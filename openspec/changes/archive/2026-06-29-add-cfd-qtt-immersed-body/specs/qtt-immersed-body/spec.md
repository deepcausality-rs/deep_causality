## ADDED Requirements

### Requirement: Rank-controlled body-mask tensor train

The `tensor_bridge` module SHALL provide a way to encode an immersed-body indicator as a
`CausalTensorTrain` on the `2^Lx × 2^Ly` grid: a **smoothed volume-fraction** field `χ_body ∈ [0, 1]`
(1 inside the body, 0 outside, smeared over a few cells) quantized and rounded, so its bond dimension
stays bounded. It SHALL provide a `body_mask_2d` helper for the analytic cylinder, and SHALL report the
resulting bond dimension so the smoothing width can be tuned against rank.

#### Scenario: Cylinder mask is bounded-rank
- **WHEN** a smoothed cylinder mask is built on the grid
- **THEN** it quantizes to a tensor train whose bond dimension is bounded (far below the dense element
  count), and dequantizing recovers the smoothed volume fraction within rounding tolerance

#### Scenario: Sharper masks cost more rank
- **WHEN** the smoothing width is reduced and the mask is quantized at a fixed bond cap
- **THEN** the reconstruction error (vs. the accurately-quantized mask) increases — i.e. a sharper body
  needs more rank to represent at the same fidelity, making the rank/accuracy trade-off explicit
  (the resolution-robust form, since bonds saturate the grid's rank ceiling at a fixed tolerance)

### Requirement: Brinkman-penalized no-slip tensor-train marcher

The `solvers/qtt` module SHALL provide a 2-D incompressible marcher that enforces an immersed body by
**volume penalization**: each step SHALL add the forcing `−(1/η)·χ_body ⊙ (u − u_body)` to the velocity
rate (via the fused Hadamard product + round), driving the velocity toward the body velocity inside the
body, then project as the body-free solver does. With no body (or `χ_body = 0`) it SHALL reduce to the
existing `QttIncompressible2d` behavior.

#### Scenario: No-slip is enforced inside the body
- **WHEN** the penalized marcher advances a flow past a static body to a quasi-steady state
- **THEN** the velocity magnitude inside the body region falls toward zero (to the penalization floor),
  while the field stays divergence-free and bond-bounded

#### Scenario: Reduces to the body-free solver
- **WHEN** the body mask is identically zero
- **THEN** the marched result matches `QttIncompressible2d` for the same configuration
