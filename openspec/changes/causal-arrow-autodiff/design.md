## Context

Stage of the Causal Arrow program that puts a *user-facing forward-mode AD surface* on the `Dual<T>` number shipped by `causal-arrow-foundations` (live spec `dual-numbers`). It is the first of three coordinated changes:

1. **`causal-arrow-autodiff`** (this change) — differentiation as a Layer-1 *scalar* operation.
2. **`causal-arrow-autointegration`** — integration as a Layer-2 *operator over functions* (the sibling; the asymmetry is deliberate, see that change).
3. **`causal-arrow-application`** — apply both across the example suite and add new fluid-dynamics / avionics examples.

`Dual<T: Real>` already provides: `new`/`constant`/`variable`/`value`/`derivative`; `Add`/`Sub`/`Mul`/`Neg`/`Div` (+ the assign variants except `DivAssign`); scalar `Mul<T>`; and `impl<T: Real + Div<Output = T>> Real for Dual<T>`, which makes `Dual<Dual<T>>` a `Real` and therefore gives second derivatives by nesting. The elementary functions (`sqrt`, `exp`, `ln`, `sin`, `cos`, `tan`, `powf`, the inverse and hyperbolic forms) carry their derivative rules.

Constraints (`AGENTS.md`): `unsafe_code = "forbid"`; static dispatch only, no `dyn`; no external crates; no macros in `src/`; one concern per module; tests mirror `src/`; 100% coverage; the two writing guides bind prose.

## Goals / Non-Goals

**Goals**
- A minimal, composable forward-mode AD surface: scalar `derivative`, paired `value_and_derivative`, multi-input `gradient` / `directional_derivative` / `jacobian`, and `second_derivative`.
- Establish the **bound-relaxation principle** (bound on the *operations used*, not the *named field*) and apply it once, to `solve_gm_analytical_kernel`, so a real kernel yields exact input sensitivities under `Dual`.
- Shape every signature so `causal-arrow-application` can drop it into `maxwell` (`∇A`), decompression (`dp/dt`), `magnav` (`∇B`), and the fluid kernels (`∇u`, `∇²u`, `∇p`) without further API work.

**Non-Goals (deferred or out)**
- Reverse-mode / adjoint AD; a Taylor / jet type; symbolic differentiation.
- Any change to `Dual` arithmetic or to the `dual-numbers` spec.
- Integration, quadrature, or the Leibniz bridge — `causal-arrow-autointegration`.
- Rewriting examples — `causal-arrow-application`.
- Differentiating discrete-mesh fields with no closed form — those stay with the topology exterior-calculus operators (`exterior_derivative`, `codifferential`, `laplacian`).

## Decisions

### D1 — Free functions over `Dual`, not a new trait or type

AD here is seeding plus reading: `Dual::variable(x)` injects the `ε`, the function propagates it by the chain rule, `.derivative()` reads it back. That needs no new type and no trait. The surface is a handful of generic free functions in a `num` `autodiff` module. This keeps the API discoverable, zero-cost, and additive, and it matches the crate convention (one concern per file).

```rust
pub fn derivative<R, F>(f: F, x: R) -> R
where R: Real + Div<Output = R>, F: Fn(Dual<R>) -> Dual<R>
{ f(Dual::variable(x)).derivative() }

pub fn value_and_derivative<R, F>(f: F, x: R) -> (R, R)
where R: Real + Div<Output = R>, F: Fn(Dual<R>) -> Dual<R>
{ let y = f(Dual::variable(x)); (y.value(), y.derivative()) }
```

### D2 — Bound is `Real + Div<Output = R>`, uniformly

`Real` deliberately omits division (that is what keeps `Dual` out of `Field`). But `Dual<R>` implements `Div` (and is itself `Real`) **only** when `R: Real + Div<Output = R>`. Any differentiand that uses `/` therefore needs that bound on `R`, and `Dual<Dual<R>>` (second derivatives) needs it on the inner type too. Rather than split the surface, every helper carries `R: Real + Div<Output = R>`. `f32`/`f64`/`Float106` all satisfy it; the bound only excludes scalars that have no division, which cannot be differentiands anyway.

### D3 — Multi-input by `const`-generic arrays, one seeded coordinate per pass

`gradient::<N>` takes `&[R; N]`, and for each `i` builds the seed vector where coordinate `i` is `Dual::variable(x[i])` and the rest are `Dual::constant(x[j])`, runs `f`, and collects `.du` into `[R; N]`. `N` forward passes, no heap, stack-sized output. `directional_derivative` does it in **one** pass by seeding coordinate `i` as `Dual::new(x[i], dir[i])`. `jacobian::<N, M>` returns `[[R; M]; N]` for `f: Fn(&[Dual<R>; N]) -> [Dual<R>; M]`. Const generics keep it allocation-free and fit the fixed-arity physics kernels (`[R; 3]` velocity, `[[R; 3]; 3]` Jacobian).

### D4 — Higher derivatives reuse `Dual<Dual<R>>`, no new machinery

`second_derivative(f, x)` runs `f` over `Dual::variable(Dual::variable(x))` and reads `.derivative().derivative()`. This is the systematic Taylor/jet direction expressed by nesting; a dedicated jet type is explicitly out of scope. The helper exists so callers do not hand-write the nesting.

### D5 — Bound-relaxation principle: bind on operations, not on the named field

A generic kernel should bind on the algebra it actually uses. `solve_gm_analytical_kernel` uses `+ − × ÷ .abs() From<f64>` and ordering — all available on `Real + Div`, none requiring the full `RealField`. Relaxing `RealField → Real + Div<Output = R> + From<f64>` is behavior-preserving for `f64` and additively admits `Dual<f64>`, turning the kernel into a sensitivity probe (`∂GM/∂input`) at no runtime cost. This change applies the principle **once**, to `solve_gm`, as the worked instance; a broader bound audit across physics kernels is left to `causal-arrow-application` where the consuming examples justify each relaxation. The Kalman / EKF kernels that genuinely need a matrix `inverse()` (true field structure) are *not* candidates and stay on `RealField`.

### D6 — Layer placement and the non-Kleisli framing

`Dual` is a *functor on the scalar*: differentiation is local and compositional (the chain rule is a ring homomorphism), so it lives in the number type. These helpers are its API. Integration is *not* a scalar operation — it is a non-local functional over an interval and is not algebraically closed (Liouville), so it cannot be a `Dual`-style type and instead becomes a Layer-2 operator in `causal-arrow-autointegration`. Recording this here keeps the two changes from drifting toward a false symmetry: there is no "anti-dual" number.

### D7 — Analytic-only boundary (stated, not worked around)

Forward-mode AD returns exact derivatives **only** where the differentiand is a closed-form function the caller can evaluate on `Dual`. For fields given as discrete samples on a mesh with no formula (general CFD on a grid), there is nothing to seed; the spatial derivative there remains a stencil / exterior-calculus operator. The autodiff surface targets closed-form fields, manufactured solutions, and parameter sensitivities — and the spec says so, so `causal-arrow-application` picks AD targets accordingly.

## Risks / Trade-offs

- **`gradient` cost is O(N) passes** (forward mode). For the low-dimensional physics/medicine targets (N ≤ ~9) this is irrelevant; high-dimensional gradients would want reverse mode, which is explicitly out of scope and flagged for a future stage.
- **Bound relaxation blast radius.** Mitigated by widening only (no caller breaks) and by running `gitnexus_impact` before the `solve_gm` edit; the existing `f64` tests guard behavior.

## Migration / Rollout

Purely additive. `solve_gm`'s relaxation is source-compatible. No downstream change is required until `causal-arrow-application` opts in.
