// pattern: Functional Core
//! Bounded, shared workers for independent DSP rows and coefficient fits.

use std::sync::LazyLock;

pub(crate) fn pool() -> Option<&'static rayon::ThreadPool> {
    static POOL: LazyLock<Option<rayon::ThreadPool>> = LazyLock::new(|| {
        let logical = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
        // Leave execution room for the producer and publisher; row batches contain at most 15 objects.
        let workers = logical.min(logical.div_ceil(2).clamp(8, 16));
        if workers < 2 {
            return None;
        }
        rayon::ThreadPoolBuilder::new()
            .num_threads(workers)
            .stack_size(16 * 1024 * 1024)
            .thread_name(|i| format!("joc-dsp-{i}"))
            .build()
            .ok()
    });
    POOL.as_ref()
}

/// Runs independent parts of a frame on the same bounded pool used for its DSP rows.
pub fn join_dsp_tasks<A: Send, B: Send>(
    left: impl FnOnce() -> A + Send,
    right: impl FnOnce() -> B + Send,
) -> (A, B) {
    match pool() {
        Some(workers) => workers.install(|| rayon::join(left, right)),
        None => (left(), right()),
    }
}

/// Collect independent lanes in their original order, including the first error.
pub(crate) fn map_rows<T: Send, E: Send>(
    count: usize,
    compute: impl Fn(usize) -> Result<T, E> + Sync,
) -> Result<Vec<T>, E> {
    use rayon::prelude::*;
    let Some(pool) = pool() else {
        return (0..count).map(compute).collect();
    };
    let rows: Vec<_> = pool.install(|| (0..count).into_par_iter().map(&compute).collect());
    rows.into_iter().collect()
}
