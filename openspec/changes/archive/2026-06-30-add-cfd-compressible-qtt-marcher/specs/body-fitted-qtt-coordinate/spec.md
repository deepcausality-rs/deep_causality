<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: Shock-aligned / body-fitted curvilinear coordinate

`deep_causality_cfd` SHALL provide a smooth curvilinear coordinate map that aligns the body surface and the
bow-shock surface to **coordinate surfaces** (e.g. a wall-normal / shock-normal coordinate), carried as a
**low-rank Jacobian** in tensor-train form. The map SHALL be analytic and invertible over the computational
domain, and the metric/Jacobian SHALL be computed from the geometry (dynamic-by-construction — no hardcoded
metric components).

#### Scenario: A curved shock becomes axis-aligned and low-rank
- **WHEN** a curved (spherical-shell) shock field is expressed in the body-fitted coordinate where it is a
  function of the wall-/shock-normal coordinate only, and QTT-encoded
- **THEN** its bond dimension is `O(10)` and independent of resolution — matching the measured
  `qtt_rank_study` / `qtt_rank_3d` result (vs `χ ~ √side` in Cartesian)

#### Scenario: Jacobian is low-rank
- **WHEN** the coordinate Jacobian is QTT-encoded
- **THEN** its bond dimension is bounded and small (the smooth stretch does not itself inflate rank)

### Requirement: Chain-rule operator transformation

The finite-difference operators (`qtt-codec-3d`) SHALL be transformable to the body-fitted coordinate by the
chain rule using the low-rank Jacobian, so that physical derivatives are computed on the aligned lattice. The
transformed operators SHALL recover the physical gradient/divergence on a smooth field to discretization
order.

#### Scenario: Physical gradient via the transformed operator
- **WHEN** the chain-rule-transformed gradient is applied in the body-fitted coordinate to a field with a
  known physical derivative
- **THEN** the decoded result matches the analytic physical `∇` field to the scheme's order

### Requirement: Rank-lever acceptance gate

The body-fitted coordinate SHALL be demonstrated to hold a representative reentry shock at `χ ~ O(10)`
independent of resolution, where the same shock captured on a Cartesian grid grows as `χ ~ √side`. This is the
measured precondition that makes the 3-D marcher tractable.

#### Scenario: Resolution-independent rank in the fitted coordinate
- **WHEN** the shock rank is measured in the fitted coordinate across a resolution sweep
- **THEN** the bond dimension stays approximately constant (does not grow with side), unlike the Cartesian
  capture
