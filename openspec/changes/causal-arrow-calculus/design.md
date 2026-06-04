## Context

The Causal Arrow program already has, in `deep_causality_haft`: the value-level `Arrow` algebra (`Id`/`Compose`/`First`/`Second`/`Split`/`Fanout` + builder, from `causal-arrow-strength`), the witness-level `Morphism`, and `Endomorphism` (the monoid of `T → T` arrows with `iterate_n`/`iterate_to_fixpoint`/`iterate_until`). `deep_causality_num` has the `Dual` number (`Dual<T: Real>`, nests as `Dual<Dual<_>>`).

This change unifies forward-mode differentiation and numeric integration as **operators in that algebra**, replacing the off-architecture free-function surfaces the two prior changes put in `num`. The categorical reading is exact:

- **Differentiation is the tangent functor `T`.** Object map `T(A) = A[ε]/(ε²) = Dual<A>`; morphism map `T(f) = ` run `f` over duals = forward-mode AD.
- **Integration (evolution) is endomorphism iteration.** A stepper makes a self-map `S → S`; marching = `iterate_n`, steady state = `iterate_to_fixpoint`, event = `iterate_until`.
- **Leibniz is naturality**: `T(∫f) = ∫(Tf)`.

Constraints (`AGENTS.md`): `unsafe_code = "forbid"`; static dispatch, no `dyn`; no external crates; no macros in `src/`; one concern per module; tests mirror `src/`; 100% coverage incl. error paths; the writing guides bind prose.

## Goals / Non-Goals

**Goals**
- One Arrow-native operator surface in `haft`: differentiation (tangent functor), integration (endo-arrows + the `Endomorphism` combinators), quadrature (fold), composing with `arrow-strength`.
- `num` retains only the `Dual` number, plus the precision-safe `FromPrimitive` constant lift.
- The user writes a model once, generic over a `Scalar`, and applies operators — `Dual`, ε, seeding, stepper coefficients and loops never visible.
- Precision is a free parameter (f32/f64/Float106), duals nest for higher derivatives.

**Non-Goals (deferred / out)**
- Recasting SURD/BRCD as arrows (`causal-arrow-cdl-unification`).
- Adaptive / implicit / stiff / multistep / symplectic integrators.
- The example rewrites and the avionics CFD example (`causal-arrow-application`).
- Discrete-mesh spatial derivatives (stay with the topology exterior-calculus operators).

## Decisions

### D1 — The encoding was compiled before being specified

Two candidate encodings, both prototyped against the real `deep_causality_num::Dual`:

- **Rejected — lift a concrete value-level `Arrow` over `Dual`.** A concrete `Arrow<In = f64, Out = f64>`'s `run` only accepts `f64`; there is no way to feed it a `Dual`. The prototype fails to compile with `error[E0308]: expected f64, found Dual<f64>`. A concrete arrow simply cannot be differentiated.
- **Chosen — `DifferentiableArrow` with a scalar-generic `run`.** The scalar-polymorphism lives in the trait method:

  ```rust
  pub trait Scalar: Real + Div<Output = Self> + FromPrimitive {}
  impl<T: Real + Div<Output = T> + FromPrimitive> Scalar for T {}

  pub trait DifferentiableArrow {
      fn run<S: Scalar>(&self, x: S) -> S;          // (and an N-array form for fields)
  }
  ```

  The tangent functor instantiates `S = Dual<R>`:

  ```rust
  fn derivative<R: Scalar, A: DifferentiableArrow>(a: &A, x: R) -> R {
      a.run(Dual::<R>::variable(x)).derivative()
  }
  ```

  The prototype verified: scalar derivative, `value`, **second derivative from the same model** (`a.run` at `Dual<Dual<R>>`), a parameterised model (constants lifted via the scalar), and gradient — all numerically exact.

**Coexistence with the value-level `Arrow` (also compiled).** A model implements both `DifferentiableArrow` *and* a concrete `Arrow<In=f64, Out=f64>` (value view); `Diff<A>` is a concrete `Arrow<In=Dual<f64>, Out=Dual<f64>>` (the derivative-arrow view). Both are ordinary arrows, so they drop into `Compose`/`Split`/`Fanout`. The functor extends the strength algebra; it does not replace it.

### D2 — Precision is a parameter; the constant lift is `FromPrimitive`, not `From<f64>`

A model lifts its literal constants (`0.5`, `g`, …) into the working scalar. `From<f64>` is the wrong tool: **`f32` does not implement `From<f64>`**, so bounding on it silently excludes `f32` and breaks precision-as-a-parameter. `FromPrimitive::from_f64(c) -> Option<Self>` is implemented by `f32`, `f64`, and `Float106`. The design therefore:

- adds a blanket **`impl<T: Real + Div<Output=T> + FromPrimitive> FromPrimitive for Dual<T>`** in `num` (each method forwards to `T` and wraps with `Dual::constant`), so the lift nests through `Dual<Dual<…>>`;
- sets `Scalar = Real + Div + FromPrimitive`.

Verified by prototype: the *same* model runs at f32, f64, and Float106, first and second derivative, all within precision-appropriate tolerance. (The earlier `From<f64> for Dual` stays for the f64-only `solve_gm` kernel path; it is not the generic surface's bound.)

### D3 — Integration is the value-level realization of `Endomorphism`

The existing `Endomorphism` combinators are witness-level over `Morphism`, where `P::Type<T,T>` is a `fn`-pointer (for `FnMorphism`). A stepper that captures `dt` and a rate field is **not** a bare `fn` pointer, so it cannot be a `FnMorphism` witness — the same obstruction that pushed `arrow-strength` to realize composition at the value level. So:

- `Euler` / `Rk4` construct a value-level **endo-arrow**: a concrete `Arrow<In = S, Out = S>` carrying `dt` and the rate field, advancing the state one step.
- The change adds value-level **`iterate_n` / `iterate_to_fixpoint` / `iterate_until`** on endo-arrows (an `Arrow` whose `In = Out`), mirroring the witness-level `Endomorphism` monoid (which stays for the HKT world). `iterate_to_fixpoint` needs `S: PartialEq + Clone`; `iterate_until` takes a predicate. These are the three integration modes: fixed horizon, steady state, event.

The state `S` must be a module (`Add` + scalar `Mul`) for the stepper arithmetic — satisfied by `f64`, `Dual`, `CausalTensor`, `CausalMultiVector` — so the same endo-arrow marches a scalar, a multivector orientation, or a tensor field.

### D4 — Quadrature is a fold-arrow; Leibniz is its naturality

`quadrature` is a fold over a sampled closed-form integrand, generic over `Scalar`. Because `Dual: Scalar`, running it over `Dual` returns the integral in the real part and its parameter derivative in the ε part — the naturality square `T(∫f) = ∫(Tf)`. This is a verified **law**, not a demo: `quadrature(Diff(integrand))` equals `Diff(quadrature(integrand))`.

### D5 — Home is `haft`; `num` keeps the number

The operators are categorical structure (a functor on the `Arrow` category; iteration of the `Endomorphism` monoid; a fold), so they live with the Arrow machinery in `haft`. `haft` gains a `deep_causality_num` dependency for `Dual` (acyclic: `num` has no `haft` dependency). `num` retains `Dual` + the `FromPrimitive`/`From<f64>` conversions and nothing else AD/integration-related. The `num` `autodiff` and `autointegration` modules are removed; their logic is relocated and re-expressed.

### D6 — The user-facing shape (worked avionics example)

Touchdown sink-rate sensitivity — propagate a descent to the ground (integration) and report how sensitive the impact speed is to a drag/mass uncertainty (differentiation through the solver). The model is written **once**, generic over `Scalar`:

```rust
fn descent_rate<S: Scalar>(p: &Descent<S>, s: &State<S>) -> State<S> {
    let drag = S::from_f64(0.5).unwrap() * p.rho * p.cd * p.area * s.v * s.v * s.v.signum();
    State { h: s.v, v: -p.g - drag / p.m }                 // ḣ = v, v̇ = −g − drag/m
}

// integration: endo-arrow + Endomorphism::iterate_until — no loop, no ε, no Dual:
fn impact_speed<S: Scalar>(cd: S) -> S {
    let p = Descent { cd, ..Descent::base() };
    let step = Rk4.endo(dt, move |s: &State<S>| descent_rate(&p, s));   // Arrow<State,State>
    step.iterate_until(State::start(), |s| s.h <= S::zero(), MAX).0.v.abs()
}

let touchdown   = impact_speed(0.9_f64);              // run the descent
let d_impact_dcd = derivative(&ImpactSpeed, 0.9_f64); // T lifts the WHOLE solver over Dual
```

`derivative` instantiates the entire `impact_speed` pipeline at `Dual`, so the integrator marches `State<Dual<f64>>` and the ε channel is the exact `∂(impact speed)/∂cd` **through the solver** — the one quantity the current manual finite-difference re-run can only approximate. Every line maps to existing machinery: `Rk4.endo` → endo-arrow; `iterate_until` → `Endomorphism`; `derivative` → tangent functor `T`; and the whole thing is an `Arrow`, so it drops into a `PropagatingProcess` stage. `Dual`, ε, seeding, RK4 coefficients, and the loop are all hidden.

## Risks / Trade-offs

- **Models must be named types, not closures.** `DifferentiableArrow::run<S>` is a generic method; closures cannot carry a generic call signature, so a differentiable model is a (usually zero-sized) struct. Acceptable — models are named — and it is the irreducible cost of the only encoding that compiles (D1).
- **`haft` → `num` dependency** is new but acyclic; it is the correct direction (operators depend on the number).
- **Through-solver AD** (state carried as `Dual`) needs the state's module ops to hold over `Dual` — they do (`State<Dual>` is still `Add` + scalar `Mul`).

## Migration / Rollout

Additive in `haft`; relocating in `num`. `causal-arrow-application` retargets to `arrow-calculus`. `causal-arrow-autointegration` is closed (superseded); the archived `forward-autodiff` loses its free-function requirements (relocated) and keeps the kernels-accept-dual requirement. Owner commits.
