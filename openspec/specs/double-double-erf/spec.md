# double-double-erf Specification

## Purpose
`deep_causality_num`'s `Float106` carries `exp`, `ln`, and `sqrt` but no error
function, which blocks any double-double computation that needs the normal CDF —
in particular the `Float106` normal-quantile refinement used by the Quasi-Monte-Carlo
sampler. This capability adds `erf`/`erfc` for `Float106` at double-double precision,
computed by pure arithmetic over the existing operations.

## Requirements

### Requirement: Double-double error function for Float106
`deep_causality_num` SHALL provide `erf` and `erfc` for `Float106`, accurate to
double-double precision across the full real line. The implementation SHALL use a
representation appropriate to the argument region (e.g. a Maclaurin/series form
for small `|x|` and a continued-fraction or rational form for the tail) and SHALL
avoid catastrophic cancellation — in particular `erfc(x)` for large positive `x`
SHALL be computed directly rather than as `1 - erf(x)`. The functions SHALL be
pure arithmetic over the existing `Float106` operations (`exp`, `ln`, `sqrt`,
multiply, add) with no `unsafe` and no new external dependency.

#### Scenario: erf matches reference values to double-double precision
- **WHEN** `erf` and `erfc` are evaluated at `Float106` arguments spanning small, moderate, and tail regions (e.g. 0, 0.5, 1, 3, 6 and their negatives)
- **THEN** each result agrees with a high-precision reference to within a few units in the last double-double place

#### Scenario: Identities hold
- **WHEN** `erf(x)` and `erfc(x)` are evaluated for any tested `Float106` `x`
- **THEN** `erf(x) + erfc(x)` equals one to double-double tolerance, and `erf(-x)` equals `-erf(x)` exactly

#### Scenario: Tail is computed without cancellation
- **WHEN** `erfc(x)` is evaluated for large positive `x` where `1 - erf(x)` would lose all significant digits in f64
- **THEN** the returned value retains double-double relative accuracy
