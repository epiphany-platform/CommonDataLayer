#[cfg(all(test, feature = "e2e"))]
mod api;
#[cfg(all(test, feature = "e2e"))]
mod object_builder;

#[cfg(all(test, feature = "e2e"))]
const POSTGRES_QUERY_ADDR: &str = "http://cdl-postgres-query-service:6400";
#[cfg(all(test, feature = "e2e"))]
const POSTGRES_INSERT_DESTINATION: &str = "cdl.document.data";
#[cfg(all(test, feature = "e2e"))]
const POSTGRES_MATERIALIZER_ADDR: &str = "http://cdl-postgres-materializer-general:6400";
#[cfg(all(test, feature = "e2e"))]
const GRAPHQL_ADDR: &str = "http://cdl-api:6402/graphql";
