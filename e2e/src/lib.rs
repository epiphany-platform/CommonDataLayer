#[cfg(test)]
mod api;
#[cfg(test)]
mod object_builder;

#[cfg(test)]
mod postgres_materializer;

#[cfg(test)]
const POSTGRES_QUERY_ADDR: &str = "http://cdl-postgres-query-service:6400";
#[cfg(test)]
const POSTGRES_INSERT_DESTINATION: &str = "cdl.document.data";
#[cfg(test)]
const POSTGRES_MATERIALIZER_ADDR: &str = "http://cdl-postgres-materializer-general:6400";
#[cfg(test)]
const GRAPHQL_ADDR: &str = "http://cdl-api:6402/graphql";
