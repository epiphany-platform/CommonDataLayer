use anyhow::Result;
use serde_json::Value;

use crate::view_plan::ViewPlan;
use crate::{sources::RowSource, ObjectIdPair};

/// Because objects are received on the go, and object builder needs to create joins,
/// these objects need to be tempoirairly stored in some kind of buffer until the last part
/// of the join arrives
#[derive(Debug)]
pub struct ObjectBuffer {
    plan: ViewPlan,
}

impl ObjectBuffer {
    pub fn new(plan: ViewPlan) -> Self {
        Self { plan }
    }

    #[tracing::instrument(skip(value))]
    pub fn add_object(
        &mut self,
        pair: ObjectIdPair,
        value: Value,
    ) -> Option<Result<Vec<RowSource>>> {
        match self.plan.missing.remove(&pair) {
            Some(missing_indices) => {
                if missing_indices.is_empty() {
                    tracing::error!("Got unexpected object: {}. Skipping...", pair.object_id);
                    return None;
                }
                let mut result = vec![];
                for missing_idx in missing_indices {
                    let unfinished_row_opt =
                        self.plan.unfinished_rows.get_mut(missing_idx).unwrap();
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
            None if self.plan.single_mode => {
                let row = self.plan.builder().build_single_row(pair);

                Some(row.map(|row| vec![row.into_single(value.clone())]))
            }
            None => None,
        }
    }
}
