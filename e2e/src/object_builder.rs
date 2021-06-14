use crate::{api::*, *};
use anyhow::Result;

mod simple_views {

    use super::*;

    #[tokio::test]
    async fn should_generate_empty_result_set_for_view_without_objects() -> Result<()> {
        let schema_id =
            add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
        let view_id = add_view(&schema_id, "test", POSTGRES_MATERIALIZER_ADDR).await?; // TODO: Materializer_addr - optional if none view should not me automatically materialized(only on demand)

        let view_data = materialize_view(&view_id, &schema_id).await?;
        assert_eq!(&view_data.to_string(), "[]");
        Ok(())
    }
    #[tokio::test]
    #[ignore = "todo"]
    async fn should_generate_results() -> Result<()> {
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
