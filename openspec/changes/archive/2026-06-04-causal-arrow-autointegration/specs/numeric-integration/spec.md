## ADDED Requirements

### Requirement: Reusable ODE integration operator with swappable accuracy

`deep_causality_num` SHALL provide an `Integrator` trait that advances a module-valued state under a rate field. The trait SHALL provide `step(&self, s: &S, dt: R, f: &F) -> S` for one step and a provided `integrate(&self, s0: S, dt: R, steps: usize, f: &F) -> S` that folds `step` over the requested count, bounded on `S: Clone + Add<Output = S> + Mul<R, Output = S>`, `R: RealField`, and `F: Fn(&S) -> S`. It SHALL provide `Euler` (first-order, `s + f(s)·dt`) and `Rk4` (classical fourth-order) implementations. Substituting `Rk4` for `Euler` SHALL change accuracy without any change to the rate field. The trait and structs SHALL be re-exported from the crate root, use static dispatch only, and add no dependency.

#### Scenario: Euler advances a scalar exponential

- **WHEN** `Euler.integrate(1.0_f64, dt, n, &|y| *y)` integrates `y' = y` from `y(0)=1` to `t=1` with small `dt`
- **THEN** the result approaches `e` and the error shrinks proportionally to `dt` (first-order)

#### Scenario: RK4 is fourth-order on the same problem

- **WHEN** the same `y' = y` problem is integrated with `Rk4` at step sizes `dt` and `dt/2`
- **THEN** the error at `dt/2` is smaller than at `dt` by roughly a factor of 16 (`O(dt⁴)`), and at fixed moderate `dt` `Rk4` is far closer to `e` than `Euler`

#### Scenario: The same operator integrates a non-scalar state

- **WHEN** a state of a module type (e.g. a vector / tensor / multivector) is integrated under a rate field `f: Fn(&S) -> S`
- **THEN** the integrator advances it using only `Add` and scalar `Mul`, with no type-specific code

#### Scenario: Accuracy is a type swap

- **WHEN** an integration is written against the `Integrator` trait and `Euler` is replaced by `Rk4`
- **THEN** the rate field `f` and the call shape are unchanged; only the stepper value differs

### Requirement: Composite quadrature over a closed-form integrand

`deep_causality_num` SHALL provide `quadrature(f, a, b, n) -> R` computing `∫ₐᵇ f` by composite Simpson's rule over `n` panels, for `f: Fn(R) -> R` bounded on `R: Real + Div<Output = R>`. It SHALL be exact (to floating-point tolerance) for polynomials up to degree three and SHALL converge as `n` increases for smooth transcendental integrands.

#### Scenario: Exact on a cubic

- **WHEN** `quadrature(|x| x*x*x, 0.0, 1.0, n)` is evaluated for any even `n ≥ 2`
- **THEN** the result equals `1/4` within floating-point tolerance

#### Scenario: Convergent on a transcendental

- **WHEN** `quadrature(|x| x.sin(), 0.0, PI, n)` is evaluated with increasing `n`
- **THEN** the result converges to `2.0`

### Requirement: Quadrature composes with dual numbers (differentiate under the integral)

Because `quadrature` is generic over `Real` and `Dual<R>` is `Real`, evaluating it over `Dual` SHALL realize the Leibniz rule: for `I(θ) = ∫ₐᵇ f(x, θ) dx`, seeding `θ` as `Dual::variable` and running the quadrature SHALL return `I(θ)` in the real part and `dI/dθ = ∫ₐᵇ ∂f/∂θ dx` in the infinitesimal part, in a single sweep, with no integration-specific differentiation code.

#### Scenario: Integral and its parameter-derivative in one sweep

- **WHEN** `quadrature(|x| (x*theta).sin(), 0.0, 1.0, n)` is run with `theta = Dual::variable(θ0)`
- **THEN** the real part equals `∫₀¹ sin(θ0·x) dx` and the infinitesimal part equals `d/dθ ∫₀¹ sin(θ·x) dx` evaluated at `θ0`, each matching its analytic value within tolerance
