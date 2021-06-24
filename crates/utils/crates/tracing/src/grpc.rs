use hyper::HeaderMap;
use opentelemetry::global;
use opentelemetry::propagation::Injector;
use opentelemetry_http::HeaderExtractor;
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// Method used with gRPC client to inject Span ID to gRPC metadata.
/// Used mostly in `rpc` crate in `connect()`.
/// # Example:
/// ```ignore
/// pub async fn connect(addr: String) -> Result<MyServiceClient<tonic::transport::Channel>, tonic::transport::Error> {
///     let conn = tonic::transport::Endpoint::new(addr)?.connect().await?;
///
///     Ok(MyServiceClient::with_interceptor(conn, tracing_utils::grpc::interceptor()))
/// }
/// ```
pub fn interceptor() -> tonic::Interceptor {
    tonic::Interceptor::new(|mut req| {
        global::get_text_map_propagator(|prop| {
            prop.inject_context(
                &tracing::Span::current().context(),
                &mut MetadataMap(req.metadata_mut()),
            )
        });
        Ok(req)
    })
}

/// Method used with gRPC server to set Span ID propagated via HTTP Header to every gRPC route.
/// # Example:
/// ```ignore
/// Server::builder()
///     .trace_fn(tracing_utils::grpc::trace_fn)
///     .add_service(MyServiceServer::new())
///     .serve(addr.into())
///     .await
/// ```
pub fn trace_fn(headers: &HeaderMap) -> tracing::Span {
    let span = tracing::info_span!("gRPC request", ?headers);
    let parent_cx = global::get_text_map_propagator(|prop| prop.extract(&HeaderExtractor(headers)));
    span.set_parent(parent_cx);
    span
}

struct MetadataMap<'a>(&'a mut tonic::metadata::MetadataMap);

impl<'a> Injector for MetadataMap<'a> {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(key) = tonic::metadata::MetadataKey::from_bytes(key.as_bytes()) {
            if let Ok(val) = tonic::metadata::MetadataValue::from_str(&value) {
                self.0.insert(key, val);
            }
        }
    }
}
