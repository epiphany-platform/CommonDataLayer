use crate::{api::*, *};
use anyhow::Result;
use cdl_api::types::view::NewRelation;
use cdl_dto::materialization::{
    ComplexFilter, Computation, EqualsComputation, EqualsFilter, FieldValueComputation, Filter,
    FilterValue, RawValueComputation, RawValueFilter, SchemaFieldFilter, SimpleFilter,
    SimpleFilterKind,
};
use cdl_dto::materialization::{FieldDefinition, FieldType};
use cdl_rpc::schema_registry::types::{LogicOperator, SearchFor};
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
            None,
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
            None,
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
            None,
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
            None,
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
            None,
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

        let view = add_view(schema_a, "test", "", fields, None, &[], None).await?;
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
            None,
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

    use super::*;
    mod on_standard_field {

        use cdl_dto::materialization::{ComputedFilter, ViewPathFilter};

        use super::*;

        #[tokio::test]
        #[ignore = "todo"]
        async fn should_apply_simple_filter_for_request_with_empty_relations() -> Result<()> {
            let schema_a =
                add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;

            let object_id_a = Uuid::new_v4();
            let object_id_b = Uuid::new_v4();
            insert_message(object_id_a, schema_a, r#"{"FieldA":1}"#).await?;
            insert_message(object_id_b, schema_a, r#"{"FieldA":2}"#).await?;

            let mut fields = HashMap::new();
            fields.insert(
                "field_a".to_owned(),
                FieldDefinition::Simple {
                    field_name: "FieldA".to_owned(),
                    field_type: FieldType::Numeric,
                },
            );
            let view = add_view(
                schema_a,
                "test",
                "",
                fields,
                None,
                &[],
                Some(Filter::SimpleFilter(SimpleFilter {
                    filter: SimpleFilterKind::Equals(EqualsFilter {
                        lhs: FilterValue::SchemaField(SchemaFieldFilter {
                            field_path: "FieldA".to_owned(),
                            schema_id: 0,
                        }),
                        rhs: FilterValue::RawValue(RawValueFilter {
                            value: serde_json::to_value(&1)?.into(),
                        }),
                    }),
                })),
            )
            .await?;

            sleep(Duration::from_secs(1)).await; // async insert

            let view_data = materialize_view(view, &[schema_a]).await?;
            assert_eq!(view_data.rows.len(), 1);
            let row = view_data.rows.first().unwrap();
            let field_a = row.fields.get("field_a").unwrap().0.as_u64().unwrap();
            assert_eq!(field_a, 1);

            Ok(())
        }

        #[tokio::test]
        #[ignore = "todo"]
        async fn should_properly_merge_multiple_filters_using_and_operator() -> Result<()> {
            let schema_a =
                add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;

            let object_id_a = Uuid::new_v4();
            let object_id_b = Uuid::new_v4();
            let object_id_c = Uuid::new_v4();
            insert_message(object_id_a, schema_a, r#"{"FieldA":1,"FieldB":1}"#).await?;
            insert_message(object_id_b, schema_a, r#"{"FieldA":2,"FieldB":1}"#).await?;
            insert_message(object_id_c, schema_a, r#"{"FieldA":2,"FieldB":2}"#).await?;

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
                FieldDefinition::Simple {
                    field_name: "FieldB".to_owned(),
                    field_type: FieldType::Numeric,
                },
            );
            let view = add_view(
                schema_a,
                "test",
                "",
                fields,
                None,
                &[],
                Some(Filter::ComplexFilter(ComplexFilter {
                    operator: LogicOperator::And,
                    operands: vec![
                        Filter::SimpleFilter(SimpleFilter {
                            filter: SimpleFilterKind::Equals(EqualsFilter {
                                lhs: FilterValue::SchemaField(SchemaFieldFilter {
                                    field_path: "FieldA".to_owned(),
                                    schema_id: 0,
                                }),
                                rhs: FilterValue::RawValue(RawValueFilter {
                                    value: serde_json::to_value(&1)?.into(),
                                }),
                            }),
                        }),
                        Filter::SimpleFilter(SimpleFilter {
                            filter: SimpleFilterKind::Equals(EqualsFilter {
                                lhs: FilterValue::SchemaField(SchemaFieldFilter {
                                    field_path: "FieldB".to_owned(),
                                    schema_id: 0,
                                }),
                                rhs: FilterValue::RawValue(RawValueFilter {
                                    value: serde_json::to_value(&2)?.into(),
                                }),
                            }),
                        }),
                    ],
                })),
            )
            .await?;

            sleep(Duration::from_secs(1)).await; // async insert

            let view_data = materialize_view(view, &[schema_a]).await?;
            assert_eq!(view_data.rows.len(), 1);
            let row = view_data.rows.first().unwrap();
            let field_a = row.fields.get("field_a").unwrap().0.as_u64().unwrap();
            let field_b = row.fields.get("field_b").unwrap().0.as_u64().unwrap();
            assert_eq!(field_a, 1);
            assert_eq!(field_b, 2);

            Ok(())
        }

        #[tokio::test]
        #[ignore = "todo"]
        async fn should_properly_merge_multiple_filters_using_or_operator() -> Result<()> {
            let schema_a =
                add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;

            let object_id_a = Uuid::new_v4();
            let object_id_b = Uuid::new_v4();
            let object_id_c = Uuid::new_v4();
            insert_message(object_id_a, schema_a, r#"{"FieldA":1}"#).await?;
            insert_message(object_id_b, schema_a, r#"{"FieldA":2}"#).await?;
            insert_message(object_id_c, schema_a, r#"{"FieldA":3}"#).await?;

            let mut fields = HashMap::new();
            fields.insert(
                "field_a".to_owned(),
                FieldDefinition::Simple {
                    field_name: "FieldA".to_owned(),
                    field_type: FieldType::Numeric,
                },
            );
            let view = add_view(
                schema_a,
                "test",
                "",
                fields,
                None,
                &[],
                Some(Filter::ComplexFilter(ComplexFilter {
                    operator: LogicOperator::Or,
                    operands: vec![
                        Filter::SimpleFilter(SimpleFilter {
                            filter: SimpleFilterKind::Equals(EqualsFilter {
                                lhs: FilterValue::SchemaField(SchemaFieldFilter {
                                    field_path: "FieldA".to_owned(),
                                    schema_id: 0,
                                }),
                                rhs: FilterValue::RawValue(RawValueFilter {
                                    value: serde_json::to_value(&1)?.into(),
                                }),
                            }),
                        }),
                        Filter::SimpleFilter(SimpleFilter {
                            filter: SimpleFilterKind::Equals(EqualsFilter {
                                lhs: FilterValue::SchemaField(SchemaFieldFilter {
                                    field_path: "FieldA".to_owned(),
                                    schema_id: 0,
                                }),
                                rhs: FilterValue::RawValue(RawValueFilter {
                                    value: serde_json::to_value(&2)?.into(),
                                }),
                            }),
                        }),
                    ],
                })),
            )
            .await?;

            sleep(Duration::from_secs(1)).await; // async insert

            let view_data = materialize_view(view, &[schema_a]).await?;
            assert_eq!(view_data.rows.len(), 2);
            for row in view_data.rows {
                let field_a = row.fields.get("field_a").unwrap().0.as_u64().unwrap();
                assert!(field_a == 1 || field_a == 2);
            }

            Ok(())
        }

        #[tokio::test]
        #[ignore = "todo"]
        async fn should_properly_merge_multiple_filters_complex() -> Result<()> {
            let schema_a =
                add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;

            let object_id_a = Uuid::new_v4();
            let object_id_b = Uuid::new_v4();
            let object_id_c = Uuid::new_v4();
            let object_id_d = Uuid::new_v4();
            insert_message(object_id_a, schema_a, r#"{"FieldA":1,"FieldB":1}"#).await?;
            insert_message(object_id_b, schema_a, r#"{"FieldA":2,"FieldB":1}"#).await?;
            insert_message(object_id_c, schema_a, r#"{"FieldA":1,"FieldB":2}"#).await?;
            insert_message(object_id_d, schema_a, r#"{"FieldA":2,"FieldB":2}"#).await?;

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
                FieldDefinition::Simple {
                    field_name: "FieldB".to_owned(),
                    field_type: FieldType::Numeric,
                },
            );
            let view = add_view(
                schema_a,
                "test",
                "",
                fields,
                None,
                &[],
                Some(Filter::ComplexFilter(ComplexFilter {
                    operator: LogicOperator::Or,
                    operands: vec![
                        Filter::ComplexFilter(ComplexFilter {
                            operator: LogicOperator::And,
                            operands: vec![
                                Filter::SimpleFilter(SimpleFilter {
                                    filter: SimpleFilterKind::Equals(EqualsFilter {
                                        lhs: FilterValue::SchemaField(SchemaFieldFilter {
                                            field_path: "FieldA".to_owned(),
                                            schema_id: 0,
                                        }),
                                        rhs: FilterValue::RawValue(RawValueFilter {
                                            value: serde_json::to_value(&1)?.into(),
                                        }),
                                    }),
                                }),
                                Filter::SimpleFilter(SimpleFilter {
                                    filter: SimpleFilterKind::Equals(EqualsFilter {
                                        lhs: FilterValue::SchemaField(SchemaFieldFilter {
                                            field_path: "FieldB".to_owned(),
                                            schema_id: 0,
                                        }),
                                        rhs: FilterValue::RawValue(RawValueFilter {
                                            value: serde_json::to_value(&1)?.into(),
                                        }),
                                    }),
                                }),
                            ],
                        }),
                        Filter::ComplexFilter(ComplexFilter {
                            operator: LogicOperator::And,
                            operands: vec![
                                Filter::SimpleFilter(SimpleFilter {
                                    filter: SimpleFilterKind::Equals(EqualsFilter {
                                        lhs: FilterValue::SchemaField(SchemaFieldFilter {
                                            field_path: "FieldA".to_owned(),
                                            schema_id: 0,
                                        }),
                                        rhs: FilterValue::RawValue(RawValueFilter {
                                            value: serde_json::to_value(&2)?.into(),
                                        }),
                                    }),
                                }),
                                Filter::SimpleFilter(SimpleFilter {
                                    filter: SimpleFilterKind::Equals(EqualsFilter {
                                        lhs: FilterValue::SchemaField(SchemaFieldFilter {
                                            field_path: "FieldB".to_owned(),
                                            schema_id: 0,
                                        }),
                                        rhs: FilterValue::RawValue(RawValueFilter {
                                            value: serde_json::to_value(&2)?.into(),
                                        }),
                                    }),
                                }),
                            ],
                        }),
                    ],
                })),
            )
            .await?;

            sleep(Duration::from_secs(1)).await; // async insert

            let view_data = materialize_view(view, &[schema_a]).await?;
            assert_eq!(view_data.rows.len(), 2);
            for row in view_data.rows {
                let field_a = row.fields.get("field_a").unwrap().0.as_u64().unwrap();
                let field_b = row.fields.get("field_b").unwrap().0.as_u64().unwrap();
                assert_eq!(field_a, field_b);
                assert!(field_a == 1 || field_a == 2);
            }

            Ok(())
        }

        #[tokio::test]
        #[ignore = "todo"]
        async fn should_apply_filters_to_subrelations() -> Result<()> {
            let schema_a =
                add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
            let schema_b =
                add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
            let relation_id = add_relation(schema_a, schema_b).await?;

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
                FieldDefinition::SubObject {
                    base: 1,
                    fields: vec![(
                        "field_c".to_owned(),
                        FieldDefinition::Simple {
                            field_name: "FieldB".to_owned(),
                            field_type: FieldType::Numeric,
                        },
                    )]
                    .into_iter()
                    .collect(),
                },
            );

            let object_id_a = Uuid::new_v4();
            let object_id_b = Uuid::new_v4();
            let object_id_c = Uuid::new_v4();
            insert_message(object_id_a, schema_a, r#"{"FieldA":1}"#).await?;
            insert_message(object_id_b, schema_b, r#"{"FieldB":1}"#).await?;
            insert_message(object_id_c, schema_b, r#"{"FieldB":2}"#).await?;
            add_edges(relation_id, object_id_a, &[object_id_b, object_id_c]).await?;

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
                Some(Filter::SimpleFilter(SimpleFilter {
                    filter: SimpleFilterKind::Equals(EqualsFilter {
                        lhs: FilterValue::SchemaField(SchemaFieldFilter {
                            field_path: "FieldB".to_owned(),
                            schema_id: 1,
                        }),
                        rhs: FilterValue::RawValue(RawValueFilter {
                            value: serde_json::to_value(&1)?.into(),
                        }),
                    }),
                })),
            )
            .await?;

            sleep(Duration::from_secs(1)).await; // async insert

            let view_data = materialize_view(view, &[schema_a, schema_b]).await?;
            assert_eq!(view_data.rows.len(), 1);

            let row = view_data.rows.first().unwrap();
            let field_b = row.fields.get("field_b").unwrap().0.clone();
            let field_c = (field_b["field_c"]).as_u64().unwrap();
            assert_eq!(field_c, 1);

            Ok(())
        }

        #[tokio::test]
        #[ignore = "todo"]
        async fn should_allow_filtering_using_field_not_materialized_in_view() -> Result<()> {
            let schema_a =
                add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;

            let object_id_a = Uuid::new_v4();
            let object_id_b = Uuid::new_v4();
            insert_message(object_id_a, schema_a, r#"{"FieldA":1, "FieldB":1}"#).await?;
            insert_message(object_id_b, schema_a, r#"{"FieldA":2, "FieldB":2}"#).await?;

            let mut fields = HashMap::new();
            fields.insert(
                "field_a".to_owned(),
                FieldDefinition::Simple {
                    field_name: "FieldA".to_owned(),
                    field_type: FieldType::Numeric,
                },
            );
            let view = add_view(
                schema_a,
                "test",
                "",
                fields,
                None,
                &[],
                Some(Filter::SimpleFilter(SimpleFilter {
                    filter: SimpleFilterKind::Equals(EqualsFilter {
                        lhs: FilterValue::SchemaField(SchemaFieldFilter {
                            field_path: "FieldB".to_owned(),
                            schema_id: 0,
                        }),
                        rhs: FilterValue::RawValue(RawValueFilter {
                            value: serde_json::to_value(&1)?.into(),
                        }),
                    }),
                })),
            )
            .await?;

            sleep(Duration::from_secs(1)).await; // async insert

            let view_data = materialize_view(view, &[schema_a]).await?;
            assert_eq!(view_data.rows.len(), 1);
            let row = view_data.rows.first().unwrap();
            let field_a = row.fields.get("field_a").unwrap().0.as_u64().unwrap();
            assert_eq!(field_a, 1);

            Ok(())
        }

        #[tokio::test]
        #[ignore = "todo"]
        async fn should_allow_filtering_using_field_from_schema_not_materialized_in_view(
        ) -> Result<()> {
            let schema_a =
                add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
            let schema_b =
                add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
            let relation_id = add_relation(schema_a, schema_b).await?;

            let mut fields = HashMap::new();
            fields.insert(
                "field_a".to_owned(),
                FieldDefinition::Simple {
                    field_name: "FieldA".to_owned(),
                    field_type: FieldType::Numeric,
                },
            );

            let object_id_a = Uuid::new_v4();
            let object_id_b = Uuid::new_v4();
            let object_id_c = Uuid::new_v4();
            let object_id_d = Uuid::new_v4();
            insert_message(object_id_a, schema_a, r#"{"FieldA":1}"#).await?;
            insert_message(object_id_b, schema_a, r#"{"FieldA":2}"#).await?;
            insert_message(object_id_c, schema_b, r#"{"FieldB":1}"#).await?;
            insert_message(object_id_d, schema_b, r#"{"FieldB":2}"#).await?;
            add_edges(relation_id, object_id_a, &[object_id_c]).await?;
            add_edges(relation_id, object_id_b, &[object_id_d]).await?;

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
                Some(Filter::SimpleFilter(SimpleFilter {
                    filter: SimpleFilterKind::Equals(EqualsFilter {
                        lhs: FilterValue::SchemaField(SchemaFieldFilter {
                            field_path: "FieldB".to_owned(),
                            schema_id: 1,
                        }),
                        rhs: FilterValue::RawValue(RawValueFilter {
                            value: serde_json::to_value(&1)?.into(),
                        }),
                    }),
                })),
            )
            .await?;

            sleep(Duration::from_secs(1)).await; // async insert

            let view_data = materialize_view(view, &[schema_a, schema_b]).await?;
            assert_eq!(view_data.rows.len(), 1);

            let row = view_data.rows.first().unwrap();
            let field_a = row.fields.get("field_a").unwrap().0.as_u64().unwrap();
            assert_eq!(field_a, 1);

            Ok(())
        }

        #[tokio::test]
        #[ignore = "todo"]
        async fn should_allow_filtering_using_fields_from_view() -> Result<()> {
            let schema_a =
                add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;

            let object_id_a = Uuid::new_v4();
            let object_id_b = Uuid::new_v4();
            insert_message(object_id_a, schema_a, r#"{"FieldA":1, "FieldB":1}"#).await?;
            insert_message(object_id_b, schema_a, r#"{"FieldA":1, "FieldB":2}"#).await?;

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
                FieldDefinition::Simple {
                    field_name: "FieldB".to_owned(),
                    field_type: FieldType::Numeric,
                },
            );
            let view = add_view(
                schema_a,
                "test",
                "",
                fields,
                None,
                &[],
                Some(Filter::SimpleFilter(SimpleFilter {
                    filter: SimpleFilterKind::Equals(EqualsFilter {
                        lhs: FilterValue::ViewPath(ViewPathFilter {
                            field_path: "field_a".to_owned(),
                        }),
                        rhs: FilterValue::ViewPath(ViewPathFilter {
                            field_path: "field_b".to_owned(),
                        }),
                    }),
                })),
            )
            .await?;

            sleep(Duration::from_secs(1)).await; // async insert

            let view_data = materialize_view(view, &[schema_a]).await?;
            assert_eq!(view_data.rows.len(), 1);
            let row = view_data.rows.first().unwrap();
            let field_a = row.fields.get("field_a").unwrap().0.as_u64().unwrap();
            let field_b = row.fields.get("field_b").unwrap().0.as_u64().unwrap();
            assert_eq!(field_a, 1);
            assert_eq!(field_b, 1);

            Ok(())
        }

        #[tokio::test]
        #[ignore = "todo"]
        async fn should_allow_filtering_using_raw_value() -> Result<()> {
            let schema_a =
                add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;

            let object_id_a = Uuid::new_v4();
            insert_message(object_id_a, schema_a, r#"{"FieldA":1, "FieldB":1}"#).await?;

            let mut fields = HashMap::new();
            fields.insert(
                "field_a".to_owned(),
                FieldDefinition::Simple {
                    field_name: "FieldA".to_owned(),
                    field_type: FieldType::Numeric,
                },
            );
            let view_a = add_view(
                schema_a,
                "test",
                "",
                fields.clone(),
                None,
                &[],
                Some(Filter::SimpleFilter(SimpleFilter {
                    filter: SimpleFilterKind::Equals(EqualsFilter {
                        lhs: FilterValue::RawValue(RawValueFilter {
                            value: serde_json::to_value(0)?.into(),
                        }),
                        rhs: FilterValue::RawValue(RawValueFilter {
                            value: serde_json::to_value(1)?.into(),
                        }),
                    }),
                })),
            )
            .await?;
            let view_b = add_view(
                schema_a,
                "test",
                "",
                fields,
                None,
                &[],
                Some(Filter::SimpleFilter(SimpleFilter {
                    filter: SimpleFilterKind::Equals(EqualsFilter {
                        lhs: FilterValue::RawValue(RawValueFilter {
                            value: serde_json::to_value(1)?.into(),
                        }),
                        rhs: FilterValue::RawValue(RawValueFilter {
                            value: serde_json::to_value(1)?.into(),
                        }),
                    }),
                })),
            )
            .await?;

            sleep(Duration::from_secs(1)).await; // async insert

            let view_data_a = materialize_view(view_a, &[schema_a]).await?;
            let view_data_b = materialize_view(view_b, &[schema_a]).await?;
            assert_eq!(view_data_a.rows.len(), 0);
            assert_eq!(view_data_b.rows.len(), 1);

            Ok(())
        }

        #[tokio::test]
        #[ignore = "todo"]
        async fn should_allow_filtering_using_computed_field() -> Result<()> {
            let schema_a =
                add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;

            let object_id_a = Uuid::new_v4();
            let object_id_b = Uuid::new_v4();
            insert_message(object_id_a, schema_a, r#"{"FieldA":1}"#).await?;
            insert_message(object_id_b, schema_a, r#"{"FieldA":2}"#).await?;

            let mut fields = HashMap::new();
            fields.insert(
                "field_a".to_owned(),
                FieldDefinition::Simple {
                    field_name: "FieldA".to_owned(),
                    field_type: FieldType::Numeric,
                },
            );
            let view = add_view(
                schema_a,
                "test",
                "",
                fields,
                None,
                &[],
                Some(Filter::SimpleFilter(SimpleFilter {
                    filter: SimpleFilterKind::Equals(EqualsFilter {
                        lhs: FilterValue::Computed(ComputedFilter {
                            computation: Computation::FieldValue(FieldValueComputation {
                                field_path: "FieldA".to_owned(),
                                schema_id: 0,
                            }),
                        }),
                        rhs: FilterValue::RawValue(RawValueFilter {
                            value: serde_json::to_value(1)?.into(),
                        }),
                    }),
                })),
            )
            .await?;

            sleep(Duration::from_secs(1)).await; // async insert

            let view_data = materialize_view(view, &[schema_a]).await?;
            assert_eq!(view_data.rows.len(), 1);
            let row = view_data.rows.first().unwrap();
            let field_a = row.fields.get("field_a").unwrap().0.as_u64().unwrap();
            assert_eq!(field_a, 1);

            Ok(())
        }
    }
}
