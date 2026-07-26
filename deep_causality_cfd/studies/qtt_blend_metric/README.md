<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `qtt_blend_metric` — is body-fit a valid, low-rank free parameter? (Res 4 / D8)

```bash
cargo run --release -p deep_causality_cfd --example qtt_blend_metric
```

**What it tests.** Resolution 4 promotes the coordinate to a `MetricProvider` carrying a
continuous blend `T_λ = (1−λ)·T_cart + λ·T_fit`. At `λ = 0` this is Cartesian capture: any
geometry, high rank. At `λ = 1` it is full body-fitting: this geometry, low rank. Two claims must
hold before `λ` counts as a usable dial. The blended map has to stay **valid** across the whole
sweep, meaning no folded cells, and the rank has to run **monotonically** with `λ`. Both charts
are compatibly oriented over the same `(ξ, η)` patch in front of the nose, on a 256² lattice.

**Findings (gated, exit nonzero on regression).**

| λ | min\|det J\| | det J sign | shock-field bond |
|---|---|---|---|
| 0.00 | 2.1213 | − | 114 |
| 0.25 | 1.7809 | − | 107 |
| 0.50 | 1.5757 | − | 92 |
| 0.75 | 1.5057 | − | 54 |
| 1.00 | 1.5708 | − | 5 |

- **BM-A: validity holds.** `min|det J| = 1.506 > 0` with **one sign** across the whole sweep. The
  position-blend of two compatibly-oriented charts does not fold. That closes the one open
  residual Resolution 4 flagged; the guard it depends on, a bounded λ-gradient plus a positive
  Jacobian, is satisfied here by construction.
- **BM-B: λ moves the rank monotonically, but the reduction concentrates near λ = 1.** A fixed
  *physical* curved shock, sampled on the blended lattice, falls **114 → 107 → 92 → 54 → 5** as `λ`
  goes 0 to 1. No interval reverses, so the dial is monotone. The split across intervals is uneven:
  three quarters of the dial buys a factor of 2.1 (114 → 54), and the last quarter buys the
  remaining factor of 10.8 (54 → 5). A partial blend therefore delivers little rank benefit. At
  `λ = 0.75` the bond is still about 11× the fitted value. Body-fittedness is a continuous free
  parameter rather than an all-or-nothing commitment, but the operationally useful range sits above
  `λ ≈ 0.75`. The knee is not located: only the five `λ` in the table were measured, and
  intermediate points such as 0.85 and 0.95 were not run.

**Conclusion.** The blend is valid and monotone in `λ`, so D8 can treat the coordinate as a tunable
`MetricProvider` rather than a fixed choice. The Res-4 residual is closed. A partial blend is not a
cheap middle ground, though: the rank payoff arrives above `λ ≈ 0.75`, so a tuner should expect to
run near full fit. Analysis: `openspec/notes/plasma-blackout/gap-2/`.

**Caveats.** This measures the **static** representability of a frozen analytic shock on the
blended lattice. It says nothing about whether marching holds the alignment; that is the question
`qtt_rank_fitted_dynamic` answers, where a static fit turns out not to self-bound, and
`qtt_repin_marcher` resolves. Two charts over one patch is also the easy case for non-folding. A
blend over a full domain with strong curvature needs the guard checked, not assumed.

See `output.txt` for the recorded reference output.
