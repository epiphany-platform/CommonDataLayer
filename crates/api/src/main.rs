pub mod config;
pub mod error;
pub mod events;
pub mod schema;
pub mod types;

use std::sync::Arc;

use config::Config;
use futures::FutureExt;
use schema::context::Context;
use structopt::StructOpt;
use warp::{http::Response, hyper::header::CONTENT_TYPE, hyper::Method, Filter};

const SEC_WEBSOCKET_PROTOCOL_NAME: &str = "Sec-WebSocket-Protocol";
const SEC_WEBSOCKET_PROTOCOL_VALUE: &str = "graphql-ws";

#[tokio::main]
async fn main() {
    utils::set_aborting_panic_hook();

    env_logger::init();
    let config = Arc::new(Config::from_args());

    let cors = warp::cors()
        .allow_methods(&[Method::POST, Method::OPTIONS])
        .allow_headers(&[CONTENT_TYPE])
        .allow_any_origin();

    let homepage = warp::path::end()
        .map(|| {
            Response::builder()
                .header("content-type", "text/html")
                .body(
                "<html><h1>juniper_warp</h1><div>visit <a href=\"/graphiql\">/graphiql</a></html>"
                    .to_string(),
            )
        })
        .with(cors.clone());

    let context = Context::new(config.clone());

    let state = warp::any().map({
        let context = context.clone();
        move || context.clone()
    });

    let graphql_filter = juniper_warp::make_graphql_filter(crate::schema::schema(), state.boxed())
        .with(cors.clone());

    let root_node = Arc::new(crate::schema::schema());

    let subscriptions = warp::path("subscriptions")
        .and(warp::ws())
        .map({
            let context = context.clone();
            move |ws: warp::ws::Ws| {
                let root_node = root_node.clone();
                let context = context.clone();
                ws.on_upgrade(move |websocket| async move {
                    juniper_warp::subscriptions::serve_graphql_ws(
                        websocket,
                        root_node,
                        juniper_graphql_ws::ConnectionConfig::new(context.clone()),
                    )
                    .map(|r| {
                        if let Err(e) = r {
                            log::error!("Websocket error: {}", e);
                        }
                    })
                    .await
                })
            }
        })
        .map(|reply| {
            warp::reply::with_header(
                reply,
                SEC_WEBSOCKET_PROTOCOL_NAME,
                SEC_WEBSOCKET_PROTOCOL_VALUE,
            )
        });

    warp::serve(
        warp::get()
            .and(warp::path("graphiql"))
            .and(juniper_warp::graphiql_filter("/graphql", Some("/subscriptions")).with(cors))
            .or(homepage)
            .or(warp::path("graphql").and(graphql_filter))
            .or(subscriptions),
    )
    .run(([0, 0, 0, 0], config.input_port))
    .await
}
