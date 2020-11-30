use crate::schema::query_server::Query;
use crate::schema::{ObjectIds, RawMsg, SchemaId, ValueMap};
use anyhow::Context;
use bb8_postgres::bb8::{Pool, PooledConnection};
use bb8_postgres::tokio_postgres::config::Config as PgConfig;
use bb8_postgres::tokio_postgres::{types::ToSql, NoTls, Row};
use bb8_postgres::PostgresConnectionManager;
use std::collections::HashMap;
use structopt::StructOpt;
use tonic::{Request, Response, Status};
use utils::metrics::counter;

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
                let object_id = row.get::<usize, String>(0);
                let value = row.get::<usize, String>(1).into_bytes();
                (object_id, value)
            })
            .collect()
    }
}

#[tonic::async_trait]
impl Query for PsqlQuery {
    async fn query_multiple(
        &self,
        request: Request<ObjectIds>,
    ) -> Result<Response<ValueMap>, Status> {
        counter!("cdl.query-service.query-multiple.psql", 1);
        let rows = self
            .make_query(
                "SELECT object_id, payload FROM data d1 \
                WHERE object_id in $1 AND d1.version = \
                    (SELECT MAX(version) FROM data d2 \
                     WHERE d2.object_id = d1.object_id)",
                &[&request.into_inner().object_ids],
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
        counter!("cdl.query-service.query-by-schema.psql", 1);
        let rows = self
            .make_query(
                "SELECT object_id, payload FROM data d1 \
                WHERE schema_id = $1 AND d1.version = \
                    (SELECT MAX(version) FROM data d2 \
                     WHERE d2.object_id = d1.object_id)",
                &[&request.into_inner().schema_id],
            )
            .await?;

        Ok(tonic::Response::new(ValueMap {
            values: Self::collect_id_payload_rows(rows),
        }))
    }

    async fn query_raw(&self, request: Request<RawMsg>) -> Result<Response<ValueMap>, Status> {
        counter!("cdl.query-service.query_raw.psql", 1);
        let rows = self.make_query(&request.into_inner().raw_msg, &[]).await?;

        Ok(tonic::Response::new(ValueMap {
            values: Self::collect_id_payload_rows(rows),
        }))
    }
}
