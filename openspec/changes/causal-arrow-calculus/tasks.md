## 1. `deep_causality_num` — keep the number, add the precision-safe lift, remove the operators

- [x] 1.1 Add `dual/dual_number/from_primitive.rs`: blanket `impl<T: Real + FromPrimitive> FromPrimitive for Dual<T>` (each of the 14 methods forwards to `T::from_*` and maps with `Dual::constant`; nests through `Dual<Dual<…>>`). Registered in `dual/dual_number/mod.rs`. Tests: `f32`/`f64`/`Float106` lift, nested `Dual<Dual<f32>>`, ε-channel zero, integer primitives.
- [x] 1.2 Remove the `autodiff` module (`src/autodiff/`, the `lib.rs` re-exports, `tests/autodiff/`, the `tests/mod.rs` + `BUILD.bazel` entries) — relocated.
- [x] 1.3 Remove the `autointegration` module likewise — relocated.
- [x] 1.4 Retain `Dual`, its `From<f64>` (the f64-only `solve_gm` path), and the `solve_gm` / chronometric-struct widening. `cargo test -p deep_causality_num` (4284 + 180 doctests) and `-p deep_causality_physics` (incl. the `Dual<f64>` solve_gm sensitivity) green.

## 2. New crate `deep_causality_calculus` (depends on `haft` + `num`)

- [x] 2.1 Create the crate (`Cargo.toml` deps `deep_causality_haft` + `deep_causality_num`, `[lints] workspace = true`); register in the workspace `members`. `traits/` · `types/` · `extensions/` · `ops/` layout.
- [x] 2.2 `Scalar = Real + Div<Output = Self> + FromPrimitive` (blanket marker) in `traits/`.
- [x] 2.3 Differentiation: `DifferentiableArrow` / `DifferentiableField` (`traits/`); `Diff<A, R>` implementing `Arrow<In = Dual<R>, Out = Dual<R>>` (`types/`); `DifferentiateExt::{derivative, value_and_derivative, second_derivative}` + `DifferentiateFieldExt::{gradient, directional_derivative}` blanket extension traits (`extensions/`). Verified a model is also usable as a concrete `Arrow<In = f64, Out = f64>`.
- [x] 2.4 Integration: `Euler` / `Rk4` endo-arrows `Arrow<In = S, Out = S>` over module-valued `S` (`types/`); `EndoArrow` extension trait with `iterate_n` / `iterate_to_fixpoint` (`S: PartialEq + Clone`) / `iterate_until` (`extensions/`).
- [x] 2.5 Quadrature: composite-Simpson `quadrature` fold (`ops/`), a free function generic over `Scalar`; runs over `Dual` for Leibniz naturality.

## 3. Tests (mirror `src/`, 100% incl. error paths)

- [x] 3.1 `types/`: `diff_tests` (model-as-arrow, derivative arrow, composition with strength), `euler_tests`, `rk4_tests` (order, energy conservation).
- [x] 3.2 `extensions/`: `differentiate_ext_tests` (chain/product/quotient, value-and-derivative, second derivative, gradient/directional, **precision f32/f64/Float106 + nesting**, and the avionics descent with sensitivity *through the solver* vs finite difference); `endo_arrow_ext_tests` (fixpoint converged/bound-hit, until event/bound-hit/initial-true).
- [x] 3.3 `ops/`: `quadrature_tests` (exact cubic, odd/min panels, convergent sine, precision f32, Leibniz naturality over `Dual`).
- [x] 3.4 30 tests pass; the test tree mirrors `src` (`types/` · `extensions/` · `ops/`); error/`false`-convergence/bound-hit branches exercised. (`traits/` markers have no behaviour — trait tests optional per AGENTS.)

## 4. Bookkeeping & cross-repo registration

- [x] 4.1 Close `causal-arrow-autointegration` (archived as superseded; its `numeric-integration` delta not synced). The `forward-autodiff` REMOVED delta in this change drops the two relocated free-function requirements on archive.
- [x] 4.2 Bazel: `deep_causality_calculus/BUILD.bazel` (`rust_library` + `rust_doc` + `rust_doc_test`, deps `//deep_causality_haft` + `//deep_causality_num`) and `tests/BUILD.bazel` (`types` / `extensions` / `ops` suites). Scripts: `build/scripts/{format,check,sbom}.sh` list the new crate.
- [x] 4.3 Retarget `causal-arrow-application` from (`forward-autodiff` + `numeric-integration`) to `arrow-calculus` (proposal / design / spec / tasks updated to the `deep_causality_calculus` operators and the `DifferentiateExt` / `EndoArrow` API; validates `--strict`).
- [x] 4.4 Update `openspec/notes/arrow/roadmap.md`: the analytic operators are one Arrow-native stage in the new `deep_causality_calculus` crate (not `haft`).

## 5. Verification

- [x] 5.1 `cargo test -p deep_causality_num -p deep_causality_calculus -p deep_causality_physics` green; `cargo build --workspace` green; no dangling references to the removed `num` symbols.
- [x] 5.2 `cargo fmt` + `cargo clippy -p deep_causality_calculus --all-targets` — 0 warnings, no `#[allow(...)]`; no `dyn` / macros; the only new dependency edge is `calculus → {haft, num}` (acyclic).
- [x] 5.3 Final `make format && make fix` sweep — clean across the workspace; `deep_causality_calculus` (30) + `deep_causality_num` (4284 + 180 doctests) green afterward. Commit message prepared; owner commits.
