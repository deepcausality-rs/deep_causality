[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)


# deep_causality_calculus

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
![Tests][test-url]

[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue
[crates-url]: https://crates.io/crates/deep_causality_calculus
[docs-badge]: https://img.shields.io/badge/Docs.rs-Latest-blue
[docs-url]: https://docs.rs/deep_causality_calculus/latest/deep_causality_calculus/
[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg
[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE
[test-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/run_tests.yml/badge.svg


Arrow-native differentiation and integration operators for the DeepCausality stack:
forward-mode automatic differentiation (the tangent functor over `Dual`),
endomorphism iteration for ODE integration, and composite-Simpson quadrature.
Every operator is generic over the working scalar, so precision is a free
parameter and derivatives nest. Zero external runtime dependencies.

These are the analytic operators of the *Causal Arrow*. They sit above the two
foundations they combine — the value-level `Arrow` / `Endomorphism` machinery in
`deep_causality_haft` and the `Dual` number in `deep_causality_num` — and neither
foundation depends on the other, so this crate is where they meet while both stay
self-contained.

## The three operators

| Operator | What it is | Surface |
|---|---|---|
| **Differentiation** | the tangent functor — object map `Dual`, morphism map "run an arrow over duals" | `DifferentiableArrow` / `DifferentiableField` (you implement), `DifferentiateExt` / `DifferentiateFieldExt` (fluent methods), `Diff` (the derivative-arrow view) |
| **Integration** | endomorphism iteration of a value-level endo-arrow | `Euler`, `Rk4` (you construct), `EndoArrow` (the `iterate_*` combinators) |
| **Quadrature** | a fold over a closed-form integrand | the free `quadrature` function |

### Differentiation — the tangent functor

A concrete `Arrow<In = f64>` cannot be lifted over `Dual` because its `run` only
accepts `f64`. The scalar-polymorphism therefore lives in the model you write: a
named type implementing `DifferentiableArrow`, whose `run` is generic over the
scalar. The same model then evaluates at `f64` (the value) and at `Dual` (the
derivative). The `DifferentiateExt` methods seed `Dual` internally and read back
the `ε` channel — you never name `Dual`, `ε`, or seeding.

```rust
use deep_causality_calculus::{DifferentiableArrow, DifferentiateExt, Scalar};

// f(x) = x·sin(x)   →   f'(x) = sin x + x·cos x,   f''(x) = 2·cos x − x·sin x
struct XSinX;
impl DifferentiableArrow for XSinX {
    fn run<S: Scalar>(&self, x: S) -> S {
        x * x.sin()
    }
}

let d  = XSinX.derivative(0.7_f64);            // f'(0.7)
let (v, d) = XSinX.value_and_derivative(1.2);  // (f(x), f'(x)) in one pass
let d2 = XSinX.second_derivative(0.9);         // same model, instantiated at Dual<Dual<R>>
```

Multi-input fields implement `DifferentiableField<N>` (`Rᴺ → R`) and reach
`gradient` and `directional_derivative` via `DifferentiateFieldExt`; the gradient
seeds one coordinate per pass and allocates nothing.

```rust
use deep_causality_calculus::{DifferentiableField, DifferentiateFieldExt, Scalar};

struct NormSquared; // f(x, y) = x² + y²
impl DifferentiableField<2> for NormSquared {
    fn run<S: Scalar>(&self, p: &[S; 2]) -> S {
        p[0] * p[0] + p[1] * p[1]
    }
}

let g = NormSquared.gradient(&[3.0_f64, 4.0]);                 // [6.0, 8.0]
let d = NormSquared.directional_derivative(&[1.0, 1.0], &[2.0, 0.0]); // 4.0
```

`Diff` is the same functor applied to a *morphism*: a concrete `Arrow` from
`Dual<R>` to `Dual<R>`. Because it is an ordinary arrow it composes with the
`arrow-strength` combinators (`compose` / `first` / `split` / `fanout`) — the
functor extends the arrow algebra rather than replacing it.

### Integration — endomorphism iteration

`Euler` and `Rk4` build value-level endo-arrows `Arrow<In = S, Out = S>` from a
step size `dt` and a rate field `f`. RK4 is a drop-in replacement for Euler: same
state, same rate field, far higher accuracy. The `EndoArrow` combinators supply
the three iteration modes — fixed horizon, steady state, and integrate-until-event.

```rust
use deep_causality_calculus::{Euler, Rk4, EndoArrow};

// y' = y, y(0) = 1  →  e at t = 1
let y = Euler::new(1e-4_f64, |y: &f64| *y).iterate_n(1.0, 10_000);

// Relax to a fixpoint, reporting whether it converged within the step bound
let (val, converged) = Euler::new(1.0_f64, move |x: &f64| 5.0 - *x)
    .iterate_to_fixpoint(0.0, 100);

// March until an event predicate first holds
let (val, met) = Rk4::new(0.1_f64, |_: &f64| -1.0)
    .iterate_until(5.0, |x| *x <= 0.0, 100);
```

### Quadrature

`quadrature(f, a, b, n)` is the definite integral `∫ₐᵇ f` by composite Simpson's
rule over `n` panels (`n` is normalised to an even value `≥ 2`; exact through
cubics). It is a free function, not a type extension, because its subject is a
closure. Being generic over `Scalar`, it also runs over `Dual`: seed a parameter
as `Dual::variable` and the `ε` part of the result is `d/dθ ∫ f(x, θ) dx` — the
Leibniz rule, which is the naturality of the tangent functor through the fold.

```rust
use deep_causality_calculus::quadrature;

let area = quadrature(|x: f64| x * x, 0.0, 1.0, 100); // ≈ 1/3
```

## Precision is a free parameter

Every operator is generic over `Scalar` (`Real + Div + FromPrimitive`, re-exported
from `deep_causality_num`). A model is written once and run at `f32`, `f64`, or
`Float106` extended precision, and duals nest for higher derivatives — `f''` is
just the tangent functor instantiated at `Dual<Dual<R>>` over the same model.

## `no_std`

The crate is `#![no_std]` by default-off; the default `std` feature enables `std`
on the `deep_causality_haft` and `deep_causality_num` dependencies. `alloc` is
available independently for `no_std` targets with an allocator.

## Safety

No `unsafe` — the crate opts into the workspace-wide `unsafe_code = "forbid"` lint
policy.

## Contribution

Contributions are welcomed especially related to documentation, example code, and fixes.
If unsure where to start, just open an issue and ask.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed under the MIT licence, without any additional terms or conditions.

## Licence

This project is licensed under the [MIT license](LICENSE).

## Security

For details about security, please read
the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).
