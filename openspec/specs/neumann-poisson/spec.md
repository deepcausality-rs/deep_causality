# neumann-poisson

## Purpose

The pressure-Poisson solve under Neumann boundary conditions on wall-bounded boxes: a
direct spectral solve where the geometry is uniform, and a Jacobi-preconditioned CG
fallback everywhere else.

## Requirements
### Requirement: Direct spectral Neumann solve on uniform wall-bounded boxes
`deep_causality_topology` SHALL solve the gauge-fixed grade-0 Poisson
problem with no-flux (Neumann) wall semantics directly on uniform Euclidean
lattices whose axes are each either periodic or walled: per-axis transforms
(DFT on periodic axes, DCT of the type that diagonalizes the implemented
boundary-corrected `Δ₀` on wall axes), pointwise division by the summed
per-axis eigenvalues with the gauge mode zeroed, inverse transforms. Mixed
axis sets SHALL ride a complex carrier (DCT applied independently to real
and imaginary parts). The solve SHALL have no tolerance, iteration budget,
or convergence-failure mode.

#### Scenario: Residual at rounding against the implemented operator
- **WHEN** the boundary-corrected `Δ₀` is applied to the direct solution on an all-walls box and on a mixed periodic/wall box
- **THEN** the residual against the (gauge-projected) right-hand side is at rounding level for the precision

#### Scenario: Agreement with preconditioned CG
- **WHEN** the same Neumann problem is solved directly and by the preconditioned CG fallback on multiple shapes including anisotropic spacings
- **THEN** the gauge-fixed solutions agree within the CG tolerance

#### Scenario: No flux through walls (DEC form)
- **WHEN** the projected field `ω − dφ` is computed from the direct solution
- **THEN** its divergence vanishes to rounding at every vertex including wall and corner vertices, whose clipped dual volumes realize the no-flux wall condition discretely (amended at implementation: the variational Neumann condition is encoded in the boundary control volumes, not as pointwise vanishing of boundary-normal gradient components)

### Requirement: Jacobi-preconditioned CG fallback
`deep_causality_sparse` SHALL provide a preconditioned variant of the
matrix-free CG (additive API), and the wall-aware grade-0 solve SHALL use
it with the diagonal of the boundary-corrected `Δ₀` wherever the direct
solve does not apply (per-edge metrics, non-uniform geometry). Plain
`cg_solve` semantics are unchanged.

#### Scenario: Preconditioned CG converges faster on walled lattices
- **WHEN** the same walled-lattice Poisson problem is solved by plain and Jacobi-preconditioned CG at the same tolerance
- **THEN** both converge to agreeing solutions and the preconditioned solve uses no more iterations (strictly fewer on the recorded benchmark case)

#### Scenario: Existing CG callers unaffected
- **WHEN** the existing `cg_solve` test suite runs
- **THEN** all results are unchanged
