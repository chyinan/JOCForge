// pattern: Functional Core
//! Bounded, shared workers for independent DSP rows and coefficient fits.
//!
//! The `parallel` feature owns the only rayon pool. Targets without that feature,
//! including `wasm32-unknown-unknown`, run the same closures in input order.

#[cfg(feature = "parallel")]
use std::sync::LazyLock;

#[cfg(feature = "parallel")]
fn pool() -> Option<&'static rayon::ThreadPool> {
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
    #[cfg(feature = "parallel")]
    if let Some(workers) = pool() {
        return workers.install(|| rayon::join(left, right));
    }
    (left(), right())
}

/// Collect independent lanes in their original order, including the first error.
pub(crate) fn map_rows<T: Send, E: Send>(
    count: usize,
    compute: impl Fn(usize) -> Result<T, E> + Sync,
) -> Result<Vec<T>, E> {
    #[cfg(feature = "parallel")]
    if let Some(workers) = pool() {
        use rayon::prelude::*;
        let rows: Vec<_> = workers.install(|| (0..count).into_par_iter().map(&compute).collect());
        return rows.into_iter().collect();
    }
    (0..count).map(compute).collect()
}

/// Map a slice in input order. `min_len` is the rayon chunk size when a pool exists.
pub(crate) fn map_slice<T, U, E>(
    items: &[T],
    min_len: usize,
    compute: impl Fn(&T) -> Result<U, E> + Sync,
) -> Result<Vec<U>, E>
where
    T: Sync,
    U: Send,
    E: Send,
{
    #[cfg(feature = "parallel")]
    if let Some(workers) = pool() {
        use rayon::prelude::*;
        let rows: Vec<_> = workers.install(|| {
            items
                .par_iter()
                .with_min_len(min_len)
                .map(&compute)
                .collect()
        });
        return rows.into_iter().collect();
    }
    let _ = min_len;
    items.iter().map(compute).collect()
}

/// Map `(index, item)` in input order. `parallel` is an additional caller predicate.
pub(crate) fn map_enumerated<T, U>(
    items: &[T],
    parallel: bool,
    compute: impl Fn(usize, &T) -> U + Sync,
) -> Vec<U>
where
    T: Sync,
    U: Send,
{
    #[cfg(feature = "parallel")]
    if parallel {
        if let Some(workers) = pool() {
            use rayon::prelude::*;
            return workers.install(|| {
                items
                    .par_iter()
                    .enumerate()
                    .map(|(index, item)| compute(index, item))
                    .collect()
            });
        }
    }
    let _ = parallel;
    items
        .iter()
        .enumerate()
        .map(|(index, item)| compute(index, item))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::map_slice;

    #[test]
    fn mapped_slices_keep_input_order() {
        let items: Vec<u32> = (0..64).collect();
        let mapped = map_slice(&items, 8, |value| Ok::<_, ()>(value.wrapping_mul(3))).unwrap();
        let sequential: Vec<_> = items.iter().map(|value| value.wrapping_mul(3)).collect();
        assert_eq!(mapped, sequential);
    }
}
