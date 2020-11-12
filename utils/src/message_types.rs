use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CommandServiceInsertMessage<'a> {
    pub object_id: Uuid,
    pub schema_id: Uuid,
    pub timestamp: i64,
    #[serde(borrow)]
    pub payload: &'a RawValue,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataRouterInputData<'a> {
    pub schema_id: Uuid,
    pub object_id: Uuid,
    #[serde(borrow)]
    pub data: &'a RawValue,
}
