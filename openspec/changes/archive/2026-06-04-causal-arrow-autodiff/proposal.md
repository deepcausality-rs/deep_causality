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
- **Relax over-strict kernel bounds where division is the only field operation used,** beginning with `solve_gm_analytical_kernel`: `R: RealField` → `R: Real + Div<Output = R> + From<f64>`, so `Dual<f64>` flows through and `∂GM/∂clock_drift` (and the other input sensitivities) become available. Because the kernel's inputs are themselves `RealField`-bound public structs, this *also* widens `SpaceTimeCoordinate<R>` and `CentralBody<R>` (and their impls) to `R: Real + Div<Output = R>` — a contained, source-compatible widening in `chronometric_quantities.rs`; field-needing impls keep their stronger bounds. Every change here is a bound *widening*: strictly more types are accepted, no existing `f64` caller is affected.
- **Add `impl<T: Real + From<f64>> From<f64> for Dual<T>`** — a real literal converts to a *constant* dual (`x + 0·ε`, zero derivative). This lets `Dual` satisfy the `From<f64>` bound that generic kernels use to build constants (`R::from(0.5)`), so AD reaches `f64`-literal-driven arithmetic without those literals contaminating the `ε` channel. It is the one additive change to the `dual-numbers` surface (a conversion, not arithmetic).
- Document the **layer placement**: `Dual` is the Layer-1 analytic scalar; these helpers are its forward-mode API. The Layer-2 *integration* operator and the differentiate-under-the-integral (Leibniz) bridge are the sibling change `causal-arrow-autointegration`. Applying both to the example suite is `causal-arrow-application`.
- **Out of scope:** reverse-mode / adjoint AD; a Taylor / jet type (the systematic higher-derivative generalization of `Dual<Dual<…>>`); symbolic differentiation; any change to `Dual` arithmetic itself; rewriting examples (that is `causal-arrow-application`); discrete-mesh spatial derivatives (those stay in the topology exterior-calculus operators — AD gives exact derivatives only for closed-form functions).
- **No new dependency.** Pure generic functions over the existing `Dual` + algebra tower; stays inside `unsafe_code = "forbid"`, static dispatch, no macros in `src/`.

## Capabilities

### New Capabilities

- `forward-autodiff`: a forward-mode automatic-differentiation surface over `Dual<T>` in `deep_causality_num` — `derivative`, `value_and_derivative`, `gradient`, `directional_derivative`, `jacobian`, `second_derivative` — plus the bound-relaxation principle (`RealField → Real + Div`) that lets `Dual` flow through division-only generic kernels, demonstrated end-to-end on `solve_gm_analytical_kernel`.

### Modified Capabilities

- `dual-numbers`: one additive conversion — `impl<T: Real + From<f64>> From<f64> for Dual<T>` (a real literal → a constant dual). The existing `Dual` arithmetic and elementary-function surface is unchanged; this only lets `Dual` flow through `From<f64>`-bounded generic code. (The `solve_gm` / chronometric-struct bound relaxations touch physics code that has no tracked capability spec, so they are recorded under Impact, not as a spec delta.)

## Impact

- **New code, `deep_causality_num`:** an `autodiff` module (one helper concern per file, per the crate's module convention) under `src/`, re-exported from `lib.rs`; a `dual/dual_number/convert.rs` with the `From<f64>` impl; mirrored tests under `tests/`.
- **Modified code, `deep_causality_physics`:** `solve_gm_analytical_kernel` (and `inv_r_effective`) relaxed `RealField → Real + Div<Output = R> + From<f64>`; the input structs `SpaceTimeCoordinate<R>` and `CentralBody<R>` (and their impls) widened `RealField → Real + Div<Output = R>` in `chronometric_quantities.rs`. All behavior-preserving for `f64`. `gitnexus_impact`: MEDIUM (14 direct callers + 1 example, all `f64`); a widening breaks none.
- **APIs:** additive free functions, one additive `From<f64>` impl, and bound widenings only. No existing signature narrows; no caller breaks.
- **Dependencies:** none added.
- **Consumers (later changes):** `causal-arrow-application` uses these helpers to rewrite `maxwell`, diving-decompression (`dp/dt`), `magnav` (`∇B`), and the fluid examples; `causal-arrow-autointegration` reuses `derivative`/`gradient` for the Leibniz differentiate-under-the-integral bridge.
- **Verification:** analytic checks (polynomials, `exp`/`sin`/`ln`, chain / product / quotient rules), gradient and Jacobian on known fields, second derivative via nesting, and a finite-difference cross-check of `∂GM/∂inputs`; 100% coverage of new code, **including every error path** (each `Err` / validation / panic branch) for maximum coverage.
