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

- [x] 3.1 `fit_ridge` (port of `_fit_ridge` L312) in `brcd::gaussian`: `β = solve(XᵀX + λI, Xᵀy)` via the shared `linalg::solve_spd` (ridge on every column incl. intercept, matching the reference), `σ² = ‖resid‖²/max(n−p,1)` floored 1e-12; `RidgeFit<T>{beta, sigma2}` + `predict`. `RIDGE_DEFAULT = 1e-4`.
- [x] 3.2 `gaussian_single_expert_logdensity` composes `deep_causality_tensor::gaussian_log_density` (exact `_normal_logpdf_1d`) via the per-row residual `rᵢ = zᵢ − μᵢ` → `gaussian_log_density(0, σ²)`; ports the finite-row masking + sample-mean fallback + parentless branch (L455–480).
- [x] 3.3 Transform ladder `none/log/log1p` with original-scale Jacobian (`transform_and_jacobian`, L279) + the `log → log1p → yeojohnson` auto-downgrade (`effective_transform`, L357–381); `Yeojohnson` is selected but returns `GaussianError::YeojohnsonUnsupported` (deferred, D7) so the deferral is surfaced, not silent.
- [x] 3.4 Tests (15): parentless density vs the closed form `−½(ln2π + (z−μ)²)`; sharp-line recovery with parents; `fit_ridge` line recovery + variance floor (λ=0 exact-fit) + shape errors; log/log1p Jacobian values; the downgrade ladder; auto-downgrade-keeps-finite; yeojohnson-surfaces-unsupported; f32/f64 sweep.

## 4. F-integration (mixture of experts) + discrete Dirichlet

- [x] 4.1 `gaussian::gaussian_family_logdensity` ports the 3-branch `gaussian_conditional_postpred_rowwise` (L324–L552): F∈parents → per-regime ridge-Gaussian (with the `n≤p` small-sample guard, L433); F∉parents → two-expert mixture through the stage-2 gate, combined via a stable `logaddexp` (`_logsumexp2`); F absent → single expert. `transform_parents` applies the node's effective transform to continuous parents (no Jacobian, L409–421); `GaussianFamilyConfig<T>` carries it + the gate config. Empirical-prior gate fallback on gate-fit failure.
- [x] 4.2 `dirichlet::dirichlet_logdensity` — discrete Dirichlet posterior-predictive, prequential per parent-configuration stream, `α₀ = α*/K`, `α* = 5.0` default (L596/L659); returns per-row log-probabilities. `DirichletError::{EmptyData, DimensionMismatch, ZeroCardinality, StateOutOfRange}`.
- [x] 4.3 Tests (7 family + 5 dirichlet): F-absent ≡ single expert; per-regime closed form (`logpdf(·;μ,2) = −½(ln4π+½)`); per-regime linear fit (n>p); mixture-all-anomalous collapses to the present expert; mixture finite + deterministic; `transform_parents` changes the fit; dirichlet parentless + grouped closed forms, marginal-likelihood ≤ 0, error paths, f32/f64.

## 5. F-node augmentation + cut-configuration enumeration

- [x] 5.1 `brcd_augment::f_node_indicator(n_normal, n_anomalous)` builds the `0/1` FNODE column for the concatenated frame; `augmented_graph(config, roots)` adds the `FNODE` vertex + `FNODE → root` arcs to a completed config, yielding the structural `MixedGraph<()>` the MEC stage sizes/samples.
- [x] 5.2 `brcd_augment::get_configurations_multi` ports `getConfigurations_multi` (L1213): collects undirected edges incident on the candidate set, enumerates all `2^E` orientations, validates each via the landed `is_valid_configuration` (Meek-complete + acyclicity + no-new-unshielded-collider) — returns the **Meek-completed** valid configs (folds in `sampleAugmentedGraphs`'s `to_complete_pdag`). Bounded at `MAX_CONFIG_EDGES = 16` → `BrcdErrorEnum::ConfigSpaceTooLarge`; out-of-range target → `NodeOutOfBounds`.
- [x] 5.3 `brcd_cache::FamilyCache<T>` + `family_key(node, parents)` (sorts/dedups parents) — `get_or_try_insert_with` computes each unique family once and reuses it (D5).
- [x] 5.4 Tests (11 augment + 5 cache): arcs-only → 1 config; single incident edge → 2; new-unshielded-collider excluded (3 of 4); shielded-collider-allowed-but-cycle-pruned (3 of 4); determinism; both error paths; FNODE augmentation (single/multi root, bounds); cache compute-once / order-independent key / error-caches-nothing.

## 6. Posterior assembly + ranking (driver) + public API

- [ ] 6.1 `BrcdConfig` (seed, family kind = continuous/discrete, `node_transform`, `transform_parents`, `num_root_causes_candidates` = k, prior, enumeration bound, optional weighted-CPDAG list for the future bootstrap path) and `BrcdResult<T>` exposing `ranks` (the ranked candidate ordering, best first — mirrors the reference `result["ranks"]`) over the posterior; no `dyn`.
- [ ] 6.2 `brcd_update` (L1756): per root, sum cached log-factors into `log P(D|G)`, add `log(mec_size/Σ)`, `logsumexp` over the root's DAGs, sum over rows, add `log(prior)`, normalize → posterior over roots.
- [ ] 6.3 `brcd_helper` (L1863) supplied-CPDAG branch only: validate aligned columns + required CPDAG, run `brcd_update`, rank. Errors for missing CPDAG / misaligned datasets. Public entry point exported from `brcd::mod` and the crate root.
- [ ] 6.4 Tests: end-to-end single-root continuous; determinism under fixed seed; missing-CPDAG and misaligned-datasets error paths.

## 7. Verification tier 1 — golden fixtures vs the reference posterior

- [ ] 7.1 **Primary fixture: the `X → Y → Z` toy** (`ctx/next/brcd/README.md` / `ctx/next/example.txt`). Undirected CPDAG `arcs=[], edges=[(X,Y),(Y,Z)]`; anomaly perturbs `p(Y|X)`; `node_transform="none"`; expected `ranks == ['Y','X','Z']`. Commit the Python-generated `df_obs`/`df_a` as CSV golden inputs (numpy PCG64 is not bit-reproducible in Rust, so the *data* is the fixture, not the seed) with provenance. Assert Rust BRCD returns `['Y','X','Z']`.
- [ ] 7.2 Capture reference posteriors from `ctx/next/brcd/brcd.py` on fixed inputs for small fixtures covering every estimator mode (F∈parents, F∉parents mixture, F absent, discrete); commit as golden data with provenance. Pin the tolerance `ε`.
- [ ] 7.3 Tests: the toy ranking is exact; per-root log-posteriors within `ε`; one case per mode.

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
