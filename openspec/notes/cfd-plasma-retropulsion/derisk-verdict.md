<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# De-risk verdict — SRP plume coupling (roadmap M1, measured 2026-07-17)

**What this is.** The recorded go/no-go of the front-loaded risk milestone
(`plasma-retropulsion-de-risk`; roadmap
[`plasma-retropulsion-roadmap.md`](plasma-retropulsion-roadmap.md) §2/M1; design note
[`plasma-retropulsion-descent.md`](plasma-retropulsion-descent.md) §6, measurement 2). Three
risks were measured on a bare marched layer before any production stage or example exists.
Downstream milestones (M3, M5) cite this note rather than re-litigating the measurements.

Evidence: the committed outputs beside the binaries —
`deep_causality_cfd/reverted/srp_drag_decrement/output.txt` (the original pinned-envelope
verification, reverted 2026-07-17 — see the addendum below and `reverted/README.md`),
`deep_causality_cfd/studies/srp_momentum_jet/output.txt` (the superseding momentum-jet
measurement), and `deep_causality_cfd/studies/qtt_rank_plume/output.txt`. Harness: 2-D plane flow at the
Jarvinen–Adams anchor condition (M∞ = 2, γ = 1.4, central nozzle), 32×32 quantized layer,
plume geometry from `cordell_braun_plume_boundary`, interior pinned to the fully-expanded jet
state through the new `ForcingRegion` seam. Honesty convention as elsewhere: **[measured]**
throughout.

---

## The call: **AMBER**

| Risk | Question | Measured answer | Call |
|---|---|---|---|
| 1 — imprint fidelity | Does a compressible forcing region reproduce the Jarvinen–Adams drag collapse? | **No.** The imprint shields the forebody *monotonically* (preserved fraction 1.21 → 0.65 across C_T 0.25 → 4.0 — the direction is right and the coupling is live), but there is **no collapse** (0.895 preserved at C_T ≈ 1 vs the correlation's 0.124; max deviation 0.83) and **no sign-flip dip** (total axial force monotone). A pinned-state obstruction behaves like a drag-reduction spike; the J–A collapse is a jet-driven flowfield reorganization a static imprint cannot produce. | **AMBER** |
| 2 — fork economics | Does O(1) copy-on-write survive flow genuinely coupled to a per-branch throttle intervention? | **Yes.** Fork setup 83 ns with `Arc`-sharing verified (`shares_fluid_with`/`shares_field_with` true); per-branch continuation cost 0.68×–1.05× the unforked trunk (pinned band 2.0×); branch flow observables spread monotonically with throttle (final-field L2 vs coast: 0.18 / 0.28 / 0.35 / 0.61 for C_T 0.5 / 1.0 / 1.5 / 4.0) — the intervention feeds back into each branch's own flow, the corridor's branch-invariant foil broken. | **GREEN** |
| 3 — plume rank | What is the tensor-train rank of the colliding-structure plume layer, and does the blend-metric coordinate help? | **Bounded.** Marched peak bond 16 under the 32 ceiling at every swept C_T (tolerance rounding, rank free) — and flat: the imprint did not raise the unforced baseline's peak. Static coordinate dial on the analytic plume+shock proxy: Cartesian capture saturates (bond 32) while the fitted chart holds 10–12 — the `qtt_blend_metric` lever carries over to the plume system. Post-fork bond through every branch continuation: 16, flat. | **GREEN** |

## Consequences (per the `srp-derisk-verdict` capability's decision table)

The amber call on risk 1 pivots the coupling depth and the centerpiece, exactly as the specs
pre-committed:

- **M3 (`add-retropulsion-coupled-stages`)** carries the **A0 force-channel depth**: the drag
  decrement in flight comes from the cited Jarvinen–Adams correlation kernels
  (`srp_preserved_drag_fraction`, `srp_total_axial_force_coefficient`), not from field
  contraction. The `PlumeObstruction` stage may still imprint the plume on the marched layer
  (the seam exists, is inert-safe, and is rank-cheap — risks 2/3), but the imprint is *state
  realism*, never the drag authority.
- **M5 (`wire-plasma-retropulsion-example`)** pivots the counterfactual centerpiece to the
  **parameter-fork design** (design note §6, the shallow version), with the state-fork result
  recorded as this measured limitation. Nuance the M5 proposal should weigh: the state-fork
  *mechanics* measured green (cheap, coupled, rank-stable) — what failed is reading a
  J–A-faithful drag decrement *from the field*. A hybrid (A0 correlation as the force channel
  + state-fork mechanics for the flow-realism witnesses) is measurement-consistent and may be
  proposed to the user at M5 design time; this verdict does not preselect it.

## Pinned bands (first-run provenance)

| Band | Value | Where pinned |
|---|---|---|
| Verification R-B: sweep-top preserved fraction | ≤ 0.70 (measured 0.647 at C_T 4) | `reverted/srp_drag_decrement/config.rs` (reverted; superseded — see addendum) |
| Verification R-A: fraction monotone non-increasing in C_T | structural | `reverted/srp_drag_decrement/main.rs` gate |
| Study: per-branch continuation cost ratio | ≤ 2.0 (measured 0.68–1.05) | `studies/qtt_rank_plume/main.rs` |
| Study: rank ceiling (no compression = fail) | < 32 (measured 16) | `studies/qtt_rank_plume/main.rs` |

**Re-pin record (required by the bands requirement):** the verification's originally
anticipated structural gates — collapse < 0.10 by C_T ≈ 1 and the sign-flip dip — were
converted from hard gates to the **reported amber finding** after the first measured run,
because the harness physics cannot produce them (see risk 1) and a permanently failing
verification regresses nothing. The measured monotone-shielding structure replaced them as the
regression net. The delta spec (`srp-drag-decrement-verification`) was updated in the same
change, before archive.

## What would upgrade this to green

Named for M3/M-later, not scheduled here: a momentum-carrying jet interaction rather than a
pinned obstruction state (inject the jet's mass/momentum flux and let the plume form), an
axisymmetric or higher-resolution harness (the 32² plume spans ~2–6 cells of radius), or a
resolved barrel-shock/Mach-disk structure. Each attacks the specific mechanism the static
imprint lacks. Until one is measured, the A0 correlation remains the only cited drag authority
— which is precisely the fallback the design note (§3.1) reserved for this outcome.

**Superseded 2026-07-17 (same day, addendum below): the first path was measured.** The
momentum-carrying jet alone does not upgrade the call at this fidelity; the revised upgrade
bar is in the addendum's closing section.

---

# Addendum — the momentum-jet re-measurement (2026-07-17)

**Why re-measured.** A user-directed investigation asked whether the amber's risk-1 basis was
an artifact of the toy model: a pinned-state obstruction cannot express a jet-driven
flowfield reorganization *in principle*, so the original harness may never have had the power
to answer the imprint-fidelity question. The verdict's own first upgrade path — "inject the
jet's mass/momentum flux and let the plume form" — was built and measured the same day:
`deep_causality_cfd/studies/srp_momentum_jet/` (committed `output.txt` beside it), the same
harness with the whole-envelope ambient-pressure pin replaced by a nozzle-exit patch pinned to
a supersonic exit state through the same `ForcingRegion` seam, tail-averaged strip reads
(mean over the last 500 of 2000 steps; drift ≤ 0.02%), three strip bands (full / jet-excluding
annulus / outer), an exact injected-momentum audit, and interface / upstream-probe / floor
witnesses. Honesty convention as elsewhere: **[measured]** throughout.

## What the re-measurement found

1. **The momentum-carrying jet does not reproduce the collapse either — it inverts it.** The
   annulus fraction rises monotonically 1.031 → 3.614 across C_T 0.25 → 8 (2-D per-depth
   definition), with no collapse and no total-axial-force dip. The centerline stagnation
   interface stays frozen at x̂ = 0.469–0.531 across the 32× thrust range: the dissipation
   floor (ν = ½·s_ref·Δx; jet cell Péclet ≈ 1.3–1.8) prevents jet penetration, so injected
   momentum accumulates as face pressure — the inverse of the Jarvinen–Adams blanketing
   reorganization.
2. **Tensor-train compression is innocent.** A cap-32 companion (`SRP_MJ_CAP=32`, exact at
   2⁵ — truncation off) reproduces every fraction at C_T 1 and 4 to all displayed digits.
   The limit is the discretization, not the compression; the committed record itself is
   bit-reproducible across runs.
3. **The domain is a second, independent limit.** The upstream freestream probe departs
   continuously: +6.5% at C_T 0.25, +10.8% at 1, +69% at 4, +285% at 8. The correlation's own
   transition variable (p_e/p∞ ≈ 7.0–7.2, the sharp jet-penetration → blunt-flow transition;
   the Cordell kernel rejects below it) is unreachable: the fixed-nozzle sweep tops out at
   p_e/p∞ = 4.78 with the measurement already blockage-invalidated.
4. **Resolution is the binding constraint, measured directly.** At 2⁶ (ν halved;
   `SRP_MJ_L=6`, 2500 steps, C_T 4): the interface detaches and penetrates to x̂ = 0.266
   (~10 cells upstream vs frozen at 2⁵), the augmentation dissolves (annulus 1.111), and the
   outer band drops **below one** (0.905) — the first measured trace of genuine off-axis
   shielding, the direction of the J–A mechanism. The probe at +553% keeps this a
   direction-of-convergence datum, not a magnitude: once penetration starts, the 4 m periodic
   domain cannot contain the reorganized flowfield.
5. **The original harness's amber curve carried three artifacts, now quantified.** The pinned
   envelope overlapped the measurement strip (20–72% of strip height across the sweep, pinned
   cells reading gauge ≈ 0 mechanically); the pin held the plume at p_e/p∞ = 1.0, the wrong
   side of the correlation's transition variable by construction; and the terminal-snapshot
   read sat on a still-drifting field (+2.4% between steps 500 and 2000; the masks are
   node-sampled, so the nominal 2×8 strip is effectively 3×9). The superseding study
   reproduces the original baseline to 4.1e-9 before fixing each of these, so the deltas are
   attributable.
6. **The literature answer is settled at fidelity.** Central-nozzle SRP drag destruction is
   reproduced by resolved, shock-capturing, axisymmetric/3-D computation (the NASA SRP CFD
   validation campaign: DPLR / FUN3D / OVERFLOW / US3D against the Langley 4'×4' and Ames
   9'×7' tests; J–A's own momentum-balance model; Korzun's analytical plume structures). The
   mechanism is leading-order inviscid — a momentum-carrying jet suffices *given* transport
   that respects fronts, a resolved jet, and a domain that holds the displaced bow shock.

## The revised attribution

The **call stands**: risk 1 remains **AMBER**, the A0 correlation remains the only cited drag
authority, and every downstream consequence in the table above is unchanged — now doubly
measured, since both coupling model classes were tried on the same harness. What this
addendum revises is the **cause**. The original finding sentence ("a jet-driven flowfield
reorganization a static imprint cannot produce") was correct but incomplete: the momentum
model class cannot produce it here either. The risk-1 question was capped by the harness —
the isotropic dissipation floor and the domain — not by the seam, not by the compression, and
not only by the model class. The `ForcingRegion` seam expresses a momentum-carrying jet
without any new API; risks 2 and 3 (fork economics, rank) survive the stronger model class
untouched.

Consequences sharpened for M3/M5 (beyond the unchanged decision table): the hybrid split is
now a hard rule, not a preference — branch deceleration, drag decrement, and trajectory come
from the A0 kernels at the branch's C_T; branch *fields* carry coupling and realism witnesses
only, and any field-touching witness keeps C_T ≲ 5 on this domain (finding 3). Branch
continuations parallelize per the design note §6 pin (`deep_causality_par::scoped_map`).

## Supersession record (the re-pin, per the bands requirement)

`verification/srp_drag_decrement/` is **reverted** — moved to
`deep_causality_cfd/reverted/srp_drag_decrement/` (git mv, history preserved), detached from
Cargo, its first-run `output.txt` retained as the provenance of the original amber call and of
the R-A/R-B pins. The regression duty transfers to `studies/srp_momentum_jet/`, whose bands
are pinned from its first measured run (2026-07-17, committed `output.txt`): R-A′ monotone
non-decreasing annulus fraction; R-B′ sweep-top annulus band [3.2, 4.0] (measured 3.614);
R-C′ frozen-interface band [0.44, 0.56] at every point; plus the always-on witnesses (exact
momentum audit ±5%, pressure/density floors). The delta spec
(`srp-drag-decrement-verification`) was retargeted in the same change, before archive —
the same pattern as the original re-pin record above.

## What would upgrade this to green (revised)

The momentum-carrying jet is measured and is **not sufficient alone** at this fidelity. The
bar, in dependency order, each behind a measured gate:

1. **Front-respecting transport** — locally wave-speed-scaled dissipation (a low-rank
   Hadamard; the freestream needs ~3, not the global 8) and/or the Stage-4 tracked
   Rankine–Hugoniot interface treatment from the rank studies — the tensor-train-native
   analogue of the shock-capturing every literature reproduction uses.
2. **A domain that holds the displaced bow shock** — finding 4 shows penetration reaching the
   sponge the moment it exists; several body diameters upstream, no periodic image.
3. **Axisymmetric (or 3-D) formulation** — the J–A numbers are axisymmetric; a 2-D plane
   harness can share the structure, never the curve.

Resolution alone (2⁶) already buys the mechanism's direction (finding 4); the three items
above are what make the magnitude measurable. Until they are, the A0 correlation remains the
only cited drag authority — the same fallback as above, now with its boundary measured
rather than presumed.

**Making higher L affordable (recorded 2026-07-17).** The upgrade bar is compute-bound —
steps double per L (convective CFL) while per-step cost rises, putting the domain-widened
2⁷–2⁸ runs the bar implies at roughly a hundred single-core hours today. Measured profile
(this is what the levers attack): each step dequantizes four components to dense, evaluates
flux/EOS pointwise, then re-quantizes **eight flux fields (a TT-SVD each)** through ~20
cap-limited rounding passes — so the nonlinear round-trip is **dense O(n²) every step**
(defeating the TT step asymptotic) and **rounding, not contraction, dominates the TT side**.
The ladder, in the honest order — **algorithm first (~10–100×, precision-controlled),
parallelism second (~4–20×, zero precision risk), GPU last (situational)**, each rung behind a
timing gate (`compressible_carrier_timing`):
- **Algorithmic (CFD-side, not tensor-crate-gated):** (a) **TT cross-interpolation for the
  nonlinear flux** — rank-adaptive cross approximation removes the dense round-trip at the same
  tolerance as rounding (the QTT-CFD standard move; the single largest lever); (b) **fused +
  randomized rounding** — round once per component per step, not per operation (~2–5×);
  (c) **locally wave-speed-scaled dissipation**, which is upgrade item 1 *and* a cost lever —
  it lowers the L needed at all.
- **Parallel (now, zero precision impact):** roster-/sweep-level `deep_causality_par::scoped_map`
  (independent branch marches, the §6 pin) and component-level `scoped_map` inside the step
  (four independent component chains + eight flux encodes, ~3–4×).
- **Tensor-crate-side** (gated behind
  `openspec/notes/tensor-network/ACCELERATION-SOTA-FIRST.md`, whose own thesis — truncation,
  not contraction, is the hot spot — the measured profile above now corroborates):
  randomized/sketched TT rounding.
- **GPU: deferred** (user decision, 2026-07-17) — a single bond-≤64 train is latency-bound
  small-matrix work and FP64 rules out the local hardware; it wins only on **batched branch
  rosters** (batched GEMM/SVD) or 3-D/bond≥128, so revisit only at the batched-roster stage.

Full ladder with per-lever magnitudes and the CFD-vs-tensor-crate split:
`openspec/notes/cfd-roadmap/cfd-industry-scaling.md` §4a. **[plan]**
