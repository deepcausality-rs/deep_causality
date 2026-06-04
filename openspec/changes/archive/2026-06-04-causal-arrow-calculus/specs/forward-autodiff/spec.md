## REMOVED Requirements

### Requirement: Forward-mode scalar derivative surface

**Reason**: Relocated to the `arrow-calculus` capability in the new `deep_causality_calculus` crate. Differentiation is the tangent functor applied to a scalar-generic `DifferentiableArrow`, not a free function over `Dual` in `deep_causality_num`. The `Dual` number itself remains in `num`; only the operator surface moves.

### Requirement: Forward-mode gradient, directional derivative, and Jacobian

**Reason**: Relocated to the `arrow-calculus` capability in the new `deep_causality_calculus` crate as the multi-input form of the tangent functor (`gradient` / `directional_derivative` over a scalar-generic `DifferentiableArrow`). The "division-only generic kernels accept dual numbers" requirement of this capability is retained, since it concerns `num`'s `Dual` flowing through `solve_gm`, which stays.
