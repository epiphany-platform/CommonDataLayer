use anyhow::Result;
use cdl_dto::{edges::TreeResponse, materialization::FullView};
use futures::{ready, Stream};
use pin_project_lite::pin_project;
use serde_json::Value;
use std::{collections::HashMap, num::NonZeroU8, task::Poll};

mod buffer;

use crate::{ObjectIdPair, RowSource};
use buffer::ObjectBuffer;

pin_project! {
    pub struct ObjectBufferedStream<S> {
        buffer: ObjectBuffer,
        vec: Vec<RowSource>,
        #[pin]
        input: S,
    }
}

impl<S> ObjectBufferedStream<S>
where
    S: Stream<Item = Result<(ObjectIdPair, Value)>> + Unpin,
{
    pub fn try_new(
        input: S,
        view: FullView,
        edges: &HashMap<NonZeroU8, TreeResponse>,
    ) -> Result<Self> {
        Ok(Self {
            buffer: ObjectBuffer::try_new(view, edges)?,
            vec: Default::default(),
            input,
        })
    }
}

impl<S> Stream for ObjectBufferedStream<S>
where
    S: Stream<Item = Result<(ObjectIdPair, Value)>> + Unpin,
{
    type Item = Result<RowSource>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.project();

        Poll::Ready(loop {
            if let Some(row) = this.vec.pop() {
                break Some(Ok(row));
            } else if let Some(s) = ready!(this.input.as_mut().poll_next(cx)) {
                match s {
                    Err(e) => break (Some(Err(e))),
                    Ok((object_pair, object)) => {
                        match this.buffer.add_object(object_pair, object) {
                            None => return Poll::Pending,
                            Some(Err(e)) => {
                                break (Some(Err(e)));
                            }
                            Some(Ok(rows)) => {
                                *this.vec = rows;
                            }
                        }
                    }
                }
            } else {
                break None;
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::FieldDefinitionSource;

    use super::*;
    use cdl_dto::{
        edges::TreeResponse,
        materialization::{FieldDefinition, FieldType},
    };
    use futures::{pin_mut, FutureExt, StreamExt};
    use maplit::*;
    use serde_json::json;
    use tokio::sync::mpsc::{channel, Sender};
    use tokio_stream::wrappers::ReceiverStream;
    use uuid::Uuid;

    fn view_no_relations(object: ObjectIdPair) -> FullView {
        FullView {
            id: Uuid::new_v4(),
            base_schema_id: object.schema_id,
            name: "".into(),
            materializer_address: "".into(),
            materializer_options: json!({}),
            fields: hashmap! {
                "foo".into() => FieldDefinition::Simple { field_name: "foo".into(), field_type: FieldType::String }
            },
            relations: vec![],
        }
    }

    #[tokio::test]
    async fn when_there_are_no_edges() {
        let edges = hashmap! {};
        let obj = new_obj(None);
        let view = view_no_relations(obj.0);

        let (tx, stream) = act(view, &edges);
        pin_mut!(stream);

        assert!(stream.next().now_or_never().is_none());

        tx.send(Ok(obj.clone())).await.unwrap();

        assert_eq!(
            stream.next().now_or_never().unwrap().unwrap().unwrap(),
            RowSource::Single {
                root_object: obj.0,
                value: obj.1,
                fields: hashmap! {
                    "foo".into() => FieldDefinitionSource::Simple {
                        field_name: "foo".into(),
                        field_type: FieldType::String,
                        object: obj.0
                    }
                }
            }
        );
    }

    // #[tokio::test]
    // async fn when_there_are_edges() {
    //     let child_schema = Uuid::new_v4();
    //     let objects = vec![new_obj(None), new_obj(child_schema), new_obj(child_schema)];
    //     // Reversed order - to simulate the fact that objects can arrive via network in any order;
    //     let mut objects_it = objects.clone().into_iter().rev();

    //     let edges = hashmap! {
    //         1 => TreeResponse {
    //             objects: vec![
    //                 new_tree_obj(&objects, vec![])
    //             ]
    //         }
    //     };

    //     let (tx, stream) = act(&edges);
    //     pin_mut!(stream);

    //     // No object arrived, pending
    //     assert!(stream.next().now_or_never().is_none());

    //     tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
    //     // First object arrived, but the row is not finished
    //     assert!(stream.next().now_or_never().is_none());

    //     tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
    //     // Second object arrived, but the row is not finished
    //     assert!(stream.next().now_or_never().is_none());

    //     tx.send(Ok(objects_it.next().unwrap())).await.unwrap();

    //     // Last object arrived, row is finished
    //     assert_eq!(
    //         stream.next().now_or_never().unwrap().unwrap().unwrap(),
    //         RowSource::Join {
    //             objects: objects.into_iter().collect(),
    //         }
    //     );
    // }

    // #[tokio::test]
    // async fn when_there_is_more_than_one_edge() {
    //     let child_schema = Uuid::new_v4();
    //     let edge_1 = vec![new_obj(None), new_obj(child_schema), new_obj(child_schema)];
    //     let edge_2 = vec![new_obj(None), new_obj(child_schema), new_obj(child_schema)];

    //     let edges = hashmap! {
    //         1 => TreeResponse {
    //             objects: vec![
    //                 new_tree_obj(&edge_1, vec![]),
    //                 new_tree_obj(&edge_2, vec![])
    //             ]
    //         },
    //     };

    //     let objects: Vec<_> = edge_1
    //         .iter()
    //         .cloned()
    //         .chain(edge_2.iter().cloned())
    //         .collect();
    //     let mut objects_it = objects.into_iter();

    //     let (tx, stream) = act(&edges);
    //     pin_mut!(stream);

    //     // No object arrived, pending
    //     assert!(stream.next().now_or_never().is_none());

    //     tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
    //     // First object arrived, but the row is not finished
    //     assert!(stream.next().now_or_never().is_none());

    //     tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
    //     // Second object arrived, but the row is not finished
    //     assert!(stream.next().now_or_never().is_none());

    //     tx.send(Ok(objects_it.next().unwrap())).await.unwrap();

    //     // Last object arrived, row is finished
    //     assert_eq!(
    //         stream.next().now_or_never().unwrap().unwrap().unwrap(),
    //         RowSource::Join {
    //             objects: edge_1.into_iter().collect(),
    //         }
    //     );

    //     // No object arrived, pending
    //     assert!(stream.next().now_or_never().is_none());

    //     tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
    //     // First object arrived, but the row is not finished
    //     assert!(stream.next().now_or_never().is_none());

    //     tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
    //     // Second object arrived, but the row is not finished
    //     assert!(stream.next().now_or_never().is_none());

    //     tx.send(Ok(objects_it.next().unwrap())).await.unwrap();

    //     // Last object arrived, row is finished
    //     assert_eq!(
    //         stream.next().now_or_never().unwrap().unwrap().unwrap(),
    //         RowSource::Join {
    //             objects: edge_2.into_iter().collect(),
    //         }
    //     );
    // }

    // #[tokio::test]
    // async fn when_there_are_subedges() {
    //     let child_schema = Uuid::new_v4();
    //     let edge_1 = vec![new_obj(None), new_obj(child_schema), new_obj(child_schema)];
    //     let edge_2 = vec![new_obj(None), new_obj(child_schema), new_obj(child_schema)];

    //     let edges = hashmap! {
    //         1 => TreeResponse {
    //             objects: vec![
    //                 new_tree_obj(&edge_1, vec![
    //                     TreeResponse {
    //                         objects: vec![
    //                             new_tree_obj(&edge_2, vec![])
    //                         ]
    //                     }
    //                 ]),
    //             ]
    //         },
    //     };

    //     let objects: Vec<_> = edge_1
    //         .iter()
    //         .cloned()
    //         .chain(edge_2.iter().cloned())
    //         .collect();
    //     let mut objects_it = objects.clone().into_iter();

    //     let (tx, stream) = act(&edges);
    //     pin_mut!(stream);

    //     // No object arrived, pending
    //     assert!(stream.next().now_or_never().is_none());

    //     tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
    //     // First object arrived, but the row is not finished
    //     assert!(stream.next().now_or_never().is_none());

    //     tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
    //     // Second object arrived, but the row is not finished
    //     assert!(stream.next().now_or_never().is_none());

    //     tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
    //     assert!(stream.next().now_or_never().is_none());

    //     tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
    //     assert!(stream.next().now_or_never().is_none());

    //     tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
    //     assert!(stream.next().now_or_never().is_none());

    //     tx.send(Ok(objects_it.next().unwrap())).await.unwrap();

    //     // Last object arrived, row is finished
    //     assert_eq!(
    //         stream.next().now_or_never().unwrap().unwrap().unwrap(),
    //         RowSource::Join {
    //             objects: objects.into_iter().collect(),
    //         }
    //     );
    // }

    // fn new_tree_obj(edge: &[(ObjectIdPair, Value)], subtrees: Vec<TreeResponse>) -> TreeObject {
    //     let child_schema_id = edge[1].0.schema_id;
    //     let mut edge = edge.iter();
    //     let parent = edge.next().unwrap();
    //     let children: Vec<_> = edge.map(|e| e.0.object_id).collect();
    //     TreeObject {
    //         object_id: parent.0.object_id,
    //         children,
    //         subtrees,
    //         relation_id: Uuid::new_v4(),
    //         relation: SchemaRelation {
    //             parent_schema_id: parent.0.schema_id,
    //             child_schema_id,
    //         },
    //     }
    // }

    type TestStream = ObjectBufferedStream<ReceiverStream<Result<(ObjectIdPair, Value)>>>;

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
        view: FullView,
        edges: &HashMap<u8, TreeResponse>,
    ) -> (Sender<Result<(ObjectIdPair, Value)>>, TestStream) {
        let edges = edges
            .iter()
            .filter_map(|(k, v)| NonZeroU8::new(*k).map(|k| (k, v.clone())))
            .collect();
        let (tx, rx) = channel(16);
        let rx_stream = ReceiverStream::new(rx);
        let stream = ObjectBufferedStream::try_new(rx_stream, view, &edges).unwrap();

        (tx, stream)
    }
}
