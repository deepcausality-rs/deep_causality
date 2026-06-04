## ADDED Requirements

### Requirement: Differentiation as the tangent functor over a scalar-generic arrow

`deep_causality_haft` SHALL provide forward-mode differentiation as the tangent functor over the `Dual` number, not as free functions over `Dual`. It SHALL define a `DifferentiableArrow` trait whose evaluation is generic over the scalar (`fn run<S: Scalar>(&self, …) -> …`, where `Scalar = Real + Div + FromPrimitive`), a `Diff<A>` combinator that is itself a value-level `Arrow` from `Dual<…>` to `Dual<…>` (the derivative arrow), and the desugared entry points `derivative`, `value_and_derivative`, `second_derivative`, `gradient`, and `directional_derivative`. These SHALL instantiate the model at `Dual` (and `Dual<Dual<…>>` for second derivatives) internally, so a caller writes a model once over `Scalar` and never names `Dual`, `ε`, or a seed.

#### Scenario: Differentiate a scalar-generic model

- **WHEN** a model implementing `DifferentiableArrow` with `run(x) = x·sin(x)` is differentiated at `x` via `derivative`
- **THEN** the result equals `sin(x) + x·cos(x)` within tolerance, and the caller's code names neither `Dual` nor a seed

#### Scenario: Higher derivatives reuse the same model

- **WHEN** the same model is passed to `second_derivative`
- **THEN** the functor instantiates it at `Dual<Dual<…>>` and returns `f''(x)` within tolerance, with no change to the model

#### Scenario: Gradient of a multi-input model

- **WHEN** a multi-input `DifferentiableArrow` for `f(x, y) = x² + y²` is passed to `gradient` at `(3, 4)`
- **THEN** the result is `[6, 8]`

### Requirement: Differentiable models compose with the value-level Arrow algebra

A model that implements `DifferentiableArrow` SHALL also be usable as a concrete value-level `Arrow`, and its `Diff<A>` derivative view SHALL be a concrete `Arrow` over `Dual`. Both SHALL therefore compose with the `arrow-strength` combinators (`Compose`, `First`, `Second`, `Split`, `Fanout`) and the arrow builder. The tangent functor SHALL extend the existing arrow algebra, not replace it.

#### Scenario: A model is both a plain arrow and a derivative arrow

- **WHEN** a model is viewed as `Arrow<In = f64, Out = f64>` and its `Diff` view as `Arrow<In = Dual<f64>, Out = Dual<f64>>`
- **THEN** both run as ordinary arrows and can be placed into `Compose`/`Split` pipelines

### Requirement: Precision is a parameter via a nesting-safe constant lift

The differentiation and integration operators SHALL be generic over the base precision, working for `f32`, `f64`, and `Float106`, with duals nesting for higher derivatives. To lift literal constants into the working scalar precision-safely, `deep_causality_num` SHALL provide a blanket `FromPrimitive` implementation for `Dual<T>` (a primitive converts to a derivative-free constant dual, forwarded through every nesting level). `From<f64>` SHALL NOT be the constant mechanism of the generic surface, because `f32` does not implement it.

#### Scenario: The same model differentiates at every precision

- **WHEN** a model with lifted constants is differentiated at `f32`, `f64`, and `Float106`
- **THEN** each yields the correct first and second derivative within precision-appropriate tolerance

#### Scenario: Constants do not contaminate the derivative

- **WHEN** a constant is lifted into `Dual` via the `FromPrimitive` blanket
- **THEN** its `ε` channel is zero, so lifted constants never appear in a computed sensitivity

### Requirement: Integration as endo-arrows driven by the Endomorphism combinators

`deep_causality_haft` SHALL express ODE integration as iteration of an endo-arrow. `Euler` and `Rk4` SHALL each construct, from a step size and a rate field, a value-level **endo-arrow** — a concrete `Arrow<In = S, Out = S>` over a module-valued state `S` (`Clone + Add + scalar Mul`). Evolution SHALL be provided as the value-level realization of the `Endomorphism` monoid on endo-arrows: `iterate_n` (advance a fixed number of steps), `iterate_to_fixpoint` (advance to a steady state, `S: PartialEq + Clone`), and `iterate_until` (advance until a predicate holds). Substituting `Rk4` for `Euler` SHALL change accuracy with no change to the rate field.

#### Scenario: Fixed-horizon march

- **WHEN** `Euler` (or `Rk4`) builds an endo-arrow for `y' = y` and it is iterated `n` steps from `y(0)=1` to `t=1` via `iterate_n`
- **THEN** the result approaches `e`, with Euler error `O(dt)` and RK4 error `O(dt⁴)`

#### Scenario: Steady state via fixpoint

- **WHEN** an endo-arrow with a stable equilibrium is run through `iterate_to_fixpoint` with a step bound
- **THEN** it returns the equilibrium and `true`, or the last iterate and `false` if the bound was hit first

#### Scenario: Integrate until an event

- **WHEN** a descent endo-arrow is run through `iterate_until` with the predicate "altitude ≤ 0"
- **THEN** it stops at touchdown and reports whether the event was reached within the step bound

### Requirement: Quadrature as a fold-arrow with Leibniz naturality

`deep_causality_haft` SHALL provide definite integration as a fold over a closed-form integrand, generic over `Scalar`. Because `Dual` is a `Scalar`, evaluating the fold over `Dual` SHALL realize differentiation under the integral sign — the naturality of the tangent functor through the fold (`T(∫f) = ∫(Tf)`).

#### Scenario: Exact on a cubic

- **WHEN** the quadrature fold integrates `x³` over `[0, 1]`
- **THEN** the result equals `1/4` within tolerance

#### Scenario: Differentiate under the integral by naturality

- **WHEN** the quadrature fold integrates `sin(θ·x)` over `[0, 1]` with `θ` differentiated
- **THEN** the real part equals `∫₀¹ sin(θx) dx` and the `ε` part equals `d/dθ ∫₀¹ sin(θx) dx`, each matching its analytic value
