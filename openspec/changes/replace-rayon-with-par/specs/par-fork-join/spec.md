## ADDED Requirements

### Requirement: Order-preserving parallel map over a range
`deep_causality_par` SHALL export `scoped_map_range(n: usize, min_len: usize, f: F) -> Vec<U>` where `F: Fn(usize) -> U + MaybeParallel` and `U: MaybeParallel`.
It returns `f(0), f(1), …, f(n-1)` in index order. Under the `parallel` feature it runs on
`min(available_parallelism, n / min_len)` threads (the caller included) within one
`std::thread::scope`, and it runs inline when that count is at most one. Without the feature it
is `(0..n).map(f).collect()`.

#### Scenario: Results follow index order
- **WHEN** `scoped_map_range(1000, 1, |i| i * i)` runs with or without `parallel`
- **THEN** the result equals `(0..1000).map(|i| i * i).collect::<Vec<_>>()`

#### Scenario: Empty range
- **WHEN** `scoped_map_range(0, 1, f)` runs
- **THEN** it returns an empty `Vec` and never calls `f`

#### Scenario: Fewer items than cores
- **WHEN** `n` is smaller than the available parallelism, including `n == 1`
- **THEN** every index is computed exactly once and no thread is spawned for an empty chunk

### Requirement: Indexed in-place parallel write
`deep_causality_par` SHALL export `scoped_for_each_mut(out: &mut [T], min_len: usize, f: F)` where `F: Fn(usize, &mut T) + MaybeParallel` and `T: MaybeParallel`.
It calls `f(i, &mut out[i])` exactly once for every index `i`. Under the `parallel` feature `out` is
split into contiguous chunks that workers take from a shared queue, and the index passed to `f` is
the global index into `out`, not the index within the chunk.

#### Scenario: Global index is passed to the closure
- **WHEN** `scoped_for_each_mut(&mut v, 1, |i, x| *x = i)` runs on a `Vec` of length 10 007
- **THEN** `v[i] == i` for every `i`, in both feature modes

#### Scenario: Empty slice
- **WHEN** `out` is empty
- **THEN** the function returns without calling `f`

### Requirement: Fixed-length chunk processing with per-worker state
`deep_causality_par` SHALL export `scoped_chunks_exact_mut_init(data: &mut [T], len: usize, min_len: usize, init: I, f: F)` where `I: Fn() -> S + MaybeParallel` and `F: Fn(&mut S, &mut [T]) + MaybeParallel`.
It calls `f` once on each of the `data.len() / len` consecutive chunks of length `len`, leaves any
trailing remainder shorter than `len` untouched, and gives each worker its own `S` built by one
`init()` call reused across that worker's chunks. Chunk boundaries SHALL never be split across
workers. Without the `parallel` feature, `init` is called once and every chunk is processed inline.

#### Scenario: Every exact chunk is processed once
- **WHEN** `data` has length `len * m + r` with `r < len`
- **THEN** each of the `m` chunks is passed to `f` exactly once and the last `r` elements are unchanged

#### Scenario: init runs at most once per worker
- **WHEN** the function runs under `parallel` with `m` chunks
- **THEN** `init` is called at least once and at most `max(1, min(m / max(min_len, 1), available_parallelism))` times

#### Scenario: Zero chunk length is rejected
- **WHEN** `len == 0`
- **THEN** the function panics with a message naming `len`, matching `slice::chunks_exact_mut`

### Requirement: Work-sized thread count
Every function that takes `min_len` SHALL start no thread when `n / min_len <= 1`, where `n` counts items, or whole chunks for `scoped_chunks_exact_mut_init`, and SHALL treat `min_len == 0` as 1.
Under the `parallel` feature the number of threads taking part, the caller included, SHALL be at
most `min(available_parallelism, n / min_len)`.

#### Scenario: Input below one worker's share runs inline
- **WHEN** `scoped_for_each_mut` runs on 3 000 elements with `min_len = 4096` under `parallel`
- **THEN** every closure call runs on the calling thread, as `std::thread::current().id()` shows

#### Scenario: Thread count is bounded by the work
- **WHEN** `scoped_map_range(4 * m, m, f)` runs under `parallel` on a machine with at least 4 cores
- **THEN** the closure observes at most 4 distinct thread ids

### Requirement: Feature-independent results and panic propagation
Every function in the fork-join surface (`scoped_map`, `scoped_map_range`, `scoped_for_each_mut`, `scoped_chunks_exact_mut_init`) SHALL produce identical output with and without the `parallel` feature for a deterministic closure.
A panic in any closure SHALL propagate to the caller: on scope join under `parallel`, immediately
otherwise. All four functions SHALL be implemented without `unsafe` and without external
dependencies.

#### Scenario: Serial and parallel builds agree
- **WHEN** the test suite runs against both the `parallel` and the serial Bazel library targets
- **THEN** every function returns bit-identical results for the same deterministic inputs

#### Scenario: A panicking closure surfaces to the caller
- **WHEN** the closure panics on one element
- **THEN** the call panics and `std::panic::catch_unwind` around it observes the panic
