use std::{collections::HashMap, convert::TryFrom, convert::TryInto};

use super::MaterializerPlugin;
use bb8_postgres::tokio_postgres::{types::Type, Config, NoTls};
use bb8_postgres::{bb8, PostgresConnectionManager};
use bb8_postgres::{
    bb8::{Pool, PooledConnection},
    tokio_postgres::{binary_copy::BinaryCopyInWriter, types::ToSql},
};
use cdl_dto::materialization::PostgresMaterializerOptions;
use futures::pin_mut;
use itertools::Itertools;
use metrics_utils::{self as metrics, counter};
use rpc::materializer_general::MaterializedView;
use serde_json::Value;
use settings_utils::PostgresSettings;
use uuid::Uuid;

pub struct PostgresMaterializer {
    pool: Pool<PostgresConnectionManager<NoTls>>,
    schema: String,
}

#[derive(Debug)]
struct PsqlView {
    options: PostgresMaterializerOptions,
    rows: Vec<RowDefinition>,
}

#[derive(Debug)]
struct RowDefinition {
    object_ids: Vec<Uuid>,
    fields: HashMap<String, Value>,
}

impl TryFrom<MaterializedView> for PsqlView {
    type Error = anyhow::Error;

    fn try_from(view: MaterializedView) -> Result<Self, Self::Error> {
        let options = serde_json::from_str(&view.options.options)?;

        let rows = view
            .rows
            .into_iter()
            .map(|row| {
                let object_ids = row
                    .object_ids
                    .into_iter()
                    .map(|oid| oid.parse())
                    .collect::<Result<_, _>>()?;
                let fields = row
                    .fields
                    .into_iter()
                    .map(|(key, field)| {
                        let field = serde_json::from_str(&field)?;
                        Ok((key, field))
                    })
                    .collect::<anyhow::Result<_>>()?;
                Ok(RowDefinition { object_ids, fields })
            })
            .collect::<anyhow::Result<_>>()?;

        Ok(PsqlView { options, rows })
    }
}

#[async_trait::async_trait]
impl MaterializerPlugin for PostgresMaterializer {
    fn validate_options(&self, _options: Value) -> anyhow::Result<()> {
        // TODO: Validate if options have proper structure
        // TODO: Validate if table contains only ASCII letters (prevent sql injection)
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn upsert_view(&self, view: MaterializedView) -> anyhow::Result<()> {
        counter!("cdl.materializer.postgres.upsert-materialized-view", 1);

        let psql_view: PsqlView = view.try_into()?;
        tracing::trace!(?psql_view, "PSQL View");

        let mut conn = self.connect().await?;

        //TODO: Validate if column names are only in ASCII letters and numbers (prevent sql injection)

        if let Some((copy_stm, insert_stm, types, columns)) = self.build_query(&psql_view).await? {
            let tx = conn.transaction().await?;
            // Temporary table is unique per session
            tx.batch_execute(&format!(
                "CREATE TABLE IF NOT EXISTS {table} ( \
                    object_ids UUID[] NOT NULL,\
                    {columns},\
                    PRIMARY KEY (object_ids)
                 );\
                 CREATE TEMP TABLE upserts ON COMMIT DROP \
                 AS TABLE {table} WITH NO DATA;",
                table = psql_view.options.table,
                columns = columns
            ))
            .await?;
            let sink = tx.copy_in(copy_stm.as_str()).await?;

            let writer = BinaryCopyInWriter::new(sink, &types); // Batch insert

            let num_written = self.write(writer, &psql_view).await?;

            tx.batch_execute(&insert_stm).await?;

            let store_result = tx.commit().await;
            tracing::trace!("PSQL `UPSERT` {:?}", store_result);
            store_result?;

            counter!("cdl.materializer.postgres.store", num_written as u64);
        } else {
            tracing::warn!("Materialized view is empty, skipping upserting");
        }

        Ok(())
    }
}

impl PostgresMaterializer {
    async fn write(&self, writer: BinaryCopyInWriter, view: &PsqlView) -> anyhow::Result<usize> {
        pin_mut!(writer);

        let mut row: Vec<&'_ (dyn ToSql + Sync)> = Vec::with_capacity(view.rows.len());
        for m in view.rows.iter() {
            row.clear();
            row.push(&m.object_ids);
            row.extend(
                m.fields
                    .iter()
                    .map(|(_, field)| field as &(dyn ToSql + Sync)),
            );
            writer.as_mut().write(&row).await?;
        }
        writer.finish().await?;

        Ok(view.rows.len())
    }

    async fn build_query(
        &self,
        view: &PsqlView,
    ) -> anyhow::Result<Option<(String, String, Vec<Type>, String)>> {
        let table = &view.options.table;

        let first_row = view.rows.get(0);

        match first_row {
            Some(first_row) => {
                let columns = first_row.fields.keys().join(", ");
                let update_columns = first_row
                    .fields
                    .keys()
                    .map(|f| format!("{field} = EXCLUDED.{field}", field = f))
                    .join(", ");
                let insert_stm = format!(
                    "INSERT INTO {} \
                     SELECT * FROM upserts \
                     ON CONFLICT (object_ids) DO UPDATE SET {}",
                    table, update_columns
                );
                let copy_stm = format!("COPY upserts (object_ids, {}) FROM STDIN BINARY", columns);
                let mut types = vec![Type::UUID_ARRAY];
                // TODO: For now each column is stored as a JSON field.
                // Later we can introduce some kind of type infering mechanism here, so each field in
                // Materialized view would be stored with better column type.
                // We can achieve that by matching `Value` variants (leaving `JSON` for complex objects like arrays or structs).
                types.extend(first_row.fields.keys().map(|_| Type::JSON));

                let columns = first_row
                    .fields
                    .keys()
                    .map(|f| format!("{} JSON NOT NULL", f))
                    .join(", ");

                tracing::debug!(?insert_stm, ?copy_stm, ?types, "Build query");
                Ok(Some((copy_stm, insert_stm, types, columns)))
            }
            None => Ok(None),
        }
    }

    pub async fn new(args: &PostgresSettings) -> anyhow::Result<Self> {
        let mut pg_config = Config::new();
        pg_config
            .user(&args.username)
            .password(&args.password)
            .host(&args.host)
            .port(args.port)
            .dbname(&args.dbname);

        let manager = PostgresConnectionManager::new(pg_config, NoTls);
        let pool = bb8::Pool::builder()
            .max_size(20)
            .connection_timeout(std::time::Duration::from_secs(30))
            .build(manager)
            .await?;

        Ok(Self {
            pool,
            schema: args.schema.clone(),
        })
    }
}

impl PostgresMaterializer {
    async fn set_schema(
        &self,
        conn: &PooledConnection<'_, PostgresConnectionManager<NoTls>>,
    ) -> anyhow::Result<()> {
        conn.execute(
            format!("SET search_path TO '{}'", &self.schema).as_str(),
            &[],
        )
        .await?;

        Ok(())
    }

    async fn connect(
        &self,
    ) -> anyhow::Result<PooledConnection<'_, PostgresConnectionManager<NoTls>>> {
        let conn = self.pool.get().await?;

        self.set_schema(&conn).await?;

        Ok(conn)
    }
}
