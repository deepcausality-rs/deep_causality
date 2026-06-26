## ADDED Requirements

### Requirement: Alternating-sweep solve engine

The crate SHALL provide a shared alternating one-/two-site sweep driver (`solve`) parameterized by a
`SolveConfig<T>` (maximum sweeps, residual tolerance), reused by the linear solve, fit, and eigensolver. A
sweep that fails to meet the residual target within the sweep budget SHALL return
`CausalTensorError::SweepDidNotConverge`.

#### Scenario: Non-convergence reported, not looped
- **WHEN** a sweep does not reach the residual tolerance within `max_sweeps`
- **THEN** the call returns `CausalTensorError::SweepDidNotConverge` rather than running unbounded

### Requirement: ALS linear solve in TT form

`solve::linear` SHALL solve `A x = b` for `x` in tensor-train form, where `A` is a
`CausalTensorTrainOperator<T>` and `b` is a `CausalTensorTrain<T>`.

#### Scenario: Recovers the known solution
- **WHEN** `solve::linear` runs on a well-conditioned `A` and `b = A·x*` for a known `x*`
- **THEN** the returned train equals `x*` to the configured residual tolerance

### Requirement: ALS fit and completion from samples

`solve::fit` SHALL build a least-squares tensor train from sampled values (TT regression / completion).

#### Scenario: Recovers a known train from samples
- **WHEN** `solve::fit` is given samples drawn from a known low-rank train with adequate coverage
- **THEN** the fitted train reproduces the known train to the configured tolerance

### Requirement: Contraction-based integration

The `TensorTrain` trait SHALL provide `integrate`, contracting each site against a supplied per-site weight
vector to produce a scalar (quadrature / expectation / normalization).

#### Scenario: Integration matches dense weighted sum
- **WHEN** `integrate(weights)` is evaluated on a small train
- **THEN** the result equals the dense contraction of the tensor against the weight vectors to `≤ tol`

### Requirement: DMRG eigensolver

`solve::eigen` SHALL compute the lowest eigenpair of a `CausalTensorTrainOperator<T>` via two-site DMRG on
the shared sweep driver, returning the eigenvalue and its tensor-train eigenvector.

#### Scenario: Recovers a known smallest eigenpair
- **WHEN** `solve::eigen` runs on an operator with a known smallest eigenvalue and eigenvector
- **THEN** the returned eigenvalue and eigenvector match the reference to the configured tolerance

### Requirement: Optional TDVP time step

`solve::tdvp_step`, when provided, SHALL perform one time-dependent variational propagation step under a `Truncation<T>` and SHALL conserve norm to the truncation tolerance for a unitary generator. It is an optional deliverable, gated on a concrete dynamics consumer.

#### Scenario: Norm conservation under unitary evolution
- **WHEN** `tdvp_step` advances a state under a unitary (skew-Hermitian) generator
- **THEN** the state norm is conserved to the truncation tolerance
