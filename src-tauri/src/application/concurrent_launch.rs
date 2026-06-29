use std::sync::atomic::{AtomicUsize, Ordering};

/// Maximum number of independent application or browser launches in parallel.
pub const MAX_CONCURRENT_LAUNCHES: usize = 3;

/// Runs `operation` over each item with at most `limit` worker threads.
pub fn map_with_bounded_concurrency<T, R, F>(items: &[T], limit: usize, operation: F) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> R + Sync + Send,
{
    if items.is_empty() {
        return Vec::new();
    }

    let worker_count = limit.max(1).min(items.len());
    let next_index = AtomicUsize::new(0);
    let mut results: Vec<Option<R>> = (0..items.len()).map(|_| None).collect();

    std::thread::scope(|scope| {
        let handles = (0..worker_count)
            .map(|_| {
                scope.spawn(|| {
                    let mut local_results = Vec::new();
                    loop {
                        let index = next_index.fetch_add(1, Ordering::SeqCst);
                        if index >= items.len() {
                            break;
                        }
                        local_results.push((index, operation(&items[index])));
                    }
                    local_results
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            for (index, result) in handle.join().expect("worker thread") {
                results[index] = Some(result);
            }
        }
    });

    results
        .into_iter()
        .map(|value| value.expect("missing concurrent result"))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        time::Duration,
    };

    use super::{MAX_CONCURRENT_LAUNCHES, map_with_bounded_concurrency};

    #[test]
    fn preserves_input_order() {
        let items = vec![1, 2, 3, 4, 5];
        let results = map_with_bounded_concurrency(&items, 2, |value| value * 2);
        assert_eq!(results, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn bounds_parallelism() {
        let active = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        let items = vec![0; 8];

        let _ = map_with_bounded_concurrency(&items, MAX_CONCURRENT_LAUNCHES, |_| {
            let current = active.fetch_add(1, Ordering::SeqCst) + 1;
            peak.fetch_max(current, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(20));
            active.fetch_sub(1, Ordering::SeqCst);
        });

        assert!(peak.load(Ordering::SeqCst) <= MAX_CONCURRENT_LAUNCHES);
    }
}
