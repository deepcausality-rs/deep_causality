<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## MODIFIED Requirements

### Requirement: Shot statistics follow the scalar and never pin `f64`

Shot statistics SHALL be generic over the scalar and bounded at `R: RealField + FromPrimitive`, covering the point estimate, its standard error and every separation quantity derived from a histogram, and they SHALL NOT name `f64` outside a display or verdict boundary.

The design note's §6.4 had this row at `Real + FromPrimitive`, on the reasoning that `sqrt`, `log2`
and ratios touch no complex carrier so dual numbers should stay admissible. The premise fails on
the first line of the estimator: `p = k / n` is a ratio, and `Real` in `deep_causality_algebra` is
`CommutativeRing + PartialOrd + Neg + …` with no `Div`. Division arrives with `Field`, so the
weakest structure that carries a frequency is `RealField`, and the row is corrected rather than
worked around. Dual numbers are not admissible here, and the surface says so.

Two shipped functions pin the scalar and are the pattern this requirement excludes.
`shots_to_qubit_bernoulli` accumulates `ones as f64 / total as f64` and returns `Uncertain<bool>`;
`shots_to_observable` takes `F: Fn(usize) -> f64` and returns `Uncertain<f64>`. Both keep their
signatures, and the scalar-generic estimator is a sibling beside them.

The `Uncertain` boundary no longer narrows the scalar. `deep_causality_uncertain` previously
admitted only the scalars for which it implemented `ProbabilisticType` — `f64` and `Float106`, and
not `f32` — so a scalar-generic estimate crossing into `Uncertain<R>` had to carry that bound and
inherit that restriction. That trait is removed and the uncertain crate's bound is blanket over the
same algebra this requirement names, so the boundary imposes nothing the estimator did not already
require.

#### Scenario: The same histogram is summarised at two precisions

- **WHEN** the read-out surface is instantiated at `f64` and again at `Float106` over the same
  `CountHistogram`
- **THEN** both compile, the standard error and the shot-noise threshold are computed at the
  instantiated scalar, and §10.4's precision sweep records a different tolerance for each

#### Scenario: The `Uncertain` boundary adds no restriction

- **WHEN** a scalar-generic estimate crosses into `Uncertain<R>`
- **THEN** the signature carries no bound from `deep_causality_uncertain` that narrows the admissible scalars, and the pipeline does not widen its `FloatType` alias to compensate

#### Scenario: A scalar the boundary once excluded now crosses it

- **WHEN** a scalar-generic estimate is instantiated at `f32` and crossed into `Uncertain<f32>`
- **THEN** it compiles
