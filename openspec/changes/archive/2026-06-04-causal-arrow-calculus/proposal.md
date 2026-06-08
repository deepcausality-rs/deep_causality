## Why

`causal-arrow-autodiff` (archived) and `causal-arrow-autointegration` (implemented) put forward-mode AD and numeric integration into `deep_causality_num` as **plain generic free functions**. That is wrong twice over.

First, `num` is about *numbers*. The differentiating *number* `Dual` belongs there — but the *act* of differentiating or integrating a *function* is a numerical method, not a number (`Rk4` and Simpson never even touch `Dual`). They have no business in a crate about numbers.

Second, and more importantly, these operations only make sense **inside the categorical mechanism already built**. Forward-mode AD is the **tangent functor** `T` whose object map is exactly `Dual` and whose morphism map is "run the arrow over duals". Integration is **endomorphism iteration** — the `Endomorphism::{iterate_n, iterate_to_fixpoint, iterate_until}` combinators shipped in foundations. The two compose by **naturality** (`T(∫) = ∫(T)`, i.e. Leibniz). As free functions bolted onto `num`, they are low-level and off-architecture, and they force the user to see `Dual`, `ε`, seeding, and a hand-rolled time loop.

This change relocates the operator surface into a **new crate `deep_causality_calculus`** that depends on both `deep_causality_haft` (for the `Arrow` / `Endomorphism` machinery) and `deep_causality_num` (for `Dual`), so both foundations stay self-contained. It expresses the operators as Arrow-native machinery, so a user writes a model **once** (generic over a scalar) and *applies operators* — never touching `Dual`. The encoding was prototyped and compiled against the real `Dual` before specifying (see design D1): the scalar-polymorphism that lets a model be lifted over `Dual` must live in a trait with a generic `run` — a concrete value-level `Arrow<In=f64>` provably **cannot** be lifted (`E0308`).

## What Changes

- **`num` keeps only the number.** `Dual` stays. Add a **`FromPrimitive` blanket for `Dual<T>`** (a primitive → a derivative-free constant dual, forwarded through every nesting level). This is the precision-safe constant lift: `f32`/`f64`/`Float106` all implement `FromPrimitive`, whereas `From<f64>` excludes `f32` and would silently break precision-as-a-parameter. Remove the `num` `autodiff` and `autointegration` modules (relocated to the new crate).
- **A new crate `deep_causality_calculus`** hosts the analytic operators. It depends on `deep_causality_haft` and `deep_causality_num`; neither foundation gains a dependency (`haft` stays zero-dependency; `num` does not depend on `haft`), so the new edge is `calculus → {haft, num}`, acyclic. It follows the repo's `traits/` · `types/` · `extensions/` structure and the `…Ext` type-extension convention:
  - **Differentiation = the tangent functor.** A `DifferentiableArrow` trait whose `run` is generic over the scalar (`fn run<S: Scalar>(&self, …) -> …`) — the only construct that can host lifting a model over `Dual`. `Diff<A>` is the derivative-arrow view (a concrete `Arrow<In = Dual<…>, Out = Dual<…>>`); the `DifferentiateExt` / `DifferentiateFieldExt` extension methods (`model.derivative(x)`, `field.gradient(&x)`) are the fluent surface. A model is simultaneously a plain `Arrow` and, via `Diff`, its derivative arrow, so it composes with the `arrow-strength` algebra.
  - **Integration = endo-arrows + the `Endomorphism` combinators.** `Euler` / `Rk4` construct a value-level **endo-arrow** `Arrow<In = S, Out = S>` from a rate field; evolution is `iterate_n` (fixed horizon), `iterate_to_fixpoint` (steady state), `iterate_until` (event). Because a capturing stepper cannot be a bare `fn`-pointer witness, these are the **value-level** realization of the `Endomorphism` monoid — exactly the move `arrow-strength` made for composition.
  - **Quadrature = a fold-arrow** over a closed-form integrand. Run over `Dual` it yields differentiate-under-the-integral — the **naturality** of `T` through the fold (Leibniz) — as a verified law.
- **Precision is a parameter.** `Scalar = Real + Div + FromPrimitive`; every operator is generic over the base precision and duals nest for higher derivatives. Verified at f32 / f64 / Float106 including `Dual<Dual<_>>`.
- **The user never sees `Dual` / ε / seeding / stepper coefficients / loops.** Models are written generic over `Scalar`; operators are applied.
- **Out of scope:** recasting SURD/BRCD as arrows (`causal-arrow-cdl-unification`); adaptive / implicit / stiff ODE solvers; the example rewrites and the avionics CFD example (those are `causal-arrow-application`, which now consumes `arrow-calculus`).
- **Supersedes** the `num`-resident surfaces: the archived `forward-autodiff` free-function requirements (relocated) and the `causal-arrow-autointegration` change (closed; its `num` module is relocated). The `Dual`, `From<f64>` (the `solve_gm` path), and the `solve_gm` / chronometric-struct widening are **retained**.

## Capabilities

### New Capabilities

- `arrow-calculus`: the analytic operators of the Causal Arrow, in the new `deep_causality_calculus` crate — differentiation as the tangent functor over a scalar-generic `DifferentiableArrow` (object map = `Dual`; `DifferentiateExt` methods), integration as `Euler`/`Rk4` endo-arrows driven by the `EndoArrow` value-level iteration combinators, and `quadrature` as a fold with verified Leibniz naturality. Precision-generic, composes with the `arrow-strength` algebra, and hides `Dual` from the user entirely.

### Removed Capabilities

- `forward-autodiff`: the scalar-derivative (`derivative` / `value_and_derivative` / `second_derivative`) and multi-input (`gradient` / `directional_derivative` / `jacobian`) **free-function** requirements are REMOVED — relocated to `arrow-calculus`. The "division-only generic kernels accept dual numbers" requirement is **retained** (it concerns `num`'s `Dual` and `solve_gm`, which stay).

## Impact

- **`deep_causality_num`:** `+` a `FromPrimitive` blanket for `Dual` (one module); `−` the `autodiff` and `autointegration` modules (relocated). `Dual`, its `From<f64>`, and the `solve_gm` / chronometric-struct widening are retained.
- **`deep_causality_calculus` (new crate):** the operator surface — `traits/` (`Scalar`, `DifferentiableArrow`, `DifferentiableField`), `types/` (`Diff`, `Euler`, `Rk4`), `extensions/` (`DifferentiateExt`, `DifferentiateFieldExt`, `EndoArrow`), and `ops/quadrature`. Depends on `deep_causality_haft` + `deep_causality_num`; registered in the workspace, Bazel `BUILD.bazel`/`tests/BUILD.bazel`, and the `format`/`check`/`sbom` scripts. `haft` and `num` are unchanged structurally (no new dependency on either).
- **Consumers:** `causal-arrow-application` switches its dependency from (`forward-autodiff` + `numeric-integration`) to `arrow-calculus`; the example rewrites and the avionics descent/CFD examples apply these operators to scalar-generic models.
- **Verification:** tangent-functor numeric correctness at f32/f64/Float106 incl. nesting; the value-level `Arrow` coexistence (`Model: Arrow<f64,f64>`, `Diff<Model>: Arrow<Dual,Dual>`); endo-arrow order (Euler `O(dt)`, RK4 `O(dt⁴)`), fixpoint and until; quadrature exactness and the Leibniz naturality law; 100% coverage including **every error path**.
