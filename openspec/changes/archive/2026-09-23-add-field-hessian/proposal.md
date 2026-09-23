## Why

`deep_causality_calculus` differentiates a scalar field `Rᴺ → R` to first order only: `gradient`
and `directional_derivative`. The scalar model has `second_derivative`; the field has no second
order. Newton steps, curvature checks and Laplace approximations need the Hessian, and a caller
today has to finite-difference the gradient to get one. Issue #799.

## What Changes

- `DifferentiateFieldExt<N>` gains `hessian<R: Scalar>(&self, x: &[R; N]) -> [[R; N]; N]`,
  returning `H[i][j] = ∂²f / ∂xᵢ∂xⱼ`.
- The model is run over `Dual<Dual<R>>`: entry `(i, j)` seeds `xᵢ` in the inner `ε` and `xⱼ` in
  the outer `ε` and reads the `ε₁ε₂` channel. Only the upper triangle is evaluated, `N(N+1)/2`
  passes, and each value is mirrored, so the result is exactly symmetric. No allocation.
- The crate README and the crate-level rustdoc list the new method.

No existing signature changes. No new dependency.

## Capabilities

### New Capabilities

- `calculus-field-hessian`: the forward-mode Hessian of a `DifferentiableField<N>`.

## Impact

- `deep_causality_unified_math/deep_causality_calculus/src/extensions/differentiate_ext.rs`
- `deep_causality_unified_math/deep_causality_calculus/tests/extensions/differentiate_ext_tests.rs`
- `deep_causality_unified_math/deep_causality_calculus/README.md`, `src/lib.rs` (docs)
