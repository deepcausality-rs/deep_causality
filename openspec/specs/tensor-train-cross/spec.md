# tensor-train-cross Specification

## Purpose
TBD - created by archiving change add-tensor-network. Update Purpose after archive.
## Requirements
### Requirement: TT-cross construction from an oracle

The crate SHALL provide `CausalTensorTrain::cross`, building a tensor train from an oracle closure
`Fn(&[usize]) -> T` over a given shape **without forming the dense tensor**, controlled by a
`CrossConfig<T>` (maximum sweeps, rank cap, tolerance). It SHALL return a residual/accuracy estimate and
SHALL be bounded: it MUST NOT exceed the configured sweep/rank budget.

#### Scenario: Recovers a known low-rank oracle
- **WHEN** `cross` is run on an oracle that is exactly low TT-rank, with a rank cap at or above that rank
- **THEN** the constructed train reproduces the oracle to the configured tolerance and the reported
  residual is within tolerance

#### Scenario: Never materializes the dense tensor
- **WHEN** `cross` builds a high-order train
- **THEN** it queries the oracle only at sampled multi-indices and never allocates the `nᵈ` dense buffer

#### Scenario: Non-finite oracle sample rejected
- **WHEN** the oracle returns a non-finite value during sampling
- **THEN** `cross` returns `CausalTensorError::CrossSampleFailure`

#### Scenario: Budget is respected
- **WHEN** the oracle does not converge within the configured sweep/rank budget
- **THEN** `cross` stops at the budget and reports the achieved residual rather than looping unbounded

### Requirement: Nonlinear elementwise map via re-approximation

The `TensorTrain` trait SHALL provide `apply_nonlinear(f, &CrossConfig) -> Result<(Self, residual), _>`,
realizing a general elementwise map `B[i] = f(A[i])` by cross re-approximation over
`i ↦ f(self.eval(i))`. The crate SHALL NOT provide a `map_elementwise` method that silently applies an
unconstrained closure to core storage.

#### Scenario: Nonlinear map returns an explicit residual
- **WHEN** `apply_nonlinear(f, cfg)` is called with a nonlinear `f`
- **THEN** the result is the cross re-approximation of `f∘A` together with a residual estimate of the
  approximation error

#### Scenario: Exact linear cases use the exact methods
- **WHEN** the transform is linear or affine
- **THEN** the caller uses `scale`/`add_scalar` (exact) rather than `apply_nonlinear`

