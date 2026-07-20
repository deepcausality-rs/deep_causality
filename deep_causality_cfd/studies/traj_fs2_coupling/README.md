<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `traj_fs2_coupling` — can non-conformal aero ride a between-step kick?

```bash
cargo run --release -p deep_causality_cfd --example traj_fs2_coupling
```

**What it tests.** Gap-3 Resolution-3, de-risking item ③, which is the corridor's standing
`[open]`: "coupling Bars 2T to non-conformal external forcing is a research move, not textbook".
The thesis under test is that you do **not** need to express aero inside the conformal or
regularised algebra. Split the step instead: an exact inverse-square core, the FS-1 generator, here
exact Kepler drift, plus a between-step perturbation **kick applied in physical Cartesian
velocity**. If the symmetric Strang composition is 2nd-order accurate and its error vanishes with
the perturbation, the `[open]` concern dissolves.

**Method.** A bound ellipse perturbed by a **mock drag** `a = −k·v`, non-conservative and therefore
the hard energy-changing aero analog. Strang per macro-step `H`: `v ·= e^(−kH/2)`, then exact
Kepler drift `H`, then `v ·= e^(−kH/2)`. Both sub-flows are exact, so the only error is the
operator non-commutator. The reference is RK4 on the full EOM `ẍ = −μx/r³ − k·v` at a tiny step.
Orbit period 5876.0 s, `k = 1.0e-6/s`, giving `ε = |a_aero|/|a_grav| = 9.301e-4`.

**Findings (gated, exit nonzero on regression).**

| N | H | \|error\| |
|---|---|---|
| 50 | 117.5 s | 4.4075e1 m |
| 100 | 58.8 s | 1.1017e1 m |
| 200 | 29.4 s | 2.7542e0 m |
| 400 | 14.7 s | 6.8855e-1 m |
| 800 | 7.3 s | 1.7215e-1 m |

- **G1: the split is 2nd-order.** Observed order on the finest pair is **2.000**.
- **G2: the error vanishes with the perturbation.** At N = 200, shrinking `ε` by 10× drops the split
  error from 2.754e0 to 2.756e-1 m, a clean **10.0×**. As `ε` goes to 0 the exact Kepler drift is
  the whole answer.
- **G3: a moderate macro-step tracks the reference.** Relative accuracy at N = 200 is
  `|error|/a = 3.913e-7`.

**Conclusion.** Non-conformal aero rides a between-step Cartesian kick at 2nd order with the
inverse-square core left an untouched exact matrix exponential. The `[open]` concern **dissolves**:
you split in physical space, Encke/Strang, and you do not express aero in the conformal algebra.
B1's perturbation factoring holds, and Resolution 1 simplifies, since a physical-space split
replaces a hand-set conformal-coupling law. Analysis:
`openspec/notes/plasma-blackout/gap-3/gap-three-resolution-3-trajectory-axis.md`.

**Caveats.** The perturbation is a **mock** drag `−k·v`, not a real atmospheric model. It is chosen
as the hard case because it is non-conservative, but a real aero force is state- and
attitude-dependent and much stiffer at low altitude. The setup is 2-D, on a bound orbit, at a small
`ε ≈ 9e-4`. Second order is a property of the split; the *magnitude* of the error at flight `ε` is
not established here. The real aero interface (Tier-B Stage-4+) is what gates the corridor's
Phase 2.

See `output.txt` for the recorded reference output.
