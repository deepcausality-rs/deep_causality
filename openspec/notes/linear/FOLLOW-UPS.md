<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Follow-ups from `add-linear-algebra-crate`

Three defects surfaced while porting the linear algebra layer. None is inside
`deep_causality_linear`, and none needs a crate boundary to fix, so `add-linear-algebra-crate`
task group 8 recorded them as "file separately" rather than widening its own scope. This file is
where they were filed.

All three were **re-verified against the tree on 2026-08-30**, at archive time, and all three still
hold. Line numbers are from that check.

## 1. Two spatial-metric inversions with thresholds 100× apart

`deep_causality_physics/src/theories/general_relativity/gr_utils.rs:114` — `invert_3x3` — and
`adm_state.rs:126` — `inverse_spatial_metric` — compute the same 3×3 inverse and reject a singular
metric on different tests.

| site | threshold | carrier of the comparison |
|---|---|---|
| `gr_utils.rs:122` | `1e-14` | `T`, via `<T as From<f64>>::from(1e-14)` |
| `adm_state.rs:148` | `1e-12` | `f64`, via `det_f64.abs()` |

Two problems, not one. The thresholds differ by a factor of 100, so a metric rejected by one path
is accepted by the other. And the second compares in `f64`, which is lossy for `Float106` — the
extended-precision carrier the ADM state can be instantiated at — so the test discards the
precision the type exists to provide before deciding whether the matrix is singular.

Merging them is a `pub(crate) fn` inside `deep_causality_physics`. Deciding *which* threshold is
right, and in which carrier the comparison belongs, is the actual work; the merge is the easy half.

## 2. `CausalMultiField::inverse` documents an algorithm it does not run

`deep_causality_multivector/src/types/multifield/algebra/mod.rs:115` says "Uses matrix inverse for
each cell." The body at line 116 maps the multivector reversion inverse over the coefficients.

A doc/code mismatch inside `deep_causality_multivector`, unrelated to the linear crate. The two
agree on the answer only where reversion inverts, which is not every multivector; the docstring
should say what the code does, or the code should do what the docstring says. Which of those is
intended is a question for the geometric-algebra owner.

## 3. `CausalTensor::matmul` is over-bounded on `PartialOrd`

`deep_causality_tensor/src/types/causal_tensor/api/mod.rs:35` bounds the trait method
`T: Clone + Default + PartialOrd + Add + Mul`, and
`ops/tensor_product/mod.rs:13` bounds the implementation `T: Ring + Copy + Default + PartialOrd`.

Matrix multiplication is a sum of products. It needs no ordering, and `PartialOrd` on a published
surface excludes every number set that has none — the complex numbers, the quaternions and the
octonions, all of which live in `deep_causality_num_complex` and are `Ring`.

Before loosening it, record the set it currently excludes and confirm nothing downstream depends on
the bound being present. The change is small; the verification that it is safe is the work.
