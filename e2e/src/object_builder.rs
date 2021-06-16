use crate::{api::*, *};
use anyhow::Result;
use cdl_api::types::view::NewRelation;
use cdl_rpc::schema_registry::types::SearchFor;
use std::num::NonZeroU8;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

mod simple_views {

    use super::*;

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    async fn should_generate_empty_result_set_for_view_without_objects() -> Result<()> {
        let schema_id =
            add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
        let view_id = add_view(
            schema_id,
            "test",
            POSTGRES_MATERIALIZER_ADDR,
            Default::default(),
            Default::default(),
        )
        .await?; // TODO: Materializer_addr - should be optional if none view should not be automatically materialized(only on demand)

        let view_data = materialize_view(view_id, schema_id).await?;
        assert!(view_data.rows.is_empty());
        Ok(())
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    async fn should_generate_results() -> Result<()> {
        let schema_id =
            add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
        let view_id = add_view(
            schema_id,
            "test",
            POSTGRES_MATERIALIZER_ADDR,
            Default::default(),
            Default::default(),
        )
        .await?;
        let object_id = Uuid::new_v4();
        insert_message(object_id, schema_id, "{}").await?;

        sleep(Duration::from_secs(1)).await; // async insert

        let view_data = materialize_view(view_id, schema_id).await?;
        assert_eq!(view_data.rows.len(), 1);
        assert!(view_data
            .rows
            .iter()
            .any(|x| x.object_ids.contains(&object_id)));
        Ok(())
    }
}

mod relations {
    use super::*;

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    #[ignore = "todo"]
    async fn should_return_no_results_when_one_of_related_objects_does_not_exist() -> Result<()> {
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
        add_edges(relation_id, object_id_a, &[object_id_b]).await?;

        sleep(Duration::from_secs(1)).await; // async insert

        let view_data = materialize_view(view, schema_a).await?;
        assert_eq!(view_data.rows.len(), 1);

        Ok(())
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    #[ignore = "todo"]
    async fn should_return_no_results_when_edge_was_not_added() -> Result<()> {
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

        sleep(Duration::from_secs(1)).await; // async insert

        let view_data = materialize_view(view, schema_a).await?;
        assert_eq!(view_data.rows.len(), 0);

        Ok(())
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    async fn should_apply_inner_join_strategy() -> Result<()> {
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

        Ok(())
    }
}

mod computed_fields {
    // TODO:
}

mod filtering {
    // TODO:
}
