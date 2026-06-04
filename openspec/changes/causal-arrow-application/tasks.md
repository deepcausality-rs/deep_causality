## 0. Preconditions

- [ ] 0.1 Confirm `causal-arrow-calculus` (`arrow-calculus`) is implemented and archived; the operators are available from the `deep_causality_calculus` crate (`DifferentiateExt`/`DifferentiateFieldExt` methods, `Euler`/`Rk4` + `EndoArrow`, `quadrature`). Example crates that need them add a `deep_causality_calculus` dependency.

## 1. Autodiff rewrites — P0 (behavior-preserving)

- [ ] 1.1 `maxwell` (physics_examples): replace the hand-coded `da_dt` / `da_dz` with `gradient` of a single `A_x(t, z)` closure; assert equality with `−ω·sin(phase)` / `ω·sin(phase)`; `MaxwellSolver` call unchanged.
- [ ] 1.2 `maxwell_multivector` (mathematics_examples/algebra): same rewrite on the second copy.
- [ ] 1.3 `magnav` (avionics_examples): compute `∇B(x, y)` via `gradient` of the synthetic field closure; test vs finite-difference reference.
- [ ] 1.4 diving-decompression (medicine_examples): compute Schreiner `dp/dt` via `derivative`; assert vs analytic `k·(p_inspired − p)`.

## 2. Integration rewrites — P0 (de-dup + accuracy swap)

- [ ] 2.1 Kuramoto: rewrite epilepsy and counterfactual-resection loops with `Euler.integrate` over a shared rate-field form; assert step-for-step match to the prior loop.
- [ ] 2.2 One heat-diffusion example: replace the time loop with the integrator (spatial Laplacian stays exterior calculus); show `Rk4` swap.
- [ ] 2.3 One position-Euler example (tcas / magnav / hypersonic): replace with the integrator.

## 3. New fluid-dynamics / avionics example — P0 (MMS)

- [ ] 3.1 Add an avionics CFD example: analytic Taylor–Green velocity/pressure closures → `gradient` for `∇u`, `∇²u`, `∇p` → `incompressible_ns_rhs_kernel` → `Rk4` march.
- [ ] 3.2 MMS verification test: marched field matches the exact Taylor–Green field within tolerance.
- [ ] 3.3 Register in `Cargo.toml` + `BUILD.bazel`; smoke/assertion test wired into the crate's test tree.

## 4. Leibniz-bridge example — P0

- [ ] 4.1 Add a small example: `quadrature` over `Dual` returns `∫f(x,θ)dx` (real part) and `dI/dθ` (infinitesimal part) in one sweep; assert both vs analytic. Register (Cargo + Bazel) with a test.

## 5. Extended coverage — P1 (recommended)

- [ ] 5.1 Remaining heat-diffusion (2) and position-Euler loops onto the integrator.
- [ ] 5.2 Topological-insulator Chern number → `quadrature`; assert vs prior accumulation.
- [ ] 5.3 GR gravity-family enhancements: `event_horizon_probe` (gravitational acceleration / tidal force as `−dΦ/dr`, `−d²Φ/dr²`) and `gauge_gr` (redshift / curvature gradient `d/dr`).

## 6. Optional showcases — P2

- [ ] 6.1 tumor-treatment: AD gradient ascent replacing simulated annealing (behavior-changing; documented as an improvement, not a behavior-preserving rewrite).
- [ ] 6.2 A second fluid example in `physics_examples` (e.g. vortex diagnostics via the kinematics kernels).

## 7. Verification

- [ ] 7.1 `cargo build` / `cargo test` for every touched example crate; new examples run and their assertion tests pass.
- [ ] 7.2 `make format && make fix` (multiple example crates touched); 0 clippy warnings, no `#[allow(...)]`.
- [ ] 7.2a All error paths covered: every `Err` / validation / panic branch in new and modified example code (and any new helper) is exercised by a test for maximum coverage.
- [ ] 7.3 Bazel targets for new examples build (`BUILD.bazel` updated). No file deleted (owner-approval rule); superseded lines replaced in place. Commit message prepared; owner commits.
