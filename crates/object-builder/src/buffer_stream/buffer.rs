use anyhow::Result;
use cdl_dto::{edges::TreeResponse, materialization::FullView};
use serde_json::Value;
use std::{collections::HashMap, num::NonZeroU8};

use crate::view_plan::ViewPlan;
use crate::{ObjectIdPair, RowSource};

/// Because objects are received on the go, and object builder needs to create joins,
/// these objects need to be tempoirairly stored in some kind of buffer until the last part
/// of the join arrives
pub struct ObjectBuffer {
    plan: ViewPlan,
}

impl ObjectBuffer {
    pub fn try_new(view: FullView, edges: &HashMap<NonZeroU8, TreeResponse>) -> Result<Self> {
        let plan = ViewPlan::try_new(view, edges)?;
        Ok(Self { plan })
    }

    pub fn add_object(
        &mut self,
        pair: ObjectIdPair,
        value: Value,
    ) -> Option<Result<Vec<RowSource>>> {
        match self.plan.missing.get_mut(&pair) {
            Some(missing_indices) => {
                if missing_indices.is_empty() {
                    tracing::error!("Got unexpected object: {}. Skipping...", pair.object_id);
                    return None;
                }
                let mut result = vec![];
                for missing_idx in missing_indices {
                    let unfinished_row_opt =
                        self.plan.unfinished_rows.get_mut(*missing_idx).unwrap();
                    let unfinished_row = unfinished_row_opt.as_mut()?;
                    unfinished_row.missing =
                        unfinished_row.missing.checked_sub(1).unwrap_or_default();
                    unfinished_row.objects.insert(pair, value.clone());
                    if unfinished_row.missing == 0 {
                        let row = std::mem::take(unfinished_row_opt)?; // Cant remove it because it would invalidate indices
                        result.push(row.into_join());
                    }
                }
                if result.is_empty() {
                    None
                } else {
                    Some(Ok(result))
                }
            }
            None => {
                // if we are processing object that is not missing it means either:
                // 1. there are no relations defined for this view and we should just process it
                // 2. we got an object_id event when we didnt ask for it.
                //      remember - object builder asks for specific object ids
                // Therefore only 1. option should be possible and we should return it
                let rows = self.plan.builder().prepare_unfinished_rows(
                    pair,
                    self.plan.view.fields.iter(),
                    None,
                );

                Some(rows.map(|rows| {
                    rows.into_iter()
                        .map(|(row, _)| row.into_single(value.clone()))
                        .collect()
                }))
            }
        }
    }
}
