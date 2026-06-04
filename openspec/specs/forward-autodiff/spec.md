# forward-autodiff Specification

## Purpose
TBD - created by archiving change causal-arrow-autodiff. Update Purpose after archive.
## Requirements
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

