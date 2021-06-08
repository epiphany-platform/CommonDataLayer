use anyhow::Result;
use cdl_dto::edges::{TreeObject, TreeResponse};
use futures::Stream;
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    num::NonZeroU8,
    pin::Pin,
    task::Poll,
};

use crate::{ObjectIdPair, RowSource};

pub struct ObjectBufferedStream<S> {
    buffer: ObjectBuffer,
    input: S,
}

impl<S> ObjectBufferedStream<S>
where
    S: Stream<Item = Result<(ObjectIdPair, Value)>> + Unpin,
{
    pub fn new(input: S, edges: &HashMap<NonZeroU8, TreeResponse>) -> Self {
        Self {
            buffer: ObjectBuffer::new(edges),
            input,
        }
    }
}

impl<S> Stream for ObjectBufferedStream<S>
where
    S: Stream<Item = Result<(ObjectIdPair, Value)>> + Unpin,
{
    type Item = Result<RowSource>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match Pin::new(&mut self.input).poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(Some(Ok((object_id_pair, object)))) => {
                match self.buffer.add_object(object_id_pair, object) {
                    Some(row) => Poll::Ready(Some(Ok(row))),
                    None => Poll::Pending,
                }
            }
        }
    }
}

/// Because objects are received on the go, and object builder needs to create joins,
/// these objects need to be tempoirairly stored in some kind of buffer until the last part
/// of the join arrives
struct ObjectBuffer {
    unfinished_rows: Vec<Option<UnfinishedRow>>,
    missing: HashMap<ObjectIdPair, usize>, // (_, index to unfinished_rows)
}

struct UnfinishedRow {
    /// Number of objects that are still missing to finish the join
    missing: usize,
    /// Stored objects waiting for missing ones
    objects: HashMap<ObjectIdPair, Value>,
    /// TreeObject defining edges
    tree_object: TreeObject,
}

impl UnfinishedRow {
    fn into_source(self) -> RowSource {
        RowSource::Join {
            objects: self.objects,
            tree_object: self.tree_object,
        }
    }
}

impl ObjectBuffer {
    fn new(edges: &HashMap<NonZeroU8, TreeResponse>) -> Self {
        let mut unfinished_rows: Vec<_> = Default::default();

        let missing = edges
            .values()
            .flat_map(|res| res.objects.iter())
            .map(|obj| (retrieve_all_objects(obj), obj))
            .map(|(res, obj)| {
                unfinished_rows.push(Some(UnfinishedRow {
                    missing: res.len(),
                    tree_object: obj.clone(),
                    objects: Default::default(),
                }));
                res
            })
            .enumerate()
            .flat_map(|(idx, set)| set.into_iter().map(move |o| (o, idx)))
            .collect();

        Self {
            unfinished_rows,
            missing,
        }
    }

    pub fn add_object(&mut self, pair: ObjectIdPair, value: Value) -> Option<RowSource> {
        match self.missing.remove(&pair) {
            Some(missing_idx) => {
                let unfinished_row_opt = self.unfinished_rows.get_mut(missing_idx).unwrap();
                let unfinished_row = unfinished_row_opt.as_mut()?;
                unfinished_row.missing = unfinished_row.missing.checked_sub(1).unwrap_or_default();
                unfinished_row.objects.insert(pair, value);
                if unfinished_row.missing == 0 {
                    let row = std::mem::take(unfinished_row_opt)?; // Cant remove it because it would invalidate indices
                    Some(row.into_source())
                } else {
                    None
                }
            }
            None => {
                // if we are processing object that is not missing it means either:
                // 1. there are no relations defined for this view and we should just process it
                // 2. we got an object_id event when we didnt ask for it.
                //      remember - object builder asks for specific object ids
                // Therefore only 1. option should be possible and we should return it
                Some(RowSource::Single {
                    object_pair: pair,
                    value,
                })
            }
        }
    }
}

fn retrieve_all_objects(tree_obj: &TreeObject) -> HashSet<ObjectIdPair> {
    fn retrieve_obj(set: &mut HashSet<ObjectIdPair>, tree_object: &TreeObject) {
        set.insert(ObjectIdPair {
            schema_id: tree_object.relation.parent_schema_id,
            object_id: tree_object.object_id,
        });

        for child in tree_object.children.iter() {
            set.insert(ObjectIdPair {
                schema_id: tree_object.relation.child_schema_id,
                object_id: *child,
            });
        }
        for subtree in tree_object.subtrees.iter() {
            retrieve_rec(set, subtree);
        }
    }

    fn retrieve_rec(set: &mut HashSet<ObjectIdPair>, response: &TreeResponse) {
        for tree_object in &response.objects {
            retrieve_obj(set, tree_object);
        }
    }
    let mut set = Default::default();
    retrieve_obj(&mut set, tree_obj);
    set
}
