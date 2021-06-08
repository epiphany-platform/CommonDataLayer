use anyhow::Result;
use cdl_dto::edges::TreeResponse;
use futures::Stream;
use serde_json::Value;
use std::{collections::HashMap, num::NonZeroU8, pin::Pin, task::Poll};

mod buffer;

use crate::{ObjectIdPair, RowSource};
use buffer::ObjectBuffer;

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

#[cfg(test)]
mod tests {
    use super::*;
    use cdl_dto::edges::{SchemaRelation, TreeObject, TreeResponse};
    use futures::{pin_mut, FutureExt, StreamExt};
    use maplit::*;
    use tokio::sync::mpsc::{channel, Sender};
    use tokio_stream::wrappers::ReceiverStream;
    use uuid::Uuid;

    #[tokio::test]
    async fn when_there_are_no_edges() {
        let edges = hashmap! {};
        let obj = new_obj(None);

        let (tx, stream) = act(&edges);
        pin_mut!(stream);

        assert!(stream.next().now_or_never().is_none());

        tx.send(Ok(obj.clone())).await.unwrap();

        assert_eq!(
            stream.next().now_or_never().unwrap().unwrap().unwrap(),
            RowSource::Single {
                object_pair: obj.0,
                value: obj.1
            }
        );
    }

    #[tokio::test]
    async fn when_there_are_edges() {
        let child_schema = Uuid::new_v4();
        let objects = vec![new_obj(None), new_obj(child_schema), new_obj(child_schema)];
        // Reversed order - to simulate the fact that objects can arrive via network in any order;
        let mut objects_it = objects.clone().into_iter().rev();

        let edges = hashmap! {
            1 => TreeResponse {
                objects: vec![
                    new_tree_obj(&objects, vec![])
                ]
            }
        };

        let (tx, stream) = act(&edges);
        pin_mut!(stream);

        // No object arrived, pending
        assert!(stream.next().now_or_never().is_none());

        tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
        // First object arrived, but the row is not finished
        assert!(stream.next().now_or_never().is_none());

        tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
        // Second object arrived, but the row is not finished
        assert!(stream.next().now_or_never().is_none());

        tx.send(Ok(objects_it.next().unwrap())).await.unwrap();

        // Last object arrived, row is finished
        assert_eq!(
            stream.next().now_or_never().unwrap().unwrap().unwrap(),
            RowSource::Join {
                objects: objects.into_iter().collect(),
                tree_object: edges[&1].objects[0].clone()
            }
        );
    }

    #[tokio::test]
    async fn when_there_is_more_than_one_edge() {
        let child_schema = Uuid::new_v4();
        let edge_1 = vec![new_obj(None), new_obj(child_schema), new_obj(child_schema)];
        let edge_2 = vec![new_obj(None), new_obj(child_schema), new_obj(child_schema)];

        let edges = hashmap! {
            1 => TreeResponse {
                objects: vec![
                    new_tree_obj(&edge_1, vec![]),
                    new_tree_obj(&edge_2, vec![])
                ]
            },
        };

        let objects: Vec<_> = edge_1
            .iter()
            .cloned()
            .chain(edge_2.iter().cloned())
            .collect();
        let mut objects_it = objects.into_iter();

        let (tx, stream) = act(&edges);
        pin_mut!(stream);

        // No object arrived, pending
        assert!(stream.next().now_or_never().is_none());

        tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
        // First object arrived, but the row is not finished
        assert!(stream.next().now_or_never().is_none());

        tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
        // Second object arrived, but the row is not finished
        assert!(stream.next().now_or_never().is_none());

        tx.send(Ok(objects_it.next().unwrap())).await.unwrap();

        // Last object arrived, row is finished
        assert_eq!(
            stream.next().now_or_never().unwrap().unwrap().unwrap(),
            RowSource::Join {
                objects: edge_1.into_iter().collect(),
                tree_object: edges[&1].objects[0].clone()
            }
        );

        // No object arrived, pending
        assert!(stream.next().now_or_never().is_none());

        tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
        // First object arrived, but the row is not finished
        assert!(stream.next().now_or_never().is_none());

        tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
        // Second object arrived, but the row is not finished
        assert!(stream.next().now_or_never().is_none());

        tx.send(Ok(objects_it.next().unwrap())).await.unwrap();

        // Last object arrived, row is finished
        assert_eq!(
            stream.next().now_or_never().unwrap().unwrap().unwrap(),
            RowSource::Join {
                objects: edge_2.into_iter().collect(),
                tree_object: edges[&1].objects[1].clone()
            }
        );
    }

    #[tokio::test]
    async fn when_there_are_subedges() {
        let child_schema = Uuid::new_v4();
        let edge_1 = vec![new_obj(None), new_obj(child_schema), new_obj(child_schema)];
        let edge_2 = vec![new_obj(None), new_obj(child_schema), new_obj(child_schema)];

        let edges = hashmap! {
            1 => TreeResponse {
                objects: vec![
                    new_tree_obj(&edge_1, vec![
                        TreeResponse {
                            objects: vec![
                                new_tree_obj(&edge_2, vec![])
                            ]
                        }
                    ]),
                ]
            },
        };

        let objects: Vec<_> = edge_1
            .iter()
            .cloned()
            .chain(edge_2.iter().cloned())
            .collect();
        let mut objects_it = objects.clone().into_iter();

        let (tx, stream) = act(&edges);
        pin_mut!(stream);

        // No object arrived, pending
        assert!(stream.next().now_or_never().is_none());

        tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
        // First object arrived, but the row is not finished
        assert!(stream.next().now_or_never().is_none());

        tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
        // Second object arrived, but the row is not finished
        assert!(stream.next().now_or_never().is_none());

        tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
        assert!(stream.next().now_or_never().is_none());

        tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
        assert!(stream.next().now_or_never().is_none());

        tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
        assert!(stream.next().now_or_never().is_none());

        tx.send(Ok(objects_it.next().unwrap())).await.unwrap();

        // Last object arrived, row is finished
        assert_eq!(
            stream.next().now_or_never().unwrap().unwrap().unwrap(),
            RowSource::Join {
                objects: objects.into_iter().collect(),
                tree_object: edges[&1].objects[0].clone()
            }
        );
    }

    fn new_tree_obj(edge: &[(ObjectIdPair, Value)], subtrees: Vec<TreeResponse>) -> TreeObject {
        let child_schema_id = edge[1].0.schema_id;
        let mut edge = edge.iter();
        let parent = edge.next().unwrap();
        let children: Vec<_> = edge.map(|e| e.0.object_id).collect();
        TreeObject {
            object_id: parent.0.object_id,
            children,
            subtrees,
            relation_id: Uuid::new_v4(),
            relation: SchemaRelation {
                parent_schema_id: parent.0.schema_id,
                child_schema_id,
            },
        }
    }

    fn new_obj(schema_id: impl Into<Option<Uuid>>) -> (ObjectIdPair, Value) {
        let value = "{}";
        let schema_id = schema_id.into().unwrap_or_else(Uuid::new_v4);
        let pair = ObjectIdPair {
            schema_id,
            object_id: Uuid::new_v4(),
        };
        let value: Value = serde_json::from_str(value).unwrap();
        (pair, value)
    }

    fn act(
        edges: &HashMap<u8, TreeResponse>,
    ) -> (
        Sender<Result<(ObjectIdPair, Value)>>,
        ObjectBufferedStream<ReceiverStream<Result<(ObjectIdPair, Value)>>>,
    ) {
        let edges = edges
            .into_iter()
            .filter_map(|(k, v)| NonZeroU8::new(*k).map(|k| (k, v.clone())))
            .collect();
        let (tx, rx) = channel(16);
        let rx_stream = ReceiverStream::new(rx);
        let stream = ObjectBufferedStream::new(rx_stream, &edges);

        (tx, stream)
    }
}
