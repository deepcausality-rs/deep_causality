# tensor-train-scalars Specification

## Purpose
TBD - created by archiving change add-tensor-network. Update Purpose after archive.
## Requirements
### Requirement: Complex-scalar instantiation

The tensor-network stack SHALL operate over complex scalars `Complex<T>`. The norm/orthogonality kernels
SHALL use the Hermitian (conjugate) transpose, real-valued singular values (in `Normed::Real`), and the
conjugated inner product `⟨a|b⟩ = Σ aᵢ* bᵢ`. The TT algebra (add/scale/hadamard/contraction/marginalize/
eval, MPO apply/compose) SHALL operate over `Complex<T>` unchanged.

#### Scenario: Complex SVD/QR unitarity
- **WHEN** the truncated SVD or QR runs over `Complex<f64>`
- **THEN** the factors satisfy `‖Uᴴ U − I‖ ≤ k·ε` and the singular values are real and non-negative

#### Scenario: Conjugated inner product
- **WHEN** `inner(a, b)` is evaluated over complex trains
- **THEN** it equals `Σ aᵢ* bᵢ` and `inner(a, a)` is real and non-negative

### Requirement: Forward-mode automatic differentiation via dual scalars

The real kernels SHALL run over the dual scalar `Dual<T>` to give forward-mode AD through the network. This
SHALL be achieved by bounding the algebra/AD code path on `Scalar` (`Real + Div + FromPrimitive`), not on
`Field`/`RealField`. Truncation and pivot decisions SHALL branch on the real part, so derivatives flow
through the retained subspace.

#### Scenario: Gradient matches finite differences
- **WHEN** a scalar output (e.g. `norm` or an `integrate` result) is differentiated with respect to an
  input via `Dual<f64>`
- **THEN** the dual-channel derivative matches a central finite-difference estimate to `√ε`

#### Scenario: Truncation decisions on the real part
- **WHEN** a singular value is compared for truncation over a dual scalar
- **THEN** the comparison uses the real part, and the chosen rank is locally constant in the input

### Requirement: Mixed complex-dual scalars

The stack SHALL support `Dual<Complex<T>>` (complex-valued forward-mode AD) so derivatives can be taken
through complex computations.

#### Scenario: Complex-AD round-trip
- **WHEN** a complex computation is run over `Dual<Complex<f64>>`
- **THEN** the value channel matches the plain `Complex<f64>` computation and the derivative channel is
  finite and consistent with finite differences

### Requirement: Cross-precision and cross-scalar test matrix

The applicable test classes SHALL be instantiated at `f32`, `f64`, and `Float106` for the real path, and at
`Complex<f64>` and `Dual<f64>` for the scalar-generality classes, with assertion tolerances derived from
`T::epsilon()`.

#### Scenario: Suite passes across the matrix
- **WHEN** the generic test bodies are instantiated across the precision/scalar matrix
- **THEN** every instantiation passes with its precision-appropriate tolerance

