# CFD Consolidation — Follow-ups

Date: 2026-06-16. Status: **open follow-ups.** Context: the `consolidate-causal-cfd-fluiddynamics`
change is archived (the DEC solver, theories, boundary zones, and uncertain-inflow stack now live in
`deep_causality_cfd`; the `deep_causality_physics` fluid theories were deleted). This note records the
two things that arc left open: (1) where the uncertain handling actually sits, and (2) the loose ends
that are otherwise tracked nowhere durable.

Update 2026-06-17: the IO-monad follow-up and the `uncertain_march` DSL promotion have both landed —
see §3 (Resolved). The open issues below are revised accordingly.

## 1. Uncertain handling: consolidated in the crate and promoted to the DSL

The uncertain-inflow machinery — `UncertainInflowZone`, `UncertainBoundarySource` (with the opt-in
`with_qmc_collapse`), `inflow_march_step`, `march_inflow`, `DropoutVerbosity` — is **owned by
`deep_causality_cfd`** (`src/solvers/dec/uncertain_inflow/`).

It has also been **lifted into the `CfdFlow` DSL facade** (commits `cc8f69bef`, `e2aceee3e`):
`CfdConfigBuilder::uncertain_march(...)` declares the sensor stream + zone + collapse policy, and
`CfdFlow::uncertain_march(&config).on(&manifold).run` / `run_with` drives the sensor-fed march (the
per-step `inflow_march_step` bind, packaged as `march_inflow`), surfacing an `UncertainStepView`
per-step probe seam. `dec_cylinder_wake` now declares the march through the DSL rather than hand-rolling
a `PropagatingProcess<f64, InflowMarchState, InflowContext>` bind loop. The earlier seam-vs-corpus "open
decision" is therefore **closed** (promoted to corpus) — see §3.

## 2. Open issues

Loose ends from the consolidation + DSL migrations, with pointers:

1. **Tier-2 verification examples compute *below* the CfdFlow DSL.** `dec_graded_mms_verification`
   (operator-level MMS: `exterior_derivative` / `interior_product` / `codifferential_of`, DSL only owns
   the geometry) and `dec_cylinder_verification` (hand-rolled `DecNsSolver::with_zones(...).step()` with
   inflow/outflow/slip zones and surface-force diagnostics) do not route their computation through
   `CfdFlow`. Promoting them would need new crate workflows — a graded operator-study and a zoned
   validation-march with a surface-force `StepView` seam — which is its own OpenSpec change.

## 3. Resolved

- **IO monad (was open issue #2).** The `add-io-monad` change is written, implemented, and archived
  (`openspec/changes/archive/2026-06-17-add-io-monad/`; canonical spec `openspec/specs/io-monad/`). A
  lazy `IoAction` effect lives in `deep_causality_haft` (value-level combinators, no `dyn`), with
  `std`-gated file actions (`read_text` / `write_text` / `write_csv` / `read_csv`) and a `CausalFlow`
  read/write bridge (`source` / `commit`, `read_*_from` / `write_*_to`) in `deep_causality_core`, plus
  CSV helpers (`Report::write_series_csv`, `write_xy_csv`) in `deep_causality_cfd`.
  `dec_lid_cavity_re1000` now writes its centerline CSVs through it with **byte-identical** formatting;
  `dec_cylinder_wake` writes its full wake-probe series via `write_xy_csv` (new file output, not a
  byte reproduction of the prior stdout stream).

- **DEC/MMS examples migrated to `deep_causality_cfd/verification/` and made self-verifying (was open
  issues #1, #2).** All six DEC/MMS examples moved out of `examples/avionics_examples` into the cfd
  crate's `verification/` folder, declared as `deep_causality_cfd` examples, and given a
  `_verification` suffix (`dec_cylinder_validation` → `dec_cylinder_verification`). The Tier-1 cases
  (`mms_taylor_green`, `dec_taylor_green_re1600`, `dec_lid_cavity_re1000`, `dec_cylinder_wake`) carry
  the `config.rs`/`main.rs`/`FloatType` split with native-precision arithmetic (downcast only at the
  display edge). Convention: a verification example **exits nonzero** when its invariant/reference
  breaks — `dec_taylor_green_re1600` gates kinetic-energy monotonic decay, `dec_cylinder_wake` gates
  the divergence residual + dropout/intervention log accounting, the cavity gates the Ghia-RMSE
  refinement trend (`trend` mode). See `deep_causality_cfd/verification/README.md`.

- **`uncertain_march` DSL promotion (was open issue #2 / §1's open decision).** The sensor-fed march is
  lifted into the `CfdFlow` DSL — `CfdConfigBuilder::uncertain_march` + `CfdFlow::uncertain_march`
  (`run` / `run_with`) over the `march_inflow` kernel — and `dec_cylinder_wake` declares it through the
  DSL (commits `cc8f69bef`, `e2aceee3e`).

- **`deep_causality_physics` `parallel`-feature audit (dropped).** Not vestigial after all — the feature
  is used by other physics kernels, not only the moved Navier–Stokes solver. No action needed.

- **DEC-solver benchmark gap (was open issue #3).** Replaced the removed
  `deep_causality_physics/benches/dec_solver_benchmark.rs` with three dedicated, holistic Criterion
  benches in `deep_causality_cfd/benches/` — one per solver kind: `bench_dec_ns_march` (grid + step
  sweeps), `bench_mms_verify` (pointwise viscosity sweep + amplitude-march step sweep), and
  `bench_operator_study` (viscous `δd` over growing resolution ladders). `criterion` was already a
  dev-dep; three `[[bench]]` entries declared (`autobenches = false`). No Bazel rule — benches are
  cargo-only across the whole repo.

- **`dec-ns-validation` strict-validation failure (was open issue #2).** The canonical spec was still in
  change-delta form; converted to `## Purpose` / `## Requirements` and moved `SHALL` onto the first
  line of the two requirements whose statement spilled to line 2 (openspec reads only the first line as
  the requirement statement). `openspec validate dec-ns-validation --strict` now passes. (Other live
  specs still fail `--strict` for similar reasons — out of scope here.)

## Pointers

- Consolidated home: `deep_causality_cfd/src/solvers/dec/uncertain_inflow/`,
  `deep_causality_cfd/src/{theories,solvers}/`.
- Uncertain march via the DSL: `deep_causality_cfd/src/types/flow/uncertain_march_run.rs`
  (`CfdFlow::uncertain_march`); example `deep_causality_cfd/verification/dec_cylinder_wake_verification/`.
- DEC/MMS verification examples: `deep_causality_cfd/verification/` (run with
  `cargo run -p deep_causality_cfd --example <name>_verification`).
- Related open notes: [qmc-presence-gate-followup.md](qmc-presence-gate-followup.md),
  [cfd-validation-plan.md](cfd-validation-plan.md).
- Archived (implemented) context: `archive/cfd-crate.md`, `archive/cfd-gap.md`.
