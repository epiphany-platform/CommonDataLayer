use opentelemetry::global;
use opentelemetry::propagation::{Extractor, Injector};
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// Method used with gRPC client to inject Span ID to gRPC metadata
pub fn inject_span<T: std::fmt::Debug>(t: T) -> tonic::Request<T> {
    let mut request = tonic::Request::new(t);
    global::get_text_map_propagator(|prop| {
        prop.inject_context(
            &tracing::Span::current().context(),
            &mut ClientMetadataMap(request.metadata_mut()),
        )
    });
    request
}

/// Method used in gRPC server handler to set Span ID propagated by gRPC metadata
pub fn set_parent_span<T>(request: &tonic::Request<T>) {
    let parent_cx = global::get_text_map_propagator(|prop| {
        prop.extract(&ServerMetadataMap(request.metadata()))
    });
    tracing::Span::current().set_parent(parent_cx);
}

struct ServerMetadataMap<'a>(&'a tonic::metadata::MetadataMap);
struct ClientMetadataMap<'a>(&'a mut tonic::metadata::MetadataMap);

impl<'a> Extractor for ServerMetadataMap<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|m| m.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0
            .keys()
            .map(|k| match k {
                tonic::metadata::KeyRef::Ascii(v) => v.as_str(),
                tonic::metadata::KeyRef::Binary(v) => v.as_str(),
            })
            .collect::<Vec<_>>()
    }
}

impl<'a> Injector for ClientMetadataMap<'a> {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(key) = tonic::metadata::MetadataKey::from_bytes(key.as_bytes()) {
            if let Ok(val) = tonic::metadata::MetadataValue::from_str(&value) {
                self.0.insert(key, val);
            }
        }
    }
}
