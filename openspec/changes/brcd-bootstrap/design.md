## Context

The archived `brcd-estimator` ranks root causes given a *supplied* CPDAG (e.g. a reversed service map). BRCD's Bayesian inference is defined over that CPDAG — it is a hard structural prerequisite, not optional context. Systems without a reliable service map (Petshop, and most real deployments) have no CPDAG to supply. The reference `brcd_helper` handles this by learning the CPDAG from the pre-failure observational data with **BOSS** (Best-Order Score Search) before running the estimator. This change ports BOSS so `brcd_run` can run from observational data alone.

The relevant reference files are `ctx/next/brcd/BRCD/boss.py` (404 lines), `LocalScoreFunction.py` (the `local_score_BIC_from_cov` function only), and causal-learn's `gst.GST` (76 lines) and `DAG2CPDAG` (166 lines). Reconciliation context is in `openspec/notes/brcd/BRCD.md` §4a.

## Goals / Non-Goals

**Goals:**
- Learn a CPDAG from an observational `CausalTensor<T>` and return it as a `deep_causality_topology::MixedGraph`, ready to feed the estimator.
- Make `brcd_run` accept `Option<&MixedGraph<N>>`: `Some` → supplied CPDAG; `None` → learn via BOSS, then rank.
- Reuse the existing tensor/topology/Meek primitives; add no external or numeric dependency.
- Provide the optional bootstrap CPDAG-uncertainty outer loop as a separable later stage.

**Non-Goals:**
- The RKHS (cross-validated/marginal) and BDeu scores in `LocalScoreFunction.py` — never used by the real-world default.
- Forest-KDE; any change to SURD or the existing BRCD estimator numerics.
- Byte-exact reproduction of causal-learn's learned CPDAG (BOSS is a heuristic search; see Risks).

## Decisions

**D1. Module placement — inside `brcd`, not a sibling.** BOSS is **not a standalone discovery algorithm** in this codebase; it is BRCD's optional structure-learning **preprocessor** (it learns the CPDAG that `brcd_run` requires when none is supplied). Its files therefore live **inside** `deep_causality_algorithms::causal_discovery::brcd`, prefixed `brcd_boss_` to sit beside the other `brcd_*` files without colliding (`brcd_config.rs` already exists): `brcd_boss_config`, `brcd_boss_score`, `brcd_boss_gst`, `brcd_boss_search`, `brcd_boss_cpdag`, and the entry `brcd_boss_learn`. Tests mirror the source tree under `tests/causal_discovery/brcd/` as `boss_*_tests.rs` (the `brcd_` prefix is dropped in test filenames, matching the existing `brcd_augment.rs → augment_tests.rs` convention). The public entry point `boss_learn(data: &CausalTensor<T>, config) -> Result<MixedGraph<()>, BrcdError>` returns the learned CPDAG, re-exported from the `brcd` module. Errors reuse the existing `BrcdError`/`BrcdErrorEnum` (extended with a `StructureLearning`-style variant only if a genuinely new failure case appears).

**D2. Score = BIC-from-cov only, over existing tensor primitives — at the *correct* (higher-is-better) sign.** Compute the `p×p` sample covariance **once** with `CausalTensorStatsExt::sample_covariance`. A family `(i, PA)` scores as the linear-Gaussian SEM-BIC that the search **maximizes**:
- no parents: `−½ · n · ln(cov[i,i]) − ½ · ln(n) · λ`;
- with parents: `−½ · n · ln(conditional_variance(i, PA, ε)) − ½ · ln(n) · (|PA| + 1) · λ`,
where `conditional_variance` is the ridge Schur complement `Σ_yy − Σ_yP (Σ_PP + εI)⁻¹ Σ_Py` already in `deep_causality_tensor`. The reference uses `ε = 1e-6` and `λ = 2`; these are config fields. This is the only score implemented. Per-node-constant terms do not affect the `argmax` over parent sets, so the proportional form `−n·ln(σ²) − ln(n)·|PA|·λ` is equivalent for the GST/order search; the full form above is what the score *function* returns.

> **Deviation from the reference — and why (corrects a reference bug).** The vendored reference `ctx/next/brcd/BRCD/LocalScoreFunction.py::local_score_BIC_from_cov` returns the *negated* score `n·ln(σ²) + ln(n)·|PA|·λ` (**lower = better fit**), but its BOSS (`boss.py`) **maximizes** — `GSTNode.grow`/`shrink` keep parents that *increase* the score and `better_mutation` does `argmax`. The two disagree in sign, so the vendored search maximizes *worst* fit and **adds no parents**: run on a clean linear-Gaussian chain `X→Y→Z` (600 samples) it learns the **empty graph** for every seed, whereas causal-learn's own (correctly signed) BOSS learns `X—Y—Z`. The ICML paper itself states (Appendix D, "Real-world data experiment") that it uses **"the default setting of … BOSS from causal-learn"**, and causal-learn's installed `local_score_BIC_from_cov` is the *higher-is-better* form — so the vendored lower-is-better copy is a local divergence that would not reproduce the paper's Petshop results. This port therefore implements the **correct, higher-is-better** sign (matching causal-learn and the paper), not the vendored copy. Owner-approved (2026-06-03). Reference: Andrews et al., NeurIPS 2023 (BOSS + grow-shrink trees); ICML 2026 BRCD paper, Appendix D.
>
> Also per Appendix D: BOSS "throws an error due to a singular matrix … by removing any metrics that have zero variance." We fold this in by guarding zero-variance columns (a constant metric would otherwise give `ln(0)`); such metrics are confirmed never to be true root causes.

**D3. Grow-shrink tree (GST) ported faithfully.** A `GstNode { add, grow_score, shrink_score, branches, remove }` tree over `Vec<usize>` parents, with `grow` (add improving parents, keep improving branches, sort), `shrink` (remove improving parents to a fixpoint), and `trace(prefix)` matching `gst.py`. The score is supplied as a closure/borrow over the covariance-backed `boss_score`, so the GST carries no numerics. `forbidden` starts as `[vertex]`; required-edge knowledge is carried but, as in the reference, not enforced in the trace.

**D4. Order search to a fixpoint, deterministic per seed.** Maintain an order (initialized topologically/identity); for each variable, use `better_mutation` to move it to the position maximizing the total GST score; iterate until no move improves the total. Any randomness (tie-breaking, restart) uses the project RNG (`deep_causality_rand::Xoshiro256`) seeded from `config.seed`, so a fixed seed yields a deterministic CPDAG.

**D5. DAG → CPDAG reuses Meek.** From the final order build the DAG, orient its unshielded colliders (the only genuinely new pass — find `X → Z ← Y` with `X`,`Y` non-adjacent), then call `brcd_meek::meek_complete` to propagate. Validity/collider helpers come from `brcd_validity`. The result is a `MixedGraph` — the exact type `brcd_run` consumes.

**D6. Driver integration: `brcd_run` takes `Option<&MixedGraph<N>>`.** When `Some`, behaviour is unchanged. When `None`, `brcd_run` calls `boss_learn` on the normal dataset (the pre-failure observational data) to obtain the CPDAG, then runs the existing phases. This is a small **breaking** signature change; the ~handful of internal call sites (verification examples, tests, benches) add `Some(...)`. The learned-CPDAG node type is `()`, matching `MixedGraph<()>`.

**D7. Bootstrap CPDAG-uncertainty as a separable stage.** A later sub-stage resamples the observational data `B` times, learns a CPDAG per resample, weights distinct CPDAGs by frequency-corrected posterior, and marginalizes the per-CPDAG root-cause posteriors (paper Eq. 8–10; `BRCD-B10`/`B100`). Built behind the same entry point via a config flag; the non-bootstrap path uses a single learned CPDAG. Staged last so the core (single learned CPDAG) ships and verifies first.

## Risks / Trade-offs

- **BOSS is a heuristic search; exact CPDAG reproduction is fragile.** Tie-breaking and branch ordering in the GST and the order search make the learned CPDAG sensitive to implementation details, so byte-matching causal-learn is not a sound acceptance test. *Mitigation:* verify (a) the learned skeleton + v-structures on a fixed dataset/seed, and (b) end-to-end that the learned CPDAG, fed to `brcd_run`, reproduces a published root-cause ranking. The estimator's **I-MEC invariance** (already demonstrated: the Rust estimator samples different DAGs than Python yet matches the ranking) makes the downstream ranking robust to a Markov-equivalent learned CPDAG.
- **Enumeration/search cost.** The order search is polynomial per sweep but multi-sweep; for the experiment scale (≤ ~50 variables) it is tractable, and all scoring derives from one covariance matrix. *Trade-off accepted;* bound and `log` any pathological case rather than silently degrade.
- **Breaking signature change to `brcd_run`.** Small and mechanical (`Some(...)` at call sites), and it gives one clean entry point rather than a parallel API. The owner approved making `cpdag` optional.
- **Reproducing the reference ε/λ exactly.** The score's `ε = 1e-6` ridge and `λ = 2` penalty must match the reference; they are surfaced as config fields and pinned to the reference defaults.
- **The reference's own output may be wrong (two bugs).** The vendored Python reference combines a *sign-inverted* BIC (D2 — learns the empty CPDAG on a clean chain) with the posterior-ranking underflow bug (`exp`-then-`argsort`, see `openspec/notes/brcd/brcd_python_ranking_bug.md`). The correctly-signed port may therefore **diverge from the reference precisely because the reference is wrong**. *Mitigation / verification stance:* if the port does not reproduce a reference ranking, treat that as evidence against the reference, not the port. To confirm for a bug report, temporarily re-introduce the reference bug(s) behind a clearly-marked **test-only** switch (inverted-sign BOSS and/or `exp`-then-`argsort` ranking) and check whether the port then matches; document the result. The production path never ships the wrong sign. This is recorded in the verification requirement and README.
