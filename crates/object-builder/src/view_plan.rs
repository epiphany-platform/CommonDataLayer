use anyhow::Result;
use cdl_dto::{
    edges::TreeResponse,
    materialization::{self, FullView},
};
use itertools::Itertools;
use serde::Serialize;
use serde_json::Value;
use std::{collections::HashMap, num::NonZeroU8};
use uuid::Uuid;

use crate::{FieldDefinitionSource, ObjectIdPair, RowSource};

use self::builder::ViewPlanBuilder;

mod builder;

#[derive(Clone, Debug)]
pub struct UnfinishedRowVariant {
    root_object: ObjectIdPair,
    objects: HashMap<NonZeroU8, ObjectIdPair>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UnfinishedRow {
    /// Number of objects that are still missing to finish the join
    pub missing: usize,
    /// Stored objects waiting for missing ones
    pub objects: HashMap<ObjectIdPair, Value>,

    pub root_object: ObjectIdPair,

    pub fields: HashMap<String, FieldDefinitionSource>,
}

impl UnfinishedRow {
    pub fn into_single(self, value: Value) -> RowSource {
        RowSource::Single {
            root_object: self.root_object,
            value,
            fields: self.fields,
        }
    }
    pub fn into_join(self) -> RowSource {
        RowSource::Join {
            objects: self.objects,
            root_object: self.root_object,
            fields: self.fields,
        }
    }
}

/// Because objects are received on the go, and object builder needs to create joins,
/// these objects need to be tempoirairly stored in some kind of buffer until the last part
/// of the join arrives
#[derive(Debug, Serialize)]
pub struct ViewPlan {
    pub(crate) unfinished_rows: Vec<Option<UnfinishedRow>>,
    pub(crate) missing: HashMap<ObjectIdPair, Vec<usize>>, // (_, indices to unfinished_rows)
    #[serde(skip)] // Serialize is used only for tests, we dont need to assert view
    pub(crate) view: FullView,
    #[serde(skip)] // Serialize is used only for tests, we dont need to asset it
    pub(crate) single_mode: bool,
}

impl ViewPlan {
    pub fn try_new(view: FullView, edges: &[TreeResponse]) -> Result<Self> {
        let mut missing: HashMap<ObjectIdPair, Vec<usize>> = Default::default();

        let builder = ViewPlanBuilder::new(&view);

        let unfinished_rows = builder
            .build_rows(edges)?
            .into_iter()
            .enumerate()
            .map(|(idx, (row, set))| {
                for object in set {
                    missing.entry(object).or_default().push(idx);
                }
                Some(row)
            })
            .collect();

        Ok(ViewPlan {
            unfinished_rows,
            missing,
            view,
            single_mode: edges.is_empty(),
        })
    }

    pub fn objects_filter(&self) -> HashMap<Uuid, materialization::Schema> {
        self.missing
            .keys()
            .group_by(|ObjectIdPair { schema_id, .. }| schema_id)
            .into_iter()
            .map(|(schema_id, objects)| {
                (
                    *schema_id,
                    materialization::Schema {
                        object_ids: objects.into_iter().map(|pair| pair.object_id).collect(),
                    },
                )
            })
            .collect()
    }

    pub fn builder(&self) -> ViewPlanBuilder {
        ViewPlanBuilder {
            view: &self.view,
            relations: Default::default(),
        }
    }
}

#[cfg(all(test, not(miri)))]
mod tests {
    use super::*;
    use anyhow::Result;
    use misc_utils::serde_json::{to_string_sorted, SortSettings};

    #[test]
    fn build_view_plan_test() -> Result<()> {
        snapshot_runner::test_snapshots("view_plan_build", |input| {
            let view = input.get_json("view").expect("view");
            let edges: Vec<_> = input.get_json("edges").expect("edges");

            let view_plan = ViewPlan::try_new(view, &edges).expect("valid view plan");

            to_string_sorted(
                &view_plan,
                SortSettings {
                    pretty: true,
                    sort_arrays: true,
                },
            )
            .expect("Cannot serialize")
        })
    }
}
