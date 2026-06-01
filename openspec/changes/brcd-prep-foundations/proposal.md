## Why

CDL ships one causal-discovery algorithm, SURD. Issue [#598](https://github.com/deepcausality-rs/deep_causality/issues/598) asks for a second, BRCD, and over time a collection of state-of-the-art methods. A source-level study of the BRCD reference (recorded in `openspec/notes/rca/BRCD.md`) shows that most of what BRCD needs is not specific to BRCD: shared numeric primitives, a causal-graph layer, and a discovery pipeline that can carry two datasets and return algorithm-specific results.

This change builds those foundations as reusable layers. A subsequent change then implements the BRCD estimator as a thin composition on top. Building the shared base first keeps BRCD small and lets every later algorithm (RCD, RCG, PC, GES) start from the same layer instead of re-deriving it.

**Prerequisite: `real-field-discovery`.** This change depends on `real-field-discovery`, which generifies `deep_causality_algorithms` and `deep_causality_discovery` over `RealField`. That refactor lands first, so the foundations here are precision-generic from the start. Consequently the discovery-result generalization below is expressed as `DiscoveryOutcome<T>` over the already-generic precision, not over `f64`.

## What Changes

- **Numeric primitives**, each generic over `T: RealField`. Expose the conjugate-gradient SPD solver as public API (it is `pub(crate)` in `deep_causality_topology` today and already generic; its own doc invites the lift). Add sample mean and covariance over `CausalTensor<T>`. Add `logsumexp`, a Gaussian log-density, and a conditional-variance (covariance Schur complement) helper. No new external crates. These compose with `real-field-discovery` and the wider math stack at any precision.
- **Causal-graph layer** (new shared module in `deep_causality_algorithms`, beside `surd`). A PDAG/CPDAG type with directed arcs and undirected edges; directed-graph operations (parents, topological order, acyclicity) reusing `ultragraph`; Meek orientation rules in a single implementation that serves both DAG-to-CPDAG conversion and PDAG completion; the unshielded-collider validity check; and Markov-equivalence-class size with the trivial arcs-only case. This layer is structural and carries no scalars, so it takes no precision parameter and composes with every `T` by construction.
- **Discovery pipeline generalization** (`deep_causality_discovery`). Carry two aligned datasets (normal and anomalous) through the discovery stage instead of one. Generalize the precision-generic `SurdResult<T>` (from `real-field-discovery`) into an algorithm-specific result enum `DiscoveryOutcome<T>` across the `CausalDiscovery` trait, the `CausalDiscoveryConfig` enum, the `WithCausalResults` state, the analyzer, and the formatter. Accept a user-supplied domain graph as input. SURD keeps its current behavior and output. **BREAKING** at the source-API level of `deep_causality_discovery` (trait and state signatures change); behavior-preserving for SURD.

Deferred to the subsequent BRCD change (not built here): the F-node augmented-DAG construction, per-regime versus pooled Gaussian scoring, `brcd_update`/`brcd_helper`, candidate ranking, and the microservice call-graph adapter.

Deferred further (Petshop-gated, a separate concern): BOSS structure learning and the full Wienöbst uniform MEC sampler. The prep needs only the trivial arcs-only equivalence-class case.

## Capabilities

### New Capabilities
- `linalg-numeric-primitives`: a public SPD conjugate-gradient solver, sample mean and covariance over `CausalTensor`, `logsumexp`, Gaussian log-density, and conditional variance via the covariance Schur complement.
- `causal-graph`: PDAG/CPDAG representation, directed-graph operations, Meek orientation, unshielded-collider validity, and Markov-equivalence-class size (trivial arcs-only case).
- `discovery-pipeline`: a CDL discovery stage that carries two aligned datasets, returns an algorithm-specific result enum `DiscoveryOutcome<T>`, and accepts a user-supplied domain graph, with SURD preserved.

### Modified Capabilities
<!-- None. No existing capability spec in openspec/specs/ governs these areas at the requirements level; the cross-crate edits (ultragraph accessor, topology cg_solve lift) are additive and recorded under Impact. -->

## Impact

- **Crates touched.** `deep_causality_num` (numeric primitives), `deep_causality_tensor` (mean/covariance), the chosen home for `cg_solve` (`deep_causality_sparse` or `deep_causality_num`), `deep_causality_topology` (cg_solve lifted out then consumed back; covariance de-duplicated), `ultragraph` (add a parents/predecessors accessor if absent), `deep_causality_algorithms` (new `causal_discovery::{graph, mec}` modules), `deep_causality_discovery` (pipeline generalization).
- **Public API.** `deep_causality_discovery` discovery trait, config enum, state, analyzer, and formatter signatures change (BREAKING). `cg_solve` becomes public in its new home.
- **Dependencies.** None added. Repo-wide `unsafe_code = "forbid"` preserved; static dispatch retained.
- **Prerequisite change.** `real-field-discovery` must land first (it makes both crates generic over `RealField`). The result generalization here builds on that as `DiscoveryOutcome<T>`.
- **Out of scope.** The BRCD estimator, BOSS, and the full uniform MEC sampler are explicitly excluded and tracked for later changes.
