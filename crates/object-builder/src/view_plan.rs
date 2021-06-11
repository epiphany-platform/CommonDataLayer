use anyhow::Result;
use cdl_dto::{edges::TreeResponse, materialization::FullView};
use maplit::hashset;
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    num::NonZeroU8,
};

use crate::{utils::get_base_object, FieldDefinitionSource, ObjectIdPair, RowSource};

use self::builder::ViewPlanBuilder;

mod builder;

type UnfinishedRowPair = (UnfinishedRow, HashSet<ObjectIdPair>);

#[derive(Clone)]
pub struct UnfinishedRow {
    /// Number of objects that are still missing to finish the join
    pub missing: usize,
    /// Stored objects waiting for missing ones
    pub objects: HashMap<ObjectIdPair, Value>,

    pub root_object: ObjectIdPair,

    pub fields: HashMap<String, FieldDefinitionSource>,
}

impl UnfinishedRow {
    fn new(root_object: ObjectIdPair) -> UnfinishedRowPair {
        (
            Self {
                missing: 0,
                objects: Default::default(),
                root_object,
                fields: Default::default(),
            },
            hashset![],
        )
    }

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
pub struct ViewPlan {
    pub(crate) unfinished_rows: Vec<Option<UnfinishedRow>>,
    pub(crate) missing: HashMap<ObjectIdPair, Vec<usize>>, // (_, indices to unfinished_rows)
    pub(crate) view: FullView,
}

impl ViewPlan {
    pub fn try_new(view: FullView, edges: &HashMap<NonZeroU8, TreeResponse>) -> Result<Self> {
        let mut missing: HashMap<ObjectIdPair, Vec<usize>> = Default::default();

        let builder = ViewPlanBuilder { view: &view };

        let unfinished_rows = edges
            .values()
            .flat_map(|res| res.objects.iter())
            .map(|tree_object| {
                builder.prepare_unfinished_rows(
                    get_base_object(tree_object),
                    view.fields.iter(),
                    Some(tree_object),
                )
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
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
        })
    }

    pub fn builder(&self) -> ViewPlanBuilder {
        ViewPlanBuilder { view: &self.view }
    }
}
