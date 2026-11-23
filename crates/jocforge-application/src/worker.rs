// pattern: Imperative Shell
//! Large-stack worker for one encode callback.
//!
//! `wasm32-unknown-unknown` has no threads in this phase, so the callback runs
//! on the caller. Native targets keep the dedicated stack.

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub(crate) enum WorkerRunError {
    Unavailable,
    Panicked,
}

pub(crate) fn run_on_worker<T: Send>(
    name: &str,
    stack_size: usize,
    work: impl FnOnce() -> T + Send,
) -> Result<T, WorkerRunError> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::thread::scope(|scope| {
            let worker = std::thread::Builder::new()
                .name(name.to_owned())
                .stack_size(stack_size)
                .spawn_scoped(scope, work)
                .map_err(|_| WorkerRunError::Unavailable)?;
            worker.join().map_err(|_| WorkerRunError::Panicked)
        })
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (name, stack_size);
        Ok(work())
    }
}
