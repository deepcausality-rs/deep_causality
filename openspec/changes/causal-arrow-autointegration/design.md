## Context

The Layer-2 sibling of `causal-arrow-autodiff`. Where autodiff puts a forward-mode surface on the `Dual` *scalar*, this change adds a numeric integration *operator* — the complement that the example survey shows is needed far more often (≈10 hand-rolled Euler loops, a Riemann-sum quadrature, and a fluid kernel set that returns `∂u/∂t` with nothing to march it). Three integrators already exist in the library, each welded to one type: `geodesic_integrator_kernel` (RK4, physics), the gauge-lattice gradient flow (`FlowMethod::{Euler, RungeKutta3}`, topology), and Stokes `integrate(form, chain)` (haft). None is reusable.

Constraints (`AGENTS.md`): `unsafe_code = "forbid"`; static dispatch, no `dyn`; no external crates; no macros in `src/`; one concern per module; tests mirror `src/`; 100% coverage; the writing guides bind prose.

## Goals / Non-Goals

**Goals**
- One reusable, allocation-light integration operator, generic over any module-valued state, with swappable accuracy (`Euler` ↔ `Rk4`) that needs no change to the model.
- Composite-Simpson `quadrature` over `Real` that composes with `Dual` to give differentiate-under-the-integral for free.
- Signatures shaped so `causal-arrow-application` can drop the operator into the Kuramoto / heat-diffusion / position-Euler sites and time-march the fluid RHS kernels.

**Non-Goals (deferred or out)**
- Adaptive / error-controlled, implicit / stiff (BDF), multistep (Adams), symplectic integrators.
- PDE spatial operators on a mesh — those stay in the topology exterior calculus.
- Lifting the operator into `haft`'s `Arrow` — deferred to keep `num` dependency-free.
- Forward sensitivity *through* the solver (state carried as `Dual`) — noted in D6, deferred.
- Rewriting examples — `causal-arrow-application`.

## Decisions

### D1 — An operator over functions, not a number type (the asymmetry is fundamental)

Differentiation is a ring homomorphism: `D(f∘g) = (f'∘g)·g'`, `D(fg) = f'g + fg'`. That locality is why `Dual` — one extra slot carried through arithmetic — works. Integration has neither property. `∫ₐᵇf` is a non-local functional (it depends on `f` across the whole interval, not at a point); there is no chain rule for `∫`; integration by parts is a non-terminating recurrence, not a homomorphism; and by Liouville's theorem antidifferentiation is not closed in the elementary functions. So a `Dual`-style "carry the integral" number cannot exist. The only faithful realization is an **operator that consumes a function**: a stepper for `y' = f(y)` and a quadrature rule for `∫f`. This decision is the reason the change ships a trait + structs + a free function rather than a type, and it is recorded so the two sibling changes never drift toward a false symmetry.

### D2 — Module-valued state: `S: Clone + Add<Output = S> + Mul<R, Output = S>`

Classical ODE integration assumes the state and its rate live in the same vector space (`y' = f(y)`, both in the module). The bound is exactly the `Module<R>` structure `deep_causality_num` already defines — addition plus scalar multiplication by `R: RealField`. It is satisfied by `f64`, `Complex`, `Dual`, `CausalTensor`, and `CausalMultiVector`, so the same operator marches a scalar oscillator, a multivector orientation, or a tensor field with no special-casing. States given as bare `[R; n]` or `Vec<R>` (no `Add`) are wrapped in the tensor/vector types the examples already use.

### D3 — Trait + stepper structs; accuracy is a type swap

```rust
pub trait Integrator {
    fn step<S, R, F>(&self, s: &S, dt: R, f: &F) -> S
    where S: Clone + Add<Output = S> + Mul<R, Output = S>,
          R: RealField, F: Fn(&S) -> S;

    fn integrate<S, R, F>(&self, s0: S, dt: R, steps: usize, f: &F) -> S
    where /* same bounds */ { /* fold step over 0..steps */ }
}

pub struct Euler;  // s + f(s)·dt
pub struct Rk4;    // s + (k1 + 2k2 + 2k3 + k4)·(dt/6)
```

`Euler` is `s + f(s)·dt`. `Rk4` is the classical four-stage combination, using only `Add`, scalar `Mul`, and scalar constants (`dt/2`, `dt/6` formed via `R: From<f64>`/`RealField`). Swapping `Euler` for `Rk4` changes accuracy without touching the rate field `f` — the property the duplicated example loops cannot offer today.

### D4 — Quadrature: composite Simpson over `Real`, composing with `Dual`

`quadrature(f, a, b, n)` is composite Simpson over `n` panels, bounded on `R: Real` (no division of duals is needed beyond what `Real + Div` already gives the panel weights; the helper carries `R: Real + Div<Output = R>`). Simpson is chosen for being deterministic, exact through cubics, and free of external tables (Gauss–Legendre nodes / tanh-sinh are deferred). Because the rule is generic over `Real` and `Dual<R>` is `Real`, running it over `Dual` realizes the **Leibniz bridge**: seed the parameter `θ` as `Dual::variable`, and one sweep returns `∫f(x,θ)dx` in the real part and `d/dθ∫f(x,θ)dx = ∫∂f/∂θ dx` in the infinitesimal part. No new code couples the two changes — the composition falls out of `Dual: Real`.

### D5 — Home is `deep_causality_num`; the `Arrow` lift is deferred

The operator is numeric and generic over `num`'s own algebra tower, so it lives in `num` with no new dependency. Conceptually it is a Layer-2 morphism and could implement `haft`'s `Arrow` (`In = S`, `Out = S` for one step), but realizing that would make `num` depend on `haft`. The lift is therefore deferred; the note is recorded so a later change can add it additively if a consumer wants arrows of integrators.

### D6 — Forward sensitivity through the solver is a follow-on

Seeding a *parameter* (a viscosity, an initial condition) as `Dual` and pushing it through the stepper yields the trajectory's sensitivity to that parameter, because the stepper is built only from `Add` and scalar `Mul`. This needs the state's scalar to be `Dual` (`S = CausalTensor<Dual<f64>>` and so on), which is a larger ergonomic step than the core deliverable. The core change ships the plain operator plus the self-contained quadrature/Leibniz bridge; through-solver sensitivity is flagged for a later stage.

### D7 — Relationship to the three existing integrators

`geodesic_integrator_kernel` (RK4), the gauge-lattice flow (Euler/RK3), and Stokes `integrate` are left untouched. This change provides the shared operator they should eventually delegate to; consolidating them is explicitly a later, separate change so this one stays additive and low-risk.

## Risks / Trade-offs

- **Fixed-step only.** Euler/RK4 without step control can diverge on stiff systems; acceptable because the target sites are non-stiff demonstrations and the spec states the limitation. Adaptive/implicit methods are a named follow-on.
- **State must be a module.** Examples on bare slices wrap in the tensor/vector types they already use; documented in D2.

## Migration / Rollout

Purely additive; nothing downstream changes until `causal-arrow-application` opts in.
