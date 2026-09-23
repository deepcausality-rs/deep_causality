## 1. Baseline measurements (before any code change)

- [ ] 1.1 Record the CFD baseline with rayon: `cargo bench -p deep_causality_cfd --bench bench_dec_ns_march` and `--bench bench_operator_study` (default features, so `parallel` is on), plus the wall-clock of one verification harness chosen for D5. Save the numbers and the machine in the change folder as `baseline.md`.
- [ ] 1.2 Record the fft baseline with rayon: `cargo bench -p deep_causality_fft --features parallel`.

## 2. deep_causality_par: extend the fork-join surface

- [ ] 2.1 Add the D2 scheduling core: the work-sized thread count, a `Mutex<ChunksMut>` chunk queue with about 8 × threads chunks, and a caller that works too. Add `src/functions/scoped_map_range.rs` on top of it, and rewrite `scoped_map` to delegate to it with `min_len = 1`. Register both in `functions/mod.rs` and export them from `lib.rs`.
- [ ] 2.2 Add `src/functions/scoped_for_each_mut.rs`, which passes the global index to `f`.
- [ ] 2.3 Add `src/functions/scoped_chunks_exact_mut_init.rs`: whole-`len` chunks per worker, one `init` per worker, a panic on `len == 0`, and the remainder left untouched.
- [ ] 2.4 Add tests in `tests/functions/` for each par-fork-join spec scenario:
  - index order
  - empty input
  - `n` smaller than the core count
  - a global index at a length that is not a multiple of the core count (10 007)
  - the remainder left untouched
  - the `init` call count counted with an `AtomicUsize`, bounded as in the par-fork-join `init` scenario
  - a `len == 0` panic
  - an input below `min_len` running inline, and at most `n / min_len` distinct thread ids (par-fork-join "Work-sized thread count")
  - `min_len == 0` behaving like 1
  - panic propagation through `catch_unwind`

  Both the `functions` and the `functions_serial` Bazel suites must pass.
- [ ] 2.5 Add a criterion bench `benches/scoped_threshold.rs` that runs the stencil kernel serially and through `scoped_for_each_mut` from 2^12 to 2^22 rows, sweeping `min_len` over {1024, 2048, 4096, 8192}. Confirm the design's adaptive + queue column; if it does not reproduce, stop and report before migrating any crate. Add `criterion` as a dev-dependency and wire the Bazel bench target.
- [ ] 2.6 Update the crate docs and README:
  - describe the four functions
  - list nesting as unsupported
  - remove the rayon references in `lib.rs` and `maybe_parallel.rs`
  - drop the `rayon` keyword from `Cargo.toml`
- [ ] 2.7 Run `cargo test -p deep_causality_par`, `cargo test -p deep_causality_par --features parallel`, `bazel test //deep_causality_utils/deep_causality_par/...` and clippy. All must pass.

## 3. deep_causality_fft

- [ ] 3.1 Replace both `par_chunks_exact_mut(..).for_each_init` sites in `fft_plan_nd/axis.rs` with `scoped_chunks_exact_mut_init`, and remove `use rayon::prelude::*`.
- [ ] 3.2 Convert `PARALLEL_THRESHOLD` into the lines `min_len` (D4), removing the `if` gate. Set the value from the fft bench, and record the timings and the machine in the doc comment.
- [ ] 3.3 Add a test in which an N-D transform is large enough to fan out and the forward and inverse results match the serial path bit for bit. Run it in both feature modes.
- [ ] 3.4 Remove `rayon` from the fft `Cargo.toml` `[dependencies]` and from its `parallel` feature, leaving `["std", "deep_causality_par/parallel"]`.
- [ ] 3.5 Run `cargo test -p deep_causality_fft` with and without `parallel`, the Bazel tests and clippy. All must pass.

## 4. deep_causality_topology

- [ ] 4.1 `utils_differential.rs`: migrate both matvec helpers to `scoped_map_range`.
- [ ] 4.2 `stencil_op.rs` and `bilinear_op.rs`: migrate to `scoped_for_each_mut`.
- [ ] 4.3 `wedge.rs`, `interior_product.rs` and `de_rham.rs` (`de_rham`): change the local `per_*` closures to take `&LatticeCell<D>` and use `scoped_map`.
- [ ] 4.4 `de_rham.rs` (`sharp`): replace `flat_map_iter` with `scoped_map` over per-vertex `Vec`s, then flatten (D3).
- [ ] 4.5 Convert `PAR_MATVEC_THRESHOLD`, `PAR_STENCIL_THRESHOLD` (non-zeros to rows), `PAR_BILINEAR_THRESHOLD` (non-zeros to rows) and the three inline cell cut-offs (`1 << 12`, `1 << 12`, `1 << 14`) into per-site `min_len` constants (D4). Remove the `if` gates, set each value by measurement, and record the measurement in the doc comments.
- [ ] 4.6 Add fan-out agreement tests, at sizes of at least `4 × min_len`, for each migrated operator (rayon-free-workspace spec). Use a non-identity input field and a lattice size that is not a power of two, so that a chunk-offset error cannot hide behind symmetry. Run them in both feature modes.
- [ ] 4.7 Remove `rayon` from the topology `Cargo.toml` and its `parallel` feature.
- [ ] 4.8 Run `cargo test -p deep_causality_topology` with and without `parallel`, the Bazel tests and clippy. All must pass.

## 5. deep_causality_algorithms

- [ ] 5.1 `surd_algo.rs` and `surd_algo_cdl.rs`: migrate to `scoped_map_range` and collect `Result`.
- [ ] 5.2 `brcd_algo.rs`: migrate the candidate-plan map to `scoped_map_range` with the `candidate_seed(seed, i)` indexing, and migrate `score_families` to `scoped_map` over the sorted job list, collected into `Result<BTreeMap>`.
- [ ] 5.3 `mrmr_algo.rs`: migrate both loops to `scoped_map` followed by a serial fold with the existing identity and comparison, and make the inner redundancy sum serial.
- [ ] 5.4 Add tests:
  - two failing inputs return the lowest-index error, for brcd or surd, whichever fixture can inject a failure
  - mRMR selects the same features and scores in both feature modes
  - the brcd result is unchanged against its existing verification fixture under `parallel`
- [ ] 5.5 Update the doc comments that name rayon (`surd_algo.rs:55`, `brcd_algo.rs:153`, `brcd_algo.rs:305`, `brcd/mod.rs:54`, `mrmr_algo.rs:25`, `mrmr_algo.rs:100`) and the algorithms README.
- [ ] 5.6 Remove `rayon` from the algorithms `Cargo.toml` and its `parallel` feature.
- [ ] 5.7 Run `cargo test -p deep_causality_algorithms` and `-p deep_causality_discovery` with and without `parallel`, the Bazel tests and clippy. All must pass.

## 6. Workspace removal and docs

- [ ] 6.1 Remove `rayon` from `[workspace.dependencies]` in the root `Cargo.toml`, then regenerate `Cargo.lock` and `MODULE.bazel.lock`.
- [ ] 6.2 Verify the rayon-free-workspace spec:
  - `cargo tree -e normal -i rayon --all-features -p <crate>` reports no match for fft, topology and algorithms
  - `grep -rn "rayon::" --include=*.rs` over the library `src/` trees returns nothing
- [ ] 6.3 Update the docs:
  - the `AGENTS.md` External Dependencies table (remove the three `rayon` rows) and the "other N library crates" count
  - `deep_causality_unified_math/README.md`
  - `deep_causality_cfd/verification/dec_cylinder_verification/README.md`
  - `deep_causality_cfd/src/traits/cfd_scalar.rs:15`
  - the `deep_causality_cfd/Cargo.toml` comment ("Rayon-backed")
- [ ] 6.4 Apply the fluiddynamics-dsl delta: confirm the CFD crate has no direct `rayon` dependency, as the modified requirement states.

## 7. Acceptance gate and full verification

- [ ] 7.1 Re-run the task 1.1 benchmarks and the harness, then write the before/after table into `baseline.md`. The gate (D5) is at most 5 % slower. If the gate fails, stop and report to the user instead of tuning further (design, Risks).
- [ ] 7.2 Run `make format && make fix`, `bazel test //...` and `make check_examples`. All must pass.
- [ ] 7.3 Run `cargo mutants` on the four par functions (chunk arithmetic and index offsets). Every surviving mutant is killed by a new test or documented in `.cargo/mutants.toml` with its measurement.
- [ ] 7.4 Prepare the commit message, including the feature-graph BREAKING note (the `parallel` features no longer pull `rayon`), and hand it to the user.
