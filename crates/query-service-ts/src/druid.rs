use anyhow::Context;
use bb8::{Pool, PooledConnection};
use reqwest::Client;
use rpc::query_service_ts::{
    query_service_ts_server::QueryServiceTs, Range, RawStatement, SchemaId, TimeSeries, ValueBytes,
};
use serde_json::{json, Value};
use structopt::StructOpt;
use tonic::{Request, Response, Status};
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

    async fn query_db(&self, query: &Value) -> Result<String, Status> {
        let conn = self.connect().await?;
        let request = conn.post(&self.addr).json(query);
        let response = request.send().await.map_err(|err| {
            Status::internal(format!("Error requesting value from druid: {}", err))
        })?;

        response.text().await.map_err(|err| {
            Status::internal(format!(
                "Failed to deserialize response from Druid: {}",
                err
            ))
        })
    }

    async fn query_raw(&self, query: String) -> Result<Vec<u8>, Status> {
        let conn = self.connect().await?;
        let request = conn.post(&self.addr).body(query);
        let response = request.send().await.map_err(|err| {
            Status::internal(format!("Error when querying DRUID with raw query: {}", err))
        })?;

        response
            .bytes()
            .await
            .map_err(|err| Status::internal(format!("Couldn't retrieve respones bytes: {}", err)))
            .map(|bytes| bytes.to_vec())
    }
}

#[tonic::async_trait]
impl QueryServiceTs for DruidQuery {
    #[tracing::instrument(skip(self))]
    async fn query_by_range(
        &self,
        request: Request<Range>,
    ) -> Result<Response<TimeSeries>, Status> {
        counter!("cdl.query-service.query-by-range.druid", 1);

        let request = request.into_inner();

        let intervals = format!("{}/{}", request.start, request.end);

        let query = json!({
            "queryType": "scan",
            "dataSource": &self.table_name,
            "columns": [],
            "filter": {
                "type": "and",
                "fields": [
                    { "type": "selector", "dimension": "object_id", "value": &request.object_id },
                    { "type": "selector", "dimension": "schema_id", "value": &request.schema_id },
                ]
            },
            "granularity": &request.step,
            "intervals": [ intervals ]
        });

        Ok(tonic::Response::new(TimeSeries {
            timeseries: self.query_db(&query).await?,
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn query_by_schema(
        &self,
        request: Request<SchemaId>,
    ) -> Result<Response<TimeSeries>, Status> {
        counter!("cdl.query-service.query-by-schema.druid", 1);

        let request = request.into_inner();

        let query = json!({
            "queryType": "scan",
            "dataSource": &self.table_name,
            "columns": [],
            "filter": {
                "type": "selector",
                "dimension": "schema_id",
                "value": &request.schema_id
            },
            "granularity": "none",
            "intervals": [ "2000-01-01T00:00Z/3000-01-01T00:00Z" ]
        });

        Ok(tonic::Response::new(TimeSeries {
            timeseries: self.query_db(&query).await?,
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn query_raw(
        &self,
        request: Request<RawStatement>,
    ) -> Result<Response<ValueBytes>, Status> {
        counter!("cdl.query-service.query-raw.druid", 1);

        let request = request.into_inner();

        Ok(tonic::Response::new(ValueBytes {
            value_bytes: self.query_raw(request.raw_statement).await?,
        }))
    }
}
