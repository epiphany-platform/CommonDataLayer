use log::error;
use std::{process, sync::PoisonError};

pub mod metrics;
pub mod status_endpoints;
pub mod task_limiter;

pub fn abort_on_poison<T>(_e: PoisonError<T>) -> T {
    error!("Encountered mutex poisoning. Aborting.");
    process::abort();
}
