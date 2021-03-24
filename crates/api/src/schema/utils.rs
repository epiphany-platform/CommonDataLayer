use juniper::FieldResult;
use num_traits::FromPrimitive;
use uuid::Uuid;

use super::context::SchemaRegistryConn;
use crate::error::Error;
use crate::types::schema::*;

pub async fn get_view(conn: &mut SchemaRegistryConn, id: Uuid) -> FieldResult<View> {
    tracing::debug!("get view: {:?}", id);
    let view = conn
        .get_view(rpc::schema_registry::Id { id: id.to_string() })
        .await
        .map_err(rpc::error::registry_error)?
        .into_inner();

    Ok(View {
        id,
        name: view.name,
        materializer_addr: view.materializer_addr,
        fields: view.fields,
    })
}

pub async fn get_schema(conn: &mut SchemaRegistryConn, id: Uuid) -> FieldResult<Schema> {
    tracing::debug!("get schema: {:?}", id);
    let schema = conn
        .get_schema_metadata(rpc::schema_registry::Id { id: id.to_string() })
        .await
        .map_err(rpc::error::registry_error)?
        .into_inner();

    let schema = Schema {
        id,
        name: schema.name,
        topic: schema.topic,
        query_address: schema.query_address,
        schema_type: SchemaType::from_i32(schema.schema_type)
            .ok_or(Error::InvalidSchemaType(schema.schema_type))?,
    };

    tracing::debug!("schema: {:?}", schema);

    Ok(schema)
}
