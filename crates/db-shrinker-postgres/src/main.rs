use anyhow::Context;
use postgres::{Client, Config, NoTls, Row};
use serde_json::Value as JsonValue;
use structopt::StructOpt;
use uuid::Uuid;

use tracing::{debug, info, trace};

#[derive(Debug, Clone, StructOpt)]
struct Opts {
    #[structopt(long, env = "POSTGRES_USERNAME")]
    username: String,
    #[structopt(long, env = "POSTGRES_PASSWORD")]
    password: String,
    #[structopt(long, env = "POSTGRES_HOST")]
    host: String,
    #[structopt(long, env = "POSTGRES_PORT", default_value = "5432")]
    port: u16,
    #[structopt(long, env = "POSTGRES_DBNAME")]
    dbname: String,
    #[structopt(long, env = "POSTGRES_SCHEMA", default_value = "public")]
    schema: String,
}

/// This app is called by some external scheduler (eg. cron),
/// It merges all documents with same object_id into one.
fn main() -> anyhow::Result<()> {
    let opts = Opts::from_args();

    utils::tracing::init();

    info!("Running shrinker on PSQL with {:?}", opts);

    let mut pg_config = Config::new();
    pg_config
        .user(&opts.username)
        .password(&opts.password)
        .host(&opts.host)
        .port(opts.port)
        .dbname(&opts.dbname);
    let mut client = pg_config
        .connect(NoTls)
        .context("Connection to PostgreSQL")?;

    client
        .execute(
            format!("set search_path to '{}'", opts.schema).as_str(),
            &[],
        )
        .context("Unable to select schema!")?;

    let changed_ids = client
        .query(
            "SELECT object_id FROM data GROUP BY object_id HAVING count(*) > 1",
            &[],
        )
        .context("Select duplicated object_ids")?;

    for id in changed_ids {
        let object_id: Uuid = id.get(0);

        shrink_by_id(&mut client, object_id)
            .with_context(|| format!("Shrinking of {}", object_id))?;
    }

    Ok(())
}

fn shrink_by_id(client: &mut Client, object_id: Uuid) -> anyhow::Result<()> {
    let rows = get_all_with_id(client, object_id)?;
    let version: i64 = rows
        .last()
        .context("No rows with selected ID, did database change mid-shrink?")?
        .get(0);

    debug!(
        "Shrinking object {}, {} rows, most recent version: {}",
        object_id,
        rows.len(),
        version
    );
    trace!("Rows: {:?}", rows);

    let document = minimize_document(rows).context("Failed minimizing document")?;

    trace!(
        "After minimization: {}",
        serde_json::to_string_pretty(&document)?
    );

    let mut transaction = client.transaction()?;

    transaction.query(
        "DELETE FROM data WHERE object_id = $1 AND version < $2",
        &[&object_id, &version],
    )?;
    transaction.query(
        "UPDATE data SET payload = $1 WHERE object_id = $2 AND version = $3",
        &[&document, &object_id, &version],
    )?;

    transaction.commit()?;

    Ok(())
}

fn minimize_document(rows: impl IntoIterator<Item = Row>) -> Option<JsonValue> {
    rows.into_iter()
        .map(|row| row.get::<_, JsonValue>(1))
        .reduce(|mut acc, next| {
            merge(&mut acc, next);
            acc
        })
}

fn get_all_with_id(client: &mut Client, object_id: Uuid) -> anyhow::Result<Vec<Row>> {
    let entries = client.query(
        "SELECT version, payload FROM data WHERE object_id = $1 ORDER BY version ASC",
        &[&object_id],
    )?;
    Ok(entries)
}

fn merge(a: &mut JsonValue, b: JsonValue) {
    match (a, b) {
        (JsonValue::Object(ref mut a), JsonValue::Object(b)) => {
            for (k, v) in b {
                merge(a.entry(k).or_insert(JsonValue::Null), v);
            }
        }
        (a, b) => *a = b,
    }
}

#[cfg(test)]
mod tests {
    mod describe_merge {
        use crate::merge;
        use serde_json::{json, Value};
        use test_case::test_case;

        #[test_case("{}", "{}" => json!({}))]
        #[test_case(r#"{"success": true}"#, "{}" => json!({"success": true}))]
        #[test_case(r#"{"success": true}"#, r#"{"success": false}"# => json!({"success": false}))]
        #[test_case(r#"{"success": true}"#, r#"{"success": null}"# => json!({"success": null}))]
        #[test_case(r#"{"success": {"a":1, "b":2}}"#, r#"{"success": {"c": 3}}"# => json!({"success": {"a": 1,"b": 2,"c": 3}}))]
        #[test_case(r#"{"success": [1,2]}"#, r#"{"success": [3]}"# => json!({"success": [3]}))]
        #[test_case(r#"{"top":1}"#, r#"{"level":2}"# => json!({"top": 1,"level": 2}))]
        #[test_case(r#"{"top":{"success":true}}"#, r#"{"top": null}"# => json!({"top": null}))]
        fn combines_two_jsons(left: &str, right: &str) -> Value {
            let mut left: Value = serde_json::from_str(left).unwrap();
            let right: Value = serde_json::from_str(right).unwrap();

            merge(&mut left, right);

            left
        }
    }
}
