use anyhow::Context;
use bb8::{Pool, PooledConnection};
use reqwest::Client;
use rpc::query_service_ts::{
    query_service_ts_server::QueryServiceTs, Range, RawStatement, SchemaId, TimeSeries, ValueBytes,
};
use serde::Deserialize;
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

#[derive(Deserialize)]
struct RawStatementParameters {
    method: String,
    endpoint: String,
    queries: Vec<(String, String)>,
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
        method: &str,
        endpoint: &str,
        query: &T,
    ) -> Result<String, Status> {
        let conn = self.connect().await?;
        let url = format!("{}{}", self.addr, endpoint);

        let request = match method {
            "POST" => conn.post(&url).form(query),
            "GET" => conn.get(&url).query(query),
            m => {
                return Err(Status::internal(format!(
                    "Method \"{}\" not matched. Only POST and GET are currently supported",
                    m
                )))
            }
        };

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
impl QueryServiceTs for VictoriaQuery {
    #[tracing::instrument(skip(self))]
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

        let response: String = self.query_db("GET", "/query_range", &queries).await?;

        Ok(tonic::Response::new(TimeSeries {
            timeseries: response,
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn query_by_schema(
        &self,
        request: Request<SchemaId>,
    ) -> Result<Response<TimeSeries>, Status> {
        let query = [(
            "query",
            format!("{{__name__=~\"{}_.*\"}}", request.into_inner().schema_id),
        )];

        let response: String = self.query_db("GET", "/query", &query).await?;

        Ok(tonic::Response::new(TimeSeries {
            timeseries: response,
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn query_raw(
        &self,
        request: Request<RawStatement>,
    ) -> Result<Response<ValueBytes>, Status> {
        let params: RawStatementParameters =
            serde_json::from_str(&request.into_inner().raw_statement).map_err(|e| {
                Status::internal(format!("Did not deserialize raw_statement: {}", e))
            })?;

        let response = self
            .query_db(&params.method, &params.endpoint, &params.queries)
            .await?;

        Ok(tonic::Response::new(ValueBytes {
            value_bytes: response.as_bytes().to_vec(),
        }))
    }
}
