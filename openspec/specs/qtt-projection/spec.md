# qtt-projection Specification

## Purpose
TBD - created by archiving change add-cfd-qtt-incompressible-2d. Update Purpose after archive.
## Requirements
### Requirement: Tensor-train pressure projection

The `tensor_bridge` module SHALL provide a divergence-free (Leray) projection in tensor-train form:
`divergence(u, v)` assembling `∇·u = ∂ₓu + ∂ᵧv`, a pressure-Poisson solve `∇²p = ∇·u`, and
`project(u*, v*) → (u, v)` returning the velocity with the gradient of the pressure removed. For the
periodic case the Poisson solve SHALL be **spectral/exact** — the Laplacian is diagonal in the Fourier
basis, so the solution is a per-mode division with the constant (`k=0`) mode zeroed, which pins the null
space by construction (no regularization, no iteration). The projected velocity SHALL be discretely
divergence-free to tolerance, and the projection SHALL rest on `∇p` (which is unique despite the singular
operator), not on the non-unique `p`.

#### Scenario: Projection removes the divergence
- **WHEN** an arbitrary (non-solenoidal) velocity field is projected
- **THEN** the divergence of the projected field is at or below the solver tolerance

#### Scenario: A divergence-free field is unchanged
- **WHEN** an already divergence-free field is projected
- **THEN** the result equals the input to the solver tolerance (the projection is idempotent)

#### Scenario: Singular Poisson is handled
- **WHEN** the pressure-Poisson solve is run on the singular periodic Laplacian
- **THEN** it returns a finite pressure (the constant null space pinned), not a divergent or NaN result

