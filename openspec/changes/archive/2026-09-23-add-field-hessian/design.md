## Context

`DifferentiateFieldExt<N>` differentiates a `DifferentiableField<N>` (`Rᴺ → R`) to first order:
`gradient` seeds one coordinate per pass over `Dual<R>`. `DifferentiateExt::second_derivative`
reaches second order for a scalar model by running it over `Dual<Dual<R>>`. The Hessian combines
the two: the field's model, run over nested duals, one pass per entry.

## Goals / Non-Goals

**Goals:** the Hessian of a scalar field, forward-mode, allocation-free, generic over `Scalar`.

**Non-Goals:** the rank-3 Hessian of a vector field (the crate has no vector-field trait);
a combined value, gradient and Hessian pass; sparse or reverse-mode Hessians.

## Decisions

**Placement: a default method on `DifferentiateFieldExt<N>`.** `gradient` and
`directional_derivative` already live there, so the Hessian reads `field.hessian(&x)`, and every
`DifferentiableField<N>` receives it through the blanket impl. A free function or a new trait
would split the field's differentiation surface in two.

**Seeding: one coordinate per infinitesimal.** Entry `(i, j)` builds
`xₖ = (xₖ + δₖᵢ ε₁) + δₖⱼ ε₂` and reads `ε₁ε₂`. Because `ε₁ε₂` is the mixed coefficient,
`i = j` needs no special case: seeding the same coordinate in both slots yields `∂²f/∂xᵢ²`.

**Upper triangle, mirrored.** Evaluating `i ≤ j` and writing each value to `(i, j)` and `(j, i)`
takes `N(N+1)/2` passes instead of `N²`, and makes `H[i][j] == H[j][i]` exact. Evaluating both
triangles would return the same values for a `C²` field at twice the cost.

**Return type `[[R; N]; N]`.** It matches `gradient`'s `[R; N]`: a stack array sized by the
const parameter, no allocation, no dependency on `deep_causality_linear` or `deep_causality_tensor`.

## Risks / Trade-offs

- Cost grows as `N²` passes, each over a fourfold scalar (`Dual<Dual<R>>`). For large `N` a
  caller wants reverse mode or a sparse Hessian, neither of which the crate provides.
- Returning a stack array of `N²` scalars limits practical `N` to what fits on the stack.
