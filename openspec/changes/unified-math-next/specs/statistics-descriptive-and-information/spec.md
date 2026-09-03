<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: Entropy takes its logarithm base and its zero policy as parameters

The entropy functions SHALL take the logarithm base and the treatment of zero-probability entries as explicit parameters, and SHALL NOT fix either.

The three implementations being replaced disagree on both, and on a third axis besides. Averaging
them would be wrong; picking one silently would change two callers' results.

| | base | normalisation | zero policy | invalid input |
|---|---|---|---|---|
| SURD, `surd_utils/mod.rs` | `log2`, bits | none; input assumed normalised | skip where `p > 0` | none — a negative probability contributes silently |
| SURD, `surd_utils_cdl.rs` | `log2`, bits | divides by the sum of the `Some` entries | skip where `p/Σ > ε` | returns zero when the sum is below `ε` |
| physics, `thermodynamics/stats.rs` | `ln`, **nats** | none | skip where `p > 0` | errors on empty and on negative |

The base difference is not a detail: bits and nats differ by a factor of `ln 2`, so the SURD and
physics functions are not the same quantity despite one being named for Shannon. The zero policy
differs between skipping at exactly zero and skipping below an epsilon, which are different functions
wherever a probability is small but real. Only the CDL variant normalises, and only physics
validates.

Making base and zero policy parameters lets one implementation serve all three callers with each
keeping its own semantics, and makes the difference visible at the call site instead of buried three
crates apart.

#### Scenario: Base is selectable and the two bases differ by ln 2
- **WHEN** entropy is computed over the same distribution in bits and in nats
- **THEN** the two results differ by exactly the factor `ln 2`, to the precision in use

#### Scenario: The zero policy is selectable
- **WHEN** a distribution containing an entry that is positive but below epsilon is offered under each zero policy
- **THEN** the exact-zero policy includes that entry's contribution and the epsilon policy excludes it, and the two results differ

#### Scenario: A uniform distribution matches its closed form
- **WHEN** entropy in bits is computed over a uniform distribution on `n` outcomes
- **THEN** the result is `log2 n`, compared against that closed form and not against a re-implementation

#### Scenario: A degenerate distribution has zero entropy
- **WHEN** entropy is computed over a distribution with a single outcome of probability one
- **THEN** the result is exactly zero under both zero policies

### Requirement: Entropy validates its input rather than computing on nonsense

The entropy functions SHALL return a typed error for an empty input and for a negative entry, and SHALL state whether they require a normalised input or normalise internally.

Two of the three implementations being replaced accept a negative probability and fold it into the
sum, producing a number with no meaning. One rejects it. The replacement rejects it, because the
alternative is a silent wrong answer of exactly the kind this change exists to remove.

Normalisation is separate from validation and is a parameter, because one caller supplies a
normalised distribution and another supplies unnormalised counts.

#### Scenario: An empty input is refused
- **WHEN** entropy is computed over an empty slice
- **THEN** a typed error is returned and no value is produced

#### Scenario: A negative entry is refused
- **WHEN** any entry is negative
- **THEN** a typed error naming the violation is returned

#### Scenario: Unnormalised input is handled as configured
- **WHEN** a distribution summing to something other than one is offered
- **THEN** it is either normalised or refused according to the stated parameter, and never silently treated as normalised

### Requirement: Conditional entropy is defined by the chain rule and tested against it

Conditional entropy SHALL be computed as `H(X|Y) = H(X,Y) − H(Y)` and SHALL be verified against an independently constructed joint distribution.

This is the identity both SURD copies implement, and it is the one place where an anti-circularity
failure is easiest: a test that computes `H(X,Y) − H(Y)` in the test body asserts nothing. The
verification uses distributions whose conditional entropy is known in closed form — an independent
pair, where `H(X|Y) = H(X)`, and a deterministic dependence, where `H(X|Y) = 0`.

#### Scenario: Independence gives the marginal entropy
- **WHEN** conditional entropy is computed over a joint distribution of two independent variables
- **THEN** the result equals `H(X)` to the precision in use

#### Scenario: Deterministic dependence gives zero
- **WHEN** `X` is a deterministic function of `Y`
- **THEN** the conditional entropy is zero

#### Scenario: The result is never negative
- **WHEN** conditional entropy is computed over any valid joint distribution, including generated ones
- **THEN** the result is non-negative

### Requirement: Log-sum-exp uses the max-shift form and is stable where the naive form is not

Log-sum-exp SHALL be computed by subtracting the maximum before exponentiating, SHALL guard the non-finite cases, and SHALL be verified at magnitudes where the naive form overflows or underflows.

Four copies exist today — three of the slice form and one two-term `logaddexp`, which is a distinct
function — and all use the max-shift with the same non-finite guard. They agree, so this is the
cleanest of the absorptions, and its test is the one that must not be circular: comparing against a
naive `ln(Σ exp(x))` is a valid oracle only in the range where the naive form is accurate, and the
cases that matter are outside it.

#### Scenario: It agrees with the naive form in the safe range
- **WHEN** log-sum-exp is computed over values small enough for `ln(Σ exp x)` to be accurate
- **THEN** the two agree to the precision in use

#### Scenario: It is finite where the naive form overflows
- **WHEN** the input contains values large enough that `exp` overflows
- **THEN** the result is finite and equals the shifted closed form

#### Scenario: It is accurate where the naive form underflows
- **WHEN** every input is large and negative
- **THEN** the result is finite and does not collapse to negative infinity

#### Scenario: An all-infinite input is handled explicitly
- **WHEN** the input contains a positive or negative infinity, or is empty
- **THEN** the documented value or typed error is returned, not a `NaN`

### Requirement: Descriptive statistics state their degrees of freedom

Mean, variance and standard deviation SHALL state whether they apply Bessel's correction, and both forms SHALL be available where both have a caller.

The absorbed sites include a Bessel-corrected variance in BRCD and an unbiased variance in quantum's
bridge. The distinction is one degree of freedom and it changes the answer on small samples, which is
where these are used. A single function that silently picks one would be wrong for the other caller.

#### Scenario: The two variance forms differ as their definitions require
- **WHEN** both variance forms are computed over the same sample of size `n`
- **THEN** their ratio is `n/(n-1)` to the precision in use

#### Scenario: A single-element sample is handled
- **WHEN** the Bessel-corrected variance is computed over one element
- **THEN** a typed error is returned rather than a division by zero

#### Scenario: The mean matches a hand-computed value
- **WHEN** the mean is computed over a small literal sample
- **THEN** it equals the value computed by hand and written as a literal in the test

### Requirement: Pearson correlation states its missing-data policy and its degenerate cases

Pearson correlation SHALL state how it treats missing observations, and SHALL return a typed error rather than a `NaN` when either input has zero variance.

The implementation being absorbed performs pairwise deletion and computes in `f64` behind a generic
signature — the second of which this crate removes by construction. The zero-variance case is the
one that matters: the correlation is undefined there, and returning `NaN` propagates into an
F-statistic that the same module guards with a `1e12` sentinel. A typed error at the source removes
the need for the sentinel.

#### Scenario: A known correlation is reproduced
- **WHEN** correlation is computed over a small sample with a correlation known in closed form
- **THEN** the result matches that closed form

#### Scenario: Perfect correlation is exact at the boundary
- **WHEN** one input is an exact positive affine transform of the other
- **THEN** the result is one to the precision in use, and does not exceed one

#### Scenario: Zero variance is refused
- **WHEN** either input is constant
- **THEN** a typed error is returned rather than a `NaN`

#### Scenario: The missing-data policy is applied as stated
- **WHEN** observations are missing in either input
- **THEN** they are handled by the stated policy, and the effective sample size is reported or documented

### Requirement: Ridge and logistic regression report convergence and conditioning

Ridge least squares SHALL report a conditioning failure rather than returning a silently wrong solution, and logistic regression by iteratively reweighted least squares SHALL report non-convergence rather than returning its last iterate.

Both are absorbed from BRCD, where ridge exists twice — materialised and streaming — and the
logistic gate is a Newton IRLS with a stable sigmoid and a clamped logit. The streaming and
materialised forms are documented as agreeing over the filtered design, which is a property the suite
pins rather than assumes.

Returning a last iterate without signalling is the failure mode this change removes wherever it
appears; it is specified here for the same reason it is specified for the root finders.

#### Scenario: Ridge reproduces a closed-form solution
- **WHEN** ridge is solved on a design with a known closed-form solution at a given penalty
- **THEN** the result matches it to the precision in use

#### Scenario: The penalty does what a penalty does
- **WHEN** the ridge penalty is increased
- **THEN** the norm of the coefficient vector decreases monotonically

#### Scenario: The two ridge forms agree
- **WHEN** the materialised and streaming forms are run over the same filtered design
- **THEN** their coefficients agree to the precision in use

#### Scenario: A rank-deficient design is refused
- **WHEN** ridge is solved at zero penalty on a rank-deficient design
- **THEN** a typed error is returned rather than an arbitrary solution

#### Scenario: Non-convergence is signalled
- **WHEN** the IRLS iteration reaches its cap without meeting its tolerance
- **THEN** a typed error carrying the iteration count and the achieved tolerance is returned, not the last iterate

#### Scenario: Separable data does not diverge silently
- **WHEN** logistic regression is fitted to perfectly separable data
- **THEN** the result is either a typed non-convergence error or a solution bounded by the stated regularisation, never an unbounded coefficient returned as success

### Requirement: Binning states its edge convention and its degenerate-range behaviour

Equal-width and equal-frequency binning SHALL state which edge of a bin is inclusive, SHALL place a value equal to the maximum in the last bin, and SHALL state what happens when a column's range is degenerate.

The implementation being absorbed collapses when a column's range falls to epsilon. That behaviour is
defensible — at `f64` epsilon a range genuinely is degenerate — but it is currently unstated, and an
unstated boundary is one a caller discovers through a wrong histogram.

#### Scenario: The edge convention is applied consistently
- **WHEN** a value falls exactly on a bin boundary
- **THEN** it lands in the bin the stated convention names, at every boundary in the range

#### Scenario: The maximum lands in the last bin
- **WHEN** the maximum value of the input is binned
- **THEN** it falls in the last bin rather than one past the end

#### Scenario: A constant column is handled explicitly
- **WHEN** every value in a column is identical
- **THEN** the documented degenerate result or a typed error is returned, not an unbounded bin width

#### Scenario: Equal-frequency bins are balanced where the data allows
- **WHEN** `n` distinct values are binned into `k` equal-frequency bins with `k` dividing `n`
- **THEN** each bin holds exactly `n/k` values

### Requirement: The Gaussian log-density is one function with a stated parameterisation

The Gaussian log-density SHALL be provided once, SHALL state whether its second parameter is a variance or a standard deviation, and SHALL reject a non-positive scale.

Three sites compute it today, one of which names in its own doc comment the function it duplicates.
The parameterisation is the trap: variance and standard deviation are both plausible readings of a
second argument, and a caller that reads it the other way is wrong by a square root with no error.

#### Scenario: It matches the closed form
- **WHEN** the log-density is evaluated at a point with a known closed-form value
- **THEN** it matches that value, compared against a literal derived from the closed form

#### Scenario: It integrates to one
- **WHEN** the exponentiated density is integrated numerically over a wide interval
- **THEN** the result is one to the tolerance of the quadrature

#### Scenario: A non-positive scale is refused
- **WHEN** the scale parameter is zero or negative
- **THEN** a typed error is returned rather than a `NaN` or an infinity

#### Scenario: The parameterisation is unambiguous
- **WHEN** the signature is read
- **THEN** the parameter's name and documentation state variance or standard deviation, and the suite pins the choice with a case where the two differ
