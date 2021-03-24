use std::pin::Pin;

use futures::{Stream, TryStreamExt};
use juniper::{graphql_subscription, FieldError, FieldResult};
use tracing::Instrument;

use crate::schema::context::Context;
use crate::{error::Result, types::report::Report};

type ReportStream = Pin<Box<dyn Stream<Item = FieldResult<Report>> + Send>>;

pub struct Subscription;

#[graphql_subscription(context = Context)]
impl Subscription {
    async fn reports(context: &Context) -> ReportStream {
        let span = tracing::trace_span!("subscribe_reports");
        Self::reports_inner(context).instrument(span).await?
    }
}

impl Subscription {
    async fn reports_inner(context: &Context) -> anyhow::Result<ReportStream> {
        let source = &context.config().report_source;
        match source {
            Some(source) => {
                let stream = context
                    .subscribe_on_communication_method(source)
                    .await?
                    .try_filter_map(|ev| async move { Ok(ev.payload) })
                    .and_then(|payload| async move {
                        serde_json::from_str(&payload).map_err(FieldError::from)
                    });

                Ok(Box::pin(stream))
            }
            None => {
                anyhow::bail!(
                    "Reporting is disabled. Use Kafka or AMQP communication method to enable it."
                );
            }
        }
    }
}
