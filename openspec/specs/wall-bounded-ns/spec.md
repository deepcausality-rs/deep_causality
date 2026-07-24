# wall-bounded-ns

## Purpose

The solver's acceptance of wall-bounded manifolds: which lattices the DEC Navier–Stokes
solver admits once one or more axes are open rather than periodic, and what it must reject.

## Requirements
### Requirement: Solver acceptance of wall-bounded manifolds
`DecNsSolver` SHALL accept manifolds over mixed-periodicity and all-walls
uniform lattices, wiring the constrained Leray projection, the no-slip
chain stage, the optional moving-wall lift, and wall-aware CFL sampling
(the advective bound's global max includes wall-adjacent and lid speeds).
Construction SHALL fail with a typed error when the wall substrate is
unusable: a wall axis with fewer than two vertex layers, or a metric
whose grade-1 star vends a non-positive or non-finite mass on a walled
lattice (the operational form of "lacks the boundary-corrected star" —
the correction itself is intrinsic to the cubical metric, so these are
the constructible conditions that break it). Seeding SHALL project and
constrain the initial field so the march starts divergence-free and
no-slip-consistent. Fully periodic construction and behavior SHALL be
unchanged.

#### Scenario: Wall-bounded construction succeeds and marches
- **WHEN** a solver is built over a periodic-x/wall-y manifold with the corrected star and marched
- **THEN** every step output is divergence-free at the solve's exactness, no-slip holds exactly at every step boundary, and the march completes without error

#### Scenario: Unusable wall substrate is rejected
- **WHEN** a solver is built over a walled lattice with a wall axis of extent 1, or whose metric vends a non-positive or non-finite grade-1 mass
- **THEN** construction returns a typed error naming the requirement

#### Scenario: Periodic behavior is preserved
- **WHEN** the existing periodic solver test suite runs
- **THEN** all results are unchanged
