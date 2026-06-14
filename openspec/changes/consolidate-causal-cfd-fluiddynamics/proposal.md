## Why

CFD-relevant code is scattered across `deep_causality_physics` (`src/theories/fluid_dynamics`,
the DEC NS solver) and `examples/avionics_examples` (Taylor–Green, lid cavity, graded MMS,
cylinder wake). Preparing for the CFD community challenge — and delivering the expressiveness goal
of `causal_cfd.md` — requires consolidating this into a dedicated `causal_cfd` crate and lifting
the boundary/solver/theory primitives into a **`FluidDynamics` domain-specific language**, peer to
the existing `CausalFlow` (`deep_causality_core`) and `CausalDiscovery`
(`deep_causality_discovery`) DSLs.

This is feasible only **after** the two prerequisite changes land:

- `add-boundary-zone-abstraction` makes every boundary a first-class composable zone (the DSL's
  "composable inflow and outflow zones"), with the net-flux projection behind the open-boundary
  zones.
- `add-cut-cells-and-immersed-boundaries` provides the cut-cell geometry, immersed bodies, the
  first uncertain data zone, and the validated solver substrate — and is itself closed using the
  zone abstraction's open boundaries (D2/D3).

With boundaries and geometry already composable terms, the DSL has a uniform vocabulary to
compose; without the prerequisites it would have nothing uniform to build on.

## What Changes

- **`causal_cfd` crate (NEW, `causal-cfd-consolidation`).** `publish = false`. Structure per
  `causal_cfd.md`: `src/{errors,extensions,traits,types,solvers,theories}`, `tests/` mirroring
  `src/`, `benches/`, `examples/`, `validation/`, `docs/{prompts,openspecs}`. Built from the
  no-external-dependency line (std/file IO may pull deps only where genuinely needed, e.g. example
  output). Migrate the fluid-dynamics theories and the DEC solver out of `deep_causality_physics`;
  migrate the CFD benches; migrate the validation examples (`cfd_taylor_green`,
  `dec_taylor_green_re1600`, `dec_lid_cavity_re1000`, `dec_graded_mms`, and the cut-cell cylinder)
  into `validation/`.
- **`FluidDynamics` DSL (NEW, `fluiddynamics-dsl`).** A composition surface for fluid simulations,
  built on the HKT/algebra foundation (`deep_causality_haft`, `deep_causality_num`) like
  `CausalFlow`. It composes: **solvers** (a solver uses a theory and/or physics kernels to solve a
  designated case); **boundary zones** (the `add-boundary-zone-abstraction` terms — inflow,
  outflow, walls, immersed bodies, uncertain sources); **multi-physics** pipelines (wrapping
  `CausalFlow`, as in `multi_physics_pipeline`); **counterfactuals** (`.intervene` on material,
  mesh, temperature — as in `causal_counterfactual_examples`); and **control flow** (loop / either
  / corrective, as in `causal_correction_examples`). The DSL integrates with `CausalFlow` (complex
  physics between steps / pre-processing) and `CausalDiscovery` (e.g. SURD to isolate contributing
  factors), and is easy to extend with new solvers via the HKT mechanism.
- **Theory vs solver split (NEW, `fluiddynamics-dsl`).** A *theory* is a Navier–Stokes regime
  reused across solvers; a *solver* uses a theory and/or kernels to solve one designated case. The
  DSL makes both first-class and composable.
- **Precision as a parameter (REQUIREMENT).** Every solver and theory runs natively at any
  supported float precision (`f32`, `f64`, `Float106`) with **zero downcasting** in the solver;
  each example uses a `FloatType` alias.
- **Examples in the DSL (NEW, `fluiddynamics-dsl`).** The crate's examples are written in the
  `FluidDynamics` DSL, showcasing common fluid-dynamics problems.

## Dependencies and sequencing

- **Depends on (both must be archived first):** `add-boundary-zone-abstraction` and
  `add-cut-cells-and-immersed-boundaries`.
- This change is authored now to fix the target; its design is reviewed and then derived into a
  full specification (OSPX) once the prerequisites land. It is the last of the three-change
  program.

## Non-Goals

- No behavioural change to the migrated solvers/theories beyond the crate move and the DSL surface
  (numerics are preserved; the prerequisites supply the new capabilities).
- No new IO/mesh formats beyond what migrated examples already need.
- Publishing the crate (it is `publish = false`).
