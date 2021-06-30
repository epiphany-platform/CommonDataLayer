use cache::DynamicCache;
use cdl_dto::materialization::FullView;
use plugins::{MaterializerPlugin, PostgresMaterializer};
use rpc::materializer_general::{general_materializer_server::GeneralMaterializer, Empty, Options};
use rpc::{common::RowDefinition, materializer_general::MaterializedView};
use serde::Serialize;
use settings_utils::PostgresSettings;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use utils::notification::{IntoSerialize, NotificationPublisher};
use uuid::Uuid;
use view::ViewSupplier;

mod plugins;
pub mod settings;
mod view;

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
    view_cache: Mutex<DynamicCache<Uuid, FullView>>,
}

impl MaterializerImpl {
    pub async fn new(
        args: PostgresSettings,
        notification_publisher: MaterializerNotificationPublisher,
        schema_registry_url: String,
        cache_size: usize,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            materializer: Arc::new(PostgresMaterializer::new(&args).await?),
            notification_publisher,
            view_cache: Mutex::new(DynamicCache::new(
                cache_size,
                ViewSupplier::boxed(schema_registry_url),
            )),
        })
    }
}

impl MaterializerImpl {
    fn validate_options_inner(&self, options: &str) -> anyhow::Result<()> {
        let options: serde_json::Value = serde_json::from_str(&options)?;
        self.materializer.validate_options(options)?;
        Ok(())
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
        let materialized_view = request.into_inner();
        tracing::debug!(?materialized_view, "materialized view");

        let error_handler = |err| {
            tracing::error!("Materialization error` {:?}", err);
            tonic::Status::internal(format!("{}", err))
        };

        let view_id = materialized_view
            .view_id
            .parse()
            .map_err(anyhow::Error::from)
            .map_err(error_handler)?;
        let mut view_cache = self.view_cache.lock().await;
        let view_definition = view_cache.get(view_id).await.map_err(error_handler)?;

        let publisher = self.notification_publisher.clone();
        let publisher = publisher.lock().await; // TODO: Should we have lock active for the whole time?
        let instance =
            NotificationPublisher::clone(&publisher).with_message_body(&materialized_view);

        self.materializer
            .upsert_view(materialized_view, view_definition.clone())
            .await
            .map_err(error_handler)?;
        if let Err(err) = instance.notify("success").await {
            tracing::error!("Failed to send notification {:?}", err)
        }
        Ok(tonic::Response::new(Empty {}))
    }
}
