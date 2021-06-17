use std::collections::HashMap;

use anyhow::{Context, Result};
use cdl_api::types::view::{MaterializedView, NewRelation};
use cdl_dto::materialization::{FieldDefinition, PostgresMaterializerOptions};
use lazy_static::lazy_static;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    GRAPHQL_ADDR, POSTGRES_INSERT_DESTINATION, POSTGRES_MATERIALIZER_ADDR, POSTGRES_QUERY_ADDR,
};

lazy_static! {
    static ref GRAPHQL_CLIENT: reqwest::Client = reqwest::Client::new();
}

pub async fn send_graphql_request(body: String) -> Result<Value> {
    println!("Sending request: {}", body);
    Ok(GRAPHQL_CLIENT
        .post(GRAPHQL_ADDR)
        .body(body)
        .send()
        .await?
        .json()
        .await?)
}

pub async fn add_schema(name: &str, query_addr: &str, insert_destination: &str) -> Result<Uuid> {
    let resp: Value = send_graphql_request(format!(r#"{{
            "operationName": "AddSchema",
            "variables": {{
                "sch": {{
                    "name": "{}",
                    "queryAddress": "{}",
                    "insertDestination": "{}",
                    "definition": {{}},
                    "type": "DOCUMENT_STORAGE"
                }}
            }},
            "query": "mutation AddSchema($sch: NewSchema!) {{\n  addSchema(new: $sch) {{\n    id\n  }}\n}}\n"
        }}"#, name, query_addr, insert_destination))
        .await?;

    let schema_id = resp["data"]["addSchema"]["id"]
        .as_str()
        .context("Failed to read return data")
        .map(Uuid::parse_str)??;
    println!("Added schema {}", schema_id);
    Ok(schema_id)
}

pub async fn add_view(
    schema_id: Uuid,
    name: &str,
    materializer_addr: &str,
    fields: HashMap<String, FieldDefinition>,
    materializer_options: Option<PostgresMaterializerOptions>,
    relations: &[NewRelation],
) -> Result<Uuid> {
    let materializer_options = if let Some(options) = materializer_options {
        serde_json::to_string(&options)?
    } else {
        "{}".to_owned()
    };
    let resp: Value = send_graphql_request(format!(r#"{{
            "operationName": "AddView",
            "variables": {{
                "sch": "{}",
                "newView": {{
                    "name": "{}",
                    "materializerAddress": "{}",
                    "fields": {},
                    "materializerOptions": {},
                    "relations": {}
                }}
            }},
            "query": "mutation AddView($sch: UUID!, $newView: NewView!) {{\n  addView(schemaId: $sch, newView: $newView) {{\n    id\n  }}\n}}\n"
        }}"#, schema_id, name, materializer_addr, serde_json::to_string(&fields)?,materializer_options ,serde_json::to_string(&relations)?))
        .await?;
    let view_id = resp["data"]["addView"]["id"]
        .as_str()
        .context("Failed to read return data")
        .map(Uuid::parse_str)??;
    println!("Added view {}", view_id);
    Ok(view_id)
}
pub async fn insert_message(object_id: Uuid, schema_id: Uuid, payload: &str) -> Result<()> {
    let resp: Value = send_graphql_request(format!(r#"{{
            "operationName": "InsertMessage",
            "variables": {{
                "msg": {{
                    "objectId": "{}",
                    "schemaId": "{}",
                    "payload": {}
                }}
            }},
            "query": "mutation InsertMessage($msg: InputMessage!) {{\n  insertMessage(message: $msg)\n}}\n"
        }}"#, object_id, schema_id, payload)).await?;
    let result = resp["data"]["insertMessage"].as_bool().unwrap();
    assert!(result);
    println!(
        "Added object obj_id: {} schema_id: {}",
        object_id, schema_id
    );
    Ok(())
}
pub async fn add_relation(parent_schema: Uuid, child_schema: Uuid) -> Result<Uuid> {
    let resp: Value = send_graphql_request(format!(r#"{{
            "operationName": "AddRelation",
            "variables": {{
                "parentSchema": "{}",
                "childSchema": "{}"
            }},
            "query": "mutation AddRelation($parentSchema: UUID!, $childSchema: UUID!) {{\n  addRelation(parentSchemaId: $parentSchema, childSchemaId: $childSchema)\n}}\n"
        }}"#, parent_schema, child_schema)).await?;
    let relation_id = resp["data"]["addRelation"]
        .as_str()
        .context("Failed to read return data")
        .map(Uuid::parse_str)??;
    println!("Added relation {} -> {}", parent_schema, child_schema);
    Ok(relation_id)
}
pub async fn add_edges(
    relation_id: Uuid,
    parent_object_id: Uuid,
    child_object_ids: &[Uuid],
) -> Result<()> {
    let resp: Value =send_graphql_request(format!(r#"{{
            "operationName": "AddEdges",
            "variables": {{
                "relations": [
                    {{
                        "relationId": "{}",
                        "parentObjectId": "{}",
                        "childObjectIds": {}
                    }}
                ]
            }},
            "query": "mutation AddEdges($relations: [ObjectRelations!]!) {{\n  addEdges(relations: $relations)\n}}\n"
        }}"#, relation_id, parent_object_id, serde_json::to_string(&child_object_ids)?)).await?;
    let result = resp["data"]["addEdges"].as_bool().unwrap();
    assert!(result);
    println!(
        "Added edges relation_id:{}, {} -> {:?}",
        relation_id, parent_object_id, child_object_ids
    );
    Ok(())
}

pub async fn materialize_view(view_id: Uuid, schema_id: Uuid) -> Result<MaterializedView> {
    let resp: Value = send_graphql_request(format!(r#"{{
            "operationName": "OnDemandRequest",
            "variables": {{
                "req": {{
                    "viewId": "{}",
                    "schemas": [
                        {{
                            "id": "{}",
                            "objectIds": []
                        }}
                    ]
                }}
            }},
            "query": "query OnDemandRequest($req: OnDemandViewRequest!) {{\n  onDemandView(request: $req) {{\n    id\n    rows {{\n      objectIds\n      fields\n    }}\n  }}\n}}\n"
        }}"#, view_id,schema_id)).await?;
    let result: MaterializedView = serde_json::from_value(resp["data"]["onDemandView"].clone())?;
    Ok(result)
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn general_api_compatibility_test() -> Result<()> {
    let schema_id1 = add_schema(
        "test_schema",
        POSTGRES_QUERY_ADDR,
        POSTGRES_INSERT_DESTINATION,
    )
    .await?;
    let schema_id2 = add_schema(
        "test_schema2",
        POSTGRES_QUERY_ADDR,
        POSTGRES_INSERT_DESTINATION,
    )
    .await?;
    let view_id = add_view(
        schema_id1,
        "test_view",
        POSTGRES_MATERIALIZER_ADDR,
        Default::default(),
        None,
        Default::default(),
    )
    .await?;
    let relation_id = add_relation(schema_id1, schema_id2).await?;

    let obj1_id = Uuid::new_v4();
    let obj2_id = Uuid::new_v4();
    insert_message(obj1_id, schema_id1, "{}").await?;
    insert_message(obj2_id, schema_id2, "{}").await?;

    add_edges(relation_id, obj1_id, &[obj2_id]).await?;

    materialize_view(view_id, schema_id1).await?;
    Ok(())
}
