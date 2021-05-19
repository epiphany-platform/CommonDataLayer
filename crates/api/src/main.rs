pub mod error;
pub mod events;
pub mod schema;
pub mod settings;
pub mod types;

use std::convert::Infallible;

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::Schema;
use async_graphql_warp::{graphql_subscription, Response};
use warp::{http::Response as HttpResponse, hyper::header::CONTENT_TYPE, hyper::Method, Filter};

use crate::schema::context::EdgeRegistryConnectionManager;
use schema::context::{
    MQEvents, OnDemandMaterializerConnectionManager, SchemaRegistryConnectionManager,
};
use schema::{mutation::MutationRoot, query::QueryRoot, subscription::SubscriptionRoot};
use settings::Settings;
use utils::settings::load_settings;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();

    let settings: Settings = load_settings()?;
    tracing_tools::init(settings.log.rust_log.as_str(), "web-api")?;

    tracing::debug!(?settings, "application environment");

    let input_port = settings.input_port;

    let cors = warp::cors()
        .allow_methods(&[Method::POST, Method::GET, Method::OPTIONS])
        .allow_headers(&[CONTENT_TYPE])
        .allow_any_origin();

    let sr_pool = bb8::Pool::builder()
        .build(SchemaRegistryConnectionManager {
            address: settings.services.schema_registry_url.clone(),
        })
        .await
        .unwrap();

    let er_pool = bb8::Pool::builder()
        .build(EdgeRegistryConnectionManager {
            address: settings.services.edge_registry_url.clone(),
        })
        .await
        .unwrap();

    let odm_pool = bb8::Pool::builder()
        .build(OnDemandMaterializerConnectionManager {
            address: settings.services.on_demand_materializer_url.clone(),
        })
        .await
        .unwrap();

    let schema = Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .data(settings)
        .data(sr_pool)
        .data(er_pool)
        .data(odm_pool)
        .data(MQEvents {
            events: Default::default(),
        })
        .finish();

    let graphql_post = async_graphql_warp::graphql(schema.clone()).and_then(
        |(schema, request): (
            Schema<QueryRoot, MutationRoot, SubscriptionRoot>,
            async_graphql::Request,
        )| async move { Ok::<_, Infallible>(Response::from(schema.execute(request).await)) },
    );

    let graphql_playground = warp::path!("graphiql").and(warp::get()).map(|| {
        HttpResponse::builder()
            .header("content-type", "text/html")
            .body(playground_source(
                GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/subscriptions"),
            ))
    });

    let routes = warp::path!("subscriptions")
        .and(graphql_subscription(schema))
        .or(graphql_playground)
        .or(warp::path!("graphql").and(graphql_post).with(cors));

    tracing_tools::http::serve(routes, ([0, 0, 0, 0], input_port)).await;

    Ok(())
}
