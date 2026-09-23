# Operators

A witness carries data; an `Arrow` carries a computation. `Euler`, `Rk4` and `Diff` are values
that *are* transformations: built now, composed with each other, and run later against whatever
they are handed.

The integrator and the thing being integrated are two values, so swapping `Euler` for `Rk4`
raises the time order and leaves the rate field as written.

## Integrating

| Example | What it shows | Command |
|---|---|---|
| [cubical_heat_diffusion](cubical_heat_diffusion/) | The explicit-Euler update as an endo-arrow over a cubical complex; the Moore-neighbourhood Laplacian stays a pure `Fn(&Field) -> Field` | `cargo run -p mathematics_examples --example cubical_heat_diffusion_examples` |
| [differential_field](differential_field/) | The heat equation on a simplicial manifold, with the discrete exterior calculus Laplacian as the rate field | `cargo run -p mathematics_examples --example differential_field_examples` |

## Differentiating

| Example | What it shows | Command |
|---|---|---|
| [field_gradient_flow](field_gradient_flow/) | `∇f` by forward-mode AD, exact to the last bit because no step size is chosen; `fmap` spreads it over a tensor grid, then `Euler` integrates `-∇f` into gradient descent | `cargo run -p mathematics_examples --example field_gradient_flow_examples` |
| [sampled_integration](sampled_integration/) | One integral by Simpson, Monte Carlo and Sobol, then `dI/dθ` out of the very same draws by running each fold over `Dual` | `cargo run -p mathematics_examples --example sampled_integration_examples` |

## Where the derivative comes from

`field_gradient_flow` and `sampled_integration` take a derivative by evaluating over `Dual` and
reading the `ε` channel, which makes `∇f` exact. `extension/manifold_laplacian_stencil` takes a
three-point difference, where the step size *is* the accuracy. Each prints the residual that
follows from its method.

In `sampled_integration`, every estimator is generic over `Scalar` and `Dual` is a `Scalar`, so
differentiation passes straight through a *sampled* fold. The draws that estimate
`I(θ)` estimate `dI/dθ` at the same time.
