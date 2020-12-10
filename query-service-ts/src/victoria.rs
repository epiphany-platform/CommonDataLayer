use crate::schema::{query_server::Query, Range, SchemaId, TimeSeries};
use anyhow::Context;
use bb8::{Pool, PooledConnection};
use reqwest::Client;
use structopt::StructOpt;
use tonic::{Request, Response, Status};

#[derive(Debug, StructOpt)]
pub struct VictoriaConfig {
    #[structopt(long = "victoria-query-url", env = "VICTORIA_QUERY_URL")]
    victoria_url: String,
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
        })
    }

    async fn connect(&self) -> Result<PooledConnection<'_, VictoriaConnectionManager>, Status> {
        self.pool
            .get()
            .await
            .map_err(|err| Status::internal(format!("Unable to connect to database: {}", err)))
    }

    async fn query_db<T: serde::ser::Serialize + std::fmt::Debug>(
        &self,
        query: &T,
    ) -> Result<String, Status> {
        let conn = self.connect().await?;
        let request = conn.get(&self.addr).query(query);
        let response = request.send().await.map_err(|err| {
            Status::internal(format!(
                "Error requesting value from VictoriaMetrics: {}",
                err
            ))
        })?;

        let response_payload = response.text().await.map_err(|err| {
            Status::internal(format!(
                "Failed to deserialize response from VictoriaMetrics: {}",
                err
            ))
        })?;
        Ok(response_payload)
    }
}

#[tonic::async_trait]
impl Query for VictoriaQuery {
    async fn query_by_range(
        &self,
        request: Request<Range>,
    ) -> Result<Response<TimeSeries>, Status> {
        let request_payload = request.into_inner();
        let mut queries = Vec::new();
        queries.push((
            "query",
            format!("{{object_id=\"{}\"}}", request_payload.object_id),
        ));
        if !request_payload.start.is_empty() {
            queries.push(("start", request_payload.start));
        }
        if !request_payload.end.is_empty() {
            queries.push(("end", request_payload.end));
        }
        if !request_payload.step.is_empty() {
            queries.push(("step", request_payload.step));
        }

        let response: String = self.query_db(&queries).await?;
        Ok(tonic::Response::new(TimeSeries {
            timeseries: response,
        }))
    }

    async fn query_by_schema(
        &self,
        request: Request<SchemaId>,
    ) -> Result<Response<TimeSeries>, Status> {
        let query = ([("query", request.into_inner().schema_id)]);

        let response: String = self.query_db(&query).await?;
        Ok(tonic::Response::new(TimeSeries {
            timeseries: response,
        }))
    }
}
