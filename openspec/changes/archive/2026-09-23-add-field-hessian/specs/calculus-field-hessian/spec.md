## ADDED Requirements

### Requirement: A differentiable field returns its Hessian

`DifferentiateFieldExt<N>` SHALL provide `hessian<R: Scalar>(&self, x: &[R; N]) -> [[R; N]; N]`, whose entry `[i][j]` is `∂²f / ∂xᵢ∂xⱼ` at `x`, computed by forward-mode differentiation of the same scalar-generic model over `Dual<Dual<R>>`.

#### Scenario: Every entry matches the closed form
- **WHEN** `hessian` is called on a field whose Hessian has distinct, non-zero entries
- **THEN** every entry equals the hand-derived second partial derivative

#### Scenario: Agreement with an independent algorithm
- **WHEN** `hessian` is compared with central differences of `gradient`
- **THEN** the two agree to within the finite-difference truncation error

### Requirement: The Hessian is exactly symmetric

The returned matrix SHALL satisfy `H[i][j] == H[j][i]` bit for bit, by evaluating only the upper triangle and writing each value to both positions.

#### Scenario: Symmetry holds bitwise
- **WHEN** `hessian` is called on a transcendental field
- **THEN** `H[i][j].to_bits() == H[j][i].to_bits()` for every `i`, `j`

### Requirement: Degenerate and non-finite inputs return a value

`hessian` SHALL return without panicking for `N = 0` (an empty matrix) and `N = 1` (the scalar second derivative), and SHALL propagate a NaN coordinate rather than substitute a value.

#### Scenario: Zero inputs
- **WHEN** `hessian` is called on a `DifferentiableField<0>`
- **THEN** it returns an empty `[[R; 0]; 0]`

#### Scenario: NaN propagates
- **WHEN** a coordinate the field depends on is NaN
- **THEN** every entry that depends on it is NaN

### Requirement: Precision is a parameter

`hessian` SHALL be generic over `Scalar` and SHALL produce the closed-form Hessian at `f32`, `f64` and `Float106` from the same model.

#### Scenario: Same field at three precisions
- **WHEN** the same field and point are evaluated at `f32`, `f64` and `Float106`
- **THEN** each result equals the closed-form Hessian at that precision
