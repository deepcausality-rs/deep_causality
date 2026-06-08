## Why

The Causal Arrow program (see `openspec/notes/arrow/causal-arrow-generalization.md`) reframes DeepCausality's pipeline as one Arrow whose fragments — discover, infer, govern, act — share an interface, with the existing causal monad as its Kleisli fragment. Two foundational primitives must exist before any of that work can begin, and both are named in `openspec/notes/arrow/num-dual-endomorphism.md` as the "discrete" and "continuous" halves of how a system changes:

- `deep_causality_haft` has the categorical scaffolding (`Profunctor`, `Promonad`, `Bifunctor`, `CyberneticLoop`, `Monad`, `CoMonad`) but **no explicit typed-arrow base** and **no iteration/fixpoint primitive**. The BRCD Meek-rule loop, context propagation, and the causal-monad time step are each a hand-rolled `loop { … }` over a `T → T` transform with no shared, tested combinator and no compile-time guarantee that the transform is actually type-preserving.
- `deep_causality_num` has the full algebra tower (`Field`, `Ring`, `Module`, `Algebra`) and hypercomplex types (`Complex`, `Quaternion`, `Octonion`) but **no dual numbers**, the canonical type-based forward-mode automatic-differentiation primitive and the algebraic model of a tangent vector.

This change lands those two primitives only. It is the smallest self-justifying step of the program: it adds named, tested groundwork that later stages consume, and it does not touch any live caller. The fixpoint integration, automatic differentiation in physics/topology, and the CDL unification are **separate later changes**, sketched as a roadmap in this change's `design.md` and proposed individually when their prerequisites land.

## What Changes

- Add a `Morphism` trait to `deep_causality_haft`: the explicit typed-arrow / `Category` base (`id`, `compose`, `run`) over the crate's existing `HKT2Unbound` witness pattern. This is the base every discovery operator (SURD, BRCD, PC/GES/BOSS) will instance in later stages.
- Add an `Endomorphism` trait to `deep_causality_haft`: the marker for the type-preserving fragment (source = target), supertraited on `Morphism`, that makes "safe to iterate" a compile-time fact. It carries the iteration combinators:
  - `iterate_n(f, x, n)` — apply exactly `n` times.
  - `iterate_to_fixpoint(f, x, eq, max_steps)` — apply until `f(x) == x` (a fixpoint) or a step bound is hit.
  - `iterate_until(f, x, predicate, max_steps)` — apply until a convergence predicate holds or a step bound is hit.
- Add a `Dual<T>` type to `deep_causality_num`: dual numbers `a + b·ε` with `ε² = 0`, the ring of dual numbers over a `Real` base (`Dual<T: Real>`). Overloading `Add`/`Sub`/`Mul`/`Neg` (and `Div` where defined) makes forward-mode automatic differentiation fall out of the trait impls: evaluating any composed function on `a + 1·ε` yields `f(a) + f'(a)·ε`. `Dual<T>` itself **implements `Real`** (analytic, commutative-ring) — **not** `Field`/`RealField`, because `ε` is a zero divisor — so a dual is a first-class real scalar: nestable as `Dual<Dual<f64>>` for second derivatives and droppable into any `Real`-generic code (the basis of drop-in AD in the later autodiff stage). It mirrors the `Complex`/`Quaternion` module layout and slots into the algebra tower as a `CommutativeRing`/`Module`.
- **Prerequisite:** the `dual-numbers` capability depends on the `num-real-trait` change (the `Real` trait + `RealField: Real + Field` refactor). `Dual` binds on `Real` and implements `Real`, so `Real` must exist first. The `morphism-algebra` capability (in `haft`) has no such dependency.
- No public API of any existing crate changes; no existing caller is modified; no behavior of SURD, BRCD, or the causal monad changes. This change is **purely additive**.
- **No new external or numeric dependency.** `Morphism`/`Endomorphism` use only the existing `haft` HKT machinery; `Dual<T>` uses only the existing `num` algebra traits. Stays inside `unsafe_code = "forbid"`, static-dispatch-only, and the no-external-numerics policy.

## Capabilities

### New Capabilities
- `morphism-algebra`: the typed-arrow base (`Morphism`/`Category`: identity, composition, application) and the type-preserving `Endomorphism` fragment with its iteration and fixpoint combinators, in `deep_causality_haft`.
- `dual-numbers`: the `Dual<T: Real>` ring of dual numbers in `deep_causality_num` — the type-based forward-mode automatic-differentiation primitive and algebraic tangent vector, with `f(a + ε) = f(a) + f'(a)·ε` exact to machine precision. `Dual<T>` implements `Real` (a non-field analytic scalar). **Depends on `num-real-trait`.**

### Modified Capabilities
<!-- None. This change is purely additive: it introduces two new primitives and modifies no existing spec-level behavior. The fixpoint integration that retrofits the BRCD Meek loop onto iterate_to_fixpoint is a separate later change (causal-arrow-fixpoint) and will carry the corresponding MODIFIED requirement against brcd-algorithm. -->

## Impact

- **New code, `deep_causality_haft`:** `src/traits/morphism.rs`, `src/traits/endomorphism.rs`, registered in `src/traits/mod.rs` and re-exported from `src/lib.rs`; mirrored tests under `tests/traits/`. A concrete function-carrier witness so the combinators are usable on plain `T → T` closures without dynamic dispatch.
- **New code, `deep_causality_num`:** `src/dual/dual_number/` folder module (`mod.rs` + per-trait files `arithmetic.rs`, `ops.rs`, `ops_shared.rs`, `algebra.rs`, `identity.rs`, `real.rs` (the `impl Real for Dual`), `cast.rs`, `display.rs`) mirroring `src/complex/complex_number/`; `Dual32`/`Dual64` aliases; registered in `src/lib.rs`; mirrored tests under `tests/dual/`.
- **APIs:** two new public traits in `haft`, one new public type (plus aliases and a derivative accessor) in `num`. No existing signature changes.
- **Dependencies:** none added.
- **Consumers (later changes, not here):** `causal-arrow-fixpoint` retrofits the BRCD Meek loop, context propagation, and the monad time step onto `iterate_to_fixpoint`; `causal-arrow-autodiff` consumes `Dual<T>` for tangent vectors / directional derivatives in `deep_causality_topology` and gradients in `deep_causality_physics`, and adds reverse-mode AD; `causal-arrow-cdl-unification` builds the `PropagatingEffect`-carrier algebra. See `design.md` for the roadmap.
- **Verification:** trait-law tests (identity, associativity of composition; fixpoint idempotence and the step bound) and the AD correctness property (`f(a + ε) = f(a) + f'(a)·ε` against closed-form derivatives), to 100% coverage on the new code.
