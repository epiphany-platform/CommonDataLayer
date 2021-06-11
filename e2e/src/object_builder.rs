use crate::{api::*, *};
use anyhow::Result;

mod simple_views {

    use super::*;

    #[tokio::test]
    #[ignore = "todo"]
    async fn should_generate_empty_result_set_for_view_without_objects() -> Result<()> {
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
