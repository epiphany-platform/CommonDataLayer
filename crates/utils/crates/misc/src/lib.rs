use std::{
    panic, process,
    sync::PoisonError,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn abort_on_poison<T>(_e: PoisonError<T>) -> T {
    tracing::error!("Encountered mutex poisoning. Aborting.");
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
    panic::set_hook(Box::new(move |info| {
        orig_panic_hook(info);
        process::abort();
    }));
}
