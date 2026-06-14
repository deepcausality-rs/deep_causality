# Milestone structure

Authored now to fix the target; the full task breakdown is derived (OSPX) after both prerequisites
(`add-boundary-zone-abstraction`, `add-cut-cells-and-immersed-boundaries`) archive. Per AGENTS.md
golden rules: agents never `git commit` and never delete files; each group gate prepares a commit
message and asks the user to commit.

> **Gate (must hold before any work here):** both prerequisite changes are archived.

## A. `causal_cfd` crate scaffold (causal-cfd-consolidation)

- [ ] A1 Create `causal_cfd` (`publish = false`), structure per `causal_cfd.md`
      (`src/{errors,extensions,traits,types,solvers,theories}`, `tests/`, `benches/`, `examples/`,
      `validation/`, `docs/{prompts,openspecs}`); workspace + bazel wiring.
- [ ] A2 Migrate the fluid-dynamics theories and the DEC NS solver out of
      `deep_causality_physics/src/theories/fluid_dynamics`, preserving numerics.
- [ ] A3 Migrate CFD benches and the validation examples (`cfd_taylor_green`,
      `dec_taylor_green_re1600`, `dec_lid_cavity_re1000`, `dec_graded_mms`, cut-cell cylinder) into
      `validation/`; re-run to identical reference results.

## B. `FluidDynamics` DSL (fluiddynamics-dsl)

- [ ] B1 Theory/solver trait seams (theory = NS regime reused across solvers; solver = one case);
      composable over the HKT foundation so a new solver is a small trait impl.
- [ ] B2 The `FluidDynamics` facade lowering onto `CausalFlow` (`step`/`iterate`/`branch`/
      `intervene`); compose boundary zones (the prerequisite terms), multi-physics (`bind`
      passthrough), counterfactuals (`.intervene` on material/mesh/temperature), control flow
      (loop/either).
- [ ] B3 Integration seams to `CausalFlow` (between-step / pre-processing physics) and
      `CausalDiscovery` (e.g. SURD tap on solver output).
- [ ] B4 Precision as a parameter: every solver/theory generic over `R: RealField` with no `f64`
      downcast; examples use a `FloatType` alias.

## C. Examples in the DSL (fluiddynamics-dsl)

- [ ] C1 Rewrite the validation examples in the `FluidDynamics` DSL; add showcase examples for
      common fluid-dynamics problems written in the DSL.

## D. Change gate

- [ ] D1 `openspec validate --strict`, format, clippy, full tests both feature configs and bazel
      for the new crate; prepare the final commit message; archive this change.
