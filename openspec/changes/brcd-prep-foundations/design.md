## Context

CDL (`deep_causality_discovery`) hosts one discovery algorithm, SURD, and hardcodes its result type and single-dataset flow throughout the typestate pipeline. A source-level study of the BRCD reference (`openspec/notes/rca/BRCD.md`, §12 roadmap, §13 placement, §14 verified spec) shows BRCD reuses a stack of general pieces: SPD linear solves, sample covariance, a PDAG/CPDAG with Meek orientation, Markov-equivalence-class sizing, and a discovery pipeline that carries two datasets and returns an algorithm-specific result. None of these are BRCD-specific. This change builds them once, as shared layers, so the later BRCD change is a composition.

This change depends on the prerequisite `real-field-discovery`, which generifies `deep_causality_algorithms` and `deep_causality_discovery` over `RealField`. That refactor lands first; everything here is therefore written against a precision parameter `T: RealField`, not `f64`.

Constraints carried from the repo: no external numeric crates, `unsafe_code = "forbid"` workspace-wide, static dispatch (no `dyn`), one-type-one-module layout, and full test coverage of new code.

## Goals / Non-Goals

**Goals:**
- Public, tested numeric primitives reusable repo-wide, each generic over `T: RealField`: the SPD CG solve in `deep_causality_sparse`, and mean/covariance, `logsumexp`, Gaussian log-density, and conditional variance on `deep_causality_tensor`'s `CausalTensorStatsExt`. (These are statistics over data, so they live with the tensor type — not in `deep_causality_num`, which defines number systems and the algebra tower.)
- A shared causal-graph layer (PDAG/CPDAG, Meek, validity, MEC size) sized so a second algorithm starts above it.
- A discovery pipeline that carries two aligned datasets and returns an algorithm-specific result, with SURD behavior unchanged.
- Each layer compiles and is unit-testable on its own, in dependency order (note §12: L0, L1, L2, L3-primitive, L4-trivial, L6).

**Non-Goals:**
- The BRCD estimator itself (F-node augmentation, per-regime/pooled scoring, ranking, the microservice adapter). That is the next change.
- BOSS structure learning and the full Wienöbst uniform MEC sampler. Petshop-gated, deferred. Only the trivial arcs-only MEC case is built here.
- Any change to SURD's numerical results.

## Decisions

**D1. Lift the existing `cg_solve` to a public home rather than reimplement.**
The conjugate-gradient solver in `deep_causality_topology` is already tested and matrix-free; its own doc-comment proposes lifting it into `deep_causality_sparse`. Move it to `deep_causality_sparse`, expose it publicly, and have `deep_causality_topology` consume the public version. *Alternative considered:* reimplement (~50 LOC) in the algorithms crate. Rejected: duplicates tested code and would drift from the topology consumer.

**D2. The graph *type* is `deep_causality_topology::MixedGraph`; the causal *operations* live in `deep_causality_algorithms::causal_discovery::brcd`.** *(Revised — the `mixed-graph` change landed and is archived; owner accepted the dependency, option (a).)*
The CPDAG/PDAG representation is the typed-endpoint `MixedGraph` delivered by the `mixed-graph` change in `deep_causality_topology`, which already provides the directed-arc projection (`parents`/`children`), undirected neighbors, per-kind edge enumeration, the endpoint invariant, and a self-contained topological sort / cycle check. This change adds the causal *operations* — Meek orientation, the unshielded-collider validity check, and MEC sizing — as modules under **`deep_causality_algorithms::causal_discovery::brcd`** (`meek`, `validity`, `mec`), consuming `MixedGraph`. **All BRCD-related code (these operations now, the estimator later) is rooted at the `brcd` module** per the owner's directive. *Dependency decision (owner, resolved):* `deep_causality_algorithms` takes a dependency on `deep_causality_topology`. Topology is heavyweight, but the operations are causal-discovery algorithms that belong in the algorithms crate, and BRCD genuinely needs the graph type + operations, so the dependency is justified. *Alternatives rejected:* (b) host the operations in topology beside `MixedGraph` — keeps the dep out of the prep but splits BRCD's code across two crates; (c) relocate `MixedGraph` to a lighter crate — premature.

**D3. Reuse `MixedGraph`'s built-in directed-arc projection; no `ultragraph` dependency for this layer.** *(Revised.)*
`MixedGraph` natively exposes `parents`/`children` over the directed-arc projection and a self-contained Kahn `topological_sort` plus DFS `has_cycle`/`find_cycle`, ignoring undirected/bidirected/circle edges. The Meek and validity operations read those directly; no separate directed-graph store is needed. *Consequence:* task 2.1 (the `ultragraph` parents accessor, verified present) is no longer on the consumption path — the layer uses `MixedGraph::parents` instead. `ultragraph` remains the right tool for large directed graphs elsewhere, just not here.

**D4. Generalize the discovery result with a closed enum, not generics or `dyn`.**
Replace the precision-generic `SurdResult<T>` (delivered by the prerequisite `real-field-discovery`) with a `DiscoveryOutcome<T>` enum (one `Surd(SurdResult<T>)` variant now; algorithm variants added later). The analyzer and formatter match on it. *Alternatives considered:* (a) a second generic result parameter threaded through the typestate (on top of the precision `T`), rejected for the type churn it spreads across every state; (b) a boxed trait object, rejected because it violates the repo's static-dispatch rule. The enum mirrors the existing `CausalDiscoveryConfig` enum, so the pattern is already established in this crate and stays `dyn`-free. Precision `T` comes from `real-field-discovery`; this change adds only the algorithm dimension on top.

**D5. Carry two datasets, with the second optional, rather than a regime-labeled single tensor.**
The discovery stage accepts a primary dataset and an optional second aligned dataset. SURD reads the first and is unaffected; a two-dataset algorithm reads both. *Alternative considered:* a single tensor with a regime-indicator column. Rejected: it bakes the F-node convention into the data model before the algorithm that needs it exists, and complicates SURD's path. Keeping datasets separate lets the later algorithm construct its own regime indicator.

**D6. Shape the MEC API for the full sampler, implement only the trivial case.**
`mec` exposes equivalence-class size and a representative-DAG accessor. For an arcs-only (fully directed) input the class has size one and the representative is the input. Implement exactly that, behind an API that a later uniform sampler can satisfy without a signature change. The verified spec (note §14.2) confirms the arcs-only case is all the OB/Sock Shop target needs.

**D7. Numeric code is generic over `RealField`; structural code stays precision-free.**
Every numeric component this change adds (the SPD solver, mean/covariance, `logsumexp`, Gaussian log-density, conditional variance) is generic over `T: RealField`, so it composes with `real-field-discovery` and the rest of the math stack at any precision. The causal-graph layer (PDAG/CPDAG, Meek, validity, MEC sizing) carries **no** floating-point scalars — it is structure over node indices and edges — so it takes no precision parameter and composes with every `T` by construction. *Alternative considered:* add a `T` parameter to the graph types for uniformity. Rejected: it would be a phantom parameter on code that holds no scalars, adding noise and falsely implying the graph depends on precision. Genericity follows the data: real where there are reals, absent where there are none.

## Risks / Trade-offs

- **Lifting `cg_solve` widens the blast radius into `deep_causality_topology`.** → Topology re-imports the public solver; its existing tests guard the Hodge/Laplacian consumers; the move is additive, so behavior is preserved.
- **Generalizing the discovery result is a BREAKING source-API change in `deep_causality_discovery`.** → The owner controls the whole repo; the SURD result path is behavior-preserving; the migration is mechanical and covered by tasks. No downstream external consumers depend on the old signatures.
- **De-duplicating covariance against topology's `Manifold` could shift topology output.** → Expose one covariance primitive and have `Manifold` delegate to it; guard with topology's existing tests; keep the numerical definition identical (same `ddof`, same accumulation order where it matters).
- **The PDAG/`ultragraph` seam.** `ultragraph` models directed graphs; the PDAG adds undirected edges. → The PDAG owns undirected edges and uses `ultragraph` only for the arc projection and traversal; confirm or add the parents accessor during implementation.
- **Scope creep into the full MEC sampler.** → Hard-bound to the trivial arcs-only case (D6); the API shape absorbs the sampler later without churn.

## Open Questions

- ~~`cg_solve` final home~~ **Resolved:** lives in `deep_causality_sparse` (per its own doc). The stats primitives (`logsumexp`, Gaussian log-density, conditional variance) live on `deep_causality_tensor`'s `CausalTensorStatsExt`, not `deep_causality_num` — `num` is the number-systems/algebra crate and stays unmodified by this change.
- Exact carrier for the second dataset (a small struct versus an enum on the discovery input); settled in specs/tasks.
- ~~Whether `ultragraph` already exposes a parents/predecessors accessor~~ **Resolved/moot:** the causal-graph layer now consumes `deep_causality_topology::MixedGraph`, which provides `parents` directly; `ultragraph` is not used here (D3).
- ~~`deep_causality_algorithms` → `deep_causality_topology` dependency (D2)~~ **Resolved (owner): option (a)** — accept the dependency. The Meek/MEC/validity operations live in `deep_causality_algorithms::causal_discovery::brcd`, consuming `topology::MixedGraph`; all BRCD code is rooted at that `brcd` module.
