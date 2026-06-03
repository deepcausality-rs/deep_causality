## Why

BRCD requires a CPDAG as its structural prerequisite — the partial causal structure its Bayesian inference is defined over. The archived `brcd-estimator` only accepts a *supplied* CPDAG (e.g. a reversed service map). Systems without a reliable map (Petshop, and any deployment lacking a call graph) have no CPDAG to supply, so BRCD cannot run on them at all. The reference resolves this with BOSS, which learns the CPDAG from the pre-failure observational data. Porting BOSS makes BRCD self-sufficient — it can run from observational metrics alone.

## What Changes

- Add a Best-Order Score Search (BOSS) structure learner that produces a CPDAG from an observational data matrix, returning a `deep_causality_topology::MixedGraph` ready to feed `brcd_run`.
- Implement only the `local_score_BIC_from_cov` score (the single score the real-world runs use), at the **correct higher-is-better sign** the search maximizes: `−½·n·ln(conditional_variance) − ½·ln(n)·(|PA|+1)·λ`, built on the existing `deep_causality_tensor` `sample_covariance` + `conditional_variance` (ridge Schur complement). This matches causal-learn's `local_score_BIC_from_cov` and the BOSS the paper runs ("default setting of BOSS from causal-learn", Appendix D), **correcting a sign bug** in the vendored reference `LocalScoreFunction.py` (its negated form learns the empty graph under BOSS's maximizing search). Zero-variance columns are guarded, per the paper's singular-matrix handling. The RKHS and BDeu scores are **out of scope**.
- Port the grow-shrink tree (GST) and the order search (`better_mutation`, the fixpoint) as pure combinatorics over `Vec<usize>` orders/parents.
- Convert the learned DAG to a CPDAG by orienting v-structures and **reusing** `brcd_meek::meek_complete` and the `brcd_validity` unshielded-collider checks; only a small v-structure-orientation pass is new.
- **BREAKING:** change `brcd_run`'s `cpdag` parameter from `&MixedGraph<N>` to `Option<&MixedGraph<N>>`. `Some(cpdag)` is used directly; `None` makes `brcd_run` learn the CPDAG from the observational data via BOSS as a preprocessing step, then rank as usual. Existing call sites pass `Some(&cpdag)`.
- Add the bootstrap CPDAG-uncertainty outer loop (paper Eq. 8–10; the `BRCD-B10`/`B100` variants) as a **separable, later stage** via a dedicated `brcd_run_bootstrap` entry point (a sibling of `brcd_run`), leaving `brcd_run`/`BrcdConfig` unchanged.
- **Out of scope:** RKHS/BDeu scores, Forest-KDE, and any change to SURD or the existing BRCD estimator numerics.
- **No new external or numeric dependency** — the port uses only `deep_causality_tensor`, `deep_causality_topology`, and the existing `brcd_meek`/`brcd_validity`, staying inside `unsafe_code = "forbid"` and the no-external-numerics policy.

## Capabilities

### New Capabilities
- `brcd-bootstrap`: learn a CPDAG from an observational data matrix via BOSS (BIC-from-covariance score, grow-shrink tree, order search, DAG→CPDAG), plus the optional bootstrap CPDAG-uncertainty outer loop and its verification.

### Modified Capabilities
- `brcd-algorithm`: `brcd_run`'s `cpdag` argument becomes optional (**BREAKING**); `None` triggers BOSS structure learning from the observational data before the existing ranking runs.

## Impact

- **New code:** the BOSS preprocessor files **inside** the `brcd` module, prefixed `brcd_boss_` (`brcd_boss_score`, `brcd_boss_gst`, `brcd_boss_search`, `brcd_boss_cpdag`, `brcd_boss_learn`) — BOSS is BRCD's structure-learning preprocessor, not a sibling discovery algorithm — and the `cpdag = None` branch wiring in `brcd_algo`.
- **Reused, unchanged:** `deep_causality_tensor` (`sample_covariance`, `conditional_variance`), `deep_causality_topology::MixedGraph`, `brcd_meek::meek_complete`, `brcd_validity`.
- **APIs:** `brcd_run`'s `cpdag` parameter becomes `Option<&MixedGraph<N>>` (**BREAKING**, small — call sites add `Some(...)`). A new public `boss`-style entry point produces a `MixedGraph` from a data matrix.
- **Dependencies:** none added.
- **Verification:** a new structural + end-to-end suite (not byte-exact against causal-learn; BOSS is a heuristic search, and the downstream ranking is robust to a Markov-equivalent learned CPDAG by I-MEC invariance).
