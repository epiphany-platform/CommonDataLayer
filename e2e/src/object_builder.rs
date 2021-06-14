use crate::{api::*, *};
use anyhow::Result;
use uuid::Uuid;

mod simple_views {

    use super::*;

    #[tokio::test]
    async fn should_generate_empty_result_set_for_view_without_objects() -> Result<()> {
        let schema_id =
            add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
        let view_id = add_view(
            schema_id,
            "test",
            POSTGRES_MATERIALIZER_ADDR,
            Default::default(),
        )
        .await?; // TODO: Materializer_addr - optional if none view should not me automatically materialized(only on demand)

        let view_data = materialize_view(view_id, schema_id).await?;
        assert!(view_data.rows.is_empty());
        Ok(())
    }
    #[tokio::test]
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
    use super::*;

    #[tokio::test]
    #[ignore = "todo"]
    async fn should_work_correctly_without_defined_filter() -> Result<()> {
        Ok(())
    }

    #[tokio::test]
    #[ignore = "todo"]
    async fn should_filter_results_based_on_object_id() -> Result<()> {
        Ok(())
    }
}

mod relations {
    use super::*;
}

mod computed_fields {
    use super::*;
}
