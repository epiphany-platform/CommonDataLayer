use crate::{api::*, *};
use anyhow::Result;
use bb8_postgres::{
    bb8::{Pool, PooledConnection},
    tokio_postgres::{Config, NoTls},
    PostgresConnectionManager,
};
use cdl_dto::materialization::{FieldDefinition, FieldType, PostgresMaterializerOptions};
use serde_json::Value;
use std::collections::HashMap;
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

    use super::*;

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    async fn should_create_table_and_feed_data() -> Result<()> {
        let table_name = "test_simple";
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

        let query = format!("SELECT object_ids, field_a FROM {}", table_name);
        let pg_results = pg.query(query.as_str(), &[]).await.unwrap();

        assert_eq!(pg_results.len(), 1);
        let row = pg_results.first().unwrap();

        #[derive(Debug)]
        struct TestRow {
            object_ids: Vec<Uuid>,
            field_a: Value,
        }

        let row = TestRow {
            object_ids: row.get(0),
            field_a: row.get(1),
        };
        assert_eq!(row.object_ids.first().unwrap(), &object_id);
        assert_eq!(row.field_a.as_str().unwrap(), "A");

        Ok(())
    }
}

mod relations {
    use std::num::NonZeroU8;

    use cdl_api::types::view::NewRelation;
    use cdl_rpc::schema_registry::types::SearchFor;

    use super::*;

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    async fn should_properly_name_fields_from_subobjects() -> Result<()> {
        let table_name = "test_relation";
        let pg = pg_connect().await?;
        pg.batch_execute(&format!("DROP TABLE IF EXISTS {} CASCADE;", table_name))
            .await?;

        let mut fields = HashMap::new();
        fields.insert(
            "field_a".to_owned(),
            FieldDefinition::Simple {
                field_name: "FieldA".to_owned(),
                field_type: FieldType::String,
            },
        );
        fields.insert(
            "field_b".to_owned(),
            FieldDefinition::Array {
                base: 1,
                fields: {
                    let mut subfields = HashMap::new();
                    subfields.insert(
                        "field_c".to_owned(),
                        FieldDefinition::Simple {
                            field_name: "FieldD".to_owned(),
                            field_type: FieldType::String,
                        },
                    );
                    subfields
                },
            },
        );

        let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
        let schema_b = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
        let relation_id = add_relation(schema_a, schema_b).await?;

        let _view_id = add_view(
            schema_a,
            "test",
            POSTGRES_MATERIALIZER_ADDR,
            fields,
            Some(PostgresMaterializerOptions {
                table: table_name.to_owned(),
            }),
            &[NewRelation {
                global_id: relation_id,
                local_id: NonZeroU8::new(1).unwrap(),
                relations: vec![],
                search_for: SearchFor::Children,
            }],
        )
        .await?;
        let object_id_a = Uuid::new_v4();
        let object_id_b = Uuid::new_v4();
        add_edges(relation_id, object_id_a, &[object_id_b]).await?;
        insert_message(object_id_a, schema_a, r#"{"FieldA":"A"}"#).await?;
        insert_message(object_id_b, schema_b, r#"{"FieldD":"D"}"#).await?;

        sleep(Duration::from_secs(5)).await; // async view generation

        let pg_results = pg
            .query(
                format!(
                    "SELECT object_ids, field_a, field_b_field_c FROM {}",
                    table_name
                )
                .as_str(),
                &[],
            )
            .await
            .unwrap();

        assert_eq!(pg_results.len(), 1);
        let row = pg_results.first().unwrap();

        #[derive(Debug)]
        struct TestRow {
            object_ids: Vec<Uuid>,
            field_a: Value,
            field_b_c: Value,
        }

        let row = TestRow {
            object_ids: row.get(0),
            field_a: row.get(1),
            field_b_c: row.get(2),
        };
        // TODO: Order of ids should be deterministic - required for updates to materialized data
        // assert_eq!(row.object_ids.first().unwrap(), &object_id_a);
        // assert_eq!(row.object_ids.get(1).unwrap(), &object_id_b);
        assert_eq!(row.field_a.as_str().unwrap(), "A");
        assert_eq!(row.field_b_c.as_str().unwrap(), "D");
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
