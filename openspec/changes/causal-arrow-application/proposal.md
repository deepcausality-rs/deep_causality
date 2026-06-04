## Why

`causal-arrow-autodiff` and `causal-arrow-autointegration` add the two primitives, but a primitive earns its place only when it removes real code. The example survey already located the targets. Hand-coded analytic derivatives that *are* the physics: `maxwell` writes `∂A/∂t` and `∂A/∂z` by hand in two separate copies (physics and mathematics); `magnav`'s synthetic-field gradient `∇B` — the core navigation observable — is absent; diving-decompression's Schreiner gas-loading rate `dp/dt` — the clinical driver — is never computed. Roughly ten duplicated explicit-Euler loops: Kuramoto twice (epilepsy and a counterfactual-intervention example), heat diffusion three times, position-Euler three times, rotor-Euler twice. A Riemann-sum Chern number. And a complete fluid-dynamics kernel set with *no examples at all*, because every kernel returns `∂u/∂t` and demands `∇u` / `∇²u` / `∇p` as caller-supplied inputs — exactly the two gaps the primitives fill.

This change applies the primitives, turning them into demonstrated, tested, user-facing simplifications, and adds the missing fluid-dynamics examples — notably in the avionics domain — that the new kernels were built for.

## What Changes

- **Replace hand-coded derivatives with the `forward-autodiff` surface (behavior-preserving — identical numbers):**
  - `maxwell` (physics and mathematics copies): derive `∂A/∂t`, `∂A/∂z` from a single `A_x(t, z)` closure via `gradient`, feeding the unchanged `MaxwellSolver`.
  - `magnav`: compute the field gradient `∇B(x, y)` of the synthetic anomaly field via `gradient` — previously absent.
  - diving-decompression: compute the Schreiner gas-loading rate `dp/dt` via `derivative` — previously uncomputed.
- **Replace hand-rolled stepping with the `numeric-integration` operator (de-duplication + swappable accuracy):**
  - the duplicated Kuramoto Euler loops (epilepsy, counterfactual-resection), the three heat-diffusion time loops, and at least one position-Euler loop become `Euler` / `Rk4` calls over a shared rate field — the spatial Laplacian stays exterior calculus, only the *time* loop changes;
  - the topological-insulator Chern number becomes a `quadrature` call.
- **Add new fluid-dynamics examples, including in the avionics domain,** demonstrating the full pipeline the kernels were built for: *closed-form field → `gradient` for `∇u` / `∇²u` / `∇p` → fluid RHS kernel for `∂u/∂t` → integrator for the time march*. At least one avionics CFD example SHALL use the **Method of Manufactured Solutions**: take the exact derivatives of an analytic velocity field with the autodiff surface and verify the solver reproduces it.
- **Demonstrate the Leibniz bridge** in one example: `quadrature` over `Dual` returning a definite integral and its parameter sensitivity in a single sweep.
- **Out of scope / optional showcases:** replacing the tumor-treatment simulated annealing with AD gradient ascent (changes behavior — an optional P2 demo, not a behavior-preserving rewrite); the GR gravity-family enhancements (`event_horizon_probe`, `gauge_gr`) as optional P1; any change to a library API (this change consumes the two new capabilities, it does not extend them); the broader physics-kernel bound-relaxation audit beyond what a concrete example needs.
- **No new library dependency.** Examples only; each new example is registered (Cargo + `BUILD.bazel`) with a smoke / assertion test per repo convention.

## Capabilities

### New Capabilities

- `autodiff-integration-examples`: the worked-example layer of the Causal Arrow program. Existing hand-coded derivatives and hand-rolled integration loops across the physics, avionics, medicine, and mathematics suites are re-expressed on the `forward-autodiff` and `numeric-integration` surfaces, and new fluid-dynamics examples — including an avionics CFD example with Method-of-Manufactured-Solutions verification — demonstrate the differentiate → kernel → integrate pipeline end to end.

### Modified Capabilities

<!-- None at the library-spec level. This change edits example crates and adds new ones; it consumes forward-autodiff and numeric-integration without modifying either. -->

## Impact

- **Depends on:** `causal-arrow-autodiff` (`forward-autodiff`) and `causal-arrow-autointegration` (`numeric-integration`); both SHALL be implemented and archived before this change is applied.
- **Modified examples:** maxwell (×2), magnav, diving_decompression, epilepsy, counterfactual_resection, the three heat-diffusion examples, topological_insulator, and at least one position-Euler example. Each rewrite asserts the new output equals the prior hand-computed value.
- **New examples:** at least one avionics fluid-dynamics / CFD example (MMS-verified) and at least one Leibniz-bridge example; registered in Cargo and `BUILD.bazel` with tests.
- **APIs:** none changed.
- **Verification:** per-example assertion tests that AD results equal the replaced hand-coded values; integrator rewrites reproduce the prior `Euler` trajectory and improve under `Rk4`; the MMS example verifies the solver against an exact manufactured solution within tolerance. Coverage of new and modified code includes **every error path** (each `Err` / validation / panic branch) for maximum coverage.
