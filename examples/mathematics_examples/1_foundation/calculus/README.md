# Foundation: `deep_causality_calculus`

Differentiation and integration as values, built once and applied later.

A model is written **once**, generic over `Scalar`. Evaluated at the working type it returns
the model's value; evaluated at `Dual` it returns the value together with its derivative. The
derivative comes from the same expression tree, so it is exact to the last bit on a polynomial.

| Example | What it covers | Command |
|---|---|---|
| [differentiation_and_integration.rs](differentiation_and_integration.rs) | `DifferentiableArrow` and `DifferentiableField<N>` with exact AD derivatives, `quadrature` for Simpson's rule, and `Euler` against `Rk4` on the same decay problem | `cargo run -p mathematics_examples --example differentiation_and_integration_examples` |

See also `2_composition/operators/`, where these same operators compose with tensors, manifolds
and sampled estimators.
