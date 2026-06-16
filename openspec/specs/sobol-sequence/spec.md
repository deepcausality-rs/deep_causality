# sobol-sequence Specification

## Purpose
Quasi-Monte-Carlo sampling needs a low-discrepancy point source whose `i`-th point
is a deterministic function of its index, plus a way to randomize it so variance can
be estimated. This capability adds a Sobol sequence generator to
`deep_causality_rand` with an optional seeded digital shift (randomized QMC),
leaving the existing pseudo-random RNG untouched.

## Requirements

### Requirement: Sobol low-discrepancy sequence
`deep_causality_rand` SHALL provide a Sobol sequence generator that, for a
requested dimension `d`, produces points in the half-open unit cube `[0,1)^d`. The
`i`-th point SHALL be a deterministic function of `i` and `d` (independent of call
order and prior draws), so a given index is reproducible. The generator SHALL be
backed by a static direction-number table, contain no `unsafe`, and introduce no
new external dependency. The existing `Xoshiro256` RNG and `RngCore` entropy
source SHALL be unchanged.

#### Scenario: Points are deterministic by index
- **WHEN** the `i`-th `d`-dimensional Sobol point is requested twice, in any order relative to other indices
- **THEN** both requests return identical coordinates

#### Scenario: Points are equidistributed
- **WHEN** the first `N` points of a `d`-dimensional Sobol sequence are collected for a power-of-two `N`
- **THEN** their star discrepancy is lower than that of `N` independent uniform pseudo-random points, and every coordinate lies in `[0,1)`

### Requirement: Seeded digital shift (randomized QMC)
The generator SHALL support a randomized-QMC mode in which a per-run digital shift
is applied to every point, derived deterministically from a `u64` seed. Two runs
with the same seed SHALL produce identical shifted sequences; runs with different
seeds SHALL produce different shifts. The shift SHALL preserve the equidistribution
of the underlying sequence (a shifted Sobol sequence remains low-discrepancy).

#### Scenario: Same seed reproduces the shifted sequence
- **WHEN** a digitally shifted Sobol sequence is generated twice from the same seed
- **THEN** the two shifted sequences are identical point-for-point

#### Scenario: Different seeds give independent shifts
- **WHEN** shifted sequences are generated from two different seeds
- **THEN** the shift vectors differ, and averaging an integrand over several seeds yields an unbiased estimate with a computable between-seed variance
