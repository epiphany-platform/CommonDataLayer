use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BorrowedInsertMessage<'a> {
    pub object_id: Uuid,
    pub schema_id: Uuid,
    #[serde(default = "default_timestamp")]
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

fn default_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as i64
}
