# tensor-train-operator Specification

## Purpose
TBD - created by archiving change add-tensor-network. Update Purpose after archive.
## Requirements
### Requirement: Matrix-product-operator type and trait

The crate SHALL provide a concrete type `CausalTensorTrainOperator<T>` (private fields) storing a chain of
rank-4 cores of shape `[r_k, n_out_k, n_in_k, r_{k+1}]`, and a trait `TensorTrainOperator<T>` declaring its
behavior. It SHALL provide inherent constructors `identity(dims)` and `from_dense`.

#### Scenario: Operator core invariants
- **WHEN** a `CausalTensorTrainOperator` is constructed
- **THEN** adjacent cores share bond dimensions, boundary bonds are 1, and the cached `in_dims`/`out_dims`
  equal the cores' input/output physical dimensions

#### Scenario: Identity operator acts as identity
- **WHEN** `identity(dims).apply(x, &Truncation)` is evaluated for any train `x` over `dims`
- **THEN** the result equals `x` to the truncation tolerance

### Requirement: Operator construction from dense

`CausalTensorTrainOperator::from_dense` SHALL factor a dense operator tensor into an MPO via truncated SVD
under a `Truncation<T>`, given the input and output physical dimensions.

#### Scenario: Exact recovery at sufficient bond
- **WHEN** an operator of exact MPO-rank `R` is converted with a bond cap `≥ R`
- **THEN** `to_dense` reproduces the original operator to `‖·‖ ≤ tol`

### Requirement: MPO application and composition

The trait SHALL provide `apply` (MPO·MPS → MPS) and `compose` (MPO·MPO → MPO), each growing bond dimension
exactly and paired with rounding to a `Truncation<T>`. It SHALL provide operator rounding and a
conjugate/transpose (`adjoint`).

#### Scenario: Apply matches dense matrix–vector
- **WHEN** `apply` is evaluated on small operands
- **THEN** the result equals the dense operator-times-vector product to `‖·‖ ≤ tol`

#### Scenario: Compose matches dense matmul
- **WHEN** `compose` is evaluated on small operators
- **THEN** the result equals the dense operator product to `‖·‖ ≤ tol`

### Requirement: Operator composition as an EndoArrow

`CausalTensorTrainOperator<T>` SHALL implement `EndoArrow<CausalTensorTrain<T>>` so MPO chains use the
crate's `Arrow` algebra: `compose` corresponds to `>>>`, `identity()` to `Id`, and `apply` to the arrow's
action on a state. The associativity, identity, and action laws SHALL hold exactly without truncation and
to the truncation tolerance otherwise.

#### Scenario: Associativity to tolerance
- **WHEN** `(A.compose(B)).compose(C)` and `A.compose(B.compose(C))` are each applied to a state under a
  fixed `Truncation`
- **THEN** the two results agree to the truncation tolerance

#### Scenario: Action law
- **WHEN** `A.compose(B).apply(x)` and `A.apply(B.apply(x))` are evaluated
- **THEN** the two results agree to the truncation tolerance

