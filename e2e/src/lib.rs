#![cfg(all(test, feature = "e2e"))]
mod api;
mod on_demand_materializer;
mod postgres_materializer;

const POSTGRES_QUERY_ADDR: &str = "http://cdl-postgres-query-service:6400";
const POSTGRES_INSERT_DESTINATION: &str = "cdl.document.data";
const POSTGRES_MATERIALIZER_ADDR: &str = "http://cdl-postgres-materializer-general:6400";

const GRAPHQL_ADDR: &str = "http://cdl-api:6402/graphql";
