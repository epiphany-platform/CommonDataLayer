use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use uuid::Uuid;

pub trait OwnMessage {
    type Owned;
    fn to_owned_message(&self) -> Self::Owned;
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BorrowedInsertMessage<'a> {
    pub object_id: Uuid,
    pub schema_id: Uuid,
    pub timestamp: i64,
    #[serde(borrow)]
    pub data: &'a RawValue,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnedInsertMessage {
    pub version: String,
    pub object_id: Uuid,
    pub schema_id: Uuid,
    pub timestamp: i64,
    pub data: Box<RawValue>,
}

impl OwnMessage for BorrowedInsertMessage<'_> {
    type Owned = OwnedInsertMessage;
    fn to_owned_message(&self) -> OwnedInsertMessage {
        OwnedInsertMessage {
            version: "1.0".to_owned(),
            object_id: self.object_id,
            schema_id: self.schema_id,
            timestamp: self.timestamp,
            data: self.data.to_owned(),
        }
    }
}

impl<T> OwnMessage for T
where
    T: Clone,
{
    type Owned = T;

    fn to_owned_message(&self) -> Self::Owned {
        self.clone()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataRouterInsertMessage<'a> {
    pub version: String,
    pub object_id: Uuid,
    pub schema_id: Uuid,
    #[serde(borrow)]
    pub data: &'a RawValue,
}
