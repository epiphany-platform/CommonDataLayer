#![feature(linked_list_cursors)]

use log::error;
use std::{
    process,
    sync::PoisonError,
    time::{SystemTime, UNIX_EPOCH},
};

pub mod message_types;
pub mod messaging_system;
pub mod metrics;
pub mod parallel_task_queue;
pub mod psql;
pub mod query_utils;
pub mod status_endpoints;
pub mod task_limiter;

pub fn abort_on_poison<T>(_e: PoisonError<T>) -> T {
    error!("Encountered mutex poisoning. Aborting.");
    process::abort();
}

pub fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as i64
}
