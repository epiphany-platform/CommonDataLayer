use uuid::Uuid;

use crate::{RequestError, RequestResult};

#[derive(Clone, PartialEq, Debug)]
pub struct TreeResponse {
    pub objects: Vec<TreeObject>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TreeObject {
    pub object_id: Uuid,
    pub relation_id: Uuid,
    pub relation: SchemaRelation,
    pub children: Vec<Uuid>,
    pub subtrees: Vec<TreeResponse>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct SchemaRelation {
    pub parent_schema_id: Uuid,
    pub child_schema_id: Uuid,
}

impl TreeResponse {
    pub fn from_rpc(rpc: rpc::edge_registry::TreeResponse) -> RequestResult<Self> {
        Ok(Self {
            objects: rpc
                .objects
                .into_iter()
                .map(TreeObject::from_rpc)
                .collect::<RequestResult<Vec<_>>>()?,
        })
    }
}

impl TreeObject {
    pub fn from_rpc(rpc: rpc::edge_registry::TreeObject) -> RequestResult<Self> {
        Ok(Self {
            object_id: rpc.object_id.parse()?,
            relation_id: rpc.relation_id.parse()?,
            relation: SchemaRelation::from_rpc(rpc.relation)?,
            children: rpc
                .children
                .into_iter()
                .map(|child| Uuid::parse_str(&child).map_err(RequestError::from))
                .collect::<RequestResult<Vec<_>>>()?,
            subtrees: rpc
                .subtrees
                .into_iter()
                .map(TreeResponse::from_rpc)
                .collect::<RequestResult<Vec<_>>>()?,
        })
    }
}

impl SchemaRelation {
    pub fn from_rpc(rpc: rpc::edge_registry::SchemaRelation) -> RequestResult<Self> {
        Ok(Self {
            parent_schema_id: rpc.parent_schema_id.parse()?,
            child_schema_id: rpc.child_schema_id.parse()?,
        })
    }
}
