use crate::schema::{query_server::Query, DataPoint, Range, SchemaId, TimeSeries};
use anyhow::Context;
use bb8::{Pool, PooledConnection};
use log::info;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::Value;
use structopt::StructOpt;
use tonic::{Request, Response, Status};

#[derive(Debug, StructOpt)]
pub struct VictoriaConfig {
    #[structopt(long = "victoria-query-url", env = "VICTORIA_QUERY_URL")]
    victoria_url: String,
    //#[structopt(long = "victoria-table-name", env = "VICTORIA_TABLE_NAME")]
    //victoria_table_name: String,
}

pub struct VictoriaConnectionManager;

#[tonic::async_trait]
impl bb8::ManageConnection for VictoriaConnectionManager {
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

pub struct VictoriaQuery {
    pool: Pool<VictoriaConnectionManager>,
    addr: String,
}

impl VictoriaQuery {
    pub async fn load(config: VictoriaConfig) -> anyhow::Result<Self> {
        let pool = Pool::builder()
            .build(VictoriaConnectionManager)
            .await
            .context("Failed to build connection pool")?;

        Ok(Self {
            pool,
            addr: config.victoria_url,
            //table_name: config.victoria_table_name,
        })
    }

    async fn connect(&self) -> Result<PooledConnection<'_, VictoriaConnectionManager>, Status> {
        self.pool
            .get()
            .await
            .map_err(|err| Status::internal(format!("Unable to connect to database: {}", err)))
    }

    async fn query_db<T: DeserializeOwned>(&self, query: &Value) -> Result<T, Status> {
        let conn = self.connect().await?;
        let request = conn.post(&self.addr).json(query);
        let response = request.send().await.map_err(|err| {
            Status::internal(format!(
                "Error requesting value from VictoriaMetrics: {}",
                err
            ))
        })?;

        response.json().await.map_err(|err| {
            Status::internal(format!(
                "Failed to deserialize response from VictoriaMetrics: {}",
                err
            ))
        })
    }
}

#[tonic::async_trait]
impl Query for VictoriaQuery {
    //TODO: IMPLEMENT ME!
    async fn query_by_range(
        &self,
        request: Request<Range>,
    ) -> Result<Response<TimeSeries>, Status> {
        info!("Victoria query_by_range: {:?}", request.get_ref());
        Ok(tonic::Response::new(TimeSeries {
            datapoints: vec![DataPoint {
                timestamp: "10:10:10".to_string(),
                value: 12.0,
            }],
        }))
    }

    //TODO: IMPLEMENT ME!
    async fn query_by_schema(&self, request: Request<SchemaId>) -> Result<Response<TimeSeries>, Status> {
        info!("Victoria query_by_schema: {:?}", request.get_ref());
        Ok(tonic::Response::new(TimeSeries {
            datapoints: vec![DataPoint {
                timestamp: "12:12:12".to_string(),
                value: 17.0,
            }],
        }))
    }
}
