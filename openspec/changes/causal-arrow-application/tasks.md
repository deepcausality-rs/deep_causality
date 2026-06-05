## Decisions (reconciliation)

- **Examples demonstrate, they do not test.** Per the owner, example crates show how to use the
  API by running and printing comparisons against the analytic/reference answer, not by adding
  `#[cfg(test)]` assertions. The "assert vs …" / "test vs …" clauses below are therefore satisfied
  by a printed, runnable comparison rather than an in-crate test.
- **No Bazel for examples.** The example crates are Cargo-only; there are no `BUILD.bazel` targets
  to add. The Bazel clauses in 3.3 and 7.3 are dropped.
- **7.2a dropped.** It required error-path test coverage, which the no-in-example-tests decision
  makes moot.
- **P1/P2 examples are full-stack showcases.** Each remaining example (sections 5 and 6) should
  weave the three native DeepCausality pillars together on one problem: the **Causal Monad**
  (`PropagatingEffect` bind chains sequence the workflow), the **Arrow calculus** (the tangent
  functor `gradient`/`derivative` and the integration operators `Euler`/`Rk4`/`quadrature` replace
  hand-rolled numerics), and **precision as a parameter** (a `FloatType` alias, or a scalar-generic
  field, re-runs the whole computation at f32/f64/Float106). The goal is to demonstrate the
  project's expressive load: less code, more power.

## 0. Preconditions

- [x] 0.1 Confirm `causal-arrow-calculus` (`arrow-calculus`) is implemented and archived; the operators are available from the `deep_causality_calculus` crate (`DifferentiateExt`/`DifferentiateFieldExt` methods, `Euler`/`Rk4` + `EndoArrow`, `quadrature`). Example crates that need them add a `deep_causality_calculus` dependency.

## 1. Autodiff rewrites — P0 (behavior-preserving)

- [x] 1.1 `maxwell` (physics_examples): replace the hand-coded `da_dt` / `da_dz` with `gradient` of a single `A_x(t, z)` closure; demonstrate equality with `−ω·sin(phase)` / `ω·sin(phase)`; `MaxwellSolver` call unchanged.
- [x] 1.2 `maxwell_multivector` (mathematics_examples/algebra): same rewrite on the second copy.
- [x] 1.3 `magnav` (avionics_examples): compute `∇B(x, y)` via `gradient` of the synthetic field closure; demonstrate vs finite-difference reference.
- [x] 1.4 diving-decompression (medicine_examples): compute Schreiner `dp/dt` via `derivative`; demonstrate vs analytic `k·(p_inspired − p)`.

## 2. Integration rewrites — P0 (de-dup + accuracy swap)

- [x] 2.1 Kuramoto: rewrite epilepsy and counterfactual-resection loops with `Euler` over a shared rate-field form; demonstrate step-for-step match to the prior loop.
- [x] 2.2 One heat-diffusion example: replace the time loop with the integrator (spatial Laplacian stays exterior calculus); show `Rk4` swap.
- [x] 2.3 One position-Euler example (tcas / magnav / hypersonic): replace with the integrator.

## 3. New fluid-dynamics / avionics example — P0 (MMS)

- [x] 3.1 Add an avionics CFD example: analytic Taylor–Green velocity/pressure closures → `gradient` for `∇u`, `∇²u`, `∇p` → `incompressible_ns_rhs_kernel` → `Rk4` march.
- [x] 3.2 MMS verification: marched field matches the exact Taylor–Green field within tolerance (printed comparison; precision a `FloatType` parameter at f32/f64/Float106).
- [x] 3.3 Register in `Cargo.toml`; the example runs and prints its residual/convergence.

## 4. Leibniz-bridge example — P0

- [x] 4.1 Add a small example: `quadrature` over `Dual` returns `∫f(x,θ)dx` (real part) and `dI/dθ` (infinitesimal part) in one sweep; demonstrate both vs analytic. Register (Cargo) and run.

## 5. Extended coverage — P1 (recommended)

- [ ] 5.1 Remaining heat-diffusion (2) and position-Euler loops onto the integrator.
- [ ] 5.2 Topological-insulator Chern number → `quadrature`; demonstrate vs prior accumulation.
- [ ] 5.3 GR gravity-family enhancements: `event_horizon_probe` (gravitational acceleration / tidal force as `−dΦ/dr`, `−d²Φ/dr²`) and `gauge_gr` (redshift / curvature gradient `d/dr`).

## 6. Optional showcases — P2

- [ ] 6.1 tumor-treatment: AD gradient ascent replacing simulated annealing (behavior-changing; documented as an improvement, not a behavior-preserving rewrite).
- [ ] 6.2 Second fluid example aimed at chaotic convection / turbulence, where higher precision buys something f64 cannot: a Clean-Numerical-Simulation-style integration of the Lorenz system (Saltzman/Lorenz 3-mode truncation of Rayleigh–Bénard convection) at f32/f64/Float106, showing the predictability horizon grow with precision. `Rk4` march; precision-generic rate field; printed divergence/horizon comparison.

## 7. Verification

- [x] 7.1 `cargo build` / `cargo run` for every touched example crate; new examples run and print their comparisons.
- [ ] 7.2 `make format && make fix` (multiple example crates touched); 0 clippy warnings, no `#[allow(...)]`.
- [ ] 7.3 Commit message prepared; owner commits. (No Bazel; no file deleted — superseded lines replaced in place.)
