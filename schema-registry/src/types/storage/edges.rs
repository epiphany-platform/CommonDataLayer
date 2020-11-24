use super::*;

pub trait Edge: Sized {
    fn edge_info<'a>(self) -> (EdgeKey, Vec<(&'a str, Value)>);
    fn from_properties(properties: EdgeProperties) -> Option<Self>;
}

lazy_static! {
    // Edge Types
    pub static ref SCHEMA_DEFINITION_EDGE_TYPE: Type = Type::new("SCHEMA_DEFINITION").unwrap();
    pub static ref SCHEMA_VIEW_EDGE_TYPE: Type = Type::new("SCHEMA_VIEW").unwrap();
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SchemaView {
    pub schema_id: Uuid,
    pub view_id: Uuid,
}

impl Edge for SchemaView {
    fn edge_info<'a>(self) -> (EdgeKey, Vec<(&'a str, Value)>) {
        (
            EdgeKey::new(self.schema_id, SCHEMA_VIEW_EDGE_TYPE.clone(), self.view_id),
            vec![],
        )
    }

    fn from_properties(properties: EdgeProperties) -> Option<Self> {
        let schema_id = properties.edge.key.outbound_id;
        let view_id = properties.edge.key.inbound_id;
        Some(Self { schema_id, view_id })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SchemaDefinition {
    pub schema_id: Uuid,
    pub def_id: Uuid,
    pub version: Version,
}

impl SchemaDefinition {
    pub const VERSION: &'static str = "VERSION";
}

impl Edge for SchemaDefinition {
    fn edge_info<'a>(self) -> (EdgeKey, Vec<(&'a str, Value)>) {
        (
            EdgeKey::new(
                self.schema_id,
                SCHEMA_DEFINITION_EDGE_TYPE.clone(),
                self.def_id,
            ),
            vec![(Self::VERSION, serde_json::json!(self.version))],
        )
    }

    fn from_properties(mut properties: EdgeProperties) -> Option<Self> {
        let schema_id = properties.edge.key.outbound_id;
        let def_id = properties.edge.key.inbound_id;
        Some(Self {
            schema_id,
            def_id,
            version: get_edge_property_or(&mut properties, Self::VERSION)?,
        })
    }
}
