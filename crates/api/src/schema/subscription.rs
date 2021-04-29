use std::pin::Pin;

use async_graphql::{Context, FieldError, FieldResult, Subscription};
use futures::{Stream, TryStreamExt};
use tracing::Instrument;

use crate::settings::Settings;
use crate::schema::context::MQEvents;
use crate::types::report::Report;

type ReportStream = Pin<Box<dyn Stream<Item = FieldResult<Report>> + Send>>;

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn reports(&self, context: &Context<'_>) -> FieldResult<ReportStream> {
        let span = tracing::info_span!("subscribe_reports");
        reports_inner(context).instrument(span).await
    }
}

async fn reports_inner(context: &Context<'_>) -> FieldResult<ReportStream> {
    let settings = &context.data_unchecked::<Settings>();

    match settings.notification_consumer {
        Some(ref notifications) => {
            let stream = context
                .data_unchecked::<MQEvents>()
                .subscribe_on_communication_method(&notifications.source, settings)
                .await?
                .try_filter_map(|ev| async move { Ok(ev.payload) })
                .and_then(|payload| async move {
                    serde_json::from_str(&payload).map_err(FieldError::from)
                });

            Ok(Box::pin(stream))
        }
        None => Err(FieldError::new(
            "Reporting is disabled. Use Kafka or AMQP communication method to enable it.",
        )),
    }
}
