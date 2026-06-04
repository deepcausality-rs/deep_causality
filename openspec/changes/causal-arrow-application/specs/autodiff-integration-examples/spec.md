## ADDED Requirements

### Requirement: Hand-coded derivatives are replaced by the arrow-calculus tangent functor, behavior-preserving

Examples that compute a derivative of a closed-form field by hand SHALL be rewritten to use the `arrow-calculus` tangent functor (`deep_causality_calculus`), producing the same numerical result. The rewrite SHALL define the underlying field as a scalar-generic `DifferentiableArrow` / `DifferentiableField` model and obtain its derivative(s) via the `DifferentiateExt` / `DifferentiateFieldExt` methods (`model.derivative(x)` / `field.gradient(&x)`), and a test SHALL assert the result equals the previously hand-coded value within floating-point tolerance. The downstream consumer of the derivative (e.g. `MaxwellSolver`) SHALL be unchanged.

#### Scenario: Maxwell field derivative via autodiff equals the hand-coded value

- **WHEN** the `maxwell` example derives `∂A_x/∂t` and `∂A_x/∂z` from the closure `A_x(t, z) = cos(ω(t − z))` using `gradient`
- **THEN** the values equal the previous hand-coded `−ω·sin(phase)` and `ω·sin(phase)` within tolerance, and the assembled gradient multivector fed to `MaxwellSolver` is unchanged

#### Scenario: A previously-absent gradient is now computed

- **WHEN** the `magnav` example computes the field gradient `∇B(x, y)` of its synthetic anomaly field via `gradient`
- **THEN** it obtains both partials in one place from the field closure, and a test confirms they match a finite-difference estimate of the same field within tolerance

#### Scenario: A meaningful rate is now computed

- **WHEN** the diving-decompression example computes the Schreiner gas-loading rate `dp/dt` via `derivative` of `p(t)`
- **THEN** the result equals the analytic `k·(p_inspired − p(t))` within tolerance

### Requirement: Hand-rolled integration is replaced by the integration operator

Examples that hand-write an explicit time-stepping loop SHALL be rewritten to use the `arrow-calculus` integration operators, reproducing the prior trajectory. The rate field SHALL be expressed once as `Fn(&S) -> S`, built into an `Euler` (or `Rk4`) endo-arrow, and advanced with `EndoArrow::iterate_n`; substituting `Rk4` for `Euler` SHALL require no change to the rate field. Duplicated loops across examples SHALL share the same rate-field form. A hand-written Riemann-sum quadrature SHALL be replaced by `quadrature`.

#### Scenario: Kuramoto Euler loop reproduced by the operator

- **WHEN** the epilepsy (and counterfactual-resection) Kuramoto update is run with `Euler.integrate` over the shared rate field
- **THEN** the resulting phase trajectory matches the prior hand-rolled `+= d_theta·dt` loop step-for-step within tolerance

#### Scenario: Accuracy is upgraded without touching the model

- **WHEN** a rewritten stepping example swaps `Euler` for `Rk4`
- **THEN** the rate field and call shape are unchanged and the integrated result is at least as accurate against a reference

#### Scenario: A Riemann sum becomes a quadrature call

- **WHEN** the topological-insulator Chern-number accumulation is replaced by `quadrature` over the Brillouin interval
- **THEN** the computed Chern number matches the prior accumulated value within tolerance

### Requirement: A new avionics fluid-dynamics example demonstrates differentiate → kernel → integrate with MMS verification

The change SHALL add at least one new fluid-dynamics example in the avionics domain that uses the `arrow-calculus` tangent functor to produce the spatial derivatives a fluid RHS kernel requires, the kernel to produce `∂u/∂t`, and the `arrow-calculus` endo-arrow iteration to march in time, verified by the Method of Manufactured Solutions against an exact analytic solution. The example SHALL be registered in Cargo and `BUILD.bazel` with a test.

#### Scenario: Exact spatial derivatives feed the Navier–Stokes kernel

- **WHEN** the example takes `∇u`, `∇²u`, and `∇p` of an analytic velocity field (a Taylor–Green vortex) with the arrow-calculus tangent functor and passes them to `incompressible_ns_rhs_kernel`
- **THEN** the derivatives are exact (no finite differences) and the kernel returns the field's `∂u/∂t` at the sample point

#### Scenario: The marched solution matches the manufactured solution

- **WHEN** the example marches the field with `Rk4` over one or more steps
- **THEN** the marched field matches the exact Taylor–Green field at that time within tolerance (Method of Manufactured Solutions verification)

### Requirement: An example demonstrates the differentiate-under-the-integral bridge

The change SHALL include an example demonstrating that the two primitives compose: evaluating `quadrature` over `Dual` yields a definite integral and its sensitivity to a parameter in a single sweep.

#### Scenario: Integral and parameter sensitivity in one sweep

- **WHEN** the example integrates `f(x, θ)` over a fixed interval with `θ` seeded as `Dual::variable`
- **THEN** the real part is the integral `I(θ)` and the infinitesimal part is `dI/dθ`, each matching its analytic value within tolerance
