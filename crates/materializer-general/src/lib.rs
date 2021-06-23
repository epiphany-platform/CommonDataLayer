mod plugins;
pub mod settings;

use cdl_dto::materialization::FullView;
use lru_cache::LruCache;
use misc_utils::abort_on_poison;
use plugins::{MaterializerPlugin, PostgresMaterializer};
use rpc::materializer_general::{general_materializer_server::GeneralMaterializer, Empty, Options};
use rpc::schema_registry::Id;
use rpc::{common::RowDefinition, materializer_general::MaterializedView};
use serde::Serialize;
use settings_utils::PostgresSettings;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use utils::notification::{IntoSerialize, NotificationPublisher};
use uuid::Uuid;

#[derive(Serialize, Clone)]
pub struct MaterializationNotification {
    view_id: String,
    options: String,
    rows: Vec<MaterializationRow>,
}

#[derive(Serialize, Clone)]
struct MaterializationRow {
    object_ids: Vec<String>,
    fields: HashMap<String, String>,
}

impl From<RowDefinition> for MaterializationRow {
    fn from(row: RowDefinition) -> Self {
        Self {
            object_ids: row.object_ids,
            fields: row.fields,
        }
    }
}

impl IntoSerialize<MaterializationNotification> for MaterializedView {
    fn into_serialize(self) -> MaterializationNotification {
        MaterializationNotification {
            view_id: self.view_id,
            options: self.options.options,
            rows: self
                .rows
                .into_iter()
                .map(MaterializationRow::from)
                .collect(),
        }
    }
}

type MaterializerNotificationPublisher =
    Arc<Mutex<NotificationPublisher<MaterializedView, MaterializationNotification>>>;

pub struct MaterializerImpl {
    materializer: Arc<dyn MaterializerPlugin>,
    notification_publisher: MaterializerNotificationPublisher,
    schema_registry_addr: String,
    view_cache: std::sync::Mutex<LruCache<Uuid, FullView>>,
}

impl MaterializerImpl {
    pub async fn new(
        args: PostgresSettings,
        notification_publisher: MaterializerNotificationPublisher,
        schema_registry_addr: String,
        cache_size: usize,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            materializer: Arc::new(PostgresMaterializer::new(&args).await?),
            notification_publisher,
            schema_registry_addr,
            view_cache: std::sync::Mutex::new(LruCache::new(cache_size)),
        })
    }
}

impl MaterializerImpl {
    fn validate_options_inner(&self, options: &str) -> anyhow::Result<()> {
        let options: serde_json::Value = serde_json::from_str(&options)?;
        self.materializer.validate_options(options)?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_view_definition(&self, view_id: Uuid) -> anyhow::Result<FullView> {
        let view_definition = self
            .view_cache
            .lock()
            .unwrap_or_else(abort_on_poison)
            .get_mut(&view_id)
            .cloned();
        if let Some(dest) = view_definition {
            return Ok(dest);
        }

        let mut client =
            rpc::schema_registry::connect(self.schema_registry_addr.to_owned()).await?;
        let view_definition = client
            .get_view(Id {
                id: view_id.to_string(),
            })
            .await?
            .into_inner();

        let view_definition = FullView::from_rpc(view_definition)?;

        self.view_cache
            .lock()
            .unwrap_or_else(abort_on_poison)
            .insert(view_id, view_definition.clone());

        Ok(view_definition)
    }
}

#[tonic::async_trait]
impl GeneralMaterializer for MaterializerImpl {
    #[tracing::instrument(skip(self))]
    async fn validate_options(
        &self,
        request: tonic::Request<Options>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        let options: Options = request.into_inner();
        tracing::debug!(?options, "Options to validate");

        match self.validate_options_inner(&options.options) {
            Ok(_) => Ok(tonic::Response::new(Empty {})),
            Err(err) => Err(tonic::Status::invalid_argument(format!("{}", err))),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn upsert_view(
        &self,
        request: tonic::Request<MaterializedView>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        println!("test");
        let materialized_view = request.into_inner();
        tracing::debug!(?materialized_view, "materialized view");

        let err_map = |err| {
            tracing::error!("Materialization error` {:?}", err);
            tonic::Status::internal(format!("{}", err))
        };
        let err_map2 = |err: anyhow::Error| {
            tracing::error!("Materialization error` {:?}", err);
            tonic::Status::internal(format!("{}", err))
        };

        let view_id = materialized_view.view_id.parse().map_err(err_map)?;
        let view_definition = self.get_view_definition(view_id).await.map_err(err_map2)?;

        let publisher = self.notification_publisher.clone();
        let publisher = publisher.lock().await;
        let instance =
            NotificationPublisher::clone(&publisher).with_message_body(&materialized_view);

        self.materializer
            .upsert_view(materialized_view, view_definition)
            .await
            .map_err(err_map2)?;
        if let Err(err) = instance.notify("success").await {
            tracing::error!("Failed to send notification {:?}", err)
        }
        Ok(tonic::Response::new(Empty {}))
    }
}
