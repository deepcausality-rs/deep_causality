## Why

The plasma-retropulsion roadmap front-loads one combined risk spec
(`openspec/notes/cfd-plasma-retropulsion/plasma-retropulsion-roadmap.md` §2, milestone M1 /
design-note Stage 2): the immersed-boundary forcing + force-contraction stack is
incompressible-only today, and three risks live and die on porting it — (1) does a compressible
forcing region reproduce the Jarvinen–Adams drag collapse, (2) does O(1) copy-on-write forking
survive flow genuinely coupled to a per-branch throttle intervention, (3) what is the
tensor-train rank of a colliding-shock plume layer. The answers decide the state-fork
centerpiece design before M3–M5 commit to it; measuring them on a bare marched layer, before any
production stage or example exists, is the experiment paying for itself in both outcomes.

## What Changes

- A **compressible forcing-region seam** (harness form) in `deep_causality_cfd/src/solvers/qtt`:
  a smoothed mask over the 4-component `EulerStateTt2d` and a penalization term in the
  compressible step path, porting the incompressible primitives (`body_mask_2d`, `penalize`);
  interior forced toward an analytic jet target state, outer flow left to evolve its own
  standoff shock. With no forcing configured, the marcher is bit-identical to today.
- A **drag-contraction observable** for the compressible field: forebody-strip pressure
  integration contracting the axial force (and the preserved-drag fraction against an unpowered
  baseline) from the evolved field — the quantity Jarvinen–Adams measured.
- **`verification/srp_drag_decrement/`**: a self-verifying binary driving the plume boundary
  from the existing propulsion kernels, imprinting it, contracting the decrement, and gating it
  against the digitized Jarvinen–Adams correlation across a C_T sweep — central-nozzle collapse
  and the sign-flip band included; nonzero exit on FAIL (the `qtt_ramc_stagline` pattern).
- **`studies/qtt_rank_plume/`**: a self-verifying study measuring (a) the rank of the
  plume-imprinted layer in Cartesian vs blend-metric coordinates, and (b) **fork economics** on
  the plume-coupled state: pause mid-imprint, fork copy-on-write, apply a small per-branch
  throttle roster via `publish_constant("commanded_throttle", …)`, continue, and record fork
  cost, per-branch step wall-clock ratio, and post-fork bond growth against the cap.
- A **recorded go/no-go verdict** checked in at
  `openspec/notes/cfd-plasma-retropulsion/derisk-verdict.md`: green confirms the state-fork
  centerpiece; amber/red pivots M5's centerpiece to the parameter-fork design and M3's plume
  seam to the A0 depth, with the result recorded as a measured limitation — no downstream
  rewrite either way.

## Capabilities

### New Capabilities

- `compressible-forcing-region`: the masked penalization forcing seam on the compressible QTT
  marcher — mask/target/strength configuration, application in the step path, and the
  no-forcing-bit-identity guarantee.
- `compressible-drag-contraction`: the axial-force / preserved-drag-fraction observable
  contracted from the evolved compressible field by forebody-strip pressure integration.
- `srp-drag-decrement-verification`: the verification target gating the imprinted layer's
  contracted drag decrement against the Jarvinen–Adams correlation kernels.
- `plume-rank-fork-study`: the rank + fork-economics study on the plume-imprinted layer,
  including the measured bands the later gates regress against.
- `srp-derisk-verdict`: the checked-in go/no-go verdict artifact and its decision semantics for
  M3/M5.

### Modified Capabilities

None. The existing `qtt-immersed-body` and `qtt-surface-observables` capabilities are
incompressible-scoped and stay unchanged; this change adds compressible siblings. (The active
`plasma-retropulsion-cfd-contracts` change is cross-checked and adjusted separately: its
inheritance guard's standing scope now names this change too, since the forcing seam touches
the marcher path the corridor flies.)

## Impact

- `deep_causality_cfd`: `solvers/qtt/compressible/` (forcing member on the 2-D marcher path)
  and/or `types/flow/compressible_march_run.rs` (carrier pass-through), `tensor_bridge`
  (plume-shaped mask builder reusing the smoothed-mask codec), `solvers/qtt/observe.rs`-family
  (compressible contraction observable), new `verification/srp_drag_decrement/` and
  `studies/qtt_rank_plume/` binaries, mirrored tests + Bazel registration.
- `deep_causality_physics`: consumed only (`cordell_braun_plume_boundary`,
  `srp_thrust_coefficient`, `momentum_flux_ratio`, `srp_preserved_drag_fraction`,
  `jarvinen_adams_baseline_axial_coefficient`, `srp_total_axial_force_coefficient` kernels); no
  changes.
- `openspec/notes/cfd-plasma-retropulsion/`: gains `derisk-verdict.md` after the measured runs.
- Cross-change: no file overlap with `plasma-retropulsion-cfd-contracts`; the one shared name is
  the published constant `"commanded_throttle"`, pinned identically in both. The corridor
  inheritance guard (prong A) re-runs before this change archives.
- Downstream: M3 (`PlumeObstruction` depth), M4 (unaffected), M5 (centerpiece design) read the
  verdict; the roadmap's M1 go/no-go gate is this change's exit.
