## Context

Both prerequisite changes have archived (`add-boundary-zone-abstraction` `2026-06-14`,
`add-cut-cells-and-immersed-boundaries` `2026-06-15`), so the substrate is ready: boundaries are
composable `BoundaryZone` terms with open-boundary support, cut-cell geometry and immersed bodies
exist, the DEC solver is stateless and precision-generic (`DecNsSolver`/`DecNsRate` are already
generic over `R: DecNsScalar = RealField + FromPrimitive + …`, every constant lifted via
`R::from_f64`), and the march already composes through `CausalFlow`. The f64-isms and the
orchestration sprawl live in the **examples**, not the solver — so "precision as a parameter" is
mostly lifting example orchestration into generic solvers plus a `FloatType` alias, not rewriting
numerics.

This change consolidates the scattered CFD code into a `deep_causality_cfd` crate and lifts these
primitives into a `Flow` DSL. The design below was reviewed as worked example code (per
`cfd-crate.md`'s order of next steps) before being recorded here. The capability slugs
`causal-cfd-consolidation` and `fluiddynamics-dsl` are retained as OpenSpec identifiers; the
user-facing crate is `deep_causality_cfd` and the DSL is named **Flow**.

## Goals / Non-Goals

- **Goal:** one `deep_causality_cfd` crate (`publish = false`); a `Flow` DSL peer to `CausalFlow` /
  `CausalDiscovery`; composable solvers, zones, multi-physics coupling, counterfactuals, control
  flow; precision as a parameter; examples written in the DSL with a `config.rs` / `main.rs` split.
- **Non-Goal:** changing migrated numerics; new IO; the prerequisites' capabilities (zones, open
  boundaries, cut cells) — those are inherited.

## Decisions

### D1: Flow is a fluid-typed facade that lowers onto `CausalFlow`; it does not replace it

The CFD march is already a `CausalFlow` (`iterate_n`/`iterate_until` + `bind` + `intervene`). `Flow`
is a fluid-typed facade over that monad. A `Flow::case(...)` builder assembles a domain (mesh +
geometry + zones + theory + solver + seed); `.march_for`/`.march_until`/`.observe`/`.probe` describe
the run; `.couple`/`.counterfactual`/`.continue_with` describe coupling and intervention. These
lower onto the `CausalFlow` combinators: the march onto `iterate_until`/`iterate_n` (the
arrow-algebra iterator, as in `corrective_ddos_detector`), multi-physics onto the `bind`/`bind_or_error`
passthrough (as in `multi_physics_pipeline` / `flight_envelope_monitor` / `sensor_processing`),
counterfactuals onto `.intervene`, and control flow onto `branch_with` / `either`. Integration with
`CausalDiscovery` (e.g. SURD to isolate contributing factors) is a downstream tap that consumes
solver output.

### D2: Owned declarative `Case`; the solver is materialized inside `run`

`DecNsSolver` borrows the manifold (`&'m Manifold<…>`). To keep lifetimes clean, `Flow::case(...)`
produces a **fully-owned declarative description** (`Case<R>`) that holds no borrows. At `.run()` the
mesh, geometry, manifold, and solver are materialized as locals, the march executes, and only an
owned `Report<R>` escapes — the borrows never leave `run()`. This sidesteps self-referential structs.
A `Case` is cheap to clone, so the same case seeds the factual run and every counterfactual.

### D3: Composition via the HKT foundation; new solvers and physics are cheap to add

Solvers, zones, and physics stages compose through the `deep_causality_haft` HKT/algebra foundation,
all **statically** (typed tuple/cons, no `dyn`, per AGENTS.md). The `BoundaryZone` trait (inherited)
is the zone seam. A `Solver`/`Theory` trait pair is the solver seam. A `PhysicsStage<R>` trait is the
coupling seam. Adding a new solver or a new coupled physics is a small trait impl, not a change to the
DSL core.

### D4: `.couple` is the multi-physics seam — a between-step pipeline of modular stages

`.couple(Coupling)` registers a between-step physics pipeline run once per timestep around the CFD
step as the anchor. A `Coupling<R>` is a statically-composed pipeline of `PhysicsStage<R>` values
that lowers onto the `CausalFlow` `bind_or_error` passthrough. Because everything composes over the
error algebra (`PropagatingEffect`/`bind`), a stage, a sub-process, and a whole coupling are all
first-class values: built and unit-tested independently, stored in variables, and wired with
`then`/`compose`. Large multi-physics examples decompose into per-physics sub-process modules
(`stages/thermal.rs`, `stages/structural.rs`, …), each exposing a `sub_process(&cfg) -> Coupling`
seam, wired in `main.rs` — the `flight_envelope_monitor` / `sensor_processing` module discipline
applied to physics. A coupled thermo × fluid × stress march reuses real kernels (`heat_diffusion`,
the DEC NS step, and the `triple_hkt_stress_field` stress walk); a temperature-dependent viscosity
`ν(T)` feedback makes the fluid dynamics change with the evolving context — the dynamic-causality
premise. Control flow (a corrective `Guard::branch_when`) is just another stage in the chain.

### D5: Counterfactuals — two flavors; intervene on dynamics, not just constants

The headline counterfactual intervenes on the **dynamics** — a law, schedule, or field — so the
effect is a *trajectory of divergence*, not a scalar (a plain constant swap is a parametric sweep,
not a dynamic counterfactual). Two flavors, both lowering onto `CausalFlow` `.intervene`:

- **Shared-seed:** `case.counterfactual(intervention)` returns a sibling case sharing the background
  (mesh/zones/seed = the abduction) with one `do(·)` applied; both run and are contrasted.
- **Continuation:** `report.continue_with().intervene(intervention)` abducts the developed field, so
  the expensive coupled solve runs **once** and many cheap counterfactual scenarios branch on top —
  "compute once, report many scenarios."

Interventions are a closed, discoverable `Intervene::*` vocabulary spanning static terms
(`material`, `reynolds`, `mesh`) and dynamic laws (`thrust_schedule`, `heat_gradient`,
`wall_temperature`, `prandtl`).

### D6: Theory vs solver

A *theory* is a Navier–Stokes regime (incompressible DEC, compressible, Stokes, Euler) reused across
solvers; a *solver* uses a theory and/or physics kernels to solve one designated case (lid cavity,
Taylor–Green, cylinder wake/validation). Both are first-class in the DSL; theories migrate from
`deep_causality_physics/src/theories/fluid_dynamics`.

### D7: Precision as a parameter

Every solver/theory is generic over `R: RealField` with no `f64` downcast. Each example fixes a
`FloatType` alias (`f32`/`f64`/`Float106`) so precision is a one-line switch. SI bookkeeping scalars
in coupling/context are lifted into `R` via `R::from_f64`, never downcast.

### D8: Crate layout, migration, and the config/main split

Per `cfd-crate.md`: `src/{errors,extensions,traits,types,solvers,theories}`, `tests/` mirroring
`src/`, `benches/`, `examples/` (DSL-written), `validation/` (migrated reference cases),
`docs/{prompts,openspecs}`. `publish = false`. The theories and DEC solver are **moved out** of
`deep_causality_physics` entirely (no published back-compat; downstream importers updated). The
migration preserves numerics — a move plus a DSL surface, gated by re-running the migrated validation
cases to identical results. Each example splits a `config.rs` (the `FloatType` alias plus every
solver/mesh/zone/seed configuration, built with type-state builders modeled on the Discovery
`CdlBuilder`) from a `main.rs` (pure Flow wiring that plugs in the imported configuration).

## Solver refactoring to the Flow interface

The migrated solvers already satisfy precision-as-a-parameter (`R: DecNsScalar`), but their *shape*
does not yet match the Flow interface: configuration is fused with the manifold borrow, the ambient
(ν, inflow, body force) is fixed at construction, the theory is not first-class, and each case's
orchestration is trapped in an example `main`. Today the theory layer is pointwise free functions
(`incompressible_ns_rhs_kernel`, `euler_momentum_rhs_kernel`, …) while the DEC marcher (`DecNsRate`)
is a separate concrete struct with `nu: R` fixed at construction (no setter) and `DecNsSolver<'m,D,R>`
holding both the owned config and the borrowed `rate: DecNsRate<'m>`. The following refactors land in
Phase 1 so the Phase 2 Flow features (`.couple`, counterfactuals) plug in without re-plumbing.

| # | Current (physics crate) | Target (Flow interface) |
|---|---|---|
| R1 | Regime evaluators are free functions; `DecNsRate` is a concrete struct | A `FluidTheory<R>` trait abstracts the marching rate; the DEC incompressible rate implements it, and the pointwise regime kernels are wrapped behind it for the verification solvers |
| R2 | `DecNsSolver<'m,D,R>` fuses owned config (`dt`, `cg_options`, `cfl_*`, `lift`) with the borrowed `rate: DecNsRate<'m>` | An owned `DecNsConfig<R>` (no borrow) with a type-state builder; the bound marcher is materialized from `(&manifold, zones, config)` at `run` (D2) |
| R3 | Ambient is fixed at construction: `DecNsRate.nu: R` is immutable; the inflow value is baked into the zone | A per-step `Ambient<R>` context channel (ν, freestream U, body force g) the marcher reads each step; `ambient_from_context()` wires the solver to take ambient from the march context — the seam `.couple` and dynamic counterfactuals drive |
| R4 | The march is exposed only as monolithic `run_n` / `run_until` methods | A per-step advance value (the `Rk4` arrow / `dec_ns_step` effect) over a typed `MarchState` / `CoupledField<R>` that the Flow march drives via `iterate_until`, interleaving between-step coupling stages |
| R5 | Each case (lid cavity, cylinder, TGV, MMS) hand-rolls seeding, the loop, and diagnostics in its example `main`, partly in concrete `f64` | Standalone generic case-solvers (`LidCavitySolver`, `CylinderSolver`, `TaylorGreenSolver`, `MmsSolver`, …) — thin compositions over the DEC core + zones + seed + diagnostics, each with a config + type-state builder and the Flow interface |
| R6 | Strouhal / drag-mean / Ghia compare / vortex detection / dissipation sampling live in example `main`s | Generic `R` diagnostics in a solver-side `observe` module that `.observe` / `.probe` consume |

R3 (the `Ambient<R>` channel) and R4 (the march-as-value) are load-bearing: they make the solver
dynamics *context-driven*, which is what `.couple` (ν(T) feedback) and dynamic-law counterfactuals
require. They land in Phase 1 even though the coupling pipeline that exploits them is Phase 2. The
no-coupling path SHALL reproduce the construction-fixed numerics bit-for-bit, so the migration gate
(identical validation results) still holds. The `FluidTheory<R>` trait (R1) must span both the
pointwise regime form (used by the MMS verification solver) and the DEC-native realization.

## Phasing

- **Phase 1:** crate scaffold + theory/solver move-out + benches + the solver refactoring R1–R6
  (theory trait, owned config + materialization, per-step ambient channel, march-as-value,
  case-solvers, generic diagnostics) + the six validation examples lifted into those generic solvers +
  the minimal Flow surface (mesh / solver / zones / seed / march / observe), with the `config.rs` /
  `main.rs` split.
- **Phase 2:** the advanced Flow surface — `.couple` multi-physics, `.counterfactual` /
  `.continue_with` counterfactuals, control flow (either / loop / corrective), and the
  `CausalDiscovery` (SURD) tap, plus the showcase multi-physics examples.

## Risks / Trade-offs

- **Migration blast radius** (moving theories/solver out of `deep_causality_physics`): mitigated by
  preserving numerics, gating on identical validation results, and updating downstream importers.
- **DSL over-abstraction**: mitigated by deriving the DSL from the working DSLs' patterns
  (`CausalFlow`, Discovery builder, the multi-physics/sensor/corrective examples) and from the
  already-composable march, not inventing new combinators.
- **Scope**: the largest change; mitigated by the Phase 1 / Phase 2 split landing working value first.

## Open Questions

1. The exact `PhysicsStage<R>` / `CoupledField<R>` trait surface and the `Solver`/`Theory` HKT seam
   (refined during Phase 2 implementation).
