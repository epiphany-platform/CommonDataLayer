use anyhow::Context;
use bb8::{Pool, PooledConnection};
use reqwest::Client;
use rpc::query_service::{
    query_service_server::QueryService, ObjectIds, RawStatement, SchemaId, ValueBytes, ValueMap,
};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{json, Value};
use structopt::StructOpt;
use tonic::{Code, Request, Response, Status};
use utils::metrics::counter;

#[derive(Debug, StructOpt)]
pub struct DruidConfig {
    #[structopt(long = "druid-query-url", env = "DRUID_QUERY_URL")]
    druid_url: String,
    #[structopt(long = "druid-table-name", env = "DRUID_TABLE_NAME")]
    druid_table_name: String,
}

pub struct DruidConnectionManager;

#[tonic::async_trait]
impl bb8::ManageConnection for DruidConnectionManager {
    type Connection = Client;
    type Error = reqwest::Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        Ok(Client::new())
    }

    async fn is_valid(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

pub struct DruidQuery {
    pool: Pool<DruidConnectionManager>,
    addr: String,
    table_name: String,
}

#[derive(Deserialize)]
pub struct DruidValue {
    pub timestamp: String,
    pub result: DruidInnerValue,
}

#[derive(Deserialize)]
pub struct DruidInnerValue {
    pub object_id: String,
    pub data: String,
}

impl DruidQuery {
    pub async fn load(config: DruidConfig) -> anyhow::Result<Self> {
        let pool = Pool::builder()
            .build(DruidConnectionManager)
            .await
            .context("Failed to build connection pool")?;

        Ok(Self {
            pool,
            addr: config.druid_url,
            table_name: config.druid_table_name,
        })
    }

    async fn connect(&self) -> Result<PooledConnection<'_, DruidConnectionManager>, Status> {
        self.pool
            .get()
            .await
            .map_err(|err| Status::internal(format!("Unable to connect to database: {}", err)))
    }

    async fn query_db<T: DeserializeOwned>(&self, query: &Value) -> Result<T, Status> {
        let conn = self.connect().await?;
        let request = conn.post(&self.addr).json(query);
        let response = request.send().await.map_err(|err| {
            Status::internal(format!("Error requesting value from druid: {}", err))
        })?;

        response.json().await.map_err(|err| {
            Status::internal(format!(
                "Failed to deserialize response from Druid: {}",
                err
            ))
        })
    }
}

#[tonic::async_trait]
impl QueryService for DruidQuery {
    async fn query_multiple(
        &self,
        request: Request<ObjectIds>,
    ) -> Result<Response<ValueMap>, Status> {
        counter!("cdl.query-service.query-multiple.druid", 1);
        let query = json!({
            "queryType": "timeseries",
            "dataSource": &self.table_name,
            "granularity": "all",
            "filter": {
                "type": "in",
                "dimension": "objectId",
                "values": request.into_inner().object_ids
            },
            "aggregations": [
                { "type": "stringLast", "name": "object_id", "fieldName": "objectId" },
                { "type": "stringLast", "name": "data", "fieldName": "data" }
            ],
            "intervals": [ "2000-01-01T00:00:00.000/3000-01-01T00:00:00.000" ]
        });

        let response: Vec<DruidValue> = self.query_db(&query).await?;
        let values = response
            .into_iter()
            .map(|val| (val.result.object_id, val.result.data.into_bytes()))
            .collect();

        Ok(tonic::Response::new(ValueMap { values }))
    }

    async fn query_by_schema(
        &self,
        request: Request<SchemaId>,
    ) -> Result<Response<ValueMap>, Status> {
        counter!("cdl.query-service.query-by-schema.druid", 1);
        let query = json!({
            "queryType": "timeseries",
            "dataSource": &self.table_name,
            "granularity": "all",
            "filter": {
                "type": "selector",
                "dimension": "schemaId",
                "value": request.into_inner().schema_id
            },
            "aggregations": [
                { "type": "stringLast", "name": "object_id", "fieldName": "objectId" },
                { "type": "stringLast", "name": "data", "fieldName": "data" }
            ],
            "intervals": [ "2000-01-01T00:00:00.000/3000-01-01T00:00:00.000" ]
        });

        let response: Vec<DruidValue> = self.query_db(&query).await?;
        let values = response
            .into_iter()
            .map(|val| (val.result.object_id, val.result.data.into_bytes()))
            .collect();

        Ok(tonic::Response::new(ValueMap { values }))
    }

    async fn query_raw(
        &self,
        _request: Request<RawStatement>,
    ) -> Result<Response<ValueBytes>, Status> {
        counter!("cdl.query-service.query-raw.druid", 1);

        Err(Status::new(
            Code::Unimplemented,
            "query-service-druid does not support RAW requests yet",
        ))
    }
}
