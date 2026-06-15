# Milestone structure

Per AGENTS.md golden rules: agents never `git commit` and never delete files; each group gate
prepares a commit message and asks the user to commit.

> **Gate:** both prerequisite changes (`add-boundary-zone-abstraction`,
> `add-cut-cells-and-immersed-boundaries`) are archived — **satisfied** (`2026-06-14`, `2026-06-15`).

The work is phased: Phase 1 lands the crate, the migration, and the minimal Flow surface with the six
validation examples; Phase 2 lands the advanced Flow surface (multi-physics coupling, counterfactuals,
control flow, Discovery tap) and the showcase multi-physics examples.

## Phase 1

### A. `deep_causality_cfd` crate scaffold (causal-cfd-consolidation)

- [ ] A1 Create `deep_causality_cfd` (`publish = false`), structure per `cfd-crate.md`
      (`src/{errors,extensions,traits,types,solvers,theories}`, `tests/`, `benches/`, `examples/`,
      `validation/`, `docs/{prompts,openspecs}`); workspace + bazel wiring; `[lints] workspace = true`.
- [ ] A2 **Move** the fluid-dynamics theories and the DEC NS solver **out of**
      `deep_causality_physics/src/theories/fluid_dynamics`, preserving numerics; update downstream
      importers.
- [ ] A3 Migrate CFD benches and the six validation examples (`cfd_taylor_green`,
      `dec_taylor_green_re1600`, `dec_lid_cavity_re1000`, `dec_graded_mms`, `dec_cylinder_wake`,
      `dec_cylinder_validation`) into `validation/`; re-run to identical reference results.

### B. Solver refactoring + minimal Flow (fluiddynamics-dsl)

- [ ] B1 (R1) `FluidTheory<R>` trait (NS regime) + `Solver`/`Theory` seam, static dispatch; implement
      it for the DEC incompressible rate and wrap the pointwise regime kernels behind it.
- [ ] B2 (R2) Split `DecNsSolver` into an owned `DecNsConfig<R>` (no manifold borrow) with a
      type-state builder (Discovery `CdlBuilder` style) + a manifold-bound marcher materialized from
      `(&manifold, zones, config)` at `run` (D2).
- [ ] B3 (R3/R4) Per-step `Ambient<R>` context channel (ν, freestream U, body force) + a per-step
      advance value over a typed `MarchState` / `CoupledField<R>`; the no-coupling path reproduces the
      construction-fixed validation numerics to tolerance (migration gate).
- [ ] B4 (R5/R6) Lift the six case orchestrations into standalone generic case-solvers
      (`LidCavitySolver`, `CylinderSolver`, `TaylorGreenSolver`, `MmsSolver`, …) + a generic
      diagnostics/observe module (Strouhal, drag/lift, Ghia compare, dissipation), no `f64` downcast.
- [ ] B5 The minimal `Flow` facade: owned declarative `Case` materialized at `run` (D2), lowering the
      march onto `CausalFlow` `iterate_n` / `iterate_until`; compose boundary zones (the archived
      terms); `seed` / `observe` / `probe`.
- [ ] B6 Rewrite the six validation examples in the minimal Flow DSL with the `config.rs` / `main.rs`
      split and a `FloatType` alias; re-run to identical reference results.

## Phase 2

### C. Advanced Flow surface (fluiddynamics-dsl)

- [ ] C1 `.couple` multi-physics seam: `PhysicsStage<R>` / `CoupledField<R>` over the error algebra;
      static stage composition (`then` / `compose`); modular per-physics sub-process modules; a
      corrective `Guard` stage (the `corrective_ddos_detector` `branch_with` pattern).
- [ ] C2 Counterfactuals: the `Intervene` vocabulary (static + dynamic laws); shared-seed
      `.counterfactual`; continuation `.continue_with().intervene` (compute-once / branch-many).
- [ ] C3 Control flow (either / loop / corrective) and integration seams to `CausalFlow` (between-step
      / pre-processing physics) and `CausalDiscovery` (e.g. a SURD tap on solver output).
- [ ] C4 Showcase multi-physics examples written in the Flow DSL (e.g. heated-cylinder thermo × fluid
      × stress; ascent sectional-drag dynamic counterfactual), with the config/main + sub-process
      module decomposition.

## D. Change gate

- [ ] D1 `openspec validate --strict`, format, clippy, full tests both feature configs and bazel for
      the new crate; prepare the final commit message; archive this change.
