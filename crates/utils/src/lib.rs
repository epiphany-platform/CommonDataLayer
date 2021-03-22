#![feature(linked_list_cursors)]
#![feature(box_syntax)]

use log::error;
use std::{
    panic, process,
    sync::PoisonError,
    time::{SystemTime, UNIX_EPOCH},
};

pub mod communication;
pub mod message_types;
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

pub fn set_aborting_panic_hook() {
    let orig_panic_hook = panic::take_hook();
    panic::set_hook(box move |info| {
        orig_panic_hook(info);
        process::abort();
    });
}
