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
`deep_causality_cfd/verification/srp_drag_decrement/output.txt` and
`deep_causality_cfd/studies/qtt_rank_plume/output.txt`. Harness: 2-D plane flow at the
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
| Verification R-B: sweep-top preserved fraction | ≤ 0.70 (measured 0.647 at C_T 4) | `verification/srp_drag_decrement/config.rs` |
| Verification R-A: fraction monotone non-increasing in C_T | structural | `main.rs` gate |
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
