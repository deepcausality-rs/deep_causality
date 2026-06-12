## ADDED Requirements

### Requirement: Solver acceptance of wall-bounded manifolds
`DecNsSolver` SHALL accept manifolds over mixed-periodicity and all-walls
uniform lattices, wiring the Neumann projection path, the no-slip chain
stage, and wall-aware CFL sampling. Construction SHALL fail with a typed
error if a wall axis lacks the boundary-corrected star. Seeding SHALL
project and constrain the initial field so the march starts
divergence-free and no-slip-consistent. Fully periodic construction and
behavior SHALL be unchanged.

#### Scenario: Wall-bounded construction succeeds and marches
- **WHEN** a solver is built over a periodic-x/wall-y manifold with the corrected star and marched
- **THEN** every step output is divergence-free at the solve's exactness, no-slip holds exactly at every step boundary, and the march completes without error

#### Scenario: Missing star correction is rejected
- **WHEN** a solver is built over a walled lattice whose metric does not carry the boundary correction
- **THEN** construction returns a typed error naming the requirement

#### Scenario: Periodic behavior is preserved
- **WHEN** the existing periodic solver test suite runs
- **THEN** all results are unchanged
