use std::collections::HashMap;

use opentelemetry::global;
use opentelemetry::propagation::{Extractor, Injector};
use rdkafka::{
    message::{BorrowedHeaders, BorrowedMessage, Headers, OwnedHeaders},
    Message,
};
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// Method used in kafka [parallel]_consumer handler to set Span ID propagated by kafka header
pub fn set_parent_span(message: &BorrowedMessage<'_>) {
    let parent_cx = global::get_text_map_propagator(|prop| {
        let headers = message.headers();
        prop.extract(&KafkaConsumerHeaders::new(headers))
    });
    tracing::Span::current().set_parent(parent_cx);
}

/// Method used with kafka publisher to inject Span ID to header
pub fn inject_span(headers: OwnedHeaders) -> OwnedHeaders {
    let mut headers = KafkaProducerHeaders::new(headers);
    global::get_text_map_propagator(|prop| {
        prop.inject_context(&tracing::Span::current().context(), &mut headers);
    });
    headers.headers
}

struct KafkaProducerHeaders {
    headers: OwnedHeaders,
}

impl KafkaProducerHeaders {
    fn new(headers: OwnedHeaders) -> Self {
        Self { headers }
    }
}

impl Injector for KafkaProducerHeaders {
    fn set(&mut self, key: &str, value: String) {
        let headers = std::mem::take(&mut self.headers);
        self.headers = headers.add(key, &value);
    }
}

struct KafkaConsumerHeaders<'a> {
    headers: HashMap<&'a str, &'a [u8]>,
}

impl<'a> KafkaConsumerHeaders<'a> {
    fn new(headers: Option<&'a BorrowedHeaders>) -> Self {
        let mut map = HashMap::new();
        if let Some(headers) = headers {
            for i in 0..headers.count() {
                if let Some((key, value)) = headers.get(i) {
                    map.insert(key, value);
                }
            }
        }
        Self { headers: map }
    }
}

impl<'a> Extractor for KafkaConsumerHeaders<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.headers
            .get(key)
            .and_then(|m| std::str::from_utf8(m).ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.headers.keys().copied().collect()
    }
}
