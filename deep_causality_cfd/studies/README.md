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
deliverable (`add-park2t-blackout-tier-a`) does **not** depend on these — they de-risk the *Tier-B*
compressible marcher.
