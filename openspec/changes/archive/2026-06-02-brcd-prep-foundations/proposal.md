## Why

CDL ships one causal-discovery algorithm, SURD. Issue [#598](https://github.com/deepcausality-rs/deep_causality/issues/598) asks for a second, BRCD, and over time a collection of state-of-the-art methods. A source-level study of the BRCD reference (recorded in `openspec/notes/rca/BRCD.md`) shows that most of what BRCD needs is not specific to BRCD: shared numeric primitives, a causal-graph layer, and a discovery pipeline that can carry two datasets and return algorithm-specific results.

This change builds those foundations as reusable layers. A subsequent change then implements the BRCD estimator as a thin composition on top. Building the shared base first keeps BRCD small and lets every later algorithm (RCD, RCG, PC, GES) start from the same layer instead of re-deriving it.

**Scope (revised).** This change delivers the two *pure foundation* layers only: numeric primitives (`linalg-numeric-primitives`) and causal-graph operations (`causal-graph`). The CDL discovery-pipeline generalization that was originally section 3 here (the breaking `DiscoveryOutcome<T>` / two-dataset / domain-graph rework of `deep_causality_discovery`) has been **moved out** into its own change `cdl-discovery-pipeline`, sequenced *after* the BRCD estimator is built and verified — its shape is dictated by what BRCD actually needs at the seam, so it is designed against the real interface, not a reconstructed one. The rationale and requirements are preserved in `openspec/notes/cdl-integration.md`.

**Prerequisite: `real-field-discovery`.** This change depends on `real-field-discovery`, which generifies `deep_causality_algorithms` and `deep_causality_discovery` over `RealField`. That refactor lands first, so the foundations here are precision-generic from the start. Consequently the discovery-result generalization below is expressed as `DiscoveryOutcome<T>` over the already-generic precision, not over `f64`.

## What Changes

- **Numeric primitives**, each generic over `T: RealField`. Expose the conjugate-gradient SPD solver as public API in `deep_causality_sparse` (it is `pub(crate)` in `deep_causality_topology` today and already generic; its own doc invites the lift). In `deep_causality_tensor`, extend the `CausalTensorStatsExt` extension with sample mean and covariance, `logsumexp`, a Gaussian log-density, and a conditional-variance (covariance Schur complement) helper — these are statistics computed over tensor data, so they live with the tensor type, not in the number-systems crate `deep_causality_num`. No new external crates. These compose with `real-field-discovery` and the wider math stack at any precision.
- **Causal-graph operations** (new modules in `deep_causality_algorithms::causal_discovery`, beside `surd`). The CPDAG/PDAG *type* is `deep_causality_topology::MixedGraph` — delivered by the separate, now-archived `mixed-graph` change — which already provides directed arcs + undirected edges, the parents/children projection, and a built-in topological sort / acyclicity check. This change adds only the causal *operations* over it: Meek orientation rules in a single implementation serving both DAG-to-CPDAG conversion and PDAG completion; the unshielded-collider validity check; and Markov-equivalence-class size with the trivial arcs-only case. These operations are structural and carry no scalars, so they take no precision parameter and compose with every `T` by construction. (`deep_causality_algorithms` gains a dependency on `deep_causality_topology`; see the design's open question.)
- **Discovery pipeline generalization** (`deep_causality_discovery`) — **moved out of this change.** Carrying two aligned datasets, generalizing `SurdResult<T>` into a `DiscoveryOutcome<T>` enum, and accepting a user-supplied domain graph are now the dedicated change `cdl-discovery-pipeline`, landed after the BRCD estimator is verified. Rationale and requirements: `openspec/notes/cdl-integration.md`.

Deferred to the subsequent BRCD change (not built here): the F-node augmented-DAG construction, per-regime versus pooled Gaussian scoring, `brcd_update`/`brcd_helper`, candidate ranking, and the microservice call-graph adapter.

Deferred further (Petshop-gated, a separate concern): BOSS structure learning and the full Wienöbst uniform MEC sampler. The prep needs only the trivial arcs-only equivalence-class case.

## Capabilities

### New Capabilities
- `linalg-numeric-primitives`: a public SPD conjugate-gradient solver, sample mean and covariance over `CausalTensor`, `logsumexp`, Gaussian log-density, and conditional variance via the covariance Schur complement.
- `causal-graph`: Meek orientation, unshielded-collider validity, and Markov-equivalence-class size (trivial arcs-only case), all over `deep_causality_topology::MixedGraph` (the PDAG/CPDAG representation, delivered by the archived `mixed-graph` change).

### Modified Capabilities
<!-- None. No existing capability spec in openspec/specs/ governs these areas at the requirements level; the cross-crate edits (ultragraph accessor, topology cg_solve lift) are additive and recorded under Impact. -->

## Impact

- **Crates touched.** `deep_causality_sparse` (public `cg_solve` home), `deep_causality_tensor` (the `CausalTensorStatsExt` numeric stats primitives: mean/covariance, `logsumexp`, Gaussian log-density, conditional variance), `deep_causality_topology` (cg_solve lifted out then consumed back; covariance de-duplicated; **provides `MixedGraph`** via the archived `mixed-graph` change), `deep_causality_algorithms` (new `causal_discovery::brcd::{meek, validity, mec}` operation modules; **gains a dependency on `deep_causality_topology`** to consume `MixedGraph`). `deep_causality_discovery` is **not** touched here — the pipeline generalization moved to `cdl-discovery-pipeline`. `ultragraph` is **no longer** modified by this change (the causal-graph layer uses `MixedGraph`'s built-in projection, not `ultragraph`). `deep_causality_num` is **not** modified — it stays the number-systems/algebra crate.
- **Public API.** `cg_solve` becomes public in its new home. No breaking change in this change (the `deep_causality_discovery` API break moved to `cdl-discovery-pipeline`).
- **Dependencies.** None added. Repo-wide `unsafe_code = "forbid"` preserved; static dispatch retained.
- **Prerequisite change.** `real-field-discovery` must land first (it makes both crates generic over `RealField`). The result generalization here builds on that as `DiscoveryOutcome<T>`.
- **Out of scope.** The BRCD estimator, BOSS, and the full uniform MEC sampler are explicitly excluded and tracked for later changes.
