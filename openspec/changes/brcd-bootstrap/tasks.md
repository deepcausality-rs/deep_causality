# Tasks

Each stage is independently testable and must leave the workspace green
(build + test + clippy + fmt). The owner commits; the agent never commits.

## 1. BIC-from-covariance score

- [x] 1.1 Add the BOSS preprocessor files **inside** the `brcd` module (prefixed `brcd_boss_`, e.g. `brcd_boss_score`), starting with `brcd_boss_score`; wire them into the crate. (BOSS is a BRCD preprocessor, not a sibling discovery algorithm.)
- [x] 1.2 Add a `BossConfig<T>` (`brcd_boss_config`) carrying `ridge_eps` (default `1e-6`), `bic_lambda` (default `2`), and `seed`.
- [x] 1.3 Implement `family_bic(cov: &CausalTensor<T>, n: usize, node: usize, parents: &[usize], cfg) -> T` at the **higher-is-better** sign: no-parent case `−½·n·ln(cov[i,i]) − ½·ln(n)·bic_lambda`; with-parents case `−½·n·ln(conditional_variance(node, parents, ridge_eps)) − ½·ln(n)·(|parents|+1)·bic_lambda`, using `CausalTensorStatsExt::{sample_covariance, conditional_variance}`. Guard zero-variance columns (a constant metric must not produce `ln(0)`), per the paper's singular-matrix handling. (Sign corrects the vendored reference bug; see design D2.)
- [x] 1.4 Test that the score is higher-is-better and orders parent sets correctly (a variance-reducing parent set scores strictly above the empty set), cover the no-parent and parented branches, a near-singular parent block (ridge keeps it finite), and a zero-variance column (guarded, finite).

## 2. Grow-shrink tree (GST)

- [x] 2.1 Implement `GstNode`/`Gst` over `Vec<usize>` parents with `grow` (add strictly-improving parents, keep improving branches, sort), `shrink` (remove strictly-improving parents to a fixpoint), and `trace(prefix)`, scoring through a borrow of the `boss_score` + covariance.
- [x] 2.2 Test that `trace` reproduces the reference grow-then-shrink parent set/score on a small graph, and that repeated traces reuse the cached tree (no extra score calls).

## 3. Best-order search

- [x] 3.1 Implement the order optimization: per-variable `better_mutation` (move to the score-maximizing position), iterating sweeps until the total GST score stops improving; seed any tie-breaking/restart with `Xoshiro256::from_seed(cfg.seed)`.
- [x] 3.2 Test determinism (same seed + data → same final order) and that the search recovers the correct order on a known linear-Gaussian chain.

## 4. DAG → CPDAG

- [x] 4.1 Build the DAG from the final order (each node's parents from its GST trace) as a `MixedGraph<()>`.
- [x] 4.2 Implement the v-structure orientation pass (orient `X → Z ← Y` for non-adjacent `X`,`Y`), then call `brcd_meek::meek_complete`; reuse `brcd_validity` collider helpers.
- [x] 4.3 Test that conversion keeps compelled edges directed and leaves reversible edges undirected, matching `dag2cpdag` on small DAGs (chain, fork, collider, two-parent).

## 5. `boss_learn` entry point

- [x] 5.1 Implement `boss_learn(data: &CausalTensor<T>, cfg: &BossConfig<T>) -> Result<MixedGraph<()>, BrcdError>` composing stages 1–4 (covariance once → GSTs → order search → DAG → CPDAG).
- [x] 5.2 Test end to end: BOSS on samples from `X → Y → Z` returns the chain's CPDAG; the result is a valid CPDAG accepted by `get_configurations_multi`.

## 6. Driver integration

- [x] 6.1 Change `brcd_run`'s `cpdag` parameter to `Option<&MixedGraph<N>>`; `Some` → unchanged path; `None` → `boss_learn` on the normal dataset, then the existing phases.
- [x] 6.2 Update all internal call sites (verification examples, tests, benches) to pass `Some(&cpdag)`.
- [x] 6.3 Test that `Some(cpdag)` reproduces the prior result exactly, and that `None` learns a CPDAG and returns a ranking.

## 7. Verification (structural + end-to-end)

- [ ] 7.1 Add a synthetic structural verification: BOSS on a committed dataset with a fixed seed; assert the learned skeleton + v-structures match a committed expected structure.
- [ ] 7.2 Add an end-to-end verification: feed the learned CPDAG into `brcd_run` on a real-world case (e.g. Petshop) and assert it reproduces the published root-cause ranking; deliver both as individually-runnable examples under `examples/verification/brcd`.
- [ ] 7.3 Document in the verification README that BOSS reproduction is structural + downstream-ranking, not byte-exact, and why (heuristic search; I-MEC invariance). Add the **reference-correctness caveat**: the port uses the correct BIC sign while the Python reference is sign-inverted (empty CPDAG on a clean chain) and has the ranking-underflow bug, so the reference's own outputs may be wrong; if the correctly-signed port does not reproduce a reference ranking, that is evidence the reference is wrong. To confirm for a bug report, temporarily re-introduce the reference bug(s) behind a test-only switch (inverted-sign BOSS and/or `exp`-then-`argsort` ranking), check whether the port then matches the reference, and record the finding — never ship the wrong sign.

## 8. Bootstrap CPDAG-uncertainty (separable, last)

- [x] 8.1 Add a `bootstrap` flag/count to the config; resample the observational data `B` times, learn a CPDAG per resample, and dedupe distinct CPDAGs with frequency-corrected weights.
- [x] 8.2 Marginalize the per-CPDAG root-cause posteriors (paper Eq. 8–10); the non-bootstrap path uses a single learned CPDAG.
- [x] 8.3 Test the weighting and marginalization on a tiny case; confirm the default (no bootstrap) is unaffected.

## 9. Hygiene and gates

- [x] 9.1 Workspace builds and tests pass with default features and with `--features parallel`; clippy and fmt clean; no `unsafe`, no `dyn`, no new third-party crate.
- [x] 9.2 `openspec validate brcd-bootstrap` passes; draft a commit message summarizing the change for the owner to commit.
