# qtt-immersed-body Specification

## Purpose
TBD - created by archiving change add-cfd-qtt-immersed-body. Update Purpose after archive.
## Requirements
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

### Requirement: Immersed-cylinder validation constrains the reported drag

The `qtt_cylinder_verification` gate set SHALL include at least one gate that constrains the reported drag
coefficient against a parameter it actually depends on. No gate in the present set does: one cannot fail,
one is provably invariant under the parameter that dominates the answer, and one carries eleven orders of
margin.

Specifically the harness SHALL gate a **smoothing-width ladder** and an **η ladder** as first-class checks
alongside the existing bond ladder, and SHALL tighten `CONVERGENCE_BOUND` to the scale of the difference it
measures.

Measured behaviour that motivates this, recorded so the gates are interpretable:

- `C_d` moves **6.1×** with the mask smoothing width — 7.70, 12.33, 23.76, 35.81, 47.27 at 0.5, 1, 2, 3, 4
  cells — while the no-slip gate passes identically across that entire range.
- `C_d` is **non-monotone** in the penalization parameter — 17.39, 24.02, 26.25, 23.76, 21.40 at
  η = 0.128, 0.064, 0.032, 0.016, 0.008 — and is still drifting at the finest η, so no η → 0 limit is
  demonstrated. Establishing that limit is what would license calling the penalization integral a drag
  (Angot, Bruneau & Fabrie 1999).
- `CONVERGENCE_BOUND = 0.10` gates a measured successive difference of `1.89e-11`.

Where a ladder does not converge, the harness SHALL report the non-convergence as its result rather than
passing silently. Establishing the physical η envelope is out of scope here and is handled by the Phase 2
remediation; this requirement makes the harness capable of *detecting* the condition.

#### Scenario: Smoothing-width ladder is gated

- **WHEN** the harness runs its smoothing-width ladder
- **THEN** it reports `C_d` at each width and gates the trend, so a result that scales with a purely
  numerical parameter cannot pass unremarked

#### Scenario: Penalization ladder is gated

- **WHEN** the harness runs its η ladder
- **THEN** it reports `C_d` and the interior slip at each η and gates whether the sequence is converging,
  failing or explicitly reporting non-convergence when it is not

#### Scenario: Bond-convergence bound matches the phenomenon

- **WHEN** the bond ladder completes
- **THEN** the successive-difference bound is set at the scale of the measured differences rather than
  orders of magnitude above them, so a solver that had not saturated in bond would fail

#### Scenario: A parameter-dependent result cannot pass silently

- **WHEN** the reported `C_d` changes materially under a parameter the gate set covers
- **THEN** at least one gate responds to that change, and the harness output records which parameter moved
  the answer

#### Scenario: Cross-references state their configuration

- **WHEN** the harness prints the DEC isolated-cylinder cross-reference
- **THEN** the Reynolds number of both cases is shown — the QTT case runs at `Re = 37.7` against a DEC
  reference at `Re = 100` — and the value is marked as a disclaimed cross-reference, not a gate

