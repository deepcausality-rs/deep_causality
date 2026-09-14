<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Blanket scalar sampling

## Why

`retrofit-sampling-layer` deleted the per-type distribution files and reported parity with the
unified-math convention. It had not reached it. Measured at that change's own archive point,
`deep_causality_rand` named six concrete types in sampler implementations — `f32`, `f64` and
`Float106` on `SampleUniform`, and `u32`, `u64` and `usize` in three near-identical files — and
`BFloat16`, a scalar the workspace already ships, could not be drawn at all.

The convention is `deep_causality_tensor/src/extensions`, where
`impl<T> CausalTensorMathExt<T> for CausalTensor<T> where T: RealField` is the whole story and no
concrete float appears anywhere in the directory. `BFloat16` gets `log_nat` there without `tensor`
naming it. That is the test the sampling layer failed.

## What changes

Sampler implementations become blanket over the algebraic towers, and the count of concrete types
named in `deep_causality_rand` goes from six to **zero**.

The obstacle was a circular argument in the crate's own comments: the float bindings said they
could not be generic because a blanket would collide with the integers, and the integer bindings
said the same about the floats. One trait cannot carry two blanket implementations over disjoint
towers — but two *parameterised* implementations of one trait can. `FloatKind` and `UnsignedKind`
are type parameters and nothing else, and they make the two implementations distinct items.

The width table goes with it. `RandWidth` in `stats` and `RandFloat::WORDS` in `rand` stated the
same invented fact twice, each a hand-maintained list of types, and the scalar already reports its
own precision through `Real::epsilon()` on the bound the function already had.

## Impact

- `entropy-source` — the range machinery is generic; no binding names a type.
- `stats-sampling` — the width constant is withdrawn; the draw derives its width.
- Breaking: `SampleUniform` and `SampleRange` take a `Kind` parameter, `Uniform<X>` becomes
  `Uniform<X, K = FloatKind>`, `RandFloat` is renamed `RandScalar` and loses `WORDS`, `RandWidth`
  is removed, the three integer samplers become `UniformUnsigned<T>`, and `Uniform::new` is
  strictly half-open at every scalar.
