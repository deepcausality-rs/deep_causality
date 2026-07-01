<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `qtt_rank_3d` — the 3-D upper bound from a realistically-formed shock

```bash
cargo run --release -p deep_causality_cfd --example qtt_rank_3d
```

**What it tests.** 3-D is where avionics / space CFD actually lives. The question: when a curved shock
*surface* forms in 3-D, how large is the QTT bond dimension, and how does it **scale with resolution**?
That scaling is the real Tier-B verdict for the "low tensor-train rank" thesis.

**Method (realistic, per request).** Form the shock with the canonical naive scheme — **explicit Euler +
central differences** — on the true 3-D Burgers equation (`u_t + ½∇·(u²) = ν∇²u`); a smooth radial bump
self-advects into a curved front. The march runs in the **dense** representation (so shock formation is
exact, no QTT-solver approximation) and the field is QTT-encoded each sample step to read the bond
dimension a 3-D tensor-train solver *would have to carry*. This needs no 3-D QTT marcher (the crate has
none yet) and is an honest **lower bound** on a live solver's rank.

**Findings (gated, exit nonzero on regression).** Measured on an Apple M3 Max (release):

| side | formed-shock peak χ | flat / body-fitted χ |
|---|---|---|
| 16³ | 45 | — |
| 32³ | 56 | — |
| 64³ | 89 | flat 5, fitted 6 |
| 128³ | 135 | — |

- **Upper bound: χ ~ side^0.53 (≈ √side) — unbounded in resolution.** A captured 3-D curved shock
  surface costs `45 → 135` over `16³ → 128³`; the flat and body-fitted references stay **constant at
  ~5–6**.
- **QTT-vs-dense storage flips with resolution (a crossover, not a wall).** `dense/QTT` storage ratio:
  `0.08× (16³) → 0.35× → 0.92× (64³ break-even) → 2.74× (128³)`. Because dense grows as `side³` while
  `χ² ~ side^1.1`, QTT storage **always wins asymptotically** — the 64³ "break-even" is a small-grid
  artifact, not the point.
- **The real result is the solve cost, not storage.** Tensor-train ops are `O(χ²)–O(χ³)` per core.
  `χ ~ √side` means at a flight-relevant micrometre grid (`side ~ 10⁶`) a captured curved shock implies
  `χ ~ thousands` — bounded, but expensive enough to erode the practical advantage. The **body-fitted
  shell holds χ ~ O(10) at any resolution.** That gap is the Tier-B-deciding result.

**Conclusion.** 3-D Tier-B is tractable **only** with a shock-aligned / body-fitted coordinate: it turns
the curved surface into an axis-aligned one, replacing `χ ~ √side` with `χ ~ O(10)`. Capturing the curved
shock on a Cartesian QTT grid keeps storage sub-dense but gives back most of the compression win exactly
where it is needed. This is the 3-D confirmation of the `qtt_rank_study` finding (alignment is the lever)
and the dynamic `qtt_rank_nonlinear` finding (forming shocks reach the structural rank). Analysis:
`openspec/notes/plasma-blackout/gap-2/`.

**Caveats.** Burgers is a scalar model, not compressible Euler/NS; explicit Euler + central differences
is dispersive, so the measured χ includes some dispersive-oscillation rank atop the irreducible curvature
floor (a monotone scheme would trim it, not remove the √side growth). The rank is a lower bound — a live
3-D QTT solver carries operator products before rounding. 128³ is the practical ceiling for repeated
`from_dense` here; the √side law is read from four points (16³–128³).
