## Context

Roadmap M1 (design-note Stage 2): the front-loaded combined risk spec. Survey-verified state of
the tree:

- **Incompressible-only forcing stack.** `QttImmersed2d` (`solvers/qtt/immersed_2d.rs:28-132`)
  penalizes `−(1/η)·χ ⊙ (u − u_body)` before projection; the smoothed mask `body_mask_2d`
  (`tensor_bridge/mask.rs:65-92`, `χ = ½(1 − tanh(d/δ))`) keeps rank bounded; force contraction
  is `mask.inner(deficit)·cell_volume/η` (`observe.rs:23-71`). Nothing forces or contracts on
  the compressible side: `CompressibleMarcher2d::step` (`marcher_2d.rs:218`) and the carrier's
  `enforce_inflow` (`compressible_march_run.rs:121-135`) have no masked-region seam, only a
  uniform body-acceleration in the theory residuals.
- **Fork machinery is generic and O(1).** `CarrierPause::fork` bumps two `Arc`s
  (`carrier.rs:440-450`); the single field CoW clone lands at the branch's first write
  (`carrier.rs:708`); the marched tensor train is never deep-copied (`carrier.rs:728`);
  `shares_fluid_with` proves sharing (`carrier.rs:629-636`).
- **Study/verification patterns exist.** `studies/qtt_blend_metric/` (parameter sweep →
  `max_bond` → gated trend, nonzero exit) and `verification/qtt_ramc_stagline/`
  (published-band gates in `print_utils.rs:54-83`, `exit(1)` on FAIL) are the binaries to
  mirror. Bond measurement: `max_bond` (`observe.rs:132-142`); cap: `Truncation::by_bond`.
- **Command injection.** `publish_constant(name, v)` lands on the field at the top of every
  `pre_step` (`compressible_march_run.rs:195-199`) — the per-branch throttle intervention needs
  no new machinery.
- **Kernels are done.** The propulsion family (archived `close-plasma-retropulsion-physics-gaps`)
  supplies the plume boundary, C_T, momentum-flux ratio, the digitized preserved-drag fraction
  and baseline `C_A0(M)` at M∞ = 2, and the total-axial-force composition.

Concurrent change: `plasma-retropulsion-cfd-contracts` (M2) is active. File surfaces are
disjoint (it touches `coupling.rs`/`corridor.rs`/new `retropulsion.rs`/example shared; this
change touches solvers/tensor_bridge/carrier pass-through/binaries). The one shared name is
`"commanded_throttle"`, pinned identically. Its corridor-inheritance guard applies here too:
this change touches the marcher path the corridor flies.

## Goals / Non-Goals

**Goals:**

- Answer the three §2 roadmap risks with measurements on a bare marched layer: imprint
  fidelity vs Jarvinen–Adams, fork economics of the plume-coupled state, rank of the
  colliding-shock layer (Cartesian vs blend metric).
- Land the two library seams M3's production `PlumeObstruction` will reuse: the compressible
  forcing region and the drag-contraction observable.
- Emit the recorded go/no-go verdict that fixes the M5 centerpiece design (state-fork vs
  parameter-fork) and the M3 coupling depth (A vs A0).

**Non-Goals:**

- No production `PhysicsStage` (M3), no example (M5), no guidance or envelope work (M2/M4).
- No 3-D forcing: the harness runs the 2-D marcher, where the corridor's carrier lives.
- No off-axis or angle-of-attack configurations — on-axis, inside the Cordell–Braun validated
  envelope, per the design note's §6 discipline pin.
- No tuning toward the correlation: the correlation is the *gate*, not a fit target. If the
  imprint cannot reproduce it honestly, that finding **is** the deliverable (amber/red path).

## Decisions

**D1 — The forcing region is an optional member of the compressible march path, not a wrapper
solver.** An optional `ForcingRegion` (mask train, target conserved state, strength η) rides the
compressible march configuration; when absent (`None`), the step path executes today's code —
bit-identity by construction, tested. Rationale: the incompressible side wraps
(`QttImmersed2d` around `QttIncompressible2d`), but the compressible marcher is driven through
the carrier (`CompressibleCarrier` → `CompressibleMarcher2d`), and a wrapper would fork the
carrier plumbing; an `Option` member follows the existing `enforce_inflow` precedent of
state enforcement owned by the march path. The exact insertion point (penalize the rate inside
the IMEX step vs. post-step state relaxation à la `enforce_inflow`) is chosen at implementation
against stability — the spec pins the semantics (interior driven toward the target, exterior
untouched, fused Hadamard + round, bounded bond), not the line.

**D2 — The mask is plume-shaped, static per run/branch, built from the analytic kernels.** The
plume boundary comes from `cordell_braun_plume_boundary` at the run's C_T and momentum-flux
ratio; the mask reuses the smoothed-mask codec (tanh skirt, rank-bounded) over that geometry;
the target interior state is the analytic jet state. Throttle is constant per run (verification)
and per branch (study), so the mask is static per world — an honest harness simplification,
stated in the binaries' headers. Per-step re-imprint under a *varying* throttle is M3's
production concern.

**D3 — Drag decrement is contracted by forebody-strip pressure integration, not
velocity-deficit contraction.** The incompressible `drag_lift` contraction measures the
penalization force — for the plume study that would measure the *forcing*, not the body's
preserved drag. Jarvinen–Adams measured forebody pressure; the observable therefore integrates
the evolved pressure (`ideal_gas_pressure_2d` over the conserved components) against a
forebody-strip mask, yielding the axial-force coefficient; the **preserved-drag fraction** is
its ratio against an unpowered baseline run of the same configuration. Same contraction
machinery (`mask.inner`), different integrand — a compressible sibling of
`qtt-surface-observables`, not a modification of it.

**D4 — Verification runs at the correlation's anchor point.** Freestream M∞ = 2.0, γ = 1.4
(cold air, the wind-tunnel condition), central-nozzle geometry: the digitized constants
(`JARVINEN_ADAMS_PRESERVED_DRAG_M2`, `JARVINEN_ADAMS_CA0_M2`, transition C_T ≈ 1) are all
M∞ = 2 data. The C_T sweep spans the collapse (0 → ~1) and beyond (to ~4): gates are (a)
contracted preserved-drag fraction within a pinned band of the correlation across the sweep,
(b) collapse — fraction < 0.10 by C_T ≈ 1, (c) the non-monotone total-axial-force band present
with its minimum's C_T location within tolerance of the correlation's. Bands are pinned from
the first measured run, then regress (the corridor's earned-band convention).

**D5 — Fork economics measures three quantities against structural invariants plus
first-run-pinned bands.** (i) Fork setup: `shares_fluid_with`/`shares_field_with` must hold
post-fork (structural, hard gate) with setup time recorded; (ii) per-branch continuation step
wall-clock ratio vs. an unforked trunk continuation (band pinned at first run; reported, and
regressed thereafter); (iii) post-fork `max_bond` growth per branch against the run's
`Truncation::by_bond` cap (hard gate: no branch exceeds the cap). The throttle roster is small
and purposeful: coast (0), two straddling the sign-flip band, one nominal, one high — each
injected via `publish_constant("commanded_throttle", …)` per branch world. Wall-clock gating is
ratio-based, not absolute, to survive machine variance; studies are manually-run binaries, not
CI-timed tests.

**D6 — The verdict is a hand-authored, checked-in note synthesizing both binaries.**
`openspec/notes/cfd-plasma-retropulsion/derisk-verdict.md`, written from the measured outputs,
carrying: the three risk answers with their numbers, the green/amber/red call, and the
consequence table (green → M3 depth A + M5 state-fork; amber/red → M3 depth A0 + M5
parameter-fork, limitation recorded). Rationale: the roadmap's downstream milestones (M3, M5)
need one authoritative artifact to cite, and the notes folder is where design records live; the
binaries' stdout stays the raw evidence, committed as `output.txt` beside each binary per the
family convention.

**D7 — Cross-change discipline with `plasma-retropulsion-cfd-contracts`.** No shared files; the
shared `"commanded_throttle"` name is pinned in both specs; this change re-runs the corridor
example (guard prong A) before archive because the marcher path changed, and the contracts
change's guard spec is updated to name marcher-touching changes explicitly (done as part of this
proposal's cross-check). Neither change blocks the other's implementation.

## Risks / Trade-offs

- [Penalization destabilizes the compressible IMEX step] → Smoothed skirt (the mask codec's
  rank *and* stiffness control), η chosen against the step's implicit budget, and the
  no-forcing path bit-identical so instability cannot leak into the corridor. If the term is
  irreducibly stiff at honest η, that is an amber finding, recorded — not tuned away.
- [The plume mask's rank explodes (colliding-shock system)] → That is risk 3's measurement, not
  a failure of the harness: the study reports Cartesian vs blend-metric bonds; blend-metric is
  the named lever (`qtt_blend_metric` lineage). Red only if both coordinates blow the cap.
- [Contraction disagrees with the correlation for honest reasons (blockage, smoothing skirt,
  2-D vs axisymmetric)] → The verification discloses geometry differences the way
  `qtt-surface-observables` discloses periodic blockage: the gated result is the collapse
  *structure* and band-located sign flip; absolute-fraction bands are pinned from the first
  measured run with the geometry caveat printed. A structural miss (no collapse, wrong-band
  flip) is the red signal — that is the point of the gate.
- [Wall-clock gates flake] → Ratios against an in-run trunk baseline, generous first-pinned
  bands, structural invariants (`Arc` sharing, bond cap) carry the hard PASS/FAIL.
- [Two active changes drift on the shared name] → `"commanded_throttle"` pinned verbatim in
  both changes' specs; grep is the reviewer's check.

## Migration Plan

Additive; nothing downstream consumes the new seams until M3. Land order: mask builder →
forcing member (+ bit-identity test) → contraction observable (+ tests) → verification binary →
study binary → measured runs → verdict note → corridor guard prong A re-run. Each lands green
under `bazel test //deep_causality_cfd/...` before the next. Rollback is dropping the change.

## Open Questions

- The precise penalization insertion point (rate-level inside IMEX vs. post-step relaxation) is
  an implementation choice bounded by D1's semantics; both are acceptable to the spec.
- Band values themselves are deliberately open until the first measured run — pinning them now
  would be tuning by anticipation. The specs phrase gates as "within the pinned band", with the
  pin recorded in the binaries' constants and the verdict.
