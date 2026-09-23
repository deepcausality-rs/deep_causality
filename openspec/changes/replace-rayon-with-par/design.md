## Context

`rayon` 1.12 is an optional dependency of three library crates, enabled by their `parallel`
features:

| Crate | File | Rayon shape | Gate |
|---|---|---|---|
| topology | `differential/utils_differential.rs` (×2) | `(0..rows).into_par_iter().map(f).collect()` | `rows >= 1 << 18` |
| topology | `differential/stencil/stencil_op.rs` | `out.par_iter_mut().enumerate().for_each(f)` | `nnz >= 1 << 16` |
| topology | `differential/stencil/bilinear_op.rs` | `out.par_iter_mut().enumerate().for_each(f)` | `nnz >= 1 << 16` |
| topology | `differential/wedge.rs` | `Vec<Cell>.into_par_iter().map(f).collect()` | `cells >= 1 << 12` |
| topology | `differential/interior_product.rs` | `Vec<Cell>.into_par_iter().map(f).collect()` | `cells >= 1 << 12` |
| topology | `differential/de_rham.rs` `de_rham` | `Vec<Cell>.into_par_iter().map(f).collect()` | `edges >= 1 << 14` |
| topology | `differential/de_rham.rs` `sharp` | `Vec<Cell>.into_par_iter().flat_map_iter(f).collect()` | none |
| fft | `fft_plan_nd/axis.rs` (×2) | `par_chunks_exact_mut(len).for_each_init(init, f)` | `len >= 1 << 17` |
| algorithms | `surd/surd_algo.rs`, `surd_algo_cdl.rs` | `(0..n).into_par_iter().map(f).collect::<Result<Vec<_>,_>>()` | none |
| algorithms | `brcd/brcd_algo.rs` plans | `combos.par_iter().enumerate().map(f).collect::<Result<..>>()` | none |
| algorithms | `brcd/brcd_algo.rs` `score_families` | `BTreeMap.par_iter().map(f).collect::<Result<BTreeMap>>()` | none |
| algorithms | `mrmr/mrmr_algo.rs` (×2) | `Vec.into_par_iter().map(f).reduce(id, op)`; nested `par_iter().sum::<Result>()` | none |

`deep_causality_par` already exports `MaybeParallel` and `scoped_map(&[T], f) -> Vec<U>`, which
together cover the parallel arms of `wedge`, `interior_product` and `de_rham`, and, with
restructuring, the `brcd` and `mrmr` arms. The crate is `std`-only under `parallel`, has no
dependencies, and is tested in both feature modes by Bazel targets that already exist
(`functions`, `functions_serial`).

The workspace sets `unsafe_code = "forbid"` with no exemptions. `std::thread::scope` is the only
safe way to run closures that borrow the caller's stack on other threads. A persistent pool that
accepts borrowed work needs `unsafe` lifetime erasure, which is how rayon's `scope` and `join`
are built.

Measured on the benchmark machine (M3 Max, 16 cores: 12 P + 4 E, 128 GB). The kernel is a
7-nonzero CSR-style row sum. Values are median µs per call over three runs of a scratch harness,
and task 2.5 re-measures them in-tree. *Flat* is the current `scoped_map` chunking: one
contiguous chunk per core. *Adaptive + queue* is D2. The "dispatch" row times a 16-element input,
which isolates the fan-out cost.

| rows | serial | rayon `par_iter_mut` | rayon `par_chunks_mut` | flat scoped | adaptive + queue (D2) |
|---|---|---|---|---|---|
| dispatch | 0.1–0.4 | 20–24 | 24–26 | 124–128 | 0.5 (inline) |
| 2^12 | 23 | 111–116 | 47–52 | 130–131 | 25 |
| 2^14 | 91 | 143–153 | 58–66 | 133–137 | 53 |
| 2^16 | 372 | 200–204 | 132–137 | 178–179 | 195–205 |
| 2^18 | 1490–1520 | 324–345 | 380–420 | 265–275 | 262–274 |
| 2^20 | 6080–6180 | 730–750 | 890–930 | 818–828 | 618–625 |
| 2^22 | 24500 | 2260–2330 | 3050–3270 | 3030–3340 | 2830–2960 |

Three spawn strategies were tried and rejected, because none of them reduces the fan-out cost:

| strategy | fan-out cost |
|---|---|
| caller runs a chunk itself | 120–128 µs |
| binary-tree spawning (log-depth) | 143–144 µs |
| 64 KiB thread stacks | 138–139 µs |

The cost of starting and joining each thread, about 8 µs, sets the floor.

## Goals / Non-Goals

**Goals:**
- Remove `rayon` from `[workspace.dependencies]` and from every library crate.
- Cover each shape in the table with a `deep_causality_par` function that has a serial fallback,
  uses no `unsafe` and has no dependencies.
- Keep public APIs, `MaybeParallel` bounds and feature names unchanged.
- Keep results identical to the serial arm at every migrated site.

**Non-Goals:**
- Reproducing rayon's trait-based `ParallelIterator` API (`into_par_iter`, adaptors, `prelude`).
- Work stealing, a persistent thread pool, nested parallelism, or a configurable thread count.
  A persistent pool that accepts borrowed work cannot be written without `unsafe` (Context).
- Removing `rayon` from `Cargo.lock`. It stays there through `criterion` (dev) and `candle-core`
  (one example).
- Parallelising sites that are serial today.

## Decisions

### D1. Free functions, not a rayon-shaped iterator trait

Add three functions beside `scoped_map` instead of `IntoParallelIterator` / `ParallelIterator`
traits.

- The call sites use four terminal operations (`collect`, `for_each`, `for_each_init`, `reduce`)
  over two input kinds (slice, range). A trait facade would need adaptor types for `map`,
  `enumerate` and `flat_map_iter` to cover them, which is more code than the four functions.
- Rayon's traits get their generality from a splittable producer/consumer protocol.
  Reimplementing that on scoped threads keeps the API and discards the design that justified it.
- The call sites already carry `#[cfg(feature = "parallel")]` arms. A function call in that arm is
  a smaller diff than a trait import.

*Alternative considered*: a `ParSlice` extension trait with `par_map`. Rejected. It adds a trait
and a module per shape and changes nothing at the call sites.

### D2. The added surface

```rust
pub fn scoped_map_range<U, F>(n: usize, min_len: usize, f: F) -> Vec<U>
where U: MaybeParallel, F: Fn(usize) -> U + MaybeParallel;

pub fn scoped_for_each_mut<T, F>(out: &mut [T], min_len: usize, f: F)
where T: MaybeParallel, F: Fn(usize, &mut T) + MaybeParallel;

pub fn scoped_chunks_exact_mut_init<T, S, I, F>(
    data: &mut [T], len: usize, min_len: usize, init: I, f: F)
where T: MaybeParallel, I: Fn() -> S + MaybeParallel, F: Fn(&mut S, &mut [T]) + MaybeParallel;
```

All four functions, including `scoped_map`, share one scheduling core, which the scratch
benchmark measured:

1. **Size the thread count to the work.** `threads = min(available_parallelism, n / min_len)`,
   with `n` counted in items, or in whole `len` chunks for `scoped_chunks_exact_mut_init`. With
   `threads <= 1` the call runs inline and starts no thread. This keeps small inputs at serial
   speed, and it drives the 2^12–2^14 rows in the table. `min_len == 0` is treated as 1.
2. **Hand out chunks from a shared queue.** The output is cut into about `8 × threads` contiguous
   chunks. `std::slice::ChunksMut` behind a `std::sync::Mutex` hands them out, and each worker
   loops, taking the next chunk until the queue is empty. This is safe: every chunk is a disjoint
   `&mut` borrow taken from the iterator. Faster cores take more chunks, which drives the 2^20
   row: 620 µs, against 820 µs for fixed chunks and 740 µs for rayon.
3. **The caller works too.** The scope starts `threads − 1` workers, and the calling thread runs
   the same loop.

Results stay in input order because every chunk writes back to its own index range.
`scoped_chunks_exact_mut_init` cuts chunks in whole multiples of `len`, so no line is split. It
calls `init` once per worker, including the caller, and reuses that state across the worker's
chunks. `S` needs no `Send` bound because it is created and dropped inside its worker.

`scoped_map` keeps its signature with an implicit `min_len = 1`. Its callers
(`deep_causality_cfd` `flow/carrier.rs` and `flow/sweep.rs`) fan out a few coarse branches, and
the shared queue lets a slow branch no longer hold back a fixed partner chunk.

`min_len` plays the role of rayon's `with_min_len`, and it replaces the `if n >= THRESHOLD` gate
at each call site (D4).

Each function lives in `src/functions/<name>.rs` with its test file in `tests/functions/`, per the
crate layout.

### D3. Per-site mapping

| Site | Replacement |
|---|---|
| `utils_differential` ×2 | `scoped_map_range(rows, MATVEC_MIN_LEN, per_row)` |
| `stencil_op`, `bilinear_op` | `scoped_for_each_mut(out, ROWS_MIN_LEN, \|r, o\| …)` |
| `wedge`, `interior_product`, `de_rham` | existing `scoped_map(&cells, per_x)`. `LatticeCell<D>` is `Clone` but not `Copy`, so the local `per_*` closures change to take `&LatticeCell<D>`, and the serial arm iterates by reference. |
| `sharp` (`flat_map_iter`) | `scoped_map(&vertices, \|v\| per_vertex(v).collect::<Vec<_>>())`, then `.into_iter().flatten().collect()`. `per_vertex` yields `D` values, so each inner `Vec` is small. |
| fft `axis` ×2 | `scoped_chunks_exact_mut_init(data, len, LINES_MIN_LEN, \|\| vec![czero(); s_len], \|scr, line\| plan.execute_dir_unchecked(line, scr, inverse))` |
| `surd`, `surd_cdl` | `scoped_map_range(n, 1, analyze).into_iter().collect::<Result<Vec<_>,_>>()` |
| `brcd` plans | `scoped_map_range(combos.len(), 1, \|i\| build_candidate_plan(…, &combos[i], …, candidate_seed(seed, i)))`, then collect `Result` |
| `brcd` `score_families` | `let jobs: Vec<_> = jobs.iter().collect();` then `scoped_map(&jobs, …)` and collect into `Result<BTreeMap>`. Key order is preserved because the input is already sorted. |
| `mrmr` ×2 | `scoped_map(&features, score)`, then a serial fold with the existing `(0, -1.0)` identity and `>` comparison. The inner redundancy sum becomes a serial `iter().map(...).sum::<Result<f64,_>>()`. |

**Error semantics**: rayon's `collect::<Result>` short-circuits and returns whichever error it
reaches first. The replacement computes every element and returns the error with the lowest index.
This is deterministic, and the success path is unchanged. On a failing input it costs extra work,
which is acceptable for these error paths.

**mRMR nested fan-out**: nesting `scoped_map` would spawn up to 16×16 OS threads. The inner loop
runs over the selected features, at most `num_features` short correlations, so a serial sum costs
less than any fan-out. The fluiddynamics-dsl requirement already says the two granularities do
not nest.

### D4. Thresholds become `min_len`, re-measured

A rayon dispatch costs about 20 µs. A scoped fan-out costs about 8 µs per thread started. With
`min_len`, the thread count scales with the work, so a site no longer has to choose between
all cores and no parallelism. Each `if n >= THRESHOLD` gate is removed, and its constant becomes
the `min_len` argument, in the unit the function counts. `stencil_op` and `bilinear_op` gate on
non-zeros today (`nnz >= 1 << 16`). Their `min_len` is in rows, so the value is converted using
the operator's mean non-zeros per row.

For the 7-nonzero kernel, `min_len = 4096` rows matched or beat rayon from 2^12 to 2^20. Task 2.5
adds a criterion benchmark to `deep_causality_par` that sweeps `min_len` for this kernel. Each
site's value is set from that sweep or from a benchmark of the site's own crate:
- fft uses the fft bench.
- `wedge` and `interior_product` do more work per item, so they use the CFD operator study.

Each constant's doc comment records the measurement and the machine, as the fft
`PARALLEL_THRESHOLD` comment does now.

`sharp` and the algorithms sites have no threshold today and pass `min_len = 1`. Their per-item cost
(a candidate plan, a SURD target state, an F-statistic over a column) is in the millisecond
range.

### D5. Acceptance gate: CFD wall-clock

CFD enables `parallel` by default and calls the stencil operators once per time step, so any spawn
regression shows up there. Gate: run `deep_causality_cfd/benches/bench_dec_ns_march.rs` and
`bench_operator_study.rs`, plus one verification harness (`dec_cylinder_verification`, or a
shorter one agreed during apply), before
and after the change, both with `parallel`. The change must not be more than 5 % slower. Report
the numbers with the machine.

## Risks / Trade-offs

- [Per-call spawn overhead on hot paths] → D2 sizes the thread count to the work, D4 sets
  `min_len` per site, and D5 gates the result. If D5 fails after
  tuning, stop and report. The only safe remedy is coarser fan-out (for example, per time step
  instead of per operator), which is a solver-level change outside this proposal.
- [Remaining gap to rayon at the two ends] → at 2^16 rows, rayon's chunked form runs at 134 µs
  against 195–205 µs. At 2^22 rows, rayon runs at 2.3 ms against 2.9 ms. Both gaps come from
  starting fresh threads, which begin with cold caches, where rayon reuses its pool. Closing
  either gap needs a persistent pool, which needs `unsafe`. Accepted. The D5 gate measures
  whether it matters for CFD.
- [Queue lock contention] → at about 8 × threads chunks per call, a thread takes the lock about
  8 times, and the lock is never visible in the measurements above.
- [Oversubscription when a coarse fan-out calls a fine one] → the fluiddynamics-dsl requirement
  already forbids nesting. `deep_causality_par` does not detect nesting. Documented in the crate
  docs.
- [Error-path behaviour change: all elements are evaluated] → deterministic lowest-index error.
  Covered by an algorithms test that supplies two failing inputs.
- [Downstream crates that relied on `parallel` pulling in rayon] → none in the workspace. Noted
  in the commit message for release notes (CHANGELOG.md is generated by release-plz).
- [rayon remains in `Cargo.lock`] → stated as a non-goal. To drop it from the dev graph, set
  `criterion`'s `default-features = false` with `plotters` and `cargo_bench_support`. That is left
  as an open question because it slows criterion's bootstrap analysis.

## Migration Plan

1. Add the functions and tests to par, plus the threshold benchmark. Par is Tier 0, so nothing
   else changes yet.
2. Migrate fft, then topology, then algorithms. Each crate builds and tests in both feature modes
   before the next one starts.
3. Remove `rayon` from the three `Cargo.toml` files and the workspace table, then regenerate
   `Cargo.lock` and `MODULE.bazel.lock`.
4. Run the D5 gate, update the docs, run `bazel test //...` and clippy.

To roll back, revert the change. No data or public-API migration is involved.

## Open Questions

- Should `criterion` drop its default `rayon` feature, so `rayon` leaves the dev graph as well?
  Doing so slows benchmark analysis.
- Should the `ml_rca` example (candle-core) move out of the workspace, so `rayon` leaves
  `Cargo.lock` entirely? This proposal assumes it stays.
