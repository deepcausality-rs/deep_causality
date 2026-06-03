## 1. Add the `Real` trait

- [ ] 1.1 Add `deep_causality_num/src/algebra/real.rs`: `pub trait Real: CommutativeRing + PartialOrd + Neg<Output = Self> + Copy + Clone + AddAssign + SubAssign + MulAssign` (no `Div`/`DivAssign`/`InvMonoid`/`Field`). Declare the division-independent analytic surface (constants, elementary functions, sign/rounding/shape, exceptional-value predicates) as listed in the `real-scalar` spec. Document the analytic-vs-field decoupling and the intended dual-number consumer.
- [ ] 1.2 Register `pub use crate::algebra::real::Real;` in `src/lib.rs` and the module in `src/algebra/mod.rs`.

## 2. Refactor `RealField` to `Real + Field`

- [ ] 2.1 In `src/algebra/field_real.rs`: change the declaration to `pub trait RealField: Real + Field`. Remove the analytic method declarations now hosted on `Real`; keep only the field-specific surface (e.g. `inverse`) on `RealField`. Resolve `conjugate`/`norm_sqr` placement against their call sites (default to `Real` if division-independent).
- [ ] 2.2 Relocate the analytic method **bodies** for `f32` and `f64` from `impl RealField` into new `impl Real` blocks **verbatim** (no rewrites); reduce each `impl RealField` block to the field-specific remainder.
- [ ] 2.3 In `src/float_106/traits_algebra.rs`: do the same relocation for `Float106` (`impl Real for Float106` gets the analytic bodies; `impl RealField for Float106` keeps the field remainder).

## 3. Tests

- [ ] 3.1 Add `tests/algebra/real_tests.rs` mirroring the source; register in the tests module tree and `tests/BUILD.bazel`.
- [ ] 3.2 Move the analytic-surface tests (elementary functions, constants, rounding, predicates) from the `RealField` test file(s) into the `Real` test file(s); keep field-specific tests (`inverse`, division) on `RealField`. Assert bit-identical results for `f32`/`f64`/`Float106` vs. the pre-refactor behavior.
- [ ] 3.3 Add a compile-level test that a `T: RealField` value satisfies a `Real` bound (RealField ⇒ Real), and that the analytic and field operations both still resolve under a `T: RealField` bound. 100% coverage of new/edited code.

## 4. Verification (behavior-preserving across the workspace)

- [ ] 4.1 `cargo build -p deep_causality_num && cargo test -p deep_causality_num`.
- [ ] 4.2 Workspace build to confirm every existing `T: RealField` consumer (SURD/MRMR, CDL, BRCD, linalg, fluid-dynamics, topology, physics) compiles unchanged: `make build` (or targeted `cargo build` of the dependent crates).
- [ ] 4.3 `make format && make fix`; clippy clean with no `#[allow(...)]` suppressions. Prepare a commit message; do not commit (owner commits).
