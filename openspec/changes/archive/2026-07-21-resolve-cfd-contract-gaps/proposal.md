# Resolve three CFD contract gaps

## Why

Three places in the crate where the documented contract and the shipped behaviour disagree. None is a
wrong calculation; each is a promise the code does not keep, and each needs a decision about which side
to move.

**A documented guarantee that nothing enforces.** `BlendedMap`'s module doc states that `det J_λ` keeps
one sign across `λ ∈ [0,1]` and that the constructor "rejects a fold". No such check exists in
`BlendedMap::new`. The in-function comment goes further — *"Validity (gate BM-A) holds **by
construction**"* — but the justification it offers is the `qtt_blend_metric` **measurement** for one
specific geometry, not an argument covering the inputs the constructor accepts. Meanwhile all four
inverse-metric components and the volume factor divide by `det_at(ξ, η)` with no guard, so a degenerate
geometry yields `inf`/`NaN` or entries of magnitude ~1e15 while the constructor returns `Ok`.

**An observable whose name is not what it computes.** `wall_heat_flux` returns
`(1/η)∫χ(T_w − T)dV` — a temperature-weighted volumetric rate with units `[T]·[L]²/[t]`. It contains no
gradient, no conductivity and no surface normal, so it is not a flux under any scaling; Fourier's law
is `q = −k·∂T/∂n`. The docstring states the formula honestly and marks the function *"Neutral — the
seam the Gap-2 reacting energy equation replaces"*, so the implementation is not hiding anything. What
misleads is the exposed **name** and the published series key, both `wall_heat_flux`. For a re-entry
TPS consumer, wall heat flux is the safety-critical quantity — the one number an engineer is most
likely to take at face value.

Compounding it: the production path hardcodes `t_wall = R::zero()` at `qtt_march_run.rs:214`, and
`t_wall` is **not a field of `QttMarchConfig`** — there is no way to configure the wall temperature the
quantity is defined against.

**A boundary-zone stage that was never connected.** The `BoundaryZone` trait documents five `collect_*`
hooks and states "The solver folds every zone's contribution at the matching stage". Four of the five
are wired. `collect_constrained_edges` has **zero** call sites and zero implementors outside the trait
definition — a zone that implements it silently applies no boundary condition, and nothing reports
that. The no-slip constraint set the hook was presumably meant to feed is instead built directly in
`dec_ns_solver/no_slip.rs`, which enumerates wall-tangential edges itself and feeds
`leray_project_constrained_opts`. So the abstraction is sound and used; one stage of it is vestigial.

Audit `AUDIT-REPORT.md` §4 (**B-4**), §4b, and §9 Phase 2 items 8, 11, 15.

## What Changes

- **`BlendedMap`: enforce the guarantee or withdraw it.** Either the constructor performs the fold and
  singularity check its documentation claims, or the documentation stops claiming it. Either way the
  division by `det_at` is guarded, so a degenerate map cannot silently produce `inf`-magnitude metric
  entries behind an `Ok`.
- **`wall_heat_flux`: make the name match the quantity, and make `t_wall` configurable.** The exposed
  name and the published series key describe what is computed. The wall temperature the quantity is
  defined against becomes a configurable parameter rather than a hardcoded zero.
  **BREAKING (API):** a public function and a published series key are renamed.
- **`collect_constrained_edges`: wire it or remove it.** A hook the solver never folds is worse than an
  absent one, because the trait documentation promises it is folded.

Explicitly **not** in scope: the blended-map transformation itself (the Jacobian and metric algebra
were confirmed correct by the audit), the no-slip constraint enumeration in `no_slip.rs` (working and
used), and the Gap-2 reacting energy equation that will eventually replace the heat-flux seam.

## Capabilities

### New Capabilities

None. All three gaps sit inside capabilities that already have specs; this change corrects what those
specs and their implementations promise.

### Modified Capabilities

- `body-fitted-qtt-coordinate`: the blended-map requirements gain the validity guarantee as an
  *enforced* property with a guarded inverse metric, rather than a documented one resting on a
  measurement for a single geometry.
- `qtt-surface-observables`: the surface-observable requirements gain a naming contract — an
  observable's name and published series key describe the quantity computed — and the wall-temperature
  parameter becomes part of the configuration surface rather than a hardcoded constant.
- `boundary-zone-abstraction`: the hook-set requirement is corrected so that every documented
  `collect_*` stage is one the solver actually folds, closing the gap between the trait's promise and
  the four-of-five reality.

## Impact

**Code**
- `deep_causality_cfd/src/coordinate/blended.rs` — the constructor's check (or the doc's claim) and the
  four `÷ det_at` sites plus the volume factor.
- `deep_causality_cfd/src/solvers/qtt/observe.rs` — `wall_heat_flux`'s name and documentation.
- `deep_causality_cfd/src/types/flow/qtt_march_run.rs:214` — the hardcoded `t_wall = 0` and the
  published series key.
- `deep_causality_cfd/src/types/flow_config/qtt_march_config.rs` — the wall-temperature parameter.
- `deep_causality_cfd/src/solvers/dec/boundary/boundary_zone.rs` — the hook and its trait
  documentation.
- `deep_causality_cfd/src/lib.rs` — the re-export, if the observable is renamed.

**Evidence**
- Any harness or study reading the `wall_heat_flux` series — the corridor's wall-heat-flux observable
  is folded into its branch accumulator.
- `qtt_blend_metric` (the study whose measurement is currently doing duty as the validity argument).

**Risk**
- Renaming a public item and a published series key is a breaking API change. `deep_causality_cfd` is
  `publish = false`, so the blast radius is in-repo, but every consumer of the series key must move
  with it.
- Enforcing the `BlendedMap` check may reject geometries currently constructed by studies. If so, those
  geometries were producing unguarded metric entries and the rejection is the finding.
- Removing `collect_constrained_edges` narrows a public trait; wiring it changes solver behaviour for
  any zone that implements it. Both directions need the owner's call — see the design.
