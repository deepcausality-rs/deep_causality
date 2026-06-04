## Why

The codebase integrates far more than it differentiates, and every integration is hand-rolled. A repo-wide survey finds the same explicit-Euler loop `state += f(state)·dt` re-implemented across roughly ten example sites: the Kuramoto oscillator twice (the epilepsy example and a counterfactual-intervention example, essentially verbatim), heat diffusion three times, position Euler three times (TCAS, magnav, hypersonic), and rotor-Euler twice (the physics and mathematics tilt estimators). On top of that sit a Riemann-sum quadrature (the topological-insulator Chern number, `total_flux += flux` over the Brillouin zone) and the fluid-dynamics RHS kernels that hand back `∂u/∂t` with no time-marcher to consume them.

Even the library reimplements integration three times, in three crates, with no shared abstraction: a bespoke RK4 inside `geodesic_integrator_kernel` (physics), an `Euler` / `RungeKutta3` gradient flow hard-wired to the gauge-lattice type (topology), and Stokes form-integration over a chain (haft). There is no reusable numeric integrator anywhere. The cost is duplicated boilerplate, no way to swap accuracy (Euler → RK4) without rewriting the model, and a complete fluid kernel set that cannot actually run a simulation.

A conceptual point fixes the *shape* of the fix. Integration is not the mirror of the `Dual` differentiation type, and cannot be. Differentiation is local and compositional — the chain rule is a ring homomorphism — which is exactly why a number that carries its own derivative works. Integration is a non-local functional over an interval and is not algebraically closed in elementary functions (Liouville's theorem: `∫e^{−x²}` has no elementary antiderivative), so no "anti-dual" number whose arithmetic accumulates an integral can exist. The correct realization is an **operator over functions** — a stepper / quadrature combinator generic over the state's vector space — which composes *with* `Dual` (differentiate under the integral) rather than mirroring it.

## What Changes

- Add a **numeric integration operator** to `deep_causality_num`, generic over a state that forms a module over the scalar — `S: Clone + Add<Output = S> + Mul<R, Output = S>`, `R: RealField` — the `Module<R>` structure the algebra tower already defines, satisfied by `f64`, `Complex`, `Dual`, `CausalTensor`, and `CausalMultiVector`:
  - an `Integrator` trait with `step(&self, s: &S, dt: R, f: &F) -> S` for a rate field `F: Fn(&S) -> S`, and a provided `integrate(&self, s0: S, dt: R, steps: usize, f: &F) -> S`;
  - `Euler` (first-order) and `Rk4` (classical fourth-order) stepper structs implementing it — swap the struct, keep the model.
- Add **quadrature**: `quadrature(f, a, b, n) -> R` (composite Simpson) over `R: Real`, computing `∫ₐᵇ f`. Because it is generic over `Real`, it runs over `Dual<R>` unchanged, giving the **Leibniz bridge** — a definite integral and its derivative with respect to a parameter in one sweep (`d/dθ ∫f(x,θ)dx = ∫ ∂f/∂θ dx`).
- Document the **layer placement**: integration is the Layer-2 operator over functions; differentiation (`causal-arrow-autodiff`) is the Layer-1 scalar; they meet via Leibniz, not as dual types. The operator MAY later be lifted into the `arrow-strength` `Arrow` surface as a Layer-2 morphism — that lift is out of scope here, to keep `deep_causality_num` free of a `deep_causality_haft` dependency.
- **Out of scope:** adaptive / error-controlled steppers; implicit or stiff solvers (BDF); multistep (Adams); symplectic integrators; PDE mesh operators (those stay in the topology exterior-calculus surface); forward sensitivity *through* the solver (state carried as `Dual`), noted as a follow-on; rewriting examples (`causal-arrow-application`).
- **No new dependency.** Generic functions / structs over the existing algebra tower; stays inside `unsafe_code = "forbid"`, static dispatch, no macros in `src/`.

## Capabilities

### New Capabilities

- `numeric-integration`: a reusable numeric integration operator in `deep_causality_num` — an `Integrator` trait with `Euler` and `Rk4` steppers generic over any module-valued state, plus composite-Simpson `quadrature` that composes with `Dual` for differentiate-under-the-integral. This is the Layer-2 operator complement to `forward-autodiff`, and the consolidation target for the three ad-hoc integrators that exist today.

### Modified Capabilities

<!-- None. Additive new surface. The existing per-crate integrators (geodesic RK4, gauge-flow Euler/RK3, Stokes integrate) are left in place; re-expressing them on this operator is a later, separate change. -->

## Impact

- **New code, `deep_causality_num`:** an `integration` module (the `Integrator` trait, `Euler`, `Rk4`, and `quadrature`, one concern per file), re-exported from `lib.rs`; mirrored tests under `tests/`.
- **APIs:** additive trait + two structs + one free function. No existing signature changes.
- **Dependencies:** none added.
- **Consumers (later change):** `causal-arrow-application` replaces the ~10 hand-rolled Euler loops and the Chern-number Riemann sum, time-marches the fluid RHS kernels with `Rk4`, and uses the quadrature / Leibniz bridge to demonstrate autodiff × integration.
- **Verification:** order-of-accuracy tests (Euler `O(dt)`, RK4 `O(dt⁴)` on `y' = y` → `exp`), Simpson exact on cubics and convergent on transcendentals, a harmonic-oscillator energy check, and a Leibniz test (quadrature over `Dual` equals the analytic parameter derivative); 100% coverage of new code.
