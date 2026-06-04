## 1. `deep_causality_num` — keep the number, add the precision-safe lift, remove the operators

- [ ] 1.1 Add `dual/dual_number/from_primitive.rs`: blanket `impl<T: Real + Div<Output = T> + FromPrimitive> FromPrimitive for Dual<T>` — each of the 14 methods forwards to `T::from_*` and maps with `Dual::constant` (nests through `Dual<Dual<…>>`). Register in `dual/dual_number/mod.rs`. Test: `Dual::<f32>::from_f64`, nested `Dual<Dual<f32>>`, ε-channel is zero.
- [ ] 1.2 Remove the `autodiff` module (`src/autodiff/`, its `lib.rs` re-exports, `tests/autodiff/`, the `tests/mod.rs` + `BUILD.bazel` entries) — relocated to haft. (Owner-approval for file deletion.)
- [ ] 1.3 Remove the `autointegration` module likewise — relocated to haft.
- [ ] 1.4 Retain `Dual`, its `From<f64>` (the f64-only `solve_gm` path), and the `solve_gm` / chronometric-struct widening. `cargo test -p deep_causality_num` + `-p deep_causality_physics` stay green.

## 2. `deep_causality_haft` — the calculus surface (new `deep_causality_num` dep)

- [ ] 2.1 Add `deep_causality_num` to `haft/Cargo.toml` (acyclic). Add a `calculus` module tree, re-exported from `lib.rs`.
- [ ] 2.2 `Scalar = Real + Div<Output = Self> + FromPrimitive` (blanket-impl'd marker) re-exported.
- [ ] 2.3 Differentiation: `DifferentiableArrow { fn run<S: Scalar>(&self, x: S) -> S; }` and a multi-input field trait (`&[S; N] -> S`); `Diff<A>` implementing the value-level `Arrow<In = Dual<R>, Out = Dual<R>>`; desugarings `derivative`, `value_and_derivative`, `second_derivative`, `gradient`, `directional_derivative`. Verify a model is also usable as a concrete `Arrow<In = R, Out = R>`.
- [ ] 2.4 Integration: `Euler` / `Rk4` constructing a value-level endo-arrow `Arrow<In = S, Out = S>` (carrying `dt` + rate field) over module-valued `S` (`Clone + Add + Mul<R>`). Value-level `iterate_n` / `iterate_to_fixpoint` (`S: PartialEq + Clone`) / `iterate_until` on endo-arrows, mirroring the witness-level `Endomorphism`.
- [ ] 2.5 Quadrature: a fold over a closed-form integrand, generic over `Scalar` (composite Simpson interior). Runs over `Dual` for the Leibniz naturality.

## 3. Tests (mirror `src/`, 100% incl. error paths)

- [ ] 3.1 Tangent functor: scalar derivative (chain/product/quotient), `value_and_derivative`, `second_derivative` via nesting; `gradient`/`directional_derivative` on known fields.
- [ ] 3.2 Precision-as-a-parameter: the same model differentiated at `f32`, `f64`, `Float106`, first and second derivative; constants lifted via the `FromPrimitive` blanket; ε-of-constant is zero.
- [ ] 3.3 Arrow coexistence: a model as `Arrow<f64,f64>` and `Diff<model>` as `Arrow<Dual,Dual>`, both composing via `Compose`/`Split`.
- [ ] 3.4 Integration: Euler `O(dt)` / RK4 `O(dt⁴)` order; `iterate_to_fixpoint` (converged vs bound-hit); `iterate_until` (event vs bound-hit); harmonic-oscillator energy conservation; accuracy is a `Euler→Rk4` swap; non-scalar module state.
- [ ] 3.5 Quadrature: exact on a cubic; convergent on a transcendental; the **Leibniz naturality law** (`quadrature` over `Dual` = integral + parameter derivative, matching analytic).
- [ ] 3.6 The avionics descent (design D6) as an end-to-end test: `iterate_until` to touchdown, `derivative` through the solver for `∂(impact speed)/∂cd` vs a finite-difference reference.
- [ ] 3.7 Error paths: every `Err` / `false`-convergence / bound-hit / validation branch exercised.

## 4. Bookkeeping: supersede the relocated surfaces

- [ ] 4.1 Close `causal-arrow-autointegration` (superseded; its `num` module is relocated here). The `forward-autodiff` REMOVED delta in this change drops the two relocated free-function requirements on archive.
- [ ] 4.2 Retarget `causal-arrow-application` from (`forward-autodiff` + `numeric-integration`) to `arrow-calculus`; its example rewrites apply these operators to scalar-generic models (kept as a separate change).
- [ ] 4.3 Update `openspec/notes/arrow/roadmap.md`: the analytic operators are one Arrow-native stage in `haft` (tangent functor + endomorphism integration + fold), not a `num` surface.

## 5. Verification

- [ ] 5.1 `cargo test -p deep_causality_num -p deep_causality_haft -p deep_causality_physics` green; doctests pass.
- [ ] 5.2 `make format && make fix` (3 crates touched) — 0 clippy warnings, no `#[allow(...)]`; no `dyn` / macros / new external deps (only the internal `haft → num` path).
- [ ] 5.3 The `haft → num` dependency is acyclic; `Dual` is never named in any example/user-facing model. Commit message prepared; owner commits.
