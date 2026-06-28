## ADDED Requirements

### Requirement: Greedy-pivot cross strategy

`CausalTensorTrain::cross` SHALL support an **opt-in greedy (largest-residual) pivot selection**
strategy, selectable through `CrossConfig`, as a cheaper alternative to the default maxvol/LU
rank-revealing pivot. The greedy strategy SHALL select, per step, the index of largest current
interpolation residual, SHALL preserve the nestedness of the left/right index sets across sweeps, and
SHALL recover the same low-rank oracles as the maxvol/LU default to the configured tolerance. The
maxvol/LU strategy SHALL remain the default.

#### Scenario: Greedy pivoting recovers a low-rank oracle
- **WHEN** `cross` runs the greedy strategy on an oracle that is exactly low TT-rank, with a rank cap
  at or above that rank
- **THEN** the constructed train reproduces the oracle to the configured tolerance, matching the
  maxvol/LU default's accuracy

#### Scenario: Default pivot strategy is unchanged
- **WHEN** `cross` is run with a `CrossConfig` that does not select the greedy strategy
- **THEN** the existing maxvol/LU pivot path is used unchanged

#### Scenario: Budget is still respected
- **WHEN** the greedy strategy does not converge within the configured sweep/rank budget
- **THEN** `cross` stops at the budget and reports the achieved residual rather than looping unbounded
