## 1. Adaptive randomized TT-rounding (`tensor-train`) — primary

- [x] 1.1 Gaussian sampler added: `rng::gaussian_vec<T>` (splitmix64 → Box–Muller normals injected via `from_f64` = `from_real` on the real axis). No new external crate; deterministic/reproducible.
- [x] 1.2 `Truncation` extended with `RoundStrategy` (`Deterministic` default | `Randomized { oversample, seed }`) + `.randomized(oversample, seed)` builder + `.strategy()` getter; all existing constructors default to `Deterministic`; `RoundStrategy` re-exported. (Used `oversample`+`seed`; the `f_init`/`f_inc` growth control is realized as start = `2·oversample` and geometric doubling — see 1.5.)
- [x] 1.3 Realized as a per-unfolding **randomized range-finder** (Halko–Martinsson–Tropp) rather than the literal cross-core KRP recurrence: each rounding-sweep unfolding `m = [r_left, n·r_right]` (already formed by the existing sweep — the `nᵈ` dense is never materialized) is sketched `Y = m·Ω`, giving the same `O(d·n·r²·ℓ)` cost and integrating with zero churn. The cross-core KRP "never form the per-core unfolding" refinement is noted as optional future work.
- [x] 1.4 Randomize-then-orthogonalize sweep: `Y = m·Ω` → `Q = qr(Y)` → `B = Qᴴ·m` → deterministic SVD of small `B` → lift `U = Q·U_B`. Uses the existing Householder `qr` and conjugate-aware `linalg`.
- [x] 1.5 Adaptive loop implemented: bond-capped policies set `ℓ = max_bond + oversample`; tolerance policies start at `ℓ = 2·oversample` and **double** until `‖A − Q·Qᴴ·A‖_F ≤ max(abs_tol, rel_tol·‖A‖)` (residual in `T::Real`), capped at the full rank.
- [x] 1.6 Routed through the strategy at the `svd_truncated` dispatch layer — so `round`, `from_dense`, `add_rounded`, `hadamard_rounded`, and every solver truncation inherit it. Deterministic path is byte-identical (verified); randomized only when selected.
- [x] 1.7 Tests added (`op_tensor_svd_randomized_tests.rs`, 9 tests, `f64`/`Float106` + `Complex<f64>`): low-rank reconstruction to tolerance, adaptive-growth path, randomized-vs-deterministic round agreement + original recovery, default-strategy-unchanged, seed reproducibility (bit-for-bit). Reference: Al Daas–Ballard 2023 (arXiv:2110.04393); adaptive Khatri-Rao arXiv:2511.03598; Halko–Martinsson–Tropp 2011.

## 2. Greedy-pivot TT-cross (`tensor-train-cross`) — secondary

- [ ] 2.1 Add a pivot-strategy flag to `CrossConfig` (`Maxvol` default | `Greedy`).
- [ ] 2.2 Implement residual-greedy pivot selection (largest `|A − Ã|` entry, `O(N·R²)`) with the nestedness property, reusing the existing eval-cross fibers; no dense buffer.
- [ ] 2.3 Route `cross` through the strategy; maxvol/LU stays default and byte-identical.
- [ ] 2.4 Tests (incl. `Complex<f64>`): greedy recovers the rank-1/rank-2 oracles to tolerance; nestedness holds; budget respected; default unchanged. Reference: Shi–Hayes–Qiu arXiv:2407.11290; quasi-optimality arXiv:1305.1818.

## 3. Fused Hadamard-then-truncate + dense kernels (`tensor-train`, `tensor-network-numerics`) — tertiary

- [ ] 3.1 Implement fused `hadamard_rounded`: compress each squared-bond core against the running canonical `R` as it is built, so the peak bond stays `~r·r_keep` < `r²`. Result equals build-then-round to tolerance.
- [ ] 3.2 Rewrite `linalg::matmul` as a B-transposed, cache-blocked loop (integer block constant; no float literals; no `Default` bound); result identical to the naive product.
- [ ] 3.3 Reuse ping-pong scratch buffers in the `inner`/`norm` transfer-matrix contraction (no per-site allocation); results bit-identical.
- [x] 3.4 Randomized range-finder `svd_truncated` variant behind the `Truncation` policy delivered as part of Stage 1 (`CausalTensor::svd_randomized`); deterministic Jacobi stays default. (Single-pass range-finder; block-Krylov refinement for slowly-decaying spectra — arXiv:2308.01480 / arXiv:2504.04989 — left as future work.)
- [ ] 3.5 Tests: fused-vs-build-then-round agreement; blocked-vs-naive matmul equality; inner/norm unchanged; (if 3.4) randomized SVD reconstructs to tolerance.

## 4. Benchmarks and finalization

- [ ] 4.1 Extend `bench_tensor_train_core.rs` with a high-interior-bond `round`: deterministic vs randomized rows (confirm the randomized speedup).
- [ ] 4.2 Extend `bench_tensor_train_cross.rs` with a maxvol-vs-greedy row; add a blocked-vs-naive `matmul` micro-bench and a fused-vs-build-then-round `hadamard_rounded` row.
- [ ] 4.3 Update the crate README `### CausalTensorTrain Performance` table with the before/after numbers and a note on the randomized/greedy trade-offs.
- [ ] 4.4 Run `make format && make fix`; confirm `unsafe_code = "forbid"`, no `dyn`, no lib-code macros, no concrete float literals; whole-workspace `cargo test` green; clippy `--all-targets` clean.
- [ ] 4.5 Run `openspec validate optimize-tensor-network` and reconcile any spec drift before `/opsx:apply` completion.
