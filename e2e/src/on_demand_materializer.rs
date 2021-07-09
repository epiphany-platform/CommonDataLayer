use crate::{api::*, *};
use anyhow::Result;
use cdl_api::types::view::NewRelation;
use cdl_dto::materialization::{
    Computation, EqualsComputation, FieldValueComputation, RawValueComputation,
};
use cdl_dto::materialization::{FieldDefinition, FieldType};
use cdl_rpc::schema_registry::types::SearchFor;
use std::collections::HashMap;
use std::num::NonZeroU8;
use std::time::Duration;
use tokio::time::sleep;
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
            "",
            Default::default(),
            None,
            Default::default(),
        )
        .await?; // TODO: Materializer_addr - should be optional if none view should not be automatically materialized(only on demand)

        let view_data = materialize_view(view_id, &[schema_id]).await?;
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
            "",
            Default::default(),
            None,
            Default::default(),
        )
        .await?;
        let object_id = Uuid::new_v4();
        insert_message(object_id, schema_id, "{}").await?;

        sleep(Duration::from_secs(1)).await; // async insert

        let view_data = materialize_view(view_id, &[schema_id]).await?;
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
    #[ignore = "todo"]
    async fn should_return_no_results_when_one_of_related_objects_does_not_exist() -> Result<()> {
        let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
        let schema_b = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
        let relation_id = add_relation(schema_a, schema_b).await?;

        let view = add_view(
            schema_a,
            "test",
            "",
            Default::default(),
            None,
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

        let view_data = materialize_view(view, &[schema_a, schema_b]).await?;
        assert_eq!(view_data.rows.len(), 1);

        Ok(())
    }

    #[tokio::test]
    #[ignore = "todo"]
    async fn should_return_no_results_when_edge_was_not_added() -> Result<()> {
        let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
        let schema_b = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
        let relation_id = add_relation(schema_a, schema_b).await?;

        let view = add_view(
            schema_a,
            "test",
            "",
            Default::default(),
            None,
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

        let view_data = materialize_view(view, &[schema_a, schema_b]).await?;
        assert_eq!(view_data.rows.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn should_apply_inner_join_strategy() -> Result<()> {
        let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
        let schema_b = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
        let relation_id = add_relation(schema_a, schema_b).await?;

        let view = add_view(
            schema_a,
            "test",
            "",
            Default::default(),
            None,
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

        let view_data = materialize_view(view, &[schema_a, schema_b]).await?;
        assert_eq!(view_data.rows.len(), 1);

        Ok(())
    }

    #[tokio::test]
    #[ignore = "todo"]
    async fn should_join_objects_from_parent_side() {}
    #[tokio::test]
    #[ignore = "todo"]
    async fn should_join_objects_from_child_side() {}

    #[tokio::test]
    #[ignore = "todo"]
    async fn should_join_objects_in_complex_example() {
        // TODO: Rename
        // relation tree is not simple path - multiple children of one of the schemas(not view base schema)
    }
}

mod computed_fields {

    use super::*;

    #[tokio::test]
    #[ignore = "todo"]
    async fn should_compute_field_from_another_value_in_relationless_view() -> Result<()> {
        let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;

        let mut fields = HashMap::new();
        fields.insert(
            "field_a".to_owned(),
            FieldDefinition::Simple {
                field_name: "FieldA".to_owned(),
                field_type: FieldType::Numeric,
            },
        );
        fields.insert(
            "field_b".to_owned(),
            FieldDefinition::Computed {
                computation: Computation::Equals(EqualsComputation {
                    lhs: Box::new(Computation::RawValue(RawValueComputation {
                        value: serde_json::to_value("1")?.into(),
                    })),
                    rhs: Box::new(Computation::FieldValue(FieldValueComputation {
                        field_path: "FieldA".to_owned(),
                        schema_id: 0,
                    })),
                }),
                field_type: FieldType::Json, // TODO: Boolean
            },
        );
        fields.insert(
            "field_c".to_owned(),
            FieldDefinition::Computed {
                computation: Computation::Equals(EqualsComputation {
                    lhs: Box::new(Computation::RawValue(RawValueComputation {
                        value: serde_json::to_value("2")?.into(),
                    })),
                    rhs: Box::new(Computation::FieldValue(FieldValueComputation {
                        field_path: "FieldA".to_owned(),
                        schema_id: 0,
                    })),
                }),
                field_type: FieldType::Json, // TODO: Boolean
            },
        );
        let object_id_a = Uuid::new_v4();

        let view = add_view(schema_a, "test", "", fields, None, &[]).await?;
        insert_message(object_id_a, schema_a, r#"{"FieldA":1}"#).await?;

        sleep(Duration::from_secs(1)).await; // async insert

        let view_data = materialize_view(view, &[schema_a]).await?;
        assert_eq!(view_data.rows.len(), 1);
        let row = view_data.rows.first().unwrap();
        let field_a = row.fields.get("field_a").unwrap().0.as_str().unwrap();
        let field_b = row.fields.get("field_b").unwrap().0.as_bool().unwrap();
        let field_c = row.fields.get("field_c").unwrap().0.as_bool().unwrap();
        assert_eq!(field_a, "1");
        assert!(field_b);
        assert!(!field_c);

        Ok(())
    }

    #[tokio::test]
    async fn should_compute_field_from_another_object() -> Result<()> {
        let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
        let schema_b = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
        let relation_id = add_relation(schema_a, schema_b).await?;

        let object_id_a = Uuid::new_v4();
        let object_id_b = Uuid::new_v4();
        insert_message(object_id_a, schema_a, r#"{"FieldA":1}"#).await?;
        insert_message(object_id_b, schema_b, "{}").await?;

        add_edges(relation_id, object_id_a, &[object_id_b]).await?;

        let mut fields = HashMap::new();
        fields.insert(
            "field_a".to_owned(),
            FieldDefinition::Simple {
                field_name: "FieldA".to_owned(),
                field_type: FieldType::Numeric,
            },
        );
        fields.insert(
            "field_b".to_owned(),
            FieldDefinition::Computed {
                computation: Computation::Equals(EqualsComputation {
                    lhs: Box::new(Computation::RawValue(RawValueComputation {
                        value: serde_json::to_value(1)?.into(),
                    })),
                    rhs: Box::new(Computation::FieldValue(FieldValueComputation {
                        field_path: "FieldA".to_owned(),
                        schema_id: 0,
                    })),
                }),
                field_type: FieldType::Json, // TODO: Boolean
            },
        );
        fields.insert(
            "field_c".to_owned(),
            FieldDefinition::Computed {
                computation: Computation::Equals(EqualsComputation {
                    lhs: Box::new(Computation::RawValue(RawValueComputation {
                        value: serde_json::to_value(2)?.into(),
                    })),
                    rhs: Box::new(Computation::FieldValue(FieldValueComputation {
                        field_path: "FieldA".to_owned(),
                        schema_id: 0,
                    })),
                }),
                field_type: FieldType::Json, // TODO: Boolean
            },
        );
        let view = add_view(
            schema_a,
            "test",
            "",
            fields,
            None,
            &[NewRelation {
                global_id: relation_id,
                local_id: NonZeroU8::new(1).unwrap(),
                relations: vec![],
                search_for: SearchFor::Children,
            }],
        )
        .await?;

        sleep(Duration::from_secs(1)).await; // async insert

        let view_data = materialize_view(view, &[schema_a, schema_b]).await?;
        assert_eq!(view_data.rows.len(), 1);
        let row = view_data.rows.first().unwrap();
        let field_a = row.fields.get("field_a").unwrap().0.as_u64().unwrap();
        let field_b = row.fields.get("field_b").unwrap().0.as_bool().unwrap();
        let field_c = row.fields.get("field_c").unwrap().0.as_bool().unwrap();
        assert_eq!(field_a, 1);
        assert!(field_b);
        assert!(!field_c);

        Ok(())
    }
}

mod filtering {

    mod on_standard_field {
        #[tokio::test]
        #[ignore = "todo"]
        async fn should_apply_simple_filter_for_request_with_empty_relations() {}

        #[tokio::test]
        #[ignore = "todo"]
        async fn should_properly_merge_multiple_filters_using_and_operator() {}
        #[tokio::test]
        #[ignore = "todo"]
        async fn should_properly_merge_multiple_filters_using_or_operator() {}

        #[tokio::test]
        #[ignore = "todo"]
        async fn should_properly_merge_multiple_filters_complex() {}

        #[tokio::test]
        #[ignore = "todo"]
        async fn should_apply_filters_to_subrelations() {}

        #[tokio::test]
        #[ignore = "todo"]
        async fn should_allow_filtering_using_field_from_specified_schema() {}

        #[tokio::test]
        #[ignore = "todo"]
        async fn should_allow_filtering_using_field_from_view() {}

        #[tokio::test]
        #[ignore = "todo"]
        async fn should_allow_filtering_using_raw_value() {}

        #[tokio::test]
        #[ignore = "todo"]
        async fn should_allow_filtering_using_computed_field() {}
    }
}
