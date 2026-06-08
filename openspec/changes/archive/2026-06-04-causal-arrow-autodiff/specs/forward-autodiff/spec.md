## ADDED Requirements

### Requirement: Forward-mode scalar derivative surface

`deep_causality_num` SHALL provide a forward-mode automatic-differentiation surface over `Dual<T>` consisting of generic free functions bounded on `R: Real + Div<Output = R>`. It SHALL provide `derivative(f, x) -> R` returning `f'(x)`, `value_and_derivative(f, x) -> (R, R)` returning `(f(x), f'(x))` in a single evaluation, and `second_derivative(f, x) -> R` returning `f''(x)` via nested duals. Each SHALL take a differentiand of the form `Fn(Dual<R>) -> Dual<R>`, seed the independent variable with `Dual::variable`, and read the infinitesimal part. The functions SHALL be re-exported from the crate root and SHALL introduce no new type, trait, dependency, or `unsafe`.

#### Scenario: First derivative of a closed-form function

- **WHEN** `derivative(|x| x*x*x + x + x, 3.0_f64)` is evaluated
- **THEN** the result equals `f'(3) = 3·3² + 2 = 29`

#### Scenario: Value and derivative in one pass

- **WHEN** `value_and_derivative(f, x)` is evaluated for a differentiand `f`
- **THEN** the first component equals `f(x)` and the second equals `f'(x)`, computed from a single run of `f`

#### Scenario: Elementary-function chain rule

- **WHEN** `derivative(|x| x.sin().exp(), x0)` is evaluated
- **THEN** the result equals `cos(x0)·exp(sin(x0))` within floating-point tolerance

#### Scenario: Second derivative via nesting

- **WHEN** `second_derivative(|x| x*x*x*x, 2.0_f64)` is evaluated
- **THEN** the result equals `f''(2) = 12·2² = 48`

### Requirement: Forward-mode gradient, directional derivative, and Jacobian

`deep_causality_num` SHALL provide multi-input forward-mode helpers over fixed-arity `const`-generic arrays. It SHALL provide `gradient::<N>(f, &[R; N]) -> [R; N]` for `f: Fn(&[Dual<R>; N]) -> Dual<R>` (one seeded coordinate per pass), `directional_derivative::<N>(f, &[R; N], &[R; N]) -> R` computing the derivative along a direction in a single pass, and `jacobian::<N, M>(f, &[R; N]) -> [[R; N]; M]` for vector-valued `f` (output-major: row `k` is the gradient of output `k`). All SHALL be allocation-free and bounded on `R: Real + Div<Output = R>`.

#### Scenario: Gradient of a scalar field

- **WHEN** `gradient(|p| p[0]*p[0] + p[1]*p[1], &[3.0_f64, 4.0])` is evaluated
- **THEN** the result equals `[2·3, 2·4] = [6.0, 8.0]`

#### Scenario: Directional derivative in one pass

- **WHEN** `directional_derivative(f, &x, &dir)` is evaluated
- **THEN** the result equals `∇f(x) · dir`, computed from a single evaluation of `f`

#### Scenario: Jacobian of a vector-valued map

- **WHEN** `jacobian` is evaluated for `f(x) = [x0·x1, x0 + x1]` at a point
- **THEN** row `i` equals the gradient of output component `i`, i.e. `[[x1, x0], [1, 1]]`

### Requirement: Division-only generic kernels accept dual numbers

A generic numeric kernel that uses only the operations available on `Real + Div` (addition, subtraction, multiplication, division, absolute value, ordering, `From<f64>`) SHALL be bounded on those operations rather than on the full `RealField`, so that `Dual<R>` flows through it and produces exact input sensitivities. `solve_gm_analytical_kernel` SHALL be relaxed from `R: RealField` to `R: Real + Div<Output = R> + From<f64>`, preserving its behavior for `f64` and additively admitting `Dual<f64>`. Where the kernel's inputs are themselves `RealField`-bound types, those input types (here `SpaceTimeCoordinate` and `CentralBody`) SHALL likewise be widened to `Real + Div` so the dual arguments can be constructed; field-needing impls keep their stronger bounds. To satisfy the kernel's `From<f64>` bound, a real literal SHALL be convertible to a *constant* dual via `impl<T: Real + From<f64>> From<f64> for Dual<T>` (`x + 0·ε`), which carries no derivative and so does not contaminate the `ε` channel. Kernels that require genuine field structure (e.g. a matrix inverse) are explicitly excluded from this relaxation.

#### Scenario: Existing f64 callers are unaffected

- **WHEN** `solve_gm_analytical_kernel` is called with `f64` coordinates as before
- **THEN** it returns the same recovered `GM` value as prior to the bound relaxation

#### Scenario: Recovering an input sensitivity through the kernel

- **WHEN** one input coordinate is supplied as `Dual::variable` and the others as `Dual::constant`, and `solve_gm_analytical_kernel` is run over `Dual<f64>`
- **THEN** the real part equals the recovered `GM` and the infinitesimal part equals `∂GM/∂(that input)`, matching a finite-difference estimate within tolerance

#### Scenario: A real literal converts to a derivative-free constant dual

- **WHEN** `Dual::<f64>::from(c)` is constructed for a real literal `c`
- **THEN** its value equals `c` and its derivative is zero, so constants a kernel builds with `R::from(..)` do not appear in the computed sensitivity
