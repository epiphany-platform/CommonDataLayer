use crate::schema::query_server::Query;
use crate::schema::{ObjectIds, RawStatement, SchemaId, ValueBytes, ValueMap};
use anyhow::Context;
use bb8_postgres::bb8::{Pool, PooledConnection};
use bb8_postgres::tokio_postgres::config::Config as PgConfig;
use bb8_postgres::tokio_postgres::{types::ToSql, NoTls, Row, SimpleQueryMessage};
use bb8_postgres::PostgresConnectionManager;
use log::trace;
use serde_json::Value;
use std::collections::HashMap;
use structopt::StructOpt;
use tonic::{Request, Response, Status};
use utils::metrics::counter;
use uuid::Uuid;

#[derive(Debug, StructOpt)]
pub struct PsqlConfig {
    #[structopt(long, env = "POSTGRES_USERNAME")]
    username: String,
    #[structopt(long, env = "POSTGRES_PASSWORD")]
    password: String,
    #[structopt(long, env = "POSTGRES_HOST")]
    host: String,
    #[structopt(long, env = "POSTGRES_PORT", default_value = "5432")]
    port: u16,
    #[structopt(long, env = "POSTGRES_DBNAME")]
    dbname: String,
}

pub struct PsqlQuery {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl PsqlQuery {
    pub async fn load(config: PsqlConfig) -> anyhow::Result<Self> {
        let mut pg_config = PgConfig::new();
        pg_config
            .user(&config.username)
            .password(&config.password)
            .host(&config.host)
            .port(config.port)
            .dbname(&config.dbname);
        let manager = PostgresConnectionManager::new(pg_config, NoTls);
        let pool = Pool::builder()
            .build(manager)
            .await
            .context("Failed to build connection pool")?;

        Ok(Self { pool })
    }

    async fn connect(
        &self,
    ) -> Result<PooledConnection<'_, PostgresConnectionManager<NoTls>>, Status> {
        self.pool
            .get()
            .await
            .map_err(|err| Status::internal(format!("Unable to connect to database: {}", err)))
    }

    async fn make_query(
        &self,
        query: &str,
        args: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<Row>, Status> {
        let connection = self.connect().await?;
        let statement = connection.prepare(query).await.map_err(|err| {
            Status::internal(format!("Unable to prepare query statement: {}", err))
        })?;

        connection
            .query(&statement, args)
            .await
            .map_err(|err| Status::internal(format!("Unable to query data: {}", err)))
    }

    fn collect_id_payload_rows(rows: Vec<Row>) -> HashMap<String, Vec<u8>> {
        rows.into_iter()
            .map(|row| {
                let object_id = row.get::<usize, Uuid>(0).to_string();
                let value = row.get::<usize, Value>(1).to_string().into_bytes();
                (object_id, value)
            })
            .collect()
    }

    fn collect_simple_query_messages(messages: Vec<SimpleQueryMessage>) -> Result<Vec<u8>, String> {
        Ok(messages
            .into_iter()
            .map(|msg| match msg {
                SimpleQueryMessage::Row(row) => {
                    let column_delimeter = '\t';
                    let row_delimeter = '\n';
                    let null_value = "\0";
                    let mut buffer = String::new();

                    for i in 0..row.len() {
                        buffer.push_str(
                            row.try_get(i)
                                .map_err(|e| {
                                    format!("Error getting data from row at column {}: {}", i, e)
                                })?
                                .unwrap_or(null_value),
                        );
                        buffer.push(column_delimeter);
                    }
                    buffer.push(row_delimeter);
                    Ok(buffer.into_bytes())
                }
                SimpleQueryMessage::CommandComplete(command) => {
                    Ok(command.to_string().into_bytes())
                }
                _ => Err("Could not match SimpleQueryMessage".to_string()),
            })
            .collect::<Result<Vec<Vec<u8>>, String>>()?
            .into_iter()
            .flatten()
            .collect())
    }
}

#[tonic::async_trait]
impl Query for PsqlQuery {
    async fn query_multiple(
        &self,
        request: Request<ObjectIds>,
    ) -> Result<Response<ValueMap>, Status> {
        let request = request.into_inner();

        trace!("QueryMultiple: {:?}", request);

        counter!("cdl.query-service.query-multiple.psql", 1);

        let object_ids: Vec<Uuid> = request
            .object_ids
            .into_iter()
            .map(|id| id.parse::<Uuid>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| Status::invalid_argument(err.to_string()))?;

        let rows = self
            .make_query(
                "SELECT d.object_id, d.payload \
                 FROM (\
                     SELECT object_id, max(version) as max \
                     FROM data \
                     WHERE object_id = ANY($1) \
                     GROUP BY object_id\
                 ) maxes \
                 JOIN data d \
                 ON d.object_id = maxes.object_id AND d.version = maxes.max",
                &[&object_ids.as_slice()],
            )
            .await?;

        Ok(tonic::Response::new(ValueMap {
            values: Self::collect_id_payload_rows(rows),
        }))
    }

    async fn query_by_schema(
        &self,
        request: Request<SchemaId>,
    ) -> Result<Response<ValueMap>, Status> {
        let request = request.into_inner();

        trace!("QueryBySchema: {:?}", request);

        counter!("cdl.query-service.query-by-schema.psql", 1);

        let schema_id = request
            .schema_id
            .parse::<Uuid>()
            .map_err(|err| Status::invalid_argument(err.to_string()))?;

        let rows = self
            .make_query(
                "SELECT object_id, payload \
                 FROM data d1 \
                 WHERE schema_id = $1 AND d1.version = (\
                     SELECT MAX(version) \
                     FROM data d2 \
                     WHERE d2.object_id = d1.object_id\
                 )",
                &[&schema_id],
            )
            .await?;

        Ok(tonic::Response::new(ValueMap {
            values: Self::collect_id_payload_rows(rows),
        }))
    }

    async fn query_raw(
        &self,
        request: Request<RawStatement>,
    ) -> Result<Response<ValueBytes>, Status> {
        counter!("cdl.query-service.query_raw.psql", 1);
        let connection = self.connect().await?;
        let messages = connection
            .simple_query(request.into_inner().raw_statement.as_str())
            .await
            .map_err(|err| Status::internal(format!("Unable to query_raw data: {}", err)))?;

        Ok(tonic::Response::new(ValueBytes {
            value_bytes: Self::collect_simple_query_messages(messages).map_err(|err| {
                Status::internal(format!(
                    "Unable to collect simple query messages data: {}",
                    err
                ))
            })?,
        }))
    }
}
