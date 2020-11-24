use super::*;

pub trait Vertex: Sized {
    fn vertex_info<'a>(self) -> (Type, Vec<(&'a str, Value)>);
    fn from_properties(properties: VertexProperties) -> Option<(Uuid, Self)>;
}

lazy_static! {
    // Vertex Types
    pub static ref SCHEMA_VERTEX_TYPE: Type = Type::new("SCHEMA").unwrap();
    pub static ref SCHEMA_DEFINITION_VERTEX_TYPE: Type = Type::new("DEFINITION").unwrap();
    pub static ref VIEW_VERTEX_TYPE: Type = Type::new("VIEW").unwrap();
}

// Stored vertices
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Schema {
    pub name: String,
    pub kafka_topic: String,
    pub query_address: String,
}

impl Schema {
    pub const NAME: &'static str = "SCHEMA_NAME";
    pub const TOPIC_NAME: &'static str = "SCHEMA_TOPIC_NAME";
    pub const QUERY_ADDRESS: &'static str = "SCHEMA_QUERY_ADDRESS";
}

impl Vertex for Schema {
    fn vertex_info<'a>(self) -> (Type, Vec<(&'a str, Value)>) {
        (
            SCHEMA_VERTEX_TYPE.clone(),
            vec![
                (Self::NAME, Value::String(self.name)),
                (Self::TOPIC_NAME, Value::String(self.kafka_topic)),
                (Self::QUERY_ADDRESS, Value::String(self.query_address)),
            ],
        )
    }

    fn from_properties(mut properties: VertexProperties) -> Option<(Uuid, Self)> {
        Some((
            properties.vertex.id,
            Self {
                name: get_vertex_property_or(&mut properties, Self::NAME)?,
                kafka_topic: get_vertex_property_or(&mut properties, Self::TOPIC_NAME)?,
                query_address: get_vertex_property_or(&mut properties, Self::QUERY_ADDRESS)?,
            },
        ))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Definition {
    pub definition: Value,
}

impl Definition {
    pub const VALUE: &'static str = "DEFINITION";
}

impl Vertex for Definition {
    fn vertex_info<'a>(self) -> (Type, Vec<(&'a str, Value)>) {
        (
            SCHEMA_DEFINITION_VERTEX_TYPE.clone(),
            vec![(Definition::VALUE, self.definition)],
        )
    }

    fn from_properties(mut properties: VertexProperties) -> Option<(Uuid, Self)> {
        Some((
            properties.vertex.id,
            Self {
                definition: get_vertex_property_or(&mut properties, Definition::VALUE)?,
            },
        ))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct View {
    pub name: String,
    pub jmespath: String,
}

impl View {
    pub const NAME: &'static str = "VIEW_NAME";
    pub const EXPRESSION: &'static str = "JMESPATH";
}

impl Vertex for View {
    fn vertex_info<'a>(self) -> (Type, Vec<(&'a str, Value)>) {
        (
            VIEW_VERTEX_TYPE.clone(),
            vec![
                (View::NAME, Value::String(self.name)),
                (View::EXPRESSION, Value::String(self.jmespath)),
            ],
        )
    }

    fn from_properties(mut properties: VertexProperties) -> Option<(Uuid, View)> {
        Some((
            properties.vertex.id,
            View {
                name: get_vertex_property_or(&mut properties, View::NAME)?,
                jmespath: get_vertex_property_or(&mut properties, View::EXPRESSION)?,
            },
        ))
    }
}
