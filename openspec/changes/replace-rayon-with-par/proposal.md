## Why

Three library crates (`deep_causality_topology`, `deep_causality_fft`, `deep_causality_algorithms`)
depend on `rayon` behind their `parallel` features. They use a small slice of it: ten source files
with six call shapes, all order-preserving maps or per-element writes over a slice or a range. The
workspace already owns a safe fork-join primitive, `deep_causality_par::scoped_map`, built on
`std::thread::scope`. Extending that crate to cover the six shapes removes the last external runtime
dependency behind `parallel` and puts every parallel path under the repo-wide
`unsafe_code = "forbid"` policy, which rayon's internals do not meet.

## What Changes

- `deep_causality_par` gains three functions next to `scoped_map`, all with a serial fallback
  without the `parallel` feature:
  - `scoped_map_range(n, min_len, f)` — order-preserving map over `0..n`.
  - `scoped_for_each_mut(out, min_len, f)` — writes `f(i, &mut out[i])` for every index.
  - `scoped_chunks_exact_mut_init(data, len, min_len, init, f)` — processes fixed-length chunks,
    with one `init()` state per worker thread.

  `min_len` is the minimum number of items per worker thread. Each call starts
  `min(cores, n / min_len)` threads and runs inline when that is one. Workers take chunks from a
  shared queue, so faster cores take more of the work. `scoped_map` switches to the same shared
  queue without a signature change.
- The ten rayon call sites move to these functions:
  - topology: `utils_differential`, `stencil_op`, `bilinear_op`, `wedge`, `interior_product`,
    `de_rham` (both `de_rham` and `sharp`).
  - fft: `fft_plan_nd/axis`.
  - algorithms: `surd_algo`, `surd_algo_cdl`, `brcd_algo` (two sites), `mrmr_algo` (two sites).
- The nested `par_iter` in the mRMR redundancy sum becomes a serial sum inside the outer parallel
  map, because nested scoped fan-out would spawn threads inside worker threads.
- The topology and fft threshold constants become each site's `min_len` and are re-measured
  against the new functions. Each scoped fan-out starts fresh threads at about 8 µs per thread,
  about 130 µs for 16 threads on the benchmark machine. A rayon dispatch costs about 20 µs because
  its pool is already running. Sizing the thread count to the work keeps small inputs serial.
- `rayon` is removed from `[workspace.dependencies]` and from the three crates' `[dependencies]`
  and `parallel` features. Each crate's `parallel` feature keeps its name and forwards to
  `deep_causality_par/parallel`.
- **BREAKING (feature graph only)**: a downstream build that enabled a crate's `parallel` feature
  and relied on it to bring `rayon` in transitively stops receiving it. Public Rust APIs and
  signatures do not change.

## Capabilities

### New Capabilities
- `par-fork-join`: the safe fork-join surface of `deep_causality_par`, covering the existing
  `scoped_map` and the three new functions. It specifies input-order results, identical results
  with and without the `parallel` feature, one `init` per worker, panic propagation, and the
  empty-input and single-thread paths.
- `rayon-free-workspace`: no workspace library crate depends on `rayon`. Every parallel path runs
  through `deep_causality_par`, and the migrated call sites return the same results as their
  serial arms.

### Modified Capabilities
- `fluiddynamics-dsl`: the requirement "Parallelism is an opt-in parameter threaded through the
  solver bounds" says the crate "pulls Rayon directly only where the crate fans out itself". That
  clause becomes: the crate fans out itself only through `deep_causality_par`.

## Impact

- **Code**: `deep_causality_utils/deep_causality_par` gains new functions, tests and a README
  section. The ten call-site files listed above change, along with the thresholds in
  `utils_differential.rs`, `stencil_op.rs`, `bilinear_op.rs` and `fft_plan_nd/axis.rs`, and in
  `wedge.rs`, `interior_product.rs` and `de_rham.rs` if measurement moves them.
- **Dependencies**: removed from the root `Cargo.toml` and from the `Cargo.toml` of topology, fft
  and algorithms. The lock files regenerate. `rayon` remains in `Cargo.lock` and
  `MODULE.bazel.lock` through two paths:
  - `criterion`'s default `rayon` feature, a dev-only dependency.
  - `candle-core`, used by the `examples/causal_discovery_examples` `ml_rca` example, which
    depends on rayon unconditionally.

  Neither path is part of a published library runtime.
- **Docs**:
  - The External Dependencies table in `AGENTS.md` loses its three `rayon` rows.
  - `deep_causality_unified_math/README.md`, the READMEs of `deep_causality_algorithms` and the
    CFD verification harness, and the doc comments that name rayon (`MaybeParallel`,
    `CfdScalar`, `surd_algo`, `brcd`, `mrmr`, fft `axis`) are updated to name
    `deep_causality_par`.
  - The `rayon` keyword is dropped from `deep_causality_par`'s `Cargo.toml`.
- **Performance**: the CFD crate enables `parallel` by default, so the topology stencil and matvec
  paths run per time step. A benchmark gate (design D5) is the acceptance test.
- **Bazel**: `all_crate_deps` resolves from `Cargo.toml`. No hand-written `@crates//:rayon` labels
  exist.
