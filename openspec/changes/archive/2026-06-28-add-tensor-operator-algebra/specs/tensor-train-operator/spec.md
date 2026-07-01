## ADDED Requirements

### Requirement: Operator additive algebra

`CausalTensorTrainOperator` SHALL provide `add`, `sub`, `neg`, and `scale`, completing the operator
algebra alongside `compose` (operator product) and `identity` (multiplicative one). `add` and `sub`
SHALL require matching input and output dimensions and SHALL realize the operator sum and difference
(their bond dimensions add; the caller rounds to recompress). `scale` and `neg` SHALL be
rank-preserving. The densified result SHALL equal the elementwise real-space sum / difference / scaling,
and the operations SHALL be linear under `apply`.

#### Scenario: Add and subtract densify to elementwise operations
- **WHEN** two operators with matching input and output dimensions are added or subtracted
- **THEN** the dense form of the result equals the elementwise sum or difference of their dense forms

#### Scenario: Scale and negate are rank-preserving
- **WHEN** an operator is scaled by a scalar `s` or negated
- **THEN** the dense form is the elementwise `s·` (or unary minus) of the original, with the bond
  dimensions unchanged

#### Scenario: Linear under apply
- **WHEN** `a.add(b)` is applied to a state `x`
- **THEN** the result equals `a·x + b·x` to working precision (and likewise `a.sub(b)·x = a·x − b·x`)

#### Scenario: Mismatched dimensions are rejected
- **WHEN** `add` or `sub` is called on operators whose input or output dimensions differ
- **THEN** it returns `CausalTensorError::ShapeMismatch`
