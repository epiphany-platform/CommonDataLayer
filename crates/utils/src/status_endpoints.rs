use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use lazy_static::lazy_static;
use std::convert::Infallible;
use std::sync::RwLock;

lazy_static! {
    static ref HEALTH_STATUS: RwLock<bool> = RwLock::new(true);
    static ref STARTUP_STATUS: RwLock<bool> = RwLock::new(false);
    static ref READY_STATUS: RwLock<bool> = RwLock::new(true);
}

pub fn serve() {
    tokio::spawn(serve_status());
}
pub fn get_health_status() -> bool {
    *HEALTH_STATUS.read().unwrap()
}
pub fn mark_as_unhealthy() {
    let mut status = HEALTH_STATUS.write().unwrap();
    *status = false;
}
pub fn get_startup_status() -> bool {
    *STARTUP_STATUS.read().unwrap()
}
pub fn mark_as_started() {
    let mut status = STARTUP_STATUS.write().unwrap();
    *status = true;
}
pub fn get_readiness_status() -> bool {
    *READY_STATUS.read().unwrap()
}
pub fn mark_as_ready() {
    let mut status = READY_STATUS.write().unwrap();
    *status = true;
}
pub fn mark_as_not_ready() {
    let mut status = READY_STATUS.write().unwrap();
    *status = false;
}

async fn serve_status() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([0, 0, 0, 0], 3000).into();

    let server = Server::bind(&addr).serve(make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handle_requests))
    }));

    server.await?; // TODO: What if error
    Ok(())
}
async fn handle_requests(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/status/health") => Ok(send_status(get_health_status())),
        (&Method::GET, "/status/startup") => Ok(send_status(get_startup_status())),
        (&Method::GET, "/status/readiness") => Ok(send_status(get_readiness_status())),
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()),
    }
}
fn send_status(value: bool) -> Response<Body> {
    if value {
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::empty())
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap()
    }
}
