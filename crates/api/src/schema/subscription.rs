use std::pin::Pin;

use futures::{Stream, TryStreamExt};
use juniper::{graphql_subscription, FieldError, FieldResult};

use crate::schema::context::Context;
use crate::{error::Result, types::report::Report};

type ReportStream = Pin<Box<dyn Stream<Item = FieldResult<Report>> + Send>>;

pub struct Subscription;

#[graphql_subscription(context = Context)]
impl Subscription {
    async fn reports(context: &Context) -> ReportStream {
        let source = &context.config().report_source;
        let stream = context
            .subscribe_on_communication_method(source)
            .await?
            .try_filter_map(|ev| async move { Ok(ev.payload) })
            .and_then(
                |payload| async move { serde_json::from_str(&payload).map_err(FieldError::from) },
            );

        Box::pin(stream)
    }
}
