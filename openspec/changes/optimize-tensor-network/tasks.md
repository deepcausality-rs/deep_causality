## 1. Adaptive randomized TT-rounding (`tensor-train`) — primary

- [ ] 1.1 Add a Gaussian sampler over `T::Real` to the TT layer's seeded splitmix64 generator (Box–Muller on the existing `rand_unit`-style uniforms); inject into `T` via `from_real`. No new external crate.
- [ ] 1.2 Extend `Truncation<R>` with a rounding-strategy selector (`Deterministic` default | `Randomized { oversample, f_init, f_inc }`), validated; keep all existing constructors defaulting to `Deterministic`.
- [ ] 1.3 Implement the Khatri-Rao partial-contraction recurrence `W_k = H(X_k)·(W_{k+1} ⊙ Ω_k)` (right-to-left), reusing the conjugate-aware `linalg`; no full sketch materialized. Cost `O(d·n·r²·ℓ)`.
- [ ] 1.4 Implement the fixed-rank randomize-then-orthogonalize sweep (left-to-right QR via the existing Householder `qr`), producing a left-canonical rounded train.
- [ ] 1.5 Implement the adaptive loop: seed `ℓ = ⌈max r·f_init⌉`, residual estimate `‖(I−QQᴴ)XΩ‖_F/(√s·‖X‖)` in `T::Real`, grow `ℓ` by `f_inc` until ≤ tol (cap at the deterministic bound).
- [ ] 1.6 Route `round` (and `add_rounded`/`hadamard_rounded` and the AMEn enrichment `round`) through the strategy: deterministic path byte-identical to today; randomized path used only when selected.
- [ ] 1.7 Tests (f32/f64/Float106 + `Complex<f64>`): randomized-vs-deterministic agreement to tolerance; adaptive rank matches the deterministic rank; default path unchanged; seed reproducibility. Reference: Al Daas–Ballard 2023 (arXiv:2110.04393); adaptive Khatri-Rao arXiv:2511.03598.

## 2. Greedy-pivot TT-cross (`tensor-train-cross`) — secondary

- [ ] 2.1 Add a pivot-strategy flag to `CrossConfig` (`Maxvol` default | `Greedy`).
- [ ] 2.2 Implement residual-greedy pivot selection (largest `|A − Ã|` entry, `O(N·R²)`) with the nestedness property, reusing the existing eval-cross fibers; no dense buffer.
- [ ] 2.3 Route `cross` through the strategy; maxvol/LU stays default and byte-identical.
- [ ] 2.4 Tests (incl. `Complex<f64>`): greedy recovers the rank-1/rank-2 oracles to tolerance; nestedness holds; budget respected; default unchanged. Reference: Shi–Hayes–Qiu arXiv:2407.11290; quasi-optimality arXiv:1305.1818.

## 3. Fused Hadamard-then-truncate + dense kernels (`tensor-train`, `tensor-network-numerics`) — tertiary

- [ ] 3.1 Implement fused `hadamard_rounded`: compress each squared-bond core against the running canonical `R` as it is built, so the peak bond stays `~r·r_keep` < `r²`. Result equals build-then-round to tolerance.
- [ ] 3.2 Rewrite `linalg::matmul` as a B-transposed, cache-blocked loop (integer block constant; no float literals; no `Default` bound); result identical to the naive product.
- [ ] 3.3 Reuse ping-pong scratch buffers in the `inner`/`norm` transfer-matrix contraction (no per-site allocation); results bit-identical.
- [ ] 3.4 (Optional) Randomized range-finder `svd_truncated` variant behind the `Truncation` policy; deterministic Jacobi stays default. Reference: block-Krylov arXiv:2308.01480 / arXiv:2504.04989.
- [ ] 3.5 Tests: fused-vs-build-then-round agreement; blocked-vs-naive matmul equality; inner/norm unchanged; (if 3.4) randomized SVD reconstructs to tolerance.

## 4. Benchmarks and finalization

- [ ] 4.1 Extend `bench_tensor_train_core.rs` with a high-interior-bond `round`: deterministic vs randomized rows (confirm the randomized speedup).
- [ ] 4.2 Extend `bench_tensor_train_cross.rs` with a maxvol-vs-greedy row; add a blocked-vs-naive `matmul` micro-bench and a fused-vs-build-then-round `hadamard_rounded` row.
- [ ] 4.3 Update the crate README `### CausalTensorTrain Performance` table with the before/after numbers and a note on the randomized/greedy trade-offs.
- [ ] 4.4 Run `make format && make fix`; confirm `unsafe_code = "forbid"`, no `dyn`, no lib-code macros, no concrete float literals; whole-workspace `cargo test` green; clippy `--all-targets` clean.
- [ ] 4.5 Run `openspec validate optimize-tensor-network` and reconcile any spec drift before `/opsx:apply` completion.
