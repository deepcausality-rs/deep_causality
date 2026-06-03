## 1. `Morphism` trait in `deep_causality_haft`

- [ ] 1.1 Add `src/traits/morphism.rs`: `pub trait Morphism<P: HKT2Unbound>` with `identity<A>() -> P::Type<A, A>` and `apply<A, B>(arrow: &P::Type<A, B>, input: A) -> B`, type params bounded by `Satisfies<P::Constraint>`. Document why composition is not a method here (no-`dyn` carrier limitation; deferred to `causal-arrow-strength`).
- [ ] 1.2 Add a concrete function-pointer witness (e.g. `FnMorphism`) with `Type<A, B> = fn(A) -> B`; implement `Morphism` for it (`identity` = a real `fn(A) -> A`, `apply` = call the pointer). Static dispatch only, no `dyn`.
- [ ] 1.3 Register `pub mod morphism;` in `src/traits/mod.rs` (and module doc entry) and re-export `Morphism` (and the witness) from `src/lib.rs`.
- [ ] 1.4 Tests `tests/traits/morphism_tests.rs`: identity returns input unchanged; `apply(&f, x) == f(x)`. Register in the tests module tree and `tests/BUILD.bazel`. 100% coverage of the new code.

## 2. `Endomorphism` trait + iteration/fixpoint combinators in `deep_causality_haft`

- [ ] 2.1 Add `src/traits/endomorphism.rs`: `pub trait Endomorphism<P: HKT2Unbound>: Morphism<P>` with a blanket impl over any `Morphism` witness. Host `iterate_n`, `iterate_to_fixpoint` (`T: Clone + PartialEq`, explicit `max_steps`, returns `(T, bool)`), and `iterate_until` (`Pred: FnMut(&T) -> bool`, explicit `max_steps`, returns `(T, bool)`). Implement by repeated `Morphism::apply`. Document `End(T)` as a monoid under composition (composition itself deferred to the strength stage).
- [ ] 2.2 Register `pub mod endomorphism;` in `src/traits/mod.rs` and re-export `Endomorphism` from `src/lib.rs`.
- [ ] 2.3 Tests `tests/traits/endomorphism_tests.rs`: `iterate_n` on `t ↦ t+1` from 0 yields `n`; `iterate_to_fixpoint` reaches and reports a real fixpoint (idempotent on the result); the step bound reports non-convergence on `t ↦ t+1`; `iterate_until` returns the first value meeting the predicate with the met flag. Register in the tests tree and `tests/BUILD.bazel`. 100% coverage.

## 3. `Dual<T>` type in `deep_causality_num`

> Prerequisite: the `num-real-trait` change (the `Real` trait + `RealField: Real + Field` refactor) must land first; `Dual` binds on and implements `Real`.

- [ ] 3.1 Create folder module `src/dual/dual_number/` mirroring `src/complex/complex_number/`. In `mod.rs`: `pub struct Dual<T: Real> { re: T, du: T }`; constructors `new`, `constant`, `variable`; accessors `value`, `deriv`; aliases `Dual32`, `Dual64`; derive/`impl` `Copy, Clone, PartialEq, Debug` as appropriate.
- [ ] 3.2 `arithmetic.rs` + `ops.rs` + `ops_shared.rs`: implement `Add`, `Sub`, `Mul`, `Neg`, and `Div` (invertible real part only) by the dual rules in the spec; document `Div` partiality.
- [ ] 3.3 `identity.rs`: `Zero`/`One` (and `ConstZero`/`ConstOne` if the sibling complex type provides them).
- [ ] 3.4 `algebra.rs`: the three property markers `Associative`, `Commutative`, `Distributive` (all satisfied), plus `AddMonoid`, `MulMonoid`, `Ring`, `AssociativeRing`, `CommutativeRing`, `Module<T>` — mirroring how `Complex`/`Quaternion` implement the markers they qualify for. Do **not** implement `Field`; document the zero-divisor reason (the three markers hold but invertibility fails).
- [ ] 3.5 `real.rs`: `impl Real for Dual<T> where T: Real` — the **complete** `Real` analytic surface (constants, `sqrt`/`exp`/`ln`/`log*`/`powf`, `sin`/`cos`/`tan`/`asin`/`acos`/`atan`/`atan2`, `sinh`/`cosh`/`tanh`, `abs`/`floor`/`ceil`/`round`/`clamp`, NaN/finiteness), each propagating its closed-form derivative through the `ε` channel (non-smooth ops propagate a zero `ε`). Do **not** impl `Field`/`RealField`.
- [ ] 3.6 `cast.rs` + `display.rs` to match the `Complex` module's surface where applicable.
- [ ] 3.7 Register `mod dual;` and `pub use crate::dual::dual_number::{Dual, Dual32, Dual64};` in `src/lib.rs`.

## 4. `Dual<T>` tests in `deep_causality_num`

- [ ] 4.1 Create `tests/dual/dual_number/` mirroring the source tree; register in the tests module tree and `tests/BUILD.bazel`.
- [ ] 4.2 Constructors/accessors: `variable(x0)` has `value == x0`, `deriv == 1`; `constant(c)` has `deriv == 0`.
- [ ] 4.3 AD correctness: `x³ + 2x` gives `value == x0³ + 2x0`, `deriv == 3x0² + 2`; product rule; chain rule through `sin(x)·exp(x)`; a representative sweep across the elementary functions (each derivative vs. closed form).
- [ ] 4.4 Ring laws: `d + zero == d`, `d · one == d`, `d · zero == zero`; `ε² == 0` (nilpotency); `Div` correctness on invertible real part. 100% coverage of the new code.
- [ ] 4.5 `Real`/`Field` typing: a `Dual<f64>` is accepted by a `Real`-bounded helper and rejected by a `Field`/`RealField`-bounded helper (compile-fail / trybuild); nested `Dual<Dual<f64>>` recovers a second derivative `f''(x0)`.

## 5. Verification and housekeeping

- [ ] 5.1 `cargo build -p deep_causality_haft && cargo test -p deep_causality_haft`; `cargo build -p deep_causality_num && cargo test -p deep_causality_num`.
- [ ] 5.2 `make format && make fix` (only `haft` + `num` changed; format/lint both); confirm clippy is clean with no `#[allow(...)]` suppressions.
- [ ] 5.3 Confirm purely additive: no existing public signature or behavior changed; `git diff` on existing files is limited to `mod`/`pub use` registration lines.
- [ ] 5.4 Prose in new doc comments follows the two writing guides. Prepare a commit message; do not commit (owner commits).
