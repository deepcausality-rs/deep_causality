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

1. **`dec_cylinder_validation` repointed but unverified.** Its imports were moved physics → cfd and it
   *builds*, but it was never *run* this session. The IO path it was blocked on now exists (the
   `add-io-monad` change landed and is archived — see §3), so its CSV writes can be migrated onto
   `write_csv` and A/B'd. → migrate the writes, run, and confirm output equivalence to the
   pre-consolidation version.

2. **Examples import-repointed, not fully DSL-migrated.** They were pointed at cfd but not all rewritten
   into the `config.rs` / `main.rs` + `FloatType` split the other examples use. The cavity and
   `dec_cylinder_wake` compute via `CfdFlow` (the latter via `uncertain_march`); `dec_cylinder_validation`
   does not yet.

## 3. Resolved

- **IO monad (was open issue #2).** The `add-io-monad` change is written, implemented, and archived
  (`openspec/changes/archive/2026-06-17-add-io-monad/`; canonical spec `openspec/specs/io-monad/`). A
  lazy `IoAction` effect lives in `deep_causality_haft` (value-level combinators, no `dyn`), with
  `std`-gated file actions (`read_text` / `write_text` / `write_csv` / `read_csv`) and a `CausalFlow`
  read/write bridge (`source` / `commit`, `read_*_from` / `write_*_to`) in `deep_causality_core`, plus
  CSV helpers (`Report::write_series_csv`, `write_xy_csv`) in `deep_causality_cfd`.
  `dec_lid_cavity_re1000` now writes its centerline CSVs through it with **byte-identical** formatting;
  `dec_cylinder_wake` writes its full wake-probe series via `write_xy_csv` (new file output, not a
  byte reproduction of the prior stdout stream). `dec_cylinder_validation` remains to migrate (open
  issue #1).

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
  (`CfdFlow::uncertain_march`); example `examples/avionics_examples/dec_cylinder_wake/main.rs`.
- Related open notes: [qmc-presence-gate-followup.md](qmc-presence-gate-followup.md),
  [cfd-validation-plan.md](cfd-validation-plan.md).
- Archived (implemented) context: `archive/cfd-crate.md`, `archive/cfd-gap.md`.
