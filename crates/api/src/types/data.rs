use juniper::{graphql_object, GraphQLInputObject};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, GraphQLInputObject)]
pub struct InputMessage {
    /// Object ID
    pub object_id: Uuid,
    /// Schema ID
    pub schema_id: Uuid,
    /// JSON-encoded payload
    pub payload: String,
}

#[derive(serde::Deserialize)]
pub struct CdlObject {
    pub object_id: Uuid,
    pub data: Value,
}

#[graphql_object]
impl CdlObject {
    /// The Object ID
    pub fn object_id(&self) -> Uuid {
        self.object_id
    }

    /// The Payload of the Object
    pub fn data(&self) -> String {
        serde_json::to_string(&self.data).unwrap_or_default()
    }
}

// fn json_to_graphql_value(json: Value) -> juniper::Value {
//     match json {
//         Value::Null => juniper::Value::Null,
//         Value::Number(num) => {
//             if let Some(f) = num.as_f64() {
//                 juniper::Value::scalar(f)
//             } else if let Some(i) = num.as_i64() {
//                 juniper::Value::scalar(i as i32)
//             } else {
//                 juniper::Value::scalar(num.as_u64().unwrap_or_default() as i32)
//             }
//         }
//         Value::String(s) => juniper::Value::scalar(s),
//         Value::Bool(b) => juniper::Value::scalar(b),
//         Value::Array(arr) => {
//             juniper::Value::List(arr.into_iter().map(json_to_graphql_value).collect())
//         }
//         Value::Object(obj) => juniper::Value::Object(
//             obj.into_iter()
//                 .map(|(key, value)| (key, json_to_graphql_value(value))
//                 .collect(),
//         ),
//     }
// }
