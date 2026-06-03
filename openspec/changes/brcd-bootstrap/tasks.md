# Tasks

Each stage is independently testable and must leave the workspace green
(build + test + clippy + fmt). The owner commits; the agent never commits.

## 1. BIC-from-covariance score

- [ ] 1.1 Add a `boss` module under `deep_causality_algorithms::causal_discovery` with a `boss_score` submodule; wire it into the crate.
- [ ] 1.2 Add a `BossConfig<T>` carrying `ridge_eps` (default `1e-6`), `bic_lambda` (default `2`), and `seed`.
- [ ] 1.3 Implement `family_bic(cov: &CausalTensor<T>, n: usize, node: usize, parents: &[usize], cfg) -> T`: no-parent case `n·ln(cov[i,i])`; with-parents case `n·ln(conditional_variance(node, parents, ridge_eps)) + ln(n)·|parents|·bic_lambda`, using `CausalTensorStatsExt::{sample_covariance, conditional_variance}`.
- [ ] 1.4 Test against hand-computed BIC values and against `local_score_BIC_from_cov` on a small fixed covariance (within tolerance); cover the no-parent and parented branches and a near-singular parent block (ridge keeps it finite).

## 2. Grow-shrink tree (GST)

- [ ] 2.1 Implement `GstNode`/`Gst` over `Vec<usize>` parents with `grow` (add strictly-improving parents, keep improving branches, sort), `shrink` (remove strictly-improving parents to a fixpoint), and `trace(prefix)`, scoring through a borrow of the `boss_score` + covariance.
- [ ] 2.2 Test that `trace` reproduces the reference grow-then-shrink parent set/score on a small graph, and that repeated traces reuse the cached tree (no extra score calls).

## 3. Best-order search

- [ ] 3.1 Implement the order optimization: per-variable `better_mutation` (move to the score-maximizing position), iterating sweeps until the total GST score stops improving; seed any tie-breaking/restart with `Xoshiro256::from_seed(cfg.seed)`.
- [ ] 3.2 Test determinism (same seed + data → same final order) and that the search recovers the correct order on a known linear-Gaussian chain.

## 4. DAG → CPDAG

- [ ] 4.1 Build the DAG from the final order (each node's parents from its GST trace) as a `MixedGraph<()>`.
- [ ] 4.2 Implement the v-structure orientation pass (orient `X → Z ← Y` for non-adjacent `X`,`Y`), then call `brcd_meek::meek_complete`; reuse `brcd_validity` collider helpers.
- [ ] 4.3 Test that conversion keeps compelled edges directed and leaves reversible edges undirected, matching `dag2cpdag` on small DAGs (chain, fork, collider, two-parent).

## 5. `boss_learn` entry point

- [ ] 5.1 Implement `boss_learn(data: &CausalTensor<T>, cfg: &BossConfig<T>) -> Result<MixedGraph<()>, BrcdError>` composing stages 1–4 (covariance once → GSTs → order search → DAG → CPDAG).
- [ ] 5.2 Test end to end: BOSS on samples from `X → Y → Z` returns the chain's CPDAG; the result is a valid CPDAG accepted by `get_configurations_multi`.

## 6. Driver integration

- [ ] 6.1 Change `brcd_run`'s `cpdag` parameter to `Option<&MixedGraph<N>>`; `Some` → unchanged path; `None` → `boss_learn` on the normal dataset, then the existing phases.
- [ ] 6.2 Update all internal call sites (verification examples, tests, benches) to pass `Some(&cpdag)`.
- [ ] 6.3 Test that `Some(cpdag)` reproduces the prior result exactly, and that `None` learns a CPDAG and returns a ranking.

## 7. Verification (structural + end-to-end)

- [ ] 7.1 Add a synthetic structural verification: BOSS on a committed dataset with a fixed seed; assert the learned skeleton + v-structures match a committed expected structure.
- [ ] 7.2 Add an end-to-end verification: feed the learned CPDAG into `brcd_run` on a real-world case (e.g. Petshop) and assert it reproduces the published root-cause ranking; deliver both as individually-runnable examples under `examples/verification/brcd`.
- [ ] 7.3 Document in the verification README that BOSS reproduction is structural + downstream-ranking, not byte-exact, and why (heuristic search; I-MEC invariance).

## 8. Bootstrap CPDAG-uncertainty (separable, last)

- [ ] 8.1 Add a `bootstrap` flag/count to the config; resample the observational data `B` times, learn a CPDAG per resample, and dedupe distinct CPDAGs with frequency-corrected weights.
- [ ] 8.2 Marginalize the per-CPDAG root-cause posteriors (paper Eq. 8–10); the non-bootstrap path uses a single learned CPDAG.
- [ ] 8.3 Test the weighting and marginalization on a tiny case; confirm the default (no bootstrap) is unaffected.

## 9. Hygiene and gates

- [ ] 9.1 Workspace builds and tests pass with default features and with `--features parallel`; clippy and fmt clean; no `unsafe`, no `dyn`, no new third-party crate.
- [ ] 9.2 `openspec validate brcd-bootstrap` passes; draft a commit message summarizing the change for the owner to commit.
