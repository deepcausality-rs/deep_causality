# brcd-bootstrap Specification

## Purpose
TBD - created by archiving change brcd-bootstrap. Update Purpose after archive.
## Requirements
### Requirement: Learn a CPDAG from observational data via BOSS

The system SHALL provide a structure-learning entry point in `deep_causality_algorithms::causal_discovery` that takes an observational data matrix (`CausalTensor<T>`, `T: RealField`, rows = samples, columns = variables) and a seed, and returns the learned CPDAG as a `deep_causality_topology::MixedGraph` over those variables. The learner SHALL be a Best-Order Score Search (BOSS) port of the reference `boss.py`, producing a graph that plugs directly into `brcd_run` without conversion.

#### Scenario: Recovers the equivalence class of a known chain

- **WHEN** BOSS runs on samples drawn from a linear-Gaussian chain `X → Y → Z`
- **THEN** it returns the CPDAG of that chain's Markov equivalence class (the undirected/oriented skeleton matching `X — Y — Z` with the correct v-structures)

#### Scenario: Output is a valid CPDAG

- **WHEN** BOSS returns a learned graph
- **THEN** the graph is a valid CPDAG (acyclic directed projection, no partially-directed contradictions) accepted as input by `brcd_run`

### Requirement: BIC-from-covariance score over existing tensor primitives

The learner SHALL score node families with the linear-Gaussian SEM-BIC computed from the sample covariance only, at the **higher-is-better** sign the order search maximizes: `−½·n·ln(σ²) − ½·ln(n)·(|PA|+1)·λ`, where `σ²` is the node's marginal variance when it has no parents and the ridge Schur-complement conditional variance `Σ_yy − Σ_yP (Σ_PP + εI)⁻¹ Σ_Py` otherwise. The score SHALL be built on `deep_causality_tensor::CausalTensorStatsExt::sample_covariance` and `conditional_variance`. No other score (RKHS cross-validated, RKHS marginal, or BDeu) SHALL be implemented.

The score SHALL use the **correct, higher-is-better** sign — the convention of causal-learn's `local_score_BIC_from_cov` and of the BOSS the ICML paper runs ("default setting of BOSS from causal-learn", Appendix D) — **not** the vendored reference `LocalScoreFunction.py`, whose negated `n·ln(σ²) + ln(n)·|PA|·λ` is inverted relative to BOSS's maximizing search (it learns the empty graph on a clean chain). A constant (zero-variance) column SHALL be guarded so it cannot produce `ln(0)`, consistent with the reference dropping zero-variance metrics.

#### Scenario: Score is higher-is-better and orders parent sets correctly

- **WHEN** a node family `(i, PA)` is scored on a fixed covariance matrix and sample count
- **THEN** the value equals `−½·n·ln(conditional_variance(i, PA, ε)) − ½·ln(n)·(|PA|+1)·λ` to within floating-point tolerance, and a parent set that strictly reduces the conditional variance enough to overcome its penalty scores strictly higher than the empty set (so grow/shrink and the order search add genuine parents)

#### Scenario: The covariance is computed once

- **WHEN** the learner scores many candidate parent sets during the search
- **THEN** it derives every score from a single sample covariance matrix rather than re-reading the raw data per family

### Requirement: Grow-shrink tree caches the best parent set per prefix

The learner SHALL use a grow-shrink tree (GST) per variable that, given the variables preceding it in the current order, returns that variable's best-scoring parent set and score, caching partial results across the search. Grow SHALL add parents that strictly improve the score; shrink SHALL remove parents that strictly improve the score.

#### Scenario: Grow then shrink reaches the local optimum

- **WHEN** a GST traces a prefix
- **THEN** it returns the parent set and score reachable by greedily adding improving parents and then removing improving parents, identical to the reference `GST.trace`

### Requirement: Best-order search iterates to a fixpoint

The learner SHALL optimize the variable order by moving each variable to the position that maximizes the total score (the sum of per-variable GST scores), repeating until no move improves the total. The search SHALL be deterministic for a fixed seed and initial order.

#### Scenario: A fixed seed yields a deterministic CPDAG

- **WHEN** BOSS is run twice on the same data with the same seed
- **THEN** it returns the identical learned CPDAG both times

### Requirement: DAG-to-CPDAG conversion reuses Meek completion

The learner SHALL convert the DAG implied by the final order into a CPDAG by orienting its unshielded colliders (v-structures) and then completing under Meek's rules. The Meek completion SHALL reuse `brcd_meek::meek_complete` and the unshielded-collider machinery in `brcd_validity`; only the v-structure-orientation pass SHALL be new code.

#### Scenario: Compelled edges survive, reversible edges are undirected

- **WHEN** a learned DAG is converted to a CPDAG
- **THEN** edges in every member of the Markov equivalence class are directed and edges that reverse across the class are left undirected, matching the reference `dag2cpdag`

### Requirement: Optional bootstrap CPDAG-uncertainty outer loop

The system SHALL provide, as a separable stage, a dedicated `brcd_run_bootstrap` entry point (a sibling of `brcd_run`, not a flag on it) that resamples the observational data, learns a CPDAG per resample, weights the distinct CPDAGs by their frequency-corrected posterior, and combines the per-CPDAG root-cause posteriors (paper Equations 8–10; the `BRCD-B10` / `BRCD-B100` variants). When the bootstrap is not requested, callers use `brcd_run` directly — a single learned (or supplied) CPDAG with no resampling — which is left unchanged.

#### Scenario: Bootstrap combines weighted CPDAGs

- **WHEN** the bootstrap variant is requested with `B` resamples
- **THEN** it learns up to `B` CPDAGs, assigns each a normalized weight, and returns a root-cause ranking marginalized over them

#### Scenario: Default path uses a single CPDAG

- **WHEN** the bootstrap is not requested
- **THEN** the driver learns one CPDAG and ranks against it, with no resampling

### Requirement: No new dependencies and policy compliance

The BOSS port SHALL add no external or numeric crate dependency, relying only on `deep_causality_tensor`, `deep_causality_topology`, and the existing `brcd_meek` / `brcd_validity`. It SHALL contain no `unsafe` code and no dynamic dispatch, consistent with the workspace `unsafe_code = "forbid"` lint and the no-external-numerics policy.

#### Scenario: Builds under the workspace lints

- **WHEN** the crate is built with the workspace lint configuration
- **THEN** the BOSS module compiles with no `unsafe`, no `dyn`, and no new third-party crate in `Cargo.toml`

### Requirement: Verification by structure and downstream ranking

Because BOSS is a heuristic search whose exact CPDAG depends on tie-breaking and search order, verification SHALL NOT require byte-exact reproduction of the reference CPDAG. Verification SHALL instead assert (a) the learned CPDAG's skeleton and v-structures on a fixed dataset and seed, and (b) that feeding the learned CPDAG into `brcd_run` reproduces the published root-cause ranking on a real-world case (e.g. Petshop), relying on the algorithm's I-MEC invariance to make the ranking robust to a Markov-equivalent learned CPDAG.

The verification README and the end-to-end example SHALL carry a **caveat about the reference's own correctness**: this port deliberately uses the *correct* (higher-is-better) BIC sign, whereas the vendored Python reference uses the inverted sign (it learns the empty CPDAG on a clean chain) and additionally has the posterior-ranking underflow bug. The Python real-world outputs may therefore themselves be corrupted by these two bugs. If the correctly-signed port does **not** reproduce a given reference ranking, that divergence is evidence that the reference output is wrong, not the port. Confirming this for a bug report SHALL be done by **temporarily re-introducing the reference bug(s)** (running BOSS with the inverted sign, and/or the `exp`-then-`argsort` ranking) behind a clearly-marked test-only switch, checking whether the port then matches the reference, and documenting the result — never by changing the production score to the wrong sign.

#### Scenario: Structural match on a fixed seed

- **WHEN** BOSS learns a CPDAG from a committed dataset with a fixed seed
- **THEN** the learned skeleton and v-structures match the committed expected structure

#### Scenario: End-to-end ranking reproduces the reference

- **WHEN** the learned CPDAG is passed to `brcd_run` on a real-world case whose published root-cause ranking is known
- **THEN** the produced ranking reproduces that reference ranking

