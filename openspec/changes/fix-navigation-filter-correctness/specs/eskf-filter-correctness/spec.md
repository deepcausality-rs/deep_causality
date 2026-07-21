## ADDED Requirements

### Requirement: The process noise carries a stated discretisation

The process noise SHALL be applied under a documented discretisation of a continuous-time noise
model, so that accumulated covariance depends on **elapsed time** and not on the number of steps taken
to cover it. The supplied quantity's units SHALL be stated at the API boundary — a spectral density
if the discretisation is `Q_d = Q_c·dt`, and correspondingly for any other cited form (Van Loan, or a
first-order-hold equivalent).

`NavFilter::predict` currently takes `dt`, uses it for the state propagation and the transition
matrix, and then adds the caller's diagonal unscaled: `self.cov = mat_add(&fpft, &diag(&q))`. The
diagonal is therefore not a discretisation of any continuous model — halving `dt` over a fixed horizon
doubles the accumulated process noise — and the filter's tuning is silently bound to one step size.

#### Scenario: Covariance growth is invariant under step refinement

- **WHEN** the filter propagates over a fixed horizon at `dt` and again at `dt/2`, with the same
  continuous-time noise input and no measurements
- **THEN** the terminal covariance agrees between the two runs to the discretisation's stated order

#### Scenario: The supplied quantity's units are stated

- **WHEN** a caller supplies the process-noise input
- **THEN** the API documents whether it is a spectral density or an already-discretised covariance,
  and the discretisation applied to it is named and cited

#### Scenario: A step-size change does not silently re-tune the filter

- **WHEN** an existing configuration's `dt` is changed without changing its noise input
- **THEN** the filter's behaviour over a fixed horizon is unchanged to the discretisation's order

### Requirement: The measurement update rejects degenerate inputs

The scalar measurement update SHALL NOT divide by an unvalidated innovation covariance, and SHALL
reject a measurement it cannot fold rather than propagating a non-finite value into the state or the
covariance.

`update_scalar` computes `s = h·P·hᵀ + r` and then `k[i] = ph[i] / s` with no positivity check, and
returns `()` — it has no channel through which to reject. The path is reachable from the public API:
`NavFilter::new` and `NavFilter::restore` accept an arbitrary covariance with no validation, and
`ReentryNavEngine::correct_position` passes the caller's `r_var` straight through. With `P[i][i] = 0`
and `r_var = 0` the gain is `0/0 = NaN`, which is then written into the state and into every entry of
the Joseph-form covariance, destroying the filter silently.

A negative measurement variance SHALL likewise be rejected: a variance is non-negative by definition,
and a negative one can drive `s` through zero from either side.

#### Scenario: A degenerate innovation covariance is refused

- **WHEN** an update would divide by an innovation covariance that is zero, negative or non-finite
- **THEN** the update is refused and neither the state nor the covariance is modified

#### Scenario: A negative measurement variance is refused

- **WHEN** a measurement is offered with a negative variance
- **THEN** it is rejected rather than folded

#### Scenario: The filter survives a refused measurement

- **WHEN** a degenerate measurement is refused mid-run
- **THEN** the filter's state and covariance are exactly what they were before the attempt, and the
  run can continue folding subsequent valid measurements

#### Scenario: Valid measurements are unaffected

- **WHEN** a well-posed measurement is folded
- **THEN** the resulting state and covariance are identical to those produced before this requirement

### Requirement: A covariance is validated as a covariance

Filter construction and restoration SHALL reject a covariance that is not symmetric and positive
semidefinite, rather than accepting arbitrary values and discovering the consequences during an
update.

`NavFilter::new` accepts an arbitrary diagonal and `restore` an arbitrary matrix; neither checks
symmetry, non-negativity, or finiteness. A zero or negative variance admitted here is what makes the
degenerate-update path above reachable.

#### Scenario: A non-PSD covariance is refused at construction

- **WHEN** a filter is constructed or restored with a covariance carrying a negative variance, an
  asymmetry beyond tolerance, or a non-finite entry
- **THEN** construction fails with an error naming the offending property

#### Scenario: A restored snapshot round-trips

- **WHEN** a valid filter's covariance is snapshotted and restored
- **THEN** restoration succeeds and reproduces the filter exactly

### Requirement: An error state is reset only if it was injected

An error-state component SHALL NOT be zeroed unless the estimate it carried was applied to a
corresponding nominal. Where a component has no nominal to correct, the filter SHALL NOT claim the
covariance reduction that an applied correction would justify.

`ReentryNavEngine::correct_position` runs three scalar updates on the position axes. Each shrinks the
whole covariance through the Joseph form — including the attitude block, via the position↔attitude
cross-covariance — so the filter grows more confident about attitude. It then injects only `δp` and
`δv` into the nominal and calls `reset_navigation_error()`, which zeroes position, velocity **and
attitude**. The attitude estimate is credited in the covariance and then discarded.

It cannot currently be applied: `ReentryNavEngine` carries no nominal attitude — its fields are
`position`, `velocity`, `filter`, `tau_offset`, `elapsed` — and `attitude_error` is read nowhere in the
crate. Resetting an error state after injection is sound precisely because the error was transferred to
the nominal; zeroing one that was never transferred leaves the filter permanently overconfident.

This requirement states the invariant, not the resolution. Injecting the attitude error into a nominal
attitude, retaining the error rather than zeroing it, and applying the corresponding covariance reset
transform are all admissible ways to satisfy it; silently discarding an estimated error while keeping
its covariance credit is not.

#### Scenario: Every zeroed component was applied

- **WHEN** the navigation error is reset after a measurement fold
- **THEN** each component zeroed was first injected into a nominal, and no component is zeroed that had
  no nominal to receive it

#### Scenario: Attitude confidence is not claimed without an attitude correction

- **WHEN** a position-only fix reduces the attitude block's covariance through cross-covariance
- **THEN** either the resulting attitude correction is applied, or the covariance does not retain a
  reduction the filter cannot justify

#### Scenario: Repeated fixes do not accumulate unjustified confidence

- **WHEN** many position-only fixes are folded over a long run
- **THEN** the attitude covariance does not shrink monotonically toward zero on the strength of
  corrections that were never applied
