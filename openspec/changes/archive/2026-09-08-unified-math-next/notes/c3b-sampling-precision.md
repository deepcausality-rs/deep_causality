<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# 3b — Lund sampling precision

Decision: **the entropy claim holds**. The comments are corrected; the sampling is not rerouted.

## The capability claim was stale

Both module docs said `deep_causality_rand` implements `Distribution` only for `f32` and `f64`.
`dist_float_106.rs` implements it for `Float106` under `StandardUniform`, `Open01`, `OpenClosed01`
and `StandardNormal`. A third copy of the claim sat on `generate_transverse_momentum`.

## Five draw sites, not two

| Site | Draw | Output |
|---|---|---|
| `flavor.rs:74` | uniform × weight total | discrete flavour index |
| `flavor.rs:130` | uniform vs a fixed fraction | bool |
| `flavor.rs:145-146` | two unit normals | transverse momentum |
| `kinematics.rs:135` | uniform → `z_min + (z_max − z_min)·u` | `z` in `[0.01, 0.99]` |
| `kinematics.rs:141` | uniform → accept/reject test | one bit |

Three of the five produce discrete outputs, where bits below `2^-53` change the result only when
the draw lands within about `1e-16` of a boundary. The fourth is affine on a bounded interval: the
`2^-53` uniform grid induces a spacing of about `1.1e-16` in `z`, uniform across the interval, with
no singular transform and no tail whose reach the granularity sets.

## The Gaussian, where the wider draw is the narrower one

`f64` `StandardNormal` is a ziggurat whose `zero_case` is the Marsaglia tail algorithm — a
rejection loop over two `Open01` logarithms, so its reach is unbounded.

`Float106` `StandardNormal` is Box–Muller: `radius = sqrt(-2·ln u1)` with `u1` drawn from `Open01`,
whose high part floors at `2^-53`. That caps the radius at `sqrt(2 · 53 · ln 2) = 8.5717`, and the
low part only adds below the floor, so it does not extend the reach.

Measured: analytic cap 8.5717; over 2×10⁶ draws the largest `|z|` seen was 5.50 at `Float106` and
5.77 at `f64`. `P(|z| > 8.57) ≈ 1e-17`, so the cap is unreachable at any feasible sample count and
decides nothing either way. The point stands only as a correction to the intuition that the wider
type must sample the tail better.

## Cost of the change not made

A `Float106` stream is not a refinement of the `f64` one. A `Float106` uniform is assembled from
two `f64` draws (a 53-bit high part plus a 53-bit low part scaled by `2^-53`) and its normal from
four, so rerouting would replace every seeded sequence rather than extend it. Nothing instantiates
these kernels at `Float106` today: `generic_real_field_tests.rs` covers `f32` and `f64` only.

## Not verified adversarially

A four-lens workflow was launched to check this reasoning and did not complete. The conclusion
rests on the evidence above — read from the tree and measured — reviewed by one party, and was
taken on that basis at the maintainer's direction.
