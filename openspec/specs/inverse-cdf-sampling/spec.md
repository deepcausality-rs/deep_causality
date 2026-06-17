# inverse-cdf-sampling Specification

## Purpose
A low-discrepancy sequence only retains its equidistribution if each draw is a
monotone, single-uniform transform of its coordinate; rejection methods (Ziggurat,
Box–Muller) destroy that structure. This capability adds inverse-CDF (quantile)
transforms to `deep_causality_rand` for the Normal, Uniform, and Bernoulli
distributions at `f64` and `Float106`, providing the sampling primitive for the
Quasi-Monte-Carlo path.

## Requirements

### Requirement: Inverse-CDF transforms for the supported distributions
`deep_causality_rand` SHALL provide inverse-CDF (quantile) transforms that map a
uniform input `u ∈ [0,1)` to a draw from the Normal, Uniform, and Bernoulli
distributions, for the precisions `f64` and `Float106`. Each transform SHALL be a
deterministic, monotone function of `u` that consumes exactly one uniform input
(no rejection, no variable consumption), so that it preserves the structure of a
low-discrepancy sequence. The transforms SHALL share distribution parameter types
with the existing forward `sample` surface.

#### Scenario: Uniform inversion is exact
- **WHEN** the Uniform quantile is evaluated at `u` for parameters `low`, `high`
- **THEN** the result equals `low + u·(high − low)` at the requested precision, with the `Float106` path carrying double-double values without narrowing through f64

#### Scenario: Bernoulli inversion thresholds on p
- **WHEN** the Bernoulli quantile is evaluated at `u` for probability `p`
- **THEN** it returns `true` iff `u < p`

#### Scenario: One uniform in, one draw out
- **WHEN** any supported inverse-CDF transform is evaluated
- **THEN** it consumes exactly one uniform `u` and performs no rejection step

### Requirement: Normal quantile at f64 and Float106
The Normal inverse-CDF SHALL compute the standard-normal quantile at `f64` via a
rational approximation (e.g. Acklam/Wichura) and then affine-map by `mean` and
`std_dev`. For `Float106`, the f64 quantile SHALL be used as an initial guess and
refined to double-double accuracy by Halley (or Newton) iteration on
`Φ(x) − u = 0`, using the double-double `erfc` and `exp`. The transform SHALL be
monotone non-decreasing in `u` and SHALL handle the open-interval endpoints
without producing non-finite results.

#### Scenario: Float106 normal quantile is refined beyond f64
- **WHEN** the `Float106` Normal quantile is evaluated at a `u` whose f64 quantile carries a known double-double residual
- **THEN** the refined result reduces `|Φ(x) − u|` to double-double tolerance, below what the widened f64 quantile achieves

#### Scenario: Round-trip against the CDF
- **WHEN** the Normal quantile `x = Q(u)` is computed and then pushed back through the normal CDF at the same precision
- **THEN** the recovered probability equals `u` to that precision's tolerance, at both `f64` and `Float106`

#### Scenario: Monotonicity and finite endpoints
- **WHEN** the Normal quantile is evaluated on an increasing grid of `u` values within `(0,1)`
- **THEN** the outputs are non-decreasing and finite throughout
