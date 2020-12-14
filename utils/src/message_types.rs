use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use uuid::Uuid;
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BorrowedInsertMessage<'a> {
    pub object_id: Uuid,
    pub schema_id: Uuid,
    pub timestamp: i64,
    #[serde(borrow)]
    pub data: &'a RawValue,
}

pub struct OwnedInsertMessage {
    pub object_id: Uuid,
    pub schema_id: Uuid,
    pub timestamp: i64,
    pub data: Value,
}

impl BorrowedInsertMessage<'_> {
    pub fn to_owned(&self) -> OwnedInsertMessage {
        OwnedInsertMessage {
            object_id: self.object_id,
            schema_id: self.schema_id,
            timestamp: self.timestamp,
            data: serde_json::from_str(&self.data.get()).unwrap(),
        }
    }
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataRouterInputData<'a> {
    pub schema_id: Uuid,
    pub object_id: Uuid,
    #[serde(borrow)]
    pub data: &'a RawValue,
}
