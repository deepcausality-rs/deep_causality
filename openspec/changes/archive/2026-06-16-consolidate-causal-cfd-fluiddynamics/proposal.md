## Why

CFD-relevant code is scattered across `deep_causality_physics` (`src/theories/fluid_dynamics`,
the DEC NS solver) and `examples/avionics_examples` (Taylor–Green, lid cavity, graded MMS,
cylinder wake/validation). Preparing for the CFD community challenge — and delivering the
expressiveness goal of `cfd-crate.md` — requires consolidating this into a dedicated
`deep_causality_cfd` crate and lifting the boundary/solver/theory primitives into a **`Flow`
domain-specific language**, peer to the existing `CausalFlow` (`deep_causality_core`) and
`CausalDiscovery` (`deep_causality_discovery`) DSLs.

The two prerequisite changes have **both archived**, so the substrate is ready:

- `add-boundary-zone-abstraction` (archived `2026-06-14`) makes every boundary a first-class
  composable zone (the DSL's "composable inflow and outflow zones"), with the net-flux projection
  behind the open-boundary zones.
- `add-cut-cells-and-immersed-boundaries` (archived `2026-06-15`) provides the cut-cell geometry,
  immersed bodies, the first uncertain data zone, and the validated solver substrate — itself closed
  using the zone abstraction's open boundaries (D2/D3).

With boundaries and geometry already composable terms, and the DEC solver already stateless,
precision-generic, and marching through `CausalFlow`, the DSL has a uniform vocabulary to compose.

## What Changes

- **`deep_causality_cfd` crate (NEW, `causal-cfd-consolidation`).** `publish = false`. Structure per
  `cfd-crate.md`: `src/{errors,extensions,traits,types,solvers,theories}`, `tests/` mirroring
  `src/`, `benches/`, `examples/`, `validation/`, `docs/{prompts,openspecs}`. Built from the
  no-external-dependency line (std/file IO may pull external deps only where genuinely needed, e.g.
  example output). **Move** the fluid-dynamics theories and the DEC solver **out of**
  `deep_causality_physics` (no published back-compat to preserve; downstream importers updated);
  migrate the CFD benches; migrate the validation examples (`cfd_taylor_green`,
  `dec_taylor_green_re1600`, `dec_lid_cavity_re1000`, `dec_graded_mms`, `dec_cylinder_wake`,
  `dec_cylinder_validation`) into `validation/`.
- **`Flow` DSL (NEW, `fluiddynamics-dsl`).** A composition surface for fluid simulations, built on
  the HKT/algebra foundation (`deep_causality_haft`, `deep_causality_num`) like `CausalFlow`. It
  composes: **solvers** (a solver uses a theory and/or physics kernels to solve a designated case);
  **boundary zones** (the archived `BoundaryZone` terms — inflow, outflow, walls, immersed bodies,
  uncertain sources); **multi-physics coupling** (`.couple`, a between-step physics pipeline lowering
  onto the `CausalFlow` bind passthrough, as in `multi_physics_pipeline` / `flight_envelope_monitor`);
  **counterfactuals** (`.counterfactual` and `.continue_with().intervene` on material, mesh,
  temperature, or a dynamic law — as in `causal_counterfactual_examples` / `quantum_counterfactual`);
  and **control flow** (loop / either / corrective, the arrow-algebra iterator as in
  `corrective_ddos_detector`). The DSL integrates with `CausalFlow` (complex physics between steps /
  pre-processing) and `CausalDiscovery` (e.g. SURD to isolate contributing factors), and is easy to
  extend with new solvers and physics stages via the HKT mechanism.
- **Theory vs solver split (NEW, `fluiddynamics-dsl`).** A *theory* is a Navier–Stokes regime reused
  across solvers; a *solver* uses a theory and/or kernels to solve one designated case. The DSL makes
  both first-class and composable.
- **Precision as a parameter (REQUIREMENT).** Every solver and theory runs natively at any supported
  float precision (`f32`, `f64`, `Float106`) with **zero downcasting** in the solver; each example
  fixes a `FloatType` alias.
- **Examples in the DSL (NEW, `fluiddynamics-dsl`).** The crate's examples are written in the `Flow`
  DSL. Each example splits configuration from wiring: a `config.rs` owns every solver/mesh/zone/seed
  configuration (built with type-state builders), and a `main.rs` plugs the imported configuration
  into the Flow pipeline. Larger multi-physics examples further decompose into per-physics
  sub-process modules wired into one holistic coupling.

## Sequencing

- **Prerequisites:** both archived (gate satisfied). This change is now implementable.
- **Phased delivery.** **Phase 1:** the crate scaffold, the theory/solver migration (move-out), the
  benches, the six validation examples lifted into generic reusable solvers, and the minimal Flow
  surface (mesh / solver / zones / seed / march / observe). **Phase 2:** the advanced Flow surface —
  multi-physics `.couple`, counterfactual `.counterfactual` / `.continue_with`, control flow
  (either / loop / corrective), and the `CausalDiscovery` (SURD) tap.

## Non-Goals

- No behavioural change to the migrated solvers/theories beyond the crate move and the DSL surface
  (numerics are preserved; the prerequisites supply the new capabilities).
- No new IO/mesh formats beyond what migrated examples already need.
- Publishing the crate (it is `publish = false`).
