use super::{
    schema::build_full_schema,
    types::{
        storage::edges::{
            Edge, SchemaDefinition as SchemaDefinitionEdge, SchemaView as SchemaViewEdge,
        },
        storage::vertices::{Definition, Schema, Vertex, View},
        NewSchema, NewSchemaVersion, VersionedUuid,
    },
};
use crate::{
    error::{MalformedError, RegistryError, RegistryResult},
    types::DbExport,
    types::SchemaDefinition,
    types::ViewUpdate,
};
use indradb::{
    Datastore, EdgeQueryExt, RangeVertexQuery, SledDatastore, SpecificEdgeQuery,
    SpecificVertexQuery, Transaction, VertexQueryExt,
};
use log::{trace, warn};
use rpc::schema_registry::types::SchemaType;
use semver::Version;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

pub struct SchemaDb<D: Datastore = SledDatastore> {
    pub db: D,
}

impl<D: Datastore> SchemaDb<D> {
    fn connect(&self) -> RegistryResult<D::Trans> {
        self.db
            .transaction()
            .map_err(RegistryError::ConnectionError)
    }

    fn create_vertex_with_properties<V: Vertex>(
        &self,
        vertex: V,
        uuid: Option<Uuid>,
    ) -> RegistryResult<Uuid> {
        let conn = self.connect()?;
        let properties = vertex.into_properties();
        let new_id = if let Some(uuid) = uuid {
            let vertex = indradb::Vertex {
                id: uuid,
                t: V::db_type(),
            };
            let inserted = conn.create_vertex(&vertex)?;
            if !inserted {
                return Err(RegistryError::DuplicatedUuid(uuid));
            }
            uuid
        } else {
            conn.create_vertex_from_type(V::db_type())?
        };

        for (name, value) in properties {
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

    fn set_edge_properties(&self, edge: impl Edge) -> RegistryResult<()> {
        let conn = self.connect()?;
        let (key, properties) = edge.edge_info();
        conn.create_edge(&key)?;
        for (name, value) in properties {
            conn.set_edge_properties(
                SpecificEdgeQuery::single(key.clone()).property(name),
                &value,
            )?;
        }

        Ok(())
    }

    pub fn ensure_schema_exists(&self, id: Uuid) -> RegistryResult<()> {
        let conn = self.connect()?;
        let vertices = conn.get_vertices(SpecificVertexQuery::single(id))?;

        if vertices.is_empty() {
            Err(RegistryError::NoSchemaWithId(id))
        } else {
            Ok(())
        }
    }

    pub fn get_schema(&self, id: Uuid) -> RegistryResult<Schema> {
        let conn = self.connect()?;
        let props = conn
            .get_all_vertex_properties(SpecificVertexQuery::single(id))?
            .into_iter()
            .next()
            .ok_or(RegistryError::NoSchemaWithId(id))?;

        let schema_id = props.vertex.id;
        Schema::from_properties(props)
            .map(|(_, schema)| schema)
            .ok_or_else(|| MalformedError::MalformedSchema(schema_id).into())
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
                .t(SchemaDefinitionEdge::db_type())
                .property(SchemaDefinitionEdge::VERSION),
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
            .get_vertex_properties(
                SpecificVertexQuery::single(id).property(Schema::INSERT_DESTINATION),
            )?
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

    pub fn get_schema_type(&self, id: Uuid) -> RegistryResult<SchemaType> {
        let conn = self.connect()?;
        let type_property = conn
            .get_vertex_properties(SpecificVertexQuery::single(id).property(Schema::SCHEMA_TYPE))?
            .into_iter()
            .next()
            .ok_or(RegistryError::NoSchemaWithId(id))?;

        serde_json::from_value(type_property.value)
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
        let db_type = View::db_type();
        let properties = conn
            .get_all_vertex_properties(SpecificVertexQuery::single(id))?
            .into_iter()
            .next()
            .filter(|props| props.vertex.t == db_type)
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

        self.set_edge_properties(SchemaDefinitionEdge {
            schema_id: new_id,
            definition_id: new_definition_vertex_id,
            version: Version::new(1, 0, 0),
        })?;
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

        self.set_vertex_properties(
            id,
            &[(Schema::INSERT_DESTINATION, Value::String(new_topic))],
        )?;

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

    pub fn update_schema_type(&self, id: Uuid, new_schema_type: SchemaType) -> RegistryResult<()> {
        self.ensure_schema_exists(id)?;

        self.set_vertex_properties(
            id,
            &[(
                Schema::SCHEMA_TYPE,
                Value::String(new_schema_type.to_string()),
            )],
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

        self.set_edge_properties(SchemaDefinitionEdge {
            version: schema.version,
            schema_id: id,
            definition_id: new_definition_vertex_id,
        })?;

        Ok(())
    }

    pub fn add_view_to_schema(
        &self,
        schema_id: Uuid,
        view: View,
        view_id: Option<Uuid>,
    ) -> RegistryResult<Uuid> {
        self.ensure_schema_exists(schema_id)?;
        self.validate_view(&view.jmespath)?;

        let view_id = self.create_vertex_with_properties(view, view_id)?;

        self.set_edge_properties(SchemaViewEdge { schema_id, view_id })?;

        Ok(view_id)
    }

    pub fn update_view(&self, id: Uuid, view: ViewUpdate) -> RegistryResult<View> {
        let old_view = self.get_view(id)?;

        if let Some(jmespath) = view.jmespath.as_ref() {
            self.validate_view(jmespath)?;
        }

        if let Some(name) = view.name {
            self.set_vertex_properties(id, &[(View::NAME, Value::String(name))])?;
        }

        if let Some(jmespath) = view.jmespath {
            self.set_vertex_properties(id, &[(View::EXPRESSION, Value::String(jmespath))])?;
        }

        Ok(old_view)
    }

    pub fn get_all_schemas(&self) -> RegistryResult<HashMap<Uuid, Schema>> {
        let conn = self.connect()?;
        let all_schemas = conn
            .get_all_vertex_properties(RangeVertexQuery::new(std::u32::MAX).t(Schema::db_type()))?;

        all_schemas
            .into_iter()
            .map(|props| {
                let schema_id = props.vertex.id;
                Schema::from_properties(props)
                    .ok_or_else(|| MalformedError::MalformedSchema(schema_id).into())
            })
            .collect()
    }

    pub fn get_all_schema_names(&self) -> RegistryResult<HashMap<Uuid, String>> {
        let conn = self.connect()?;
        let all_names = conn.get_vertex_properties(
            RangeVertexQuery::new(std::u32::MAX)
                .t(Schema::db_type())
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
                .t(SchemaViewEdge::db_type())
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
        if !self.get_all_schema_names()?.is_empty() {
            warn!("[IMPORT] Database is not empty, skipping importing");
            return Ok(());
        }

        for (schema_id, schema) in imported.schemas {
            self.create_vertex_with_properties(schema, Some(schema_id))?;
        }

        for (definition_id, def) in imported.definitions {
            self.create_vertex_with_properties(def, Some(definition_id))?;
        }

        for (view_id, view) in imported.views {
            self.create_vertex_with_properties(view, Some(view_id))?;
        }

        for schema_definition in imported.schema_definitions {
            self.set_edge_properties(schema_definition)?;
        }

        for schema_view in imported.schema_views {
            self.set_edge_properties(schema_view)?;
        }

        Ok(())
    }

    pub fn export_all(&self) -> RegistryResult<DbExport> {
        let conn = self.connect()?;

        let all_definitions = conn.get_all_vertex_properties(
            RangeVertexQuery::new(std::u32::MAX).t(Definition::db_type()),
        )?;

        let all_schemas = conn
            .get_all_vertex_properties(RangeVertexQuery::new(std::u32::MAX).t(Schema::db_type()))?;

        let all_views = conn
            .get_all_vertex_properties(RangeVertexQuery::new(std::u32::MAX).t(View::db_type()))?;

        let all_schema_definitions = conn.get_all_edge_properties(
            RangeVertexQuery::new(std::u32::MAX)
                .outbound(std::u32::MAX)
                .t(SchemaDefinitionEdge::db_type()),
        )?;

        let all_schema_views = conn.get_all_edge_properties(
            RangeVertexQuery::new(std::u32::MAX)
                .outbound(std::u32::MAX)
                .t(SchemaViewEdge::db_type()),
        )?;

        let definitions = all_definitions
            .into_iter()
            .map(|props| {
                let definition_id = props.vertex.id;
                Definition::from_properties(props)
                    .ok_or_else(|| MalformedError::MalformedDefinition(definition_id).into())
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

        let schema_definitions = all_schema_definitions
            .into_iter()
            .map(|props| {
                let schema_id = props.edge.key.outbound_id;
                SchemaDefinitionEdge::from_properties(props)
                    .ok_or_else(|| MalformedError::MalformedSchemaVersion(schema_id).into())
            })
            .collect::<RegistryResult<Vec<SchemaDefinitionEdge>>>()?;

        let schema_views = all_schema_views
            .into_iter()
            .map(|props| {
                SchemaViewEdge::from_properties(props).unwrap() // View edge has no params, always passes
            })
            .collect::<Vec<SchemaViewEdge>>();

        Ok(DbExport {
            schemas,
            definitions,
            views,
            schema_definitions,
            schema_views,
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
    fn import_non_empty() -> Result<()> {
        let (to_import, schema1_id, view1_id) = prepare_db_export()?;

        let db = SchemaDb {
            db: MemoryDatastore::default(),
        };
        let schema2_id = db.add_schema(schema2(), None)?;
        let view2_id = db.add_view_to_schema(schema2_id, view2(), None)?;

        db.ensure_schema_exists(schema2_id)?;
        assert!(db.ensure_schema_exists(schema1_id).is_err());
        db.get_view(view2_id)?;
        assert!(db.get_view(view1_id).is_err());

        db.import_all(to_import)?;

        // Ensure nothing changed
        db.ensure_schema_exists(schema2_id)?;
        assert!(db.ensure_schema_exists(schema1_id).is_err());
        db.get_view(view2_id)?;
        assert!(db.get_view(view1_id).is_err());

        Ok(())
    }

    #[test]
    fn import_all() -> Result<()> {
        let (original_result, original_schema_id, original_view_id) = prepare_db_export()?;

        let db = SchemaDb {
            db: MemoryDatastore::default(),
        };

        db.import_all(original_result)?;

        db.ensure_schema_exists(original_schema_id)?;

        let (schema_id, schema_name) = db.get_all_schema_names()?.into_iter().next().unwrap();
        assert_eq!(original_schema_id, schema_id);
        assert_eq!("test", schema_name);

        let defs = db.get_schema_definition(&VersionedUuid::any(original_schema_id))?;
        assert_eq!(Version::new(1, 0, 0), defs.version);
        assert_eq!(
            r#"{"definitions":{"def1":{"a":"number"},"def2":{"b":"string"}}}"#,
            serde_json::to_string(&defs.definition).unwrap()
        );

        let (view_id, view) = db
            .get_all_views_of_schema(original_schema_id)?
            .into_iter()
            .next()
            .unwrap();
        assert_eq!(original_view_id, view_id);
        assert_eq!(r#"{ a: a }"#, view.jmespath);

        Ok(())
    }

    #[test]
    fn import_export_all() -> Result<()> {
        let original_result = prepare_db_export()?.0;

        let db = SchemaDb {
            db: MemoryDatastore::default(),
        };
        db.import_all(original_result.clone())?;

        let new_result = db.export_all()?;

        assert_eq!(original_result, new_result);

        Ok(())
    }

    #[test]
    fn export_all() -> Result<()> {
        let (result, original_schema_id, original_view_id) = prepare_db_export()?;

        let (schema_id, schema) = result.schemas.into_iter().next().unwrap();
        assert_eq!(original_schema_id, schema_id);
        assert_eq!("test", schema.name);

        let (definition_id, definition) = result.definitions.into_iter().next().unwrap();
        assert!(definition.definition.is_object());
        assert_eq!(
            r#"{"definitions":{"def1":{"a":"number"},"def2":{"b":"string"}}}"#,
            serde_json::to_string(&definition.definition).unwrap()
        );

        let (view_id, view) = result.views.into_iter().next().unwrap();
        assert_eq!(original_view_id, view_id);
        assert_eq!(r#"{ a: a }"#, view.jmespath);

        let schema_definition = result.schema_definitions.into_iter().next().unwrap();
        assert_eq!(schema_id, schema_definition.schema_id);
        assert_eq!(definition_id, schema_definition.definition_id);
        assert_eq!(Version::new(1, 0, 0), schema_definition.version);

        let schema_view = result.schema_views.into_iter().next().unwrap();
        assert_eq!(schema_id, schema_view.schema_id);
        assert_eq!(view_id, schema_view.view_id);

        Ok(())
    }

    #[test]
    fn get_schema_type() -> Result<()> {
        let db = SchemaDb {
            db: MemoryDatastore::default(),
        };
        let schema_id = db.add_schema(schema1(), None)?;

        let schema_type = db.get_schema_type(schema_id)?;
        assert_eq!(SchemaType::DocumentStorage, schema_type);

        Ok(())
    }

    #[test]
    fn update_schema_type() -> Result<()> {
        let db = SchemaDb {
            db: MemoryDatastore::default(),
        };
        let schema_id = db.add_schema(schema1(), None)?;

        let schema_type = db.get_schema_type(schema_id)?;
        assert_eq!(SchemaType::DocumentStorage, schema_type);

        db.update_schema_type(schema_id, SchemaType::Timeseries)?;

        let schema_type = db.get_schema_type(schema_id)?;
        assert_eq!(SchemaType::Timeseries, schema_type);

        Ok(())
    }

    fn schema1() -> NewSchema {
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
            insert_destination: "topic1".into(),
            query_address: "query1".into(),
            schema_type: SchemaType::DocumentStorage,
        }
    }

    fn view1() -> View {
        View {
            name: "view1".into(),
            jmespath: "{ a: a }".into(),
        }
    }

    fn schema2() -> NewSchema {
        NewSchema {
            name: "test2".into(),
            definition: json! ({
                "definitions": {
                    "def3": {
                        "a": "number"
                    },
                    "def4": {
                        "b": "string"
                    }
                }
            }),
            insert_destination: "topic2".into(),
            query_address: "query2".into(),
            schema_type: SchemaType::DocumentStorage,
        }
    }

    fn view2() -> View {
        View {
            name: "view2".into(),
            jmespath: "{ a: a }".into(),
        }
    }

    fn prepare_db_export() -> Result<(DbExport, Uuid, Uuid)> {
        // SchemaId, ViewId
        let db = SchemaDb {
            db: MemoryDatastore::default(),
        };

        let schema_id = db.add_schema(schema1(), None)?;

        let view_id = db.add_view_to_schema(schema_id, view1(), None)?;

        let exported = db.export_all()?;

        Ok((exported, schema_id, view_id))
    }
}
