<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `qtt_rank_study` — static rank probe for the Tier-B compressible thesis

```bash
cargo run --release -p deep_causality_cfd --example qtt_rank_study
```

**What it tests.** The Tier-B shock-capturing plan rests on one load-bearing assumption: *the
reentry flowfield is low tensor-train rank, so a `2^L` grid costs `O(χ²·L)`.* This example measures
the actual QTT bond dimension of shock-like profiles with the real codec (`quantize` / `quantize_2d`
plus TT-SVD) and settles whether that assumption holds.

**Findings (gated, exit nonzero on regression).**

- **A discontinuity is not intrinsically high rank.** A sharp 1-D step is rank 2 or less at *any*
  position, dyadic or not, and the captured 1-D stagnation-line profile (uniform, shock, relaxation
  tail) is rank 3. **1-D is cheap either way, so shock-fitting is unnecessary in 1-D.**
- **The rank driver is coordinate *alignment*, not curvature.** In 2-D at 512², a flat axis-aligned
  shock is χ ≈ 5, a **curved** bow shock is χ ≈ 151, and a **straight 45° oblique** shock is
  χ ≈ 394, worse than the curve. Misalignment with the codec axes, amplified by its block
  bit-ordering, is the cost; straight or curved makes little difference.
- **The fix collapses it, by construction.** The same bow shock in a body-fitted polar coordinate,
  where it is a function of `r` only, is χ ≈ 5, down from 151. The oblique in an aligned coordinate
  is χ ≈ 5, down from 394, and stays near 5 across tolerances 1e-4 to 1e-12.
- **Cost verdict.** For 512², meaning 262 144 dense values: aligned χ = 5 is roughly 900 parameters
  and **291× smaller**; captured curved χ = 151 is **3.1× larger** than dense; captured oblique
  χ = 394 is **21× larger**. Capturing a misaligned shock makes QTT net-negative. Aligning it makes
  QTT a roughly 290× win.

**Conclusion.** The low-rank property is **not** automatic, but it **is** achievable by construction
through a shock-aligned or body-fitted coordinate. Tier-B must commit to such a coordinate, meaning
singularity confinement, rather than capturing on a fixed Cartesian grid. Analysis:
`openspec/notes/plasma-blackout/gap-2/`.

**Caveats.** This measures *static* representability of frozen analytic profiles in isolation, with
the codec's block bit-ordering. It does not test a live marched solution, which is `qtt_rank_dynamic`,
nor the shock plus boundary-layer plus wake superposition. A minor codec edge case is noted in-run:
a zero-norm field reports a spurious bond dimension because the relative tolerance degenerates,
whereas a true constant is rank 1.

See `output.txt` for the recorded reference output.
