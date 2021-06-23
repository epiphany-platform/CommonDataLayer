use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use maplit::hashmap;
use serde_json::Value;

use crate::{
    row_builder::field_builder::ComputationEngine,
    sources::{FieldDefinitionSource, FilterSource, RowSource},
    ObjectIdPair, RowDefinition,
};

mod field_builder;

use field_builder::FieldBuilder;
use row_filter::RowFilter;

mod row_filter;

pub struct RowBuilder {}

impl RowBuilder {
    pub fn new() -> Self {
        Self {}
    }

    #[tracing::instrument(skip(self))]
    pub(crate) fn build(&self, source: RowSource) -> Result<Option<RowDefinition>> {
        match source {
            RowSource::Join {
                objects,
                fields,
                filters,
                ..
            } => self.build_join(objects, fields, filters),
            RowSource::Single {
                root_object,
                value,
                fields,
                filters,
            } => self.build_single(root_object, value, fields, filters),
        }
    }

    fn build_join(
        &self,
        objects: HashMap<ObjectIdPair, Value>,
        fields: HashMap<String, FieldDefinitionSource>,
        filters: Option<FilterSource>,
    ) -> Result<Option<RowDefinition>> {
        let field_builder = FieldBuilder { objects: &objects };

        let fields = fields
            .iter()
            .map(|field| field_builder.build(field))
            .collect::<anyhow::Result<_>>()?;
        let object_ids = objects
            .keys()
            .map(|object_pair| object_pair.object_id)
            .collect();

        RowFilter::new(&objects).filter(RowDefinition { object_ids, fields }, filters)
    }

    fn build_single(
        &self,
        pair: ObjectIdPair,
        object_value: Value,
        fields: HashMap<String, FieldDefinitionSource>,
        filters: Option<FilterSource>,
    ) -> Result<Option<RowDefinition>> {
        let objects = hashmap!(pair => object_value);

        let object =
            objects.get(&pair).unwrap().as_object().with_context(|| {
                format!("Expected object ({}) to be a JSON object", pair.object_id)
            })?;

        use FieldDefinitionSource::*;

        let fields = fields
            .iter()
            .map(|(field_def_key, field_def)| {
                Ok((
                    field_def_key.into(),
                    match field_def {
                        Simple { field_name, .. } => {
                            //TODO: Use field_type
                            let value = object.get(field_name).with_context(|| {
                                format!(
                                    "Object ({}) does not have a field named `{}`",
                                    pair.object_id, field_name
                                )
                            })?;
                            value.clone()
                        }
                        Computed { computation, .. } => {
                            ComputationEngine::new(&objects).compute(computation)?
                        }
                        Array { .. } => {
                            anyhow::bail!(
                                "Array field definition is not supported in relation-less view"
                            )
                        }
                    },
                ))
            })
            .collect::<anyhow::Result<_>>()?;
        let mut object_ids = HashSet::new();
        object_ids.insert(pair.object_id);

        RowFilter::new(&objects).filter(RowDefinition { object_ids, fields }, filters)
    }
}

#[cfg(all(test, not(miri)))]
mod tests {
    use std::collections::BTreeMap;

    use anyhow::Result;
    use misc_utils::serde_json::{to_string_sorted, SortSettings};

    use super::*;
    use crate::{buffer_stream::ObjectBuffer, view_plan::ViewPlan};

    #[test]
    fn test_row_builder() -> Result<()> {
        snapshot_runner::test_snapshots("builded_rows", |input| {
            let view = input.get_json("view").expect("view");
            let edges: Vec<_> = input.get_json("edges").expect("edges");
            let objects: BTreeMap<ObjectIdPair, Value> =
                input.get_json("objects").expect("could not get objects");

            let view_plan = ViewPlan::try_new(view, &edges).expect("valid view plan");
            let mut buffer = ObjectBuffer::new(view_plan);
            let row_builder = RowBuilder::new();

            let rows: Vec<RowDefinition> = objects
                .into_iter()
                .filter_map(|(id, value)| buffer.add_object(id, value))
                .collect::<Result<Vec<_>>>()
                .expect("row sources")
                .into_iter()
                .flatten()
                .flat_map(|row| row_builder.build(row).transpose())
                .collect::<Result<Vec<_>>>()
                .expect("builded rows");

            to_string_sorted(
                &rows,
                SortSettings {
                    pretty: true,
                    sort_arrays: true,
                },
            )
            .expect("Cannot serialize")
        })
    }
}
