## Context

The two prerequisite changes leave the substrate ready: boundaries are composable `BoundaryZone`
terms with open-boundary support, cut-cell geometry and immersed bodies exist, the solver is
stateless and precision-generic, and the march already composes through `CausalFlow`
(`add-cut-cells-and-immersed-boundaries` Group C). This change consolidates the scattered CFD code
into a `causal_cfd` crate and lifts these primitives into a `FluidDynamics` DSL. This document is a
**design sketch for review**; the full specification is derived (OSPX) after the prerequisites
archive, per `causal_cfd.md`'s order of next steps.

## Goals / Non-Goals

- **Goal:** one `causal_cfd` crate (`publish = false`); a `FluidDynamics` DSL peer to `CausalFlow`
  / `CausalDiscovery`; composable solvers, zones, multi-physics, counterfactuals, control flow;
  precision as a parameter; examples written in the DSL.
- **Non-Goal:** changing migrated numerics; new IO; the prerequisites' capabilities (zones, open
  boundaries, cut cells) — those are inherited.

## Decisions

### D1: The DSL wraps `CausalFlow`, it does not replace it

The CFD march is already a `CausalFlow` (`iterate_n` + `bind` + `intervene`). `FluidDynamics` is a
fluid-typed facade over that monad: a `FluidDynamics::case(...)` builder assembles a domain
(geometry + zones + theory + solver), and `.step`/`.iterate`/`.branch`/`.intervene` lower onto the
`CausalFlow` combinators. Multi-physics composition reuses the `bind` passthrough (as in
`multi_physics_pipeline`); counterfactuals reuse `.intervene` (material/mesh/temperature swaps,
as in `causal_counterfactual_examples`); control flow reuses loop/either (as in
`causal_correction_examples`). Integration with `CausalDiscovery` (e.g. SURD to isolate
contributing factors) is a downstream tap that consumes solver output.

### D2: Composition via the HKT foundation; new solvers are cheap to add

Solvers and zones compose through the `deep_causality_haft` HKT/algebra foundation, so adding a new
solver is implementing a small trait surface, not re-plumbing the DSL. The `BoundaryZone` trait
(prerequisite) is the zone seam — composed **statically** (typed tuple/cons, no `dyn`, per
AGENTS.md); a `Solver`/`Theory` trait pair is the solver seam, likewise static.

### D3: Theory vs solver

A *theory* is a Navier–Stokes regime (incompressible DEC, compressible, Stokes, Euler) reused
across solvers; a *solver* uses a theory and/or physics kernels to solve one designated case
(lid cavity, Taylor–Green, cylinder wake). Both are first-class in the DSL; theories migrate from
`deep_causality_physics/src/theories/fluid_dynamics`.

### D4: Precision as a parameter

Every solver/theory is generic over `R: RealField` with no `f64` downcast (the uncertain-reduction
generalization removed the last cast islands). Each example fixes a `FloatType` alias
(`f32`/`f64`/`Float106`) so precision is a one-line switch.

### D5: Crate layout and migration

Per `causal_cfd.md`: `src/{errors,extensions,traits,types,solvers,theories}`, `tests/` mirroring
`src/`, `benches/`, `examples/` (DSL-written), `validation/` (migrated reference cases),
`docs/{prompts,openspecs}`. `publish = false`. The migration preserves numerics; it is a move plus
a DSL surface, gated by re-running the migrated validation cases to identical results.

## Risks / Trade-offs

- **Migration blast radius** (moving theories/solver out of `deep_causality_physics`): mitigated by
  preserving numerics and gating on identical validation results; downstream importers updated.
- **DSL over-abstraction**: mitigated by deriving the DSL from the two working DSLs' patterns and
  from the already-composable march, not inventing new combinators.
- **Scope**: this is the largest change; it is taken only after both prerequisites archive.

## Open Questions

1. The exact shape of the static `Solver`/`Theory` HKT seam (refined at implementation).
2. Whether the DEC NS solver/theories are **moved out** of `deep_causality_physics` entirely or
   the physics crate keeps a minimal fluid surface. Default: move out (no back-compat to preserve —
   nothing is published; downstream is updated freely).
