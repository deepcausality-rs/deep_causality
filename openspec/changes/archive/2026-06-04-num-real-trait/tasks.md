## 1. Add the `Real` trait

- [x] 1.1 Add `deep_causality_num/src/algebra/real.rs`: `pub trait Real: CommutativeRing + PartialOrd + Neg<Output = Self> + Copy + Clone + AddAssign + SubAssign + MulAssign` (no `Div`/`DivAssign`/`InvMonoid`/`Field`). Declare the division-independent analytic surface (constants, elementary functions, sign/rounding/shape, exceptional-value predicates) as listed in the `real-scalar` spec. Document the analytic-vs-field decoupling and the intended dual-number consumer.
- [x] 1.2 Register `pub use crate::algebra::real::Real;` in `src/lib.rs` and the module in `src/algebra/mod.rs`.

## 2. Refactor `RealField` to `Real + Field`

- [x] 2.1 In `src/algebra/field_real.rs`: change the declaration to `pub trait RealField: Real + Field`. The entire body of the old `RealField` was analytic (the field operations live on `DivisionAlgebra`/`Field`, not on `RealField`), so all 29 method declarations move to `Real` and `RealField` becomes an empty `Real + Field` marker. `conjugate`/`norm_sqr`/`inverse` stay on `DivisionAlgebra` (unchanged).
- [x] 2.2 Relocate the analytic method **bodies** for `f32` and `f64` into `impl Real` blocks **verbatim** (no rewrites); the `impl RealField for f32/f64` blocks become empty markers. (The `impl Real for f32/f64` and `impl DivisionAlgebra for f32/f64` blocks were subsequently moved into `algebra/real.rs` and `algebra/algebra_div.rs` respectively, matching the crate's one-impl-per-module layout.)
- [x] 2.3 In `src/float_106/traits_algebra.rs`: same relocation for `Float106` (`impl Real for Float106` holds the analytic bodies; `impl RealField for Float106 {}` is the marker).

## 3. Tests

- [x] 3.1 Add `tests/algebra/real_tests.rs`; registered in `tests/algebra/mod.rs`.
- [x] 3.2 Retarget the analytic-surface tests to `Real` (the methods' new home): `field_real_f32_tests`, `field_real_f64_tests`, and the `float_double` suite (`double_algebra`/`double_traits`/`double_transcendental`) now call `Real::…` / `<T as Real>::…`. Results are bit-identical (bodies relocated verbatim). The `assert_real_field<T: RealField>` bound test in `double_traits_tests` is preserved.
- [x] 3.3 `real_tests.rs` adds the relationship coverage: a `Real`-bounded generic accepts any `RealField` value (RealField ⇒ Real); `f32`/`f64`/`Float106` are all `Real` and `RealField`; the analytic surface resolves under both bounds.

## 4. Verification (behavior-preserving across the workspace)

- [x] 4.1 `cargo test -p deep_causality_num` — 4241 integration + 177 lib/doctests pass, 0 failed.
- [x] 4.2 `cargo build --workspace --all-targets` — 0 errors. Every `T: RealField`-generic consumer compiles unchanged. **Consumer note (revealed during apply):** code that calls the analytic methods via **method syntax on a concrete/inferred float** (not via a `T: RealField` bound) needs `Real` in scope, since the methods moved to the supertrait. This is a mechanical `use …Real` addition — applied to ~18 sites (mostly tests + 5 examples; src: the two `RealField::exp(...)` qualified calls in topology metropolis, plus a `multivector` projected-type `.sqrt()`). No behavior changed. `cargo test --workspace` passes (0 failed).
- [x] 4.3 `cargo fmt --all` clean; `cargo clippy --workspace --all-targets` — 0 warnings, 0 errors, no `#[allow(...)]`. Commit message prepared; not committed (owner commits).

## 5. Float-blanket cascade (`impl Float` ⇒ whole tower)

- [x] 5.1 Add `pi()` / `e()` to the `Float` trait and impl them for `f32`, `f64`, `Float106`.
- [x] 5.2 Strengthen `Float`'s supertraits with `AddAssign + SubAssign + MulAssign + DivAssign` so `T: Float` symbolically reaches `Field`/`Real`.
- [x] 5.3 Replace per-type marker impls with `impl<T: Float> Associative/Commutative/Distributive for T {}` (keep the integer impls); remove the `f32`/`f64`/`Float106` marker impls.
- [x] 5.4 `impl<T: Float> AbelianGroup for T {}` and `impl<T: Float> RealField for T {}` (field_real.rs); remove the explicit `f32`/`f64` impls.
- [x] 5.5 `impl<T: Float> DivisionAlgebra<T> for T { conjugate = self, norm_sqr = self*self, inverse = Float::recip(self) }` (algebra_div.rs); remove explicit `f32`/`f64`.
- [x] 5.6 Replace the two per-type `impl Real for f32/f64` with one `impl<T: Float> Real for T` delegating to `Float`; remove `Float106`'s explicit `Real`/`RealField`/marker/`AbelianGroup`/`DivisionAlgebra` impls (keep `Zero`/`One`/`Num`).
- [x] 5.7 Verify: `rustc` coherence probe (blanket + non-`Float` impls coexist, no `E0119`); `cargo build --workspace --all-targets` 0 errors; full `cargo test --workspace` 0 failures; `cargo clippy` 0 (applied 21 surfaced `assign_op_pattern` `+=` fixes in topology/physics, not suppressed); `cargo fmt` clean.
