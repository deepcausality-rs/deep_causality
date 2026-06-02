Each stage below ends green: the crate builds, all new tests pass, `cargo clippy` is clean (rewrite, don't `#[allow]`), `cargo fmt --check` is clean, no `dyn`, no `unsafe`, no external numeric crate. Draft a commit message at each stage gate for the owner to commit. All code is rooted at `deep_causality_algorithms::causal_discovery::brcd`. Line refs are into `ctx/next/brcd/`.

## 1. MEC engine — exact AMO enumeration (replaces the trivial placeholder)

- [x] 1.1 Ported `BRCD/mcs_num.py` `enumerate_amos` (the MCS recursion + `creates_invalid_collider`) to `brcd::mec` over `MixedGraph`. `mec_size = ∏ |AMO(chain component)|` (the AMP/Wienöbst independence result — no full materialization needed for the size). Chain components = connected components of the undirected subgraph; arcs-only → size 1.
- [x] 1.2 Added `mec_sample_dag<N, R: Rng>(&MixedGraph<N>, &mut R) -> Result<MixedGraph<N>, MecError>` drawing a uniform member (independent uniform AMO per component) via a seeded `deep_causality_rand` RNG; `representative_dag` shares the builder (first AMO per component). Enabler: added `Xoshiro256::from_seed(u64)` + crate-root export in `deep_causality_rand` (BRCD needs a reproducible seed, design D8). `deep_causality_algorithms` gains a `deep_causality_rand` dependency.
- [x] 1.3 Bounded both the per-component AMO count and the cross-component product at `MEC_ENUM_BOUND` (100_000) → explicit `MecError::ClassTooLarge { bound }`; no silent truncation. Replaced the `RequiresUniformSampler` placeholder: undirected edges now enumerate; bidirected/circle → `MecError::NotACpdag`; cyclic arcs → `NotAcyclic`.
- [x] 1.4 Rewrote `mec_tests.rs` (14 tests): hand-computed sizes (single edge → 2, path-of-3 → 3, triangle → 6, arc×component, disjoint product), representative is a valid moral member, sample determinism under a fixed seed, every-sample-valid, whole-class coverage, and the three error paths incl. the bound. Plus 2 `from_seed` reproducibility tests in `deep_causality_rand`.

## 2. Logistic-regression gate primitive (the new numeric component)

- [x] 2.1 Implemented ridge-penalized logistic regression by Newton/IRLS in `brcd::gate` (`fit_logistic_gate` → `LogisticGate<T>`/`predict_proba`), generic over `T: RealField + FromPrimitive`, deterministic (fixed `max_iter` + step-tol). The Newton step solves `(ZᵀWZ + Λ)·step = grad` via a new shared dense Cholesky SPD solver `brcd::linalg::solve_spd` (mirrors the proven tensor routine; reused by the stage-3 ridge fit). Objective matches `sklearn` defaults: λ=1.0 on weights, intercept unpenalized.
- [x] 2.2 Single-class label → constant base-rate gate (bias = `logit_clamped(rate)`, weights 0), matching the reference's empirical-prior behaviour; `GateError::{EmptyData, DimensionMismatch, SingularSystem}` signal the caller to fall back (stage 4 wires the empirical prior).
- [x] 2.3 Tests (11 gate + 2 linalg): symmetric 2-point fit vs the **closed form** `w = 2(1−σ(w)) ≈ 0.6749`, separable ordering, 2-feature calibration, both single-class constant gates, determinism, all three error paths, and an f32/f64 precision sweep; `solve_spd` unit-tested on a known 2×2 + identity.

## 3. Ridge-Gaussian family estimator

- [ ] 3.1 `_fit_ridge` (L312): `β = solve(XᵀX + λI, Xᵀy)`, λ = 1e-4, `σ² = resid·resid / max(n−p,1)` floored 1e-12, via the in-place Cholesky on the `p×p` normal equations.
- [ ] 3.2 Per-row family log-density composing `deep_causality_tensor::gaussian_log_density` (exact `_normal_logpdf_1d`).
- [ ] 3.3 Transform ladder none/log/log1p with Jacobian on the original scale + auto-downgrade (L279/L752); `yeojohnson` stubbed behind the same `Transform` enum (D7), recorded as the selected transform.
- [ ] 3.4 Tests: closed-form agreement on a tiny system; variance floor; transform Jacobian correctness; auto-downgrade on non-positive data.

## 4. F-integration (mixture of experts) + discrete Dirichlet

- [ ] 4.1 Three-mode integration (L324–L585): F∈parents → per-regime ridge-Gaussian; F∉parents → two-expert mixture through the gate, combined via `logsumexp`; F absent → single expert.
- [ ] 4.2 Discrete Dirichlet posterior-predictive (prequential), α* = 5.0 (L596/L659).
- [ ] 4.3 Tests: each mode's log-likelihood on a fixture; mixture vs per-regime equivalence in the degenerate gate limit; discrete prequential closed form.

## 5. F-node augmentation + cut-configuration enumeration

- [ ] 5.1 Join: concatenate normal+anomalous into a joint frame with an `FNODE` indicator (0/1) column.
- [ ] 5.2 `getConfigurations_multi` (L1213): enumerate the `2^E` orientations of undirected edges incident on the root-candidate set; validate each via the landed `meek_complete` + acyclicity + `is_valid_configuration` (no-new-unshielded-collider).
- [ ] 5.3 Family log-likelihood cache keyed on `(node, sorted parents)` (D5).
- [ ] 5.4 Tests: enumeration count on a known CPDAG; invalid orientations excluded; cache hit reuse; arcs-only → single configuration.

## 6. Posterior assembly + ranking (driver) + public API

- [ ] 6.1 `BrcdConfig` (seed, family kind, transform, prior, enumeration bound, optional weighted-CPDAG list for the future bootstrap path) and `BrcdResult<T>` (ranked posterior over candidates); no `dyn`.
- [ ] 6.2 `brcd_update` (L1756): per root, sum cached log-factors into `log P(D|G)`, add `log(mec_size/Σ)`, `logsumexp` over the root's DAGs, sum over rows, add `log(prior)`, normalize → posterior over roots.
- [ ] 6.3 `brcd_helper` (L1863) supplied-CPDAG branch only: validate aligned columns + required CPDAG, run `brcd_update`, rank. Errors for missing CPDAG / misaligned datasets. Public entry point exported from `brcd::mod` and the crate root.
- [ ] 6.4 Tests: end-to-end single-root continuous; determinism under fixed seed; missing-CPDAG and misaligned-datasets error paths.

## 7. Verification tier 1 — golden fixtures vs the reference posterior

- [ ] 7.1 Capture reference posteriors from `ctx/next/brcd/brcd.py` on fixed seeds for small fixtures covering every mode (F∈parents, F∉parents mixture, F absent, discrete, undirected-edge CPDAG); commit as golden data with provenance.
- [ ] 7.2 Pin the tolerance `ε` and seed set in the verification fixtures.
- [ ] 7.3 Tests: ranking identical to the reference; per-root log-posterior within `ε`; one case per mode.

## 8. Verification tier 2 — synthetic ground-truth recovery

- [ ] 8.1 In-repo seeded synthetic generator mirroring `experiments/synthetic/data_generation.py` (known injected root cause under a known graph).
- [ ] 8.2 Tests: single-root recovered top-1; multi-root recovered within top-k.

## 9. Verification tier 3 — authoritative oracle cross-check

- [ ] 9.1 On a handful of fixed synthetic datasets + CPDAGs, commit Python-BRCD posteriors (rankings + log-posteriors) as golden, captured offline with provenance.
- [ ] 9.2 Tests: Rust rankings exact, log-posteriors within `ε`, offline/deterministic.

## 10. Verification and hygiene

- [ ] 10.1 `cargo build -p deep_causality_algorithms` and `cargo test -p deep_causality_algorithms`; full coverage of new code.
- [ ] 10.2 Register every new test file in its module tree and `tests/BUILD.bazel`.
- [ ] 10.3 Confirm no external numeric crate added, `unsafe_code = "forbid"` intact, no `dyn` introduced, all randomness seeded.
- [ ] 10.4 `make format && make fix`, then `make build` and `make test`.
- [ ] 10.5 `openspec validate brcd-estimator`; prepare a commit message and request the owner commit.
