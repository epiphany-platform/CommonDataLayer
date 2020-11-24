use super::{
    schema::build_full_schema,
    types::{NewSchema, NewSchemaVersion, VersionedUuid, View},
};
use crate::{
    error::{MalformedError, RegistryError, RegistryResult},
    types::DbExport,
    types::Definition,
    types::Schema,
    types::SchemaDefinition,
    types::Vertex as VertexStruct,
    types::SCHEMA_DEFINITION_VERTEX_TYPE,
    types::SCHEMA_VERTEX_TYPE,
    types::VIEW_VERTEX_TYPE,
};
use indradb::{
    Datastore, EdgeKey, EdgeQueryExt, RangeVertexQuery, SledDatastore, SpecificEdgeQuery,
    SpecificVertexQuery, Transaction, Type, Vertex, VertexQueryExt,
};
use lazy_static::lazy_static;
use log::trace;
use semver::Version;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

lazy_static! {
    // Edge Types
    static ref SCHEMA_VIEW_EDGE_TYPE: Type = Type::new("SCHEMA_VIEW").unwrap();
    static ref SCHEMA_DEFINITION_EDGE_TYPE: Type = Type::new("SCHEMA_DEFINITION").unwrap();
}

mod property {
    pub const DEFINITION_VERSION: &str = "VERSION";
}

pub struct SchemaDb<D: Datastore = SledDatastore> {
    pub db: D,
}

impl<D: Datastore> SchemaDb<D> {
    fn connect(&self) -> RegistryResult<D::Trans> {
        self.db
            .transaction()
            .map_err(RegistryError::ConnectionError)
    }

    fn create_vertex_with_properties(
        &self,
        vertex: impl VertexStruct,
        uuid: Option<Uuid>,
    ) -> RegistryResult<Uuid> {
        let conn = self.connect()?;
        let (vertex_type, properties) = vertex.vertex_info();
        let new_id = if let Some(uuid) = uuid {
            let vertex = Vertex {
                id: uuid,
                t: vertex_type,
            };
            let inserted = conn.create_vertex(&vertex)?;
            if !inserted {
                return Err(RegistryError::DuplicatedUuid(uuid));
            }
            uuid
        } else {
            conn.create_vertex_from_type(vertex_type)?
        };

        for (name, value) in properties.into_iter() {
            conn.set_vertex_properties(SpecificVertexQuery::single(new_id).property(name), &value)?;
        }

        Ok(new_id)
    }

    fn set_vertex_properties<'a>(
        &self,
        id: Uuid,
        properties: impl IntoIterator<Item = &'a (&'a str, Value)>,
    ) -> RegistryResult<()> {
        let conn = self.connect()?;
        for (name, value) in properties {
            conn.set_vertex_properties(SpecificVertexQuery::single(id).property(*name), &value)?;
        }

        Ok(())
    }

    fn set_edge_properties<'a>(
        &self,
        key: EdgeKey,
        properties: impl IntoIterator<Item = &'a (&'a str, Value)>,
    ) -> RegistryResult<()> {
        let conn = self.connect()?;
        conn.create_edge(&key)?;
        for (name, value) in properties {
            conn.set_edge_properties(
                SpecificEdgeQuery::single(key.clone()).property(*name),
                &value,
            )?;
        }

        Ok(())
    }

    pub fn ensure_schema_exists(&self, id: Uuid) -> RegistryResult<()> {
        let conn = self.connect()?;
        let vertices = conn.get_vertices(
            RangeVertexQuery::new(1)
                .t(SCHEMA_VERTEX_TYPE.clone())
                .start_id(id),
        )?;

        if vertices.is_empty() {
            Err(RegistryError::NoSchemaWithId(id))
        } else {
            Ok(())
        }
    }

    pub fn get_schema_definition(&self, id: &VersionedUuid) -> RegistryResult<SchemaDefinition> {
        let conn = self.connect()?;
        let (version, version_vertex_id) = self.get_latest_valid_schema_version(id)?;
        let query = SpecificVertexQuery::single(version_vertex_id).property(Definition::VALUE);

        let prop = conn
            .get_vertex_properties(query)?
            .into_iter()
            .next()
            .ok_or_else(|| RegistryError::NoVersionMatchesRequirement(id.clone()))?;

        Ok(SchemaDefinition {
            version,
            definition: prop.value,
        })
    }

    /// Returns a schema's versions and their respective vertex ID's
    pub fn get_schema_versions(&self, id: Uuid) -> RegistryResult<Vec<(Version, Uuid)>> {
        let conn = self.connect()?;
        conn.get_edge_properties(
            SpecificVertexQuery::single(id)
                .outbound(std::u32::MAX)
                .t(SCHEMA_DEFINITION_EDGE_TYPE.clone())
                .property(property::DEFINITION_VERSION),
        )?
        .into_iter()
        .map(|prop| {
            let version = serde_json::from_value(prop.value)
                .map_err(|_| MalformedError::MalformedSchemaVersion(id))?;

            Ok((version, prop.key.inbound_id))
        })
        .collect()
    }

    pub fn get_schema_topic(&self, id: Uuid) -> RegistryResult<String> {
        let conn = self.connect()?;
        let topic_property = conn
            .get_vertex_properties(SpecificVertexQuery::single(id).property(Schema::TOPIC_NAME))?
            .into_iter()
            .next()
            .ok_or(RegistryError::NoSchemaWithId(id))?;

        serde_json::from_value(topic_property.value)
            .map_err(|_| MalformedError::MalformedSchema(id).into())
    }

    pub fn get_schema_query_address(&self, id: Uuid) -> RegistryResult<String> {
        let conn = self.connect()?;
        let query_address_property = conn
            .get_vertex_properties(SpecificVertexQuery::single(id).property(Schema::QUERY_ADDRESS))?
            .into_iter()
            .next()
            .ok_or(RegistryError::NoSchemaWithId(id))?;

        serde_json::from_value(query_address_property.value)
            .map_err(|_| MalformedError::MalformedSchema(id).into())
    }

    fn get_latest_valid_schema_version(
        &self,
        id: &VersionedUuid,
    ) -> RegistryResult<(Version, Uuid)> {
        self.get_schema_versions(id.id)?
            .into_iter()
            .filter(|(version, _vertex_id)| id.version_req.matches(version))
            .max_by_key(|(version, _vertex_id)| version.clone())
            .ok_or_else(|| RegistryError::NoVersionMatchesRequirement(id.clone()))
    }

    pub fn get_view(&self, id: Uuid) -> RegistryResult<View> {
        let conn = self.connect()?;
        let properties = conn
            .get_all_vertex_properties(SpecificVertexQuery::single(id))?
            .into_iter()
            .next()
            .filter(|props| props.vertex.t == *VIEW_VERTEX_TYPE)
            .ok_or(RegistryError::NoViewWithId(id))?;

        View::from_properties(properties)
            .ok_or_else(|| MalformedError::MalformedView(id).into())
            .map(|(_id, view)| view)
    }

    pub fn add_schema(&self, schema: NewSchema, new_id: Option<Uuid>) -> RegistryResult<Uuid> {
        let (schema, definition) = schema.vertex();
        let full_schema = build_full_schema(definition, &self)?;

        let new_id = self.create_vertex_with_properties(schema, new_id)?;
        let new_definition_vertex_id = self.create_vertex_with_properties(full_schema, None)?;

        self.set_edge_properties(
            EdgeKey::new(
                new_id,
                SCHEMA_DEFINITION_EDGE_TYPE.clone(),
                new_definition_vertex_id,
            ),
            &[(
                property::DEFINITION_VERSION,
                serde_json::json!(Version::new(1, 0, 0)),
            )],
        )?;
        trace!("Add schema {}", new_id);
        Ok(new_id)
    }

    pub fn update_schema_name(&self, id: Uuid, new_name: String) -> RegistryResult<()> {
        self.ensure_schema_exists(id)?;

        self.set_vertex_properties(id, &[(Schema::NAME, Value::String(new_name))])?;

        Ok(())
    }

    pub fn update_schema_topic(&self, id: Uuid, new_topic: String) -> RegistryResult<()> {
        self.ensure_schema_exists(id)?;

        self.set_vertex_properties(id, &[(Schema::TOPIC_NAME, Value::String(new_topic))])?;

        Ok(())
    }

    pub fn update_schema_query_address(
        &self,
        id: Uuid,
        new_query_address: String,
    ) -> RegistryResult<()> {
        self.ensure_schema_exists(id)?;

        self.set_vertex_properties(
            id,
            &[(Schema::QUERY_ADDRESS, Value::String(new_query_address))],
        )?;

        Ok(())
    }

    pub fn add_new_version_of_schema(
        &self,
        id: Uuid,
        schema: NewSchemaVersion,
    ) -> RegistryResult<()> {
        self.ensure_schema_exists(id)?;

        if let Some((max_version, _vertex_id)) = self
            .get_schema_versions(id)?
            .into_iter()
            .max_by_key(|(version, _vertex_id)| version.clone())
        {
            if max_version >= schema.version {
                return Err(RegistryError::NewVersionMustBeGreatest {
                    schema_id: id,
                    max_version,
                });
            }
        }

        let full_schema = build_full_schema(schema.definition, &self)?;
        let new_definition_vertex_id = self.create_vertex_with_properties(full_schema, None)?;

        self.set_edge_properties(
            EdgeKey::new(
                id,
                SCHEMA_DEFINITION_EDGE_TYPE.clone(),
                new_definition_vertex_id,
            ),
            &[(
                property::DEFINITION_VERSION,
                serde_json::json!(schema.version),
            )],
        )?;

        Ok(())
    }

    pub fn add_view_to_schema(
        &self,
        schema_id: Uuid,
        view: View,
        view_id: Option<Uuid>,
    ) -> RegistryResult<Uuid> {
        let conn = self.connect()?;
        self.ensure_schema_exists(schema_id)?;
        self.validate_view(&view.jmespath)?;

        let view_id = self.create_vertex_with_properties(view, view_id)?;

        conn.create_edge(&EdgeKey::new(
            schema_id,
            SCHEMA_VIEW_EDGE_TYPE.clone(),
            view_id,
        ))?;

        Ok(view_id)
    }

    pub fn update_view(&self, id: Uuid, view: View) -> RegistryResult<View> {
        let old_view = self.get_view(id)?;
        self.validate_view(&view.jmespath)?;

        self.set_vertex_properties(id, &view.vertex_info().1)?;

        Ok(old_view)
    }

    pub fn get_all_schema_names(&self) -> RegistryResult<HashMap<Uuid, String>> {
        let conn = self.connect()?;
        let all_names = conn.get_vertex_properties(
            RangeVertexQuery::new(std::u32::MAX)
                .t(SCHEMA_VERTEX_TYPE.clone())
                .property(Schema::NAME),
        )?;

        all_names
            .into_iter()
            .map(|props| {
                let schema_id = props.id;
                let name = serde_json::from_value(props.value)
                    .map_err(|_| MalformedError::MalformedSchema(schema_id))?;

                Ok((schema_id, name))
            })
            .collect()
    }

    pub fn get_all_views_of_schema(&self, schema_id: Uuid) -> RegistryResult<HashMap<Uuid, View>> {
        let conn = self.connect()?;
        self.ensure_schema_exists(schema_id)?;

        let all_views = conn.get_all_vertex_properties(
            SpecificVertexQuery::single(schema_id)
                .outbound(std::u32::MAX)
                .t(SCHEMA_VIEW_EDGE_TYPE.clone())
                .inbound(std::u32::MAX),
        )?;

        all_views
            .into_iter()
            .map(|props| {
                let view_id = props.vertex.id;

                View::from_properties(props)
                    .ok_or_else(|| MalformedError::MalformedView(view_id).into())
            })
            .collect()
    }

    fn validate_view(&self, view: &str) -> RegistryResult<()> {
        jmespatch::parse(view)
            .map(|_| ())
            .map_err(|err| RegistryError::InvalidView(err.to_string()))
    }

    pub fn import_all(&self, imported: DbExport) -> RegistryResult<()> {
        for (schema_id, schema) in imported.schemas {
            self.create_vertex_with_properties(schema, Some(schema_id))?;
        }

        for (def_id, def) in imported.definitions {
            self.create_vertex_with_properties(def, Some(def_id))?;
        }

        for (view_id, view) in imported.views {
            self.create_vertex_with_properties(view, Some(view_id))?;
        }

        let conn = self.connect()?;

        for (schema_id, def_id) in imported.def_edges {
            conn.create_edge(&EdgeKey::new(
                schema_id,
                SCHEMA_DEFINITION_EDGE_TYPE.clone(),
                def_id,
            ))?;
        }

        for (schema_id, view_id) in imported.view_edges {
            conn.create_edge(&EdgeKey::new(
                schema_id,
                SCHEMA_VIEW_EDGE_TYPE.clone(),
                view_id,
            ))?;
        }

        Ok(())
    }

    pub fn export_all(&self) -> RegistryResult<DbExport> {
        let conn = self.connect()?;

        let all_definitions = conn.get_all_vertex_properties(
            RangeVertexQuery::new(std::u32::MAX).t(SCHEMA_DEFINITION_VERTEX_TYPE.clone()),
        )?;

        let all_schemas = conn.get_all_vertex_properties(
            RangeVertexQuery::new(std::u32::MAX).t(SCHEMA_VERTEX_TYPE.clone()),
        )?;

        let all_views = conn.get_all_vertex_properties(
            RangeVertexQuery::new(std::u32::MAX).t(VIEW_VERTEX_TYPE.clone()),
        )?;

        let all_def_edges = conn.get_all_edge_properties(
            RangeVertexQuery::new(std::u32::MAX)
                .outbound(std::u32::MAX)
                .t(SCHEMA_DEFINITION_EDGE_TYPE.clone()),
        )?;

        let all_view_edges = conn.get_all_edge_properties(
            RangeVertexQuery::new(std::u32::MAX)
                .outbound(std::u32::MAX)
                .t(SCHEMA_VIEW_EDGE_TYPE.clone()),
        )?;

        let definitions = all_definitions
            .into_iter()
            .map(|props| {
                let def_id = props.vertex.id;
                Definition::from_properties(props)
                    .ok_or_else(|| MalformedError::MalformedDefinition(def_id).into())
            })
            .collect::<RegistryResult<HashMap<Uuid, Definition>>>()?;

        let schemas = all_schemas
            .into_iter()
            .map(|props| {
                let schema_id = props.vertex.id;
                Schema::from_properties(props)
                    .ok_or_else(|| MalformedError::MalformedSchema(schema_id).into())
            })
            .collect::<RegistryResult<HashMap<Uuid, Schema>>>()?;

        let views = all_views
            .into_iter()
            .map(|props| {
                let view_id = props.vertex.id;
                View::from_properties(props)
                    .ok_or_else(|| MalformedError::MalformedView(view_id).into())
            })
            .collect::<RegistryResult<HashMap<Uuid, View>>>()?;

        let def_edges = all_def_edges
            .into_iter()
            .map(|props| {
                let inbound_id = props.edge.key.inbound_id;
                let outbound_id = props.edge.key.outbound_id;
                (outbound_id, inbound_id)
            })
            .collect::<HashMap<Uuid, Uuid>>();

        let view_edges = all_view_edges
            .into_iter()
            .map(|props| {
                let inbound_id = props.edge.key.inbound_id;
                let outbound_id = props.edge.key.outbound_id;
                (outbound_id, inbound_id)
            })
            .collect::<HashMap<Uuid, Uuid>>();

        Ok(DbExport {
            schemas,
            definitions,
            views,
            def_edges,
            view_edges,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use indradb::MemoryDatastore;
    use serde_json::json;

    #[test]
    fn import_export_all() -> Result<()> {
        let db = SchemaDb {
            db: MemoryDatastore::default(),
        };

        let added_schema_id = db.add_schema(
            NewSchema {
                name: "test".into(),
                definition: json! ({
                    "definitions": {
                        "def1": {
                            "a": "number"
                        },
                        "def2": {
                            "b": "string"
                        }
                    }
                }),
                kafka_topic: "topic1".into(),
                query_address: "query1".into(),
            },
            None,
        )?;
        db.add_view_to_schema(
            added_schema_id,
            View {
                name: "view1".into(),
                jmespath: "{ a: a }".into(),
            },
            None,
        )?;

        let original_result = db.export_all()?;

        let new_db = SchemaDb {
            db: MemoryDatastore::default(),
        };
        new_db.import_all(original_result.clone())?;

        let new_result = new_db.export_all()?;

        assert_eq!(original_result, new_result);

        Ok(())
    }

    #[test]
    fn export_all() -> Result<()> {
        let db = SchemaDb {
            db: MemoryDatastore::default(),
        };

        let added_schema_id = db.add_schema(
            NewSchema {
                name: "test".into(),
                definition: json! ({
                    "definitions": {
                        "def1": {
                            "a": "number"
                        },
                        "def2": {
                            "b": "string"
                        }
                    }
                }),
                kafka_topic: "topic1".into(),
                query_address: "query1".into(),
            },
            None,
        )?;
        db.add_view_to_schema(
            added_schema_id,
            View {
                name: "view1".into(),
                jmespath: "{ a: a }".into(),
            },
            None,
        )?;

        let result = db.export_all()?;

        let (schema_id, schema) = result.schemas.iter().next().unwrap();
        assert_eq!(added_schema_id, *schema_id);
        assert_eq!("test", schema.name);

        let (def_id, definition) = result.definitions.iter().next().unwrap();
        assert!(definition.definition.is_object());
        assert_eq!(
            r#"{"definitions":{"def1":{"a":"number"},"def2":{"b":"string"}}}"#,
            serde_json::to_string(&definition.definition).unwrap()
        );

        let (view_id, view) = result.views.iter().next().unwrap();
        assert_eq!(r#"{ a: a }"#, view.jmespath);

        let (edge_in, edge_out) = result.def_edges.iter().next().unwrap();
        assert_eq!(schema_id, edge_in);
        assert_eq!(def_id, edge_out);

        let (edge_in, edge_out) = result.view_edges.iter().next().unwrap();
        assert_eq!(schema_id, edge_in);
        assert_eq!(view_id, edge_out);

        Ok(())
    }
}
