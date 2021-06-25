use anyhow::{Context, Result};
use cdl_dto::{
    edges::{TreeObject, TreeResponse},
    materialization::{
        ComplexFilter, Computation, ComputedFilter, EqualsComputation, EqualsFilter,
        FieldDefinition, FieldValueComputation, Filter, FilterValue, FullView, RawValueComputation,
        RawValueFilter, SchemaFieldFilter, SimpleFilter, SimpleFilterKind, ViewPathFilter,
    },
};
use std::collections::{HashMap, HashSet};
use std::num::NonZeroU8;

use cdl_dto::materialization::Relation;
use uuid::Uuid;

use super::{UnfinishedRow, UnfinishedRowVariant};
use crate::sources::{ComputationSource, FieldDefinitionSource, FilterSource, FilterValueSource};
use crate::{utils::get_base_object, ObjectIdPair};

#[derive(Debug)]
pub struct ViewPlanBuilder<'a> {
    pub view: &'a FullView,
    pub relations: HashMap<Uuid, NonZeroU8>,
}

impl<'a> ViewPlanBuilder<'a> {
    pub fn new(view: &'a FullView) -> Self {
        let relations = Self::relations(view);

        Self { view, relations }
    }

    pub fn build_single_row(&self, root_object: ObjectIdPair) -> Result<UnfinishedRow> {
        let variant = UnfinishedRowVariant {
            root_object,
            objects: Default::default(),
        };

        let fields = self.build_fields(&variant, &self.view.fields)?;
        let filters = self
            .view
            .filters
            .as_ref()
            .map(|filters| self.build_filters(&variant, filters))
            .transpose()?;

        Ok(UnfinishedRow {
            missing: 1,
            objects: Default::default(),
            fields,
            filters,
            root_object,
        })
    }

    pub fn build_rows(
        &self,
        tree_responses: &[TreeResponse],
    ) -> Result<Vec<(UnfinishedRow, HashSet<ObjectIdPair>)>> {
        self.build_variants(tree_responses)
            .into_iter()
            .map(|variant| {
                let fields = self.build_fields(&variant, &self.view.fields)?;
                let filters = self
                    .view
                    .filters
                    .as_ref()
                    .map(|filters| self.build_filters(&variant, filters))
                    .transpose()?;

                let mut set: HashSet<ObjectIdPair> = variant
                    .objects
                    .into_iter()
                    .map(|(_, object)| object)
                    .collect();

                set.insert(variant.root_object);

                Ok((
                    UnfinishedRow {
                        missing: set.len(),
                        objects: Default::default(),
                        fields,
                        filters,
                        root_object: variant.root_object,
                    },
                    set,
                ))
            })
            .collect()
    }

    fn build_filters(
        &self,
        variant: &UnfinishedRowVariant,
        filters: &Filter,
    ) -> Result<FilterSource> {
        Ok(match filters {
            Filter::SimpleFilter(SimpleFilter {
                filter: SimpleFilterKind::Equals(EqualsFilter { lhs, rhs }),
            }) => FilterSource::Equals {
                lhs: self.build_filter_value(variant, lhs)?,
                rhs: self.build_filter_value(variant, rhs)?,
            },
            Filter::ComplexFilter(ComplexFilter { operator, operands }) => {
                let operands = operands
                    .iter()
                    .map(|op| self.build_filters(variant, op))
                    .collect::<Result<_>>()?;
                FilterSource::Complex {
                    operator: *operator,
                    operands,
                }
            }
        })
    }

    fn build_filter_value(
        &self,
        variant: &UnfinishedRowVariant,
        filter_value: &FilterValue,
    ) -> Result<FilterValueSource> {
        Ok(match filter_value {
            FilterValue::SchemaField(SchemaFieldFilter {
                schema_id,
                field_path,
            }) => {
                let object = match NonZeroU8::new(*schema_id) {
                    None => variant.root_object,
                    Some(relation_id) => *variant.objects.get(&relation_id).with_context(|| {
                        format!(
                            "Could not find a relation {} in view definition",
                            relation_id
                        )
                    })?,
                };
                FilterValueSource::SchemaField {
                    field_path: field_path.clone(),
                    object,
                }
            }
            FilterValue::ViewPath(ViewPathFilter { field_path }) => FilterValueSource::ViewPath {
                field_path: field_path.clone(),
            },
            FilterValue::RawValue(RawValueFilter { value }) => FilterValueSource::RawValue {
                value: value.0.clone(),
            },
            FilterValue::Computed(ComputedFilter { computation }) => {
                let computation = self.build_computation(variant, computation)?;
                FilterValueSource::Computed { computation }
            }
        })
    }

    fn build_fields(
        &self,
        variant: &UnfinishedRowVariant,
        fields: &HashMap<String, FieldDefinition>,
    ) -> Result<HashMap<String, FieldDefinitionSource>> {
        fields
            .iter()
            .map(|(field_name, field)| {
                self.build_field(&variant, field)
                    .map(|field| (field_name.clone(), field))
            })
            .collect::<Result<HashMap<_, _>>>()
    }

    fn build_variants(&self, tree_responses: &[TreeResponse]) -> Vec<UnfinishedRowVariant> {
        tree_responses
            .iter()
            .flat_map(|tree_response| tree_response.objects.iter())
            .flat_map(|tree_obj| {
                let variant = UnfinishedRowVariant {
                    root_object: get_base_object(tree_obj),
                    objects: Default::default(),
                };

                self.find_variants(variant, tree_obj)
            })
            .collect()
    }

    fn find_variants(
        &self,
        variant: UnfinishedRowVariant,
        tree_object: &TreeObject,
    ) -> Vec<UnfinishedRowVariant> {
        let local_id = match self.relations.get(&tree_object.relation_id) {
            None => {
                tracing::warn!(
                    "Got relation {} that was not requested in view definition. Skipping it.",
                    tree_object.relation_id
                );
                return vec![variant];
            }
            Some(l) => l,
        };

        tree_object
            .children
            .iter()
            .map(|child| {
                let mut variant = variant.clone();
                let object_id = ObjectIdPair {
                    schema_id: tree_object.relation.child_schema_id,
                    object_id: *child,
                };
                variant.objects.insert(*local_id, object_id);
                variant
            })
            .flat_map(|variant| {
                if tree_object.subtrees.is_empty() {
                    vec![variant]
                } else {
                    tree_object
                        .subtrees
                        .iter()
                        .flat_map(|subtree| subtree.objects.iter())
                        .flat_map(move |subtree| {
                            let variant = variant.clone();
                            self.find_variants(variant, subtree)
                        })
                        .collect()
                }
            })
            .collect()
    }

    fn relations(view: &'a FullView) -> HashMap<Uuid, NonZeroU8> {
        fn flat_relation(rel: &Relation) -> Vec<&Relation> {
            Some(rel)
                .into_iter()
                .chain(rel.relations.iter().flat_map(flat_relation))
                .collect()
        }

        view.relations
            .iter()
            .flat_map(|rel| flat_relation(rel))
            .map(|rel| (rel.global_id, rel.local_id))
            .collect()
    }

    fn build_field(
        &self,
        variant: &UnfinishedRowVariant,
        field: &FieldDefinition,
    ) -> Result<FieldDefinitionSource> {
        Ok(match field {
            FieldDefinition::Simple {
                field_name,
                field_type,
            } => FieldDefinitionSource::Simple {
                object: variant.root_object,
                field_name: field_name.clone(),
                field_type: field_type.clone(),
            },
            FieldDefinition::Computed {
                computation,
                field_type,
            } => {
                let computation = self.build_computation(variant, computation)?;
                FieldDefinitionSource::Computed {
                    computation,
                    field_type: field_type.clone(),
                }
            }
            FieldDefinition::Array { base, fields } => {
                let relation_id = NonZeroU8::new(*base)
                    .context("Array field type needs a reference to relation in view definition")?;

                let object = *variant.objects.get(&relation_id).with_context(|| {
                    format!(
                        "Could not find a relation {} in view definition",
                        relation_id
                    )
                })?;

                let mut variant = variant.clone();
                variant.root_object = object;

                FieldDefinitionSource::Array {
                    fields: self.build_fields(&variant, fields)?,
                }
            }
        })
    }

    fn build_computation(
        &self,
        variant: &UnfinishedRowVariant,
        computation: &Computation,
    ) -> Result<ComputationSource> {
        Ok(match computation {
            Computation::RawValue(RawValueComputation { value }) => ComputationSource::RawValue {
                value: value.0.clone(),
            },
            Computation::FieldValue(FieldValueComputation {
                schema_id,
                field_path,
            }) => {
                let object = match NonZeroU8::new(*schema_id) {
                    None => variant.root_object,
                    Some(relation_id) => *variant.objects.get(&relation_id).with_context(|| {
                        format!(
                            "Could not find a relation {} in view definition",
                            relation_id
                        )
                    })?,
                };

                ComputationSource::FieldValue {
                    object,
                    field_path: field_path.clone(),
                }
            }
            Computation::Equals(EqualsComputation { lhs, rhs }) => {
                let lhs = self.build_computation(variant, lhs)?;
                let rhs = self.build_computation(variant, rhs)?;

                ComputationSource::Equals {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
            }
        })
    }
}
