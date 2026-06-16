# CFD Consolidation — Follow-ups

Date: 2026-06-16. Status: **open follow-ups.** Context: the `consolidate-causal-cfd-fluiddynamics`
change is archived (the DEC solver, theories, boundary zones, and uncertain-inflow stack now live in
`deep_causality_cfd`; the `deep_causality_physics` fluid theories were deleted). This note records the
two things that arc left open: (1) where the uncertain handling actually sits, and (2) the loose ends
that are otherwise tracked nowhere durable.

## 1. Uncertain handling: in the crate, not in the DSL facade

The uncertain-inflow machinery — `UncertainInflowZone`, `UncertainBoundarySource` (with the opt-in
`with_qmc_collapse`), `inflow_march_step`, `march_inflow`, `DropoutVerbosity` — is now **owned by
`deep_causality_cfd`** (`src/solvers/dec/uncertain_inflow/`). At the crate level the uncertain layer is
consolidated.

It is **not lifted into the `CfdFlow` DSL facade.** There is no `CfdConfigBuilder::uncertain_march(...)`
configuration nor a `CfdFlow` workflow for a sensor-fed march. `examples/avionics_examples/
dec_cylinder_wake` still **hand-rolls** the causal-monad march — a `PropagatingProcess<f64,
InflowMarchState, InflowContext>` driven one `inflow_march_step` bind at a time — rather than declaring
it through the DSL. So the uncertain march is *available* in the crate but *not promoted* into the
declarative surface.

**Open decision (the `uncertain_march` promotion).** Two paths, unchanged from the seam-vs-corpus model:

- **Promote to corpus** — add `CfdConfigBuilder::uncertain_march(...)` (sensor stream + zone + collapse
  policy) and a `CfdFlow` run that lowers onto `march_inflow`'s `CausalFlow::iterate_n`, with a
  `run_with`-style per-step probe seam for the wake diagnostic. Justified once a second sensor-driven
  case appears (rule of three); the physics already packages the reusable `march_inflow` kernel.
- **Keep bespoke behind the seam** — leave the hand-rolled march in the example. Defensible while
  `dec_cylinder_wake` is the only consumer.

Recommendation: keep bespoke until a second uncertain-march case exists; revisit then.

## 2. Undocumented outstanding issues

Loose ends from the consolidation + DSL migrations, with pointers:

1. **`dec_cylinder_validation` repointed but unverified.** Its imports were moved physics → cfd and it
   *builds*, but it was never *run* this session (it writes CSV files → blocked on the IO monad). Output
   equivalence to the pre-consolidation version is unconfirmed. → run + A/B once an IO path exists.

2. **IO-monad follow-up change is unwritten.** `dec_lid_cavity_re1000`, `dec_cylinder_wake`, and
   `dec_cylinder_validation` all write files; the cavity computes through `CfdFlow` but keeps its CSV
   writes inline pending an IO monad. No OpenSpec change captures this yet.

3. **`uncertain_march` DSL promotion** — see §1 (the open decision).

4. **`deep_causality_physics` `parallel` feature may be vestigial.** It forwards to
   `deep_causality_topology/parallel` "underneath the Navier–Stokes solver" — but that solver moved to
   `deep_causality_cfd`. Audit whether physics still needs the feature; the physics `README.md` blurb is
   stale on this point (the bench reference was already removed).

5. **`dec-ns-validation` live spec fails `openspec validate --strict`** (pre-existing; surfaced while
   archiving the consolidation). Unrelated to the consolidation but undocumented — needs a separate look.

6. **DEC-solver benchmark gap.** `deep_causality_physics/benches/dec_solver_benchmark.rs` was removed
   with the solver; no cfd replacement yet. Tracked task: benchmarks for all cfd fluid solvers (DEC NS
   marching, MMS-verification, operator-accuracy) — new `deep_causality_cfd/benches/` + `criterion`
   dev-dep + `[[bench]]` + Bazel.

7. **Cylinder/cavity examples are import-repointed, not DSL-migrated.** They were pointed at cfd but not
   rewritten into the `config.rs` / `main.rs` + `FloatType` split the other examples use. The cavity
   computes via `CfdFlow`; the two cylinder examples do not.

## Pointers

- Consolidated home: `deep_causality_cfd/src/solvers/dec/uncertain_inflow/`,
  `deep_causality_cfd/src/{theories,solvers}/`.
- Hand-rolled uncertain march: `examples/avionics_examples/dec_cylinder_wake/main.rs`.
- Related open notes: [qmc-presence-gate-followup.md](qmc-presence-gate-followup.md),
  [cfd-validation-plan.md](cfd-validation-plan.md).
- Archived (implemented) context: `archive/cfd-crate.md`, `archive/cfd-gap.md`.
