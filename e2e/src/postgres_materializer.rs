use crate::{api::*, *};
use anyhow::Result;
use bb8_postgres::{
    bb8::{Pool, PooledConnection},
    tokio_postgres::{Config, NoTls},
    PostgresConnectionManager,
};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

const PG_SCHEMA: &str = "public";
const PG_USER: &str = "postgres";
const PG_PASSW: &str = "CHANGEME";
const PG_HOST: &str = "infrastructure-postgresql";
const PG_PORT: u16 = 5432;
const PG_DB_NAME: &str = "CDL";
static mut PG_POOL: Option<Pool<PostgresConnectionManager<NoTls>>> = None;

mod simple_views {

    use std::collections::HashMap;

    use cdl_dto::materialization::{FieldDefinition, FieldType, PostgresMaterializerOptions};
    use serde_json::Value;

    use super::*;

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    #[ignore = "field name in postgres - todo"]
    async fn should_create_table_and_feed_data() -> Result<()> {
        let table_name = "test_view";
        let pg = pg_connect().await?;
        pg.batch_execute(&format!("DROP TABLE IF EXISTS {} CASCADE;", table_name))
            .await?;

        let mut fields = HashMap::new();
        fields.insert(
            "field_a".to_owned(),
            FieldDefinition::Simple {
                field_name: "FieldAB".to_owned(),
                field_type: FieldType::String,
            },
        );

        let schema_id =
            add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
        let _view_id = add_view(
            schema_id,
            "test",
            POSTGRES_MATERIALIZER_ADDR,
            fields,
            Some(PostgresMaterializerOptions {
                table: table_name.to_owned(),
            }),
            Default::default(),
        )
        .await?;
        let object_id = Uuid::new_v4();
        insert_message(object_id, schema_id, r#"{"FieldAB":"A"}"#).await?;

        sleep(Duration::from_secs(5)).await; // async view generation

        // TODO: should be field_a
        let pg_results = pg
            .query("SELECT object_ids, fieldab FROM test_view", &[])
            .await
            .unwrap();

        assert_eq!(pg_results.len(), 1);
        let row = pg_results.first().unwrap();

        #[derive(Debug)]
        struct TestRow {
            object_ids: Vec<Uuid>,
            field_ab: Value,
        }

        let row = TestRow {
            object_ids: row.get(0),
            field_ab: row.get(1),
        };
        assert_eq!(row.object_ids.first().unwrap(), &object_id);
        assert_eq!(row.field_ab.as_str().unwrap(), "A");

        Ok(())
    }
}

async fn pg_connect() -> anyhow::Result<PooledConnection<'static, PostgresConnectionManager<NoTls>>>
{
    if unsafe { PG_POOL.is_none() } {
        let mut pg_config = Config::new();
        pg_config
            .user(PG_USER)
            .password(PG_PASSW)
            .host(PG_HOST)
            .port(PG_PORT)
            .dbname(PG_DB_NAME);

        let manager = PostgresConnectionManager::new(pg_config, NoTls);
        let pool = Pool::builder()
            .max_size(20)
            .connection_timeout(std::time::Duration::from_secs(30))
            .build(manager)
            .await?;
        unsafe {
            PG_POOL = Some(pool);
        }
    }
    let conn = unsafe { PG_POOL.as_ref() }.unwrap().get().await?;

    conn.execute(format!("SET search_path TO '{}'", PG_SCHEMA).as_str(), &[])
        .await?;

    Ok(conn)
}
