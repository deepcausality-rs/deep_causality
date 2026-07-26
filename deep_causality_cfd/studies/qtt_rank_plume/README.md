<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `qtt_rank_plume` — plume rank + fork economics (plasma-retropulsion de-risk, M1 risks 2 and 3)

```bash
cargo run --release -p deep_causality_cfd --example qtt_rank_plume
```

**What it tests.** Two measurements on the plume-imprinted compressible layer, neither taken
before (`plasma-retropulsion-de-risk`, capability `plume-rank-fork-study`).

**Phase A, rank.** The retro-plume is a colliding-shock system of barrel shock, Mach disk, shear
layer, and displaced bow shock, and its tensor-train rank was unmeasured. **A1** marches the
imprinted layer (sponge, body, and plume, all through the forcing seam) per thrust coefficient
under a *tolerance* round policy and records the peak bond, giving the dynamic rank. **A2** encodes
an analytic plume-plus-standoff-shock proxy on the λ-blended lattice (`BlendedMap`, the
`qtt_blend_metric` and `qtt_blunt_body_2d` lineage) at λ = 0 and λ = 1, the static coordinate-dial
probe.

**Phase B, fork economics.** March a plume-coupled world to a mid-run pause on the real carrier,
fork it, and continue a purposeful throttle roster: coast, two sign-flip straddlers, nominal, and
high. Each branch carries its **own** forcing region derived from its own published
`"commanded_throttle"`, so the intervention feeds back into that branch's flow through the model.

**Findings.** A1 and Phase B march a 2⁵ × 2⁵ grid at dt 4e-4. Every encoding in the study, A1, A2
and the mirror alike, uses `Truncation::by_tol(1e-6)`, which sets `max_bond = usize::MAX`: no bond
cap is applied anywhere, so every bond below is an unconstrained measurement at that tolerance.
`RANK_CEILING = 32` is a gate threshold, the saturation bond of a 2⁵ × 2⁵ field. It is compared
against A1's peak and against the mirrored branch peaks. It is not applied to A2.

*A1, marched rank (Cartesian):* peak bond **16 at every C_T** tested (0.5, 1, 2, 4) and **16** for
the no-plume body/shock baseline. The plume imprint costs nothing in rank over the baseline.

*A2, static proxy on the blended lattice:*

| C_T | bond λ=0 (Cartesian) | bond λ=1 (fitted) |
|---|---|---|
| 0.50 | 32 | 10 |
| 1.00 | 32 | 10 |
| 2.00 | 32 | 11 |
| 4.00 | 32 | 12 |

A2 samples on its own lattice, 2⁶ × 2⁶, whose saturation bond is 64. The Cartesian 32 is therefore
neither a cap nor a ceiling but an uncapped measurement at roughly half saturation. Why all four
λ=0 rows land on exactly 32 is not explained here and was not investigated. The fitted coordinate
holds `O(10)` and grows only weakly with thrust, so the blend-metric dial works on a
plume-plus-shock field, not just on a clean bow shock.

*B, fork economics:* sharing is structural, with **shares fluid + field = true** at a setup cost of
**42 ns**. Continuation cost against an unforked trunk is a ratio of **1.00 to 1.04** for every
powered branch, and coast is cheaper at 0.67. The mirrored post-fork bond is **16, flat** across the
roster. Final-field L2 against coast spreads 0.0 to 0.608 with throttle, so the branches genuinely
diverge; the corridor's branch-invariant flow columns are the explicit foil.

**Conclusion.** Both M1 risks land **green**. Plume rank is viable, and viable cheaply under the
blend metric. Forking is structurally O(1) with a continuation cost indistinguishable from an
unforked march. Feeds the verdict in
`openspec/notes/cfd-plasma-retropulsion/derisk-verdict.md`.

**Caveats.** Only structural breaks gate: fork sharing lost, rank saturating the representational
ceiling, or an errored branch. Degraded-but-measured outcomes, such as a poor step-cost ratio or
rank viable only under the blend metric, are printed as findings for the verdict note rather than
failed. A2 is an *analytic proxy* field, not a marched plume. A1's bond 16 is read with no bond
cap at relative tolerance 1e-6, so it measures what that tolerance retains rather than the exact
rank; a tighter tolerance would admit a larger bond. The post-fork bond of 16 is read on the
bare-marcher mirror, not on the forked carrier: the mirror's pre-pause segment marches no plume
where the trunk world does, and it applies sponge and body every step where a carrier world applies
only its forcing region. No observable is compared between the two at the pause point. The
continuation-time band was pinned from the first run on one machine.

See `output.txt` for the recorded reference output.
