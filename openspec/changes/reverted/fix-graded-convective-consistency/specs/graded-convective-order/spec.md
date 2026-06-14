## ADDED Requirements

### Requirement: Second-order convective accuracy on smooth graded meshes
The solver SHALL restore approximately second-order accuracy of the convective term
`i_u(du)` on smoothly graded meshes, measured in the L2 (solution) norm, via a consistency
correction applied to the convective rate assembly. The correction SHALL be identically zero
on uniform spacing, and the diagonal Hodge star, the codifferential, and the Leray/Poisson
solve SHALL be unchanged by it (the diagonal fast-path is preserved). The correction SHALL
compose with the existing vector-slot skew-symmetrization without disturbing its energy
neutrality.

#### Scenario: Convective L2 order returns to second order under smooth grading
- **WHEN** the graded MMS example sweeps a smooth grading amplitude with the correction enabled
- **THEN** the convective operator's observed L2 order is approximately 2 across the sweep, where without the correction it collapsed toward zero

#### Scenario: The correction vanishes on uniform meshes
- **WHEN** the correction is evaluated on a uniform (ungraded) mesh
- **THEN** it is identically zero, and the Taylor–Green convergence table, Couette/Poiseuille exactness, and the energy budget are unchanged

### Requirement: Galerkin Hodge-star fallback for non-smooth meshes
The topology crate SHALL provide an opt-in Galerkin / Whitney (Q1) Hodge star that is
second-order consistent on arbitrary smooth meshes, for use where the surgical correction's
smoothness assumption breaks (cut cells, AMR). The diagonal Hodge star SHALL remain the
default; the Galerkin star SHALL reproduce the diagonal star's results to rounding on uniform
meshes and SHALL compose with the Stage-3 boundary-corrected star on walled / mixed-periodicity
lattices.

#### Scenario: Galerkin star recovers order where the surgical correction does not
- **WHEN** the convective MMS runs on a strongly / non-smoothly graded mesh with the Galerkin star selected
- **THEN** the observed L2 order is approximately 2, and on a uniform mesh the Galerkin star matches the diagonal star to rounding

### Requirement: Scope is convective-only and structure-preserving
The change SHALL leave the viscous operator (`Δ₀ = δd`) untouched, on the empirical basis
that it is already second order on graded meshes in both norms. Structure — divergence-
freeness of the Leray projection — SHALL stay exact at every grading, independent of the
accuracy correction.

#### Scenario: Viscous order and structure are unaffected
- **WHEN** the graded MMS and the divergence-free gates run before and after the change
- **THEN** the viscous operator's order is unchanged (≈ 2 at every amplitude) and the projected field stays divergence-free at the solve's exactness under strong grading
