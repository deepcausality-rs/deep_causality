## 1. `Morphism` trait in `deep_causality_haft`

- [x] 1.1 Added `src/traits/morphism.rs`: `pub trait Morphism<P: HKT2Unbound>` with `identity<A>() -> P::Type<A, A>` and `apply<A, B>(arrow: &P::Type<A, B>, input: A) -> B`, type params bounded by `Satisfies<P::Constraint>`. Documents why composition is not a method here (no-`dyn` carrier limitation; deferred to `causal-arrow-strength`).
- [x] 1.2 Added the function-pointer witness `FnMorphism` (`Type<A, B> = fn(A) -> B`) and `impl Morphism<FnMorphism> for FnMorphism` (`identity` = a real `fn(A) -> A`, `apply` = call the pointer). Static dispatch only, no `dyn`.
- [x] 1.3 Registered `pub mod morphism;` in `src/traits/mod.rs` and re-exported `Morphism` + `FnMorphism` from `src/lib.rs`.
- [x] 1.4 Tests `tests/algebra/morphism_tests.rs` (the crate's trait tests live under `tests/algebra/`, not `tests/traits/`): identity returns input unchanged, generic over the type; `apply(&f, x) == f(x)`; application changes type. Registered in `tests/algebra/mod.rs` (Bazel uses `glob`, so no manual `BUILD.bazel` edit).

## 2. `Endomorphism` trait + iteration/fixpoint combinators in `deep_causality_haft`

- [x] 2.1 Added `src/traits/endomorphism.rs`: `pub trait Endomorphism<P: HKT2Unbound>: Morphism<P>` with a blanket impl over any `Morphism` witness. Hosts `iterate_n`, `iterate_to_fixpoint` (`T: Clone + PartialEq`, explicit `max_steps`, returns `(T, bool)`), `iterate_until` (`Pred: FnMut(&T) -> bool`, explicit `max_steps`, returns `(T, bool)`), implemented by repeated `Morphism::apply`. Documents `End(T)` as a monoid under composition (composition deferred to the strength stage).
- [x] 2.2 Registered `pub mod endomorphism;` in `src/traits/mod.rs` and re-exported `Endomorphism` from `src/lib.rs`.
- [x] 2.3 Tests `tests/algebra/endomorphism_tests.rs`: `iterate_n` exact `n` times (and `n = 0` identity); `iterate_to_fixpoint` reaches and reports a real fixpoint (idempotent on the result) and reports non-convergence at the step bound; `iterate_until` returns the first value meeting the predicate (and the predicate-true-initially and step-bound cases).

## 3. `Dual<T>` type in `deep_causality_num`

> Prerequisite (landed): `num-real-trait` (the `Real` trait + `RealField: Real + Field` + the Float-blanket tower).

- [x] 3.1 Folder module `src/dual/dual_number/` mirroring `src/complex/complex_number/`. `mod.rs`: `pub struct Dual<T: Real> { re, du }`; constructors `new`/`constant`/`variable`; accessors `value`/`derivative`; `#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]`. **No `Dual32`/`Dual64` aliases** — `Dual` stays generic over the precision parameter (concrete aliases would defeat the precision-as-a-parameter design).
- [x] 3.2 `arithmetic.rs`: `Add`/`Sub`/`Mul`/`Neg`/`Div` + `AddAssign`/`SubAssign`/`MulAssign` + scalar `Mul<T>`/`MulAssign<T>` + `Sum`/`Product`, by the dual rules. **`Div` is implemented but NOT `DivAssign`**: that keeps `Dual` out of `InvMonoid`/`Field` (their blankets require both), so the zero-divisor `ε` cannot be falsely treated as invertible.
- [x] 3.3 `identity.rs`: `Zero`/`One`.
- [x] 3.4 `algebra.rs`: markers `Associative`/`Commutative`/`Distributive` + `AbelianGroup`. `Ring`/`AssociativeRing`/`CommutativeRing`/`Module<T>` come from the existing blanket impls (no explicit impl needed). **Not** `Field`/`RealField` (documented zero-divisor reason).
- [x] 3.5 `real.rs`: `impl<T: Real + Div<Output = T>> Real for Dual<T>` — the complete `Real` analytic surface, each method propagating its closed-form derivative through the `ε` channel (non-smooth ops propagate a zero `ε`). The `Div<Output = T>` bound is needed for the derivative quotients and is satisfied by `f64` **and** by `Dual` itself, so duals nest. Not `Field`/`RealField`.
- [x] 3.6 `display.rs` (`"a + bε"`). No `cast.rs` — the inherent constructors cover construction and a numeric `From`/cast surface is not part of this stage.
- [x] 3.7 Registered `mod dual;` and `pub use crate::dual::dual_number::Dual;` in `src/lib.rs`.

## 4. `Dual<T>` tests in `deep_causality_num`

- [x] 4.1 `tests/dual/dual_number/` mirroring the source (`dual_number_tests`, `arithmetic_tests`, `real_tests`, `display_tests`); registered through `tests/dual/mod.rs` + `tests/mod.rs` (Bazel `glob`).
- [x] 4.2 Constructors/accessors: `variable(x0)` has `value == x0`, `derivative == 1`; `constant(c)` has `derivative == 0`.
- [x] 4.3 AD correctness: `x³ + 2x` gives `value == x0³ + 2x0`, `derivative == 3x0² + 2`; product/quotient rules; chain rule through `sin(x)·exp(x)`; a sweep across the elementary functions (each derivative vs. closed form), `Sum`/`Product`.
- [x] 4.4 Ring laws: `d + zero == d`, `d · one == d`, `d · zero == zero`; `ε² == 0` (nilpotency); `Div` quotient rule.
- [x] 4.5 `Real` typing: `Dual<f64>`, `Dual<f32>`, and nested `Dual<Dual<f64>>` are all accepted by a `Real`-bounded helper; nested `Dual<Dual<f64>>` recovers a second derivative (`x⁴` → `f' = 32`, `f'' = 48` at `x = 2`). The `Field`/`RealField` rejection holds by construction (no such impl exists); a `trybuild` compile-fail test is deferred to avoid adding an external dev-dependency.

## 5. Verification and housekeeping

- [x] 5.1 `cargo test -p deep_causality_haft` — 163 + 11 new pass; `cargo test -p deep_causality_num` — 4276 + 180 doctests pass, 0 failed.
- [x] 5.2 `cargo fmt` + `cargo clippy -p deep_causality_haft -p deep_causality_num --all-targets` — 0 warnings/errors, no `#[allow(...)]`.
- [x] 5.3 Purely additive: only `mod`/`pub use`/test-registration lines changed in existing files; no existing public signature or behavior changed.
- [x] 5.4 New doc comments follow the writing guides. Commit message prepared; not committed (owner commits).
