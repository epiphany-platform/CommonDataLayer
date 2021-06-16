use crate::{api::*, *};
use anyhow::Result;
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
        )
        .await?;
        let object_id = Uuid::new_v4();
        insert_message(object_id, schema_id, "{}").await?;

        let view_data = materialize_view(view_id, schema_id).await?;
        assert_eq!(view_data.rows.len(), 1);
        assert!(view_data.rows.iter().any(|x| x.object_id == object_id));
        Ok(())
    }
}

mod filtering {
    // TODO:
}

mod relations {
    // TODO:
}

mod computed_fields {
    // TODO:
}
