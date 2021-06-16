use crate::{api::*, *};
use anyhow::Result;
use cdl_api::types::view::NewRelation;
use cdl_rpc::schema_registry::types::SearchFor;
use std::num::NonZeroU8;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

#[tokio::test]
#[cfg_attr(miri, ignore)]
#[ignore = "todo"]
async fn should_properly_name_nested_fields() -> Result<()> {
    let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
    let schema_b = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
    let relation_id = add_relation(schema_a, schema_b).await?;

    let view = add_view(
        schema_a,
        "test",
        POSTGRES_MATERIALIZER_ADDR,
        Default::default(),
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
    insert_message(object_id_a, schema_a, "{}").await?;
    insert_message(object_id_b, schema_b, "{}").await?;
    add_edges(relation_id, object_id_a, &[object_id_b]).await?;

    sleep(Duration::from_secs(1)).await; // async insert

    let view_data = materialize_view(view, schema_a).await?;
    assert_eq!(view_data.rows.len(), 1);

    todo!("check field names");
}
