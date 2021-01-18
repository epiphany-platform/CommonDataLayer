pub mod config;
pub mod context;
pub mod error;
pub mod queries;
pub mod schema;

use std::sync::Arc;

use config::Config;
use context::Context;
use structopt::StructOpt;
use warp::{http::Response, hyper::header::CONTENT_TYPE, hyper::Method, Filter};

#[tokio::main]
async fn main() {
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

    let state = warp::any().map({
        let config = config.clone();
        move || Context::new(config.clone())
    });

    let graphql_filter = juniper_warp::make_graphql_filter(crate::queries::schema(), state.boxed())
        .with(cors.clone());

    warp::serve(
        warp::get()
            .and(warp::path("graphiql"))
            .and(juniper_warp::graphiql_filter("/graphql", None).with(cors))
            .or(homepage)
            .or(warp::path("graphql").and(graphql_filter)),
    )
    .run(([0, 0, 0, 0], config.input_port))
    .await
}
