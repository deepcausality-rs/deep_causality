<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# CFD Studies

Exploratory **studies** — empirical probes that test an assumption or design hypothesis before it is
committed to a spec. Unlike the `verification/` examples (which check a solver against an analytic or
published reference), a study answers a *research question* about the method itself. Each is
self-verifying (gates encode the finding and **exit nonzero** on regression) so the conclusion stays
reproducible.

```bash
cargo run --release -p deep_causality_cfd --example <name>
```

| Study | Question | Finding |
|---|---|---|
| `qtt_rank_study` | Is a reentry flowfield low tensor-train rank? (the Tier-B make-or-break) | Not automatically — a *captured* misaligned shock is net-negative (χ≈151–394, larger than dense). **Low rank by construction** in a shock-aligned / body-fitted coordinate (χ≈5, ~290× win). The driver is coordinate *alignment*, not curvature. |
| `qtt_rank_dynamic` | Does a *marcher* keep the field low-rank over time? | Yes for linear transport — fixed-tolerance rounding is rank-safe, no runaway (settles at static rank). Nonlinear shock-*steepening* was left untested (needs a nonlinear marcher) → see `qtt_rank_nonlinear`. |
| `qtt_rank_nonlinear` | Does a *forming* (nonlinear) shock stay low-rank? | 1-D: yes (peak 8, cheap). 2-D *curved* shock: rank rises 7→20 dynamically — the threat is **real**. Thickening is **not** the lever (curvature sets the rank; naive over-thickening is diffusion-CFL-unstable → full rank). Levers are coordinate alignment + an implicit/IMEX step (C3). |
| `qtt_rank_3d` | What is the **upper bound** of rank in 3-D (avionics/space regime)? | A realistically-formed (explicit Euler + central diff) 3-D curved shock has **χ ~ √side, unbounded** (45→135 over 16³→128³); flat/body-fitted stay **χ~6 constant**. QTT storage still beats dense asymptotically (crossover ~64³), but the √side **solve** cost is what bites — body-fitted coordinate is **mandatory** for 3-D tractability. |

These feed the Tier-B analysis in `openspec/notes/plasma-blackout/gap-2/`. The corresponding Tier-A
deliverable (`add-park2t-blackout-tier-a`) does **not** depend on these; they de-risk the *Tier-B*
compressible marcher.

## Round 2 — de-risking the resolution-4/5/6 design nodes

A second batch, run before the compressible build, tests the make-or-break claims behind the newer Tier-B
resolutions (`openspec/notes/plasma-blackout/gap-2/`, resolutions 4–9).

| Study | Question | Finding |
|---|---|---|
| `qtt_rank_fitted_dynamic` | Does the rank lever survive *marching* (Res 5)? | Alignment bounds the bond dynamically. An axis-aligned front holds bond 7 at both 64² and 128² (flat in resolution); a misaligned curved shock grows 20 to 25. A **static** body-fitted coordinate is not enough: under Cartesian fluxes the marched front drifts off it and the bond grows 25 to 35, no better than the capture. So Res-5 **feedback re-pinning (D9)** is necessary, not optional. Alignment is the lever; maintaining it is the mechanism. |
| `qtt_acoustic_precond` | Does the split preconditioner de-risk the implicit step (Res 6)? | The constant-coefficient core inverts at bond 8, flat from L=8 to L=10 (low-rank, resolution-stable). On a smooth interior the perturbation spectral radius ρ(A₀⁻¹A₁)=0.59 < 1, so the preconditioned operator `I + A₀⁻¹A₁` contracts and the implicit solve converges geometrically. Across a captured 5× sound-speed jump ρ rises to 0.87, toward the divergence threshold at 1. The jump is the hard part, which is why fitting (Res 5), by keeping the interior smooth, keeps the implicit step cheap. |
| `qtt_blend_metric` | Is body-fit a valid, low-rank free parameter (Res 4)? | The position-blend `Tλ = (1−λ)·Cartesian + λ·fitted` stays a valid map: det J holds one sign with min‖det J‖ ≈ 1.5 across the whole λ sweep, so no cell folds. A fixed physical shock sampled on the blended lattice runs monotonically from bond 114 (λ=0, capture) to 5 (λ=1, fitted). λ is a clean rank dial. |
| `qtt_repin_marcher` | Does feedback re-pinning bound the marched rank (Res 5 / D9, the Stage-4 core)? | Two parts. Marching Cartesian fluxes *through* the curved front grows 25→35 with resolution, and re-pinning the coordinate to the live front (18 re-pins at 128²) does **not** curb it: the driver is the angular structure a flux-through-front march injects, not the front's drift. Aligning the transport with the coordinate instead (radial flux, the front as a tracked interface) holds the bond at 8, flat in resolution. So the Stage-4 lever is re-pin **and** treat the front as an exact Rankine–Hugoniot interface (smooth each side), not march fluxes across it. |

Round-2 result: two make-or-break claims confirmed (the alignment lever survives marching; the
constant-coefficient preconditioner is low-rank and contracts on a smooth interior), one residual closed
(the body-fit blend is valid and dialable), and the Stage-4 mechanism pinned down precisely. A static
fitted coordinate does not self-bound under marching, and re-pinning the coordinate alone does not fix it
either; the rank driver is carrying Cartesian fluxes *through* a curved front. The lever that works is
re-pinning **plus** treating the front as an exact Rankine–Hugoniot interface, so fluxes are never marched
across it and each side stays smooth.
