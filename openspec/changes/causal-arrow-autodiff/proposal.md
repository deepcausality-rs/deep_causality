## Why

The `dual-numbers` capability (archived in `causal-arrow-foundations`) shipped `Dual<T: Real>` — a number that carries its own derivative through ordinary arithmetic and the elementary functions, with `Dual<Dual<_>>` giving higher derivatives. What it does **not** ship is the user-facing surface that turns that type into forward-mode automatic differentiation. There is no `derivative(f, x)`, no `gradient(f, &x)`, no `directional_derivative`. A caller today constructs seed duals by hand, runs the function, and reads `.du` — the mechanics, not the intent.

A survey of the example suite (physics, avionics, medicine, mathematics) measures the cost. Derivatives that *are* the physics get hand-coded: `maxwell` writes `∂A/∂t = −ω·sin(phase)` by hand, in two separate copies (the physics and the mathematics suites); the diving-decompression Schreiner curve's gas-loading rate `dp/dt` is never computed though it is the clinical driver of decompression stress; the magnetic-navigation field gradient `∇B` — the core navigation observable — is absent; a hypersonic example finite-differences velocity and acceleration. Every fluid-dynamics RHS kernel (`incompressible_ns_rhs_kernel` and siblings) *demands* a velocity Jacobian `∇u`, a `∇²u`, and a `∇p` as caller-supplied inputs, with no turnkey way to produce them.

A second finding from foundations sharpens the need. Nearly every numeric kernel is bounded `R: RealField`, but `Dual<T>` deliberately is **not** a `RealField` (`ε` is a zero divisor, so `Dual` is not a field). A division-only generic kernel such as `solve_gm_analytical_kernel` — which uses only `+ − × ÷ .abs()` — cannot accept `Dual<f64>` today purely because its bound over-asks for `RealField`. Relaxing such a bound to the operations actually used (`Real + Div`) lets `Dual` flow through and yields exact input-sensitivities for free.

## What Changes

- Add a **forward-mode autodiff surface** over the existing `Dual<T>` in `deep_causality_num`:
  - `derivative(f, x) -> R` — `f'(x)` for `f: Fn(Dual<R>) -> Dual<R>` (seeds `Dual::variable(x)`, runs, reads `.du`).
  - `value_and_derivative(f, x) -> (R, R)` — `f(x)` and `f'(x)` in one pass.
  - `gradient::<N>(f, &[R; N]) -> [R; N]` — ∇f for `f: Fn(&[Dual<R>; N]) -> Dual<R>`, one seeded coordinate per pass.
  - `directional_derivative::<N>(f, &x, &dir) -> R` — single-pass derivative along a direction.
  - `jacobian::<N, M>(f, &[R; N]) -> [[R; M]; N]` — for vector-valued `f`.
  - `second_derivative(f, x) -> R` — via the existing `Dual<Dual<R>>` nesting.
- Each helper is a thin generic free function bounded on `R: Real + Div<Output = R>`, so it works for `f32` / `f64` / `Float106` and composes (nested duals → higher derivatives) without a new type.
- **Relax over-strict kernel bounds where division is the only field operation used,** beginning with `solve_gm_analytical_kernel`: `R: RealField` → `R: Real + Div<Output = R> + From<f64>`, so `Dual<f64>` flows through and `∂GM/∂{r, θ, v, clock_drift}` becomes available. This is a bound *widening* — strictly more types are accepted, no existing `f64` caller is affected.
- Document the **layer placement**: `Dual` is the Layer-1 analytic scalar; these helpers are its forward-mode API. The Layer-2 *integration* operator and the differentiate-under-the-integral (Leibniz) bridge are the sibling change `causal-arrow-autointegration`. Applying both to the example suite is `causal-arrow-application`.
- **Out of scope:** reverse-mode / adjoint AD; a Taylor / jet type (the systematic higher-derivative generalization of `Dual<Dual<…>>`); symbolic differentiation; any change to `Dual` arithmetic itself; rewriting examples (that is `causal-arrow-application`); discrete-mesh spatial derivatives (those stay in the topology exterior-calculus operators — AD gives exact derivatives only for closed-form functions).
- **No new dependency.** Pure generic functions over the existing `Dual` + algebra tower; stays inside `unsafe_code = "forbid"`, static dispatch, no macros in `src/`.

## Capabilities

### New Capabilities

- `forward-autodiff`: a forward-mode automatic-differentiation surface over `Dual<T>` in `deep_causality_num` — `derivative`, `value_and_derivative`, `gradient`, `directional_derivative`, `jacobian`, `second_derivative` — plus the bound-relaxation principle (`RealField → Real + Div`) that lets `Dual` flow through division-only generic kernels, demonstrated end-to-end on `solve_gm_analytical_kernel`.

### Modified Capabilities

<!-- None. `dual-numbers` is consumed unchanged: the helpers are a new surface *over* the existing `Dual` arithmetic, not a modification of it. The `solve_gm` bound relaxation touches physics code that has no tracked capability spec, so it is recorded under Impact, not as a spec delta. -->

## Impact

- **New code, `deep_causality_num`:** an `autodiff` module (one helper concern per file, per the crate's module convention) under `src/`, re-exported from `lib.rs`; mirrored tests under `tests/`.
- **Modified code, `deep_causality_physics`:** `solve_gm_analytical_kernel` bound relaxed `RealField → Real + Div<Output = R> + From<f64>` (behavior-preserving). Run `gitnexus_impact` before editing; expected blast radius is low (one generic kernel; callers pass `f64`).
- **APIs:** additive free functions plus one bound widening. No existing signature narrows; no caller breaks.
- **Dependencies:** none added.
- **Consumers (later changes):** `causal-arrow-application` uses these helpers to rewrite `maxwell`, diving-decompression (`dp/dt`), `magnav` (`∇B`), and the fluid examples; `causal-arrow-autointegration` reuses `derivative`/`gradient` for the Leibniz differentiate-under-the-integral bridge.
- **Verification:** analytic checks (polynomials, `exp`/`sin`/`ln`, chain / product / quotient rules), gradient and Jacobian on known fields, second derivative via nesting, and a finite-difference cross-check of `∂GM/∂inputs`; 100% coverage of new code.
