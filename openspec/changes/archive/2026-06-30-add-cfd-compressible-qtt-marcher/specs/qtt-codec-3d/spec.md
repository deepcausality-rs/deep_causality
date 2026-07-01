<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: 3-D QTT field codec

`deep_causality_cfd` SHALL provide `quantize_3d` and `dequantize_3d` that map a dense `2^Lx × 2^Ly × 2^Lz`
field to and from a `CausalTensorTrain`, extending the existing 1-D/2-D `tensor_bridge` codec and matching its
bit-ordering convention. The round-trip SHALL reconstruct a smooth field to the truncation tolerance.

#### Scenario: Round-trip within tolerance
- **WHEN** a smooth 3-D field is `quantize_3d`-encoded at relative tolerance `tol` and `dequantize_3d`-decoded
- **THEN** the decoded field matches the original to within `tol`

#### Scenario: Dimension mismatch rejected
- **WHEN** `quantize_3d` is given a field whose length is not `2^Lx · 2^Ly · 2^Lz`
- **THEN** it returns a `PhysicsError::DimensionMismatch` rather than a malformed train

### Requirement: 3-D finite-difference MPO operators

`deep_causality_cfd` SHALL provide `gradient_x`, `gradient_y`, `gradient_z`, `laplacian_3d`, and a divergence
helper as `CausalTensorTrainOperator`s on the 3-D QTT layout, hand-built from the existing shift-operator
construction (`from_cores`) and stencil algebra. Each operator SHALL reproduce the corresponding central
finite-difference stencil to discretization order on a smooth field.

#### Scenario: Gradient matches the analytic derivative
- **WHEN** `gradient_x` is applied to a QTT-encoded smooth field with a known analytic derivative
- **THEN** the decoded result matches the analytic `∂_x` field to the scheme's order in the grid spacing

#### Scenario: Operators are bounded-rank on smooth input
- **WHEN** any 3-D operator is applied to a low-rank smooth field and rounded
- **THEN** the output bond dimension stays bounded (no spurious rank inflation from the operator itself)
