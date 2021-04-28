use futures_util::TryFuture;
use hyper::{Body, Request};
use opentelemetry::global;
use opentelemetry_http::{HeaderExtractor, HeaderInjector};
use reqwest::RequestBuilder;
use std::convert::Infallible;
use tower_service::Service;
use tracing::Instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// Wrapper for `warp::serve` used to set Span ID propagated to HTTP Request to every defined route.
pub async fn serve<R>(routes: R, addr: ([u8; 4], u16))
where
    R: warp::Filter + Clone + Send + Sync + 'static,
    <R::Future as TryFuture>::Ok: warp::Reply,
{
    let service = warp::service(routes);

    let make_svc = hyper::service::make_service_fn(move |_| {
        let warp_svc = service.clone();
        async move {
            let svc = hyper::service::service_fn(move |req: Request<Body>| {
                let mut warp_svc = warp_svc.clone();
                let span = tracing::info_span!("handle request", ?req);
                async move {
                    let parent_cx = global::get_text_map_propagator(|prop| {
                        prop.extract(&HeaderExtractor(req.headers()))
                    });
                    tracing::Span::current().set_parent(parent_cx);
                    let resp = warp_svc.call(req).await;
                    resp
                }
                .instrument(span)
            });
            Ok::<_, Infallible>(svc)
        }
    });

    hyper::Server::bind(&addr.into())
        .serve(make_svc)
        .await
        .unwrap();
}

/// Extension trait used with reqwest::Client to inject Span ID to HTTP Headers
pub trait RequestBuilderTracingExt {
    /// Injects SpanID to HTTP Headers
    fn inject_span(self) -> Self;
}

impl RequestBuilderTracingExt for RequestBuilder {
    fn inject_span(self) -> Self {
        let mut inner = Request::new(());
        global::get_text_map_propagator(|prop| {
            prop.inject_context(
                &tracing::Span::current().context(),
                &mut HeaderInjector(&mut inner.headers_mut()),
            )
        });
        let headers = inner.headers().clone();
        self.headers(headers)
    }
}
