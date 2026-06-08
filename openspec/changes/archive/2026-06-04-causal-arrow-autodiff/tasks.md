## 1. Autodiff surface in `deep_causality_num`

- [x] 1.1 Add an `autodiff` module under `src/` (one concern per file, per crate convention), re-exported from `lib.rs`. No new type, no `unsafe`, no macros.
- [x] 1.2 `derivative(f, x)` and `value_and_derivative(f, x)` over `R: Real + Div<Output = R>`, `F: Fn(Dual<R>) -> Dual<R>` — seed `Dual::variable(x)`, read `value`/`derivative`.
- [x] 1.3 `second_derivative(f, x)` via `Dual<Dual<R>>` nesting (`f` over `Dual::variable(Dual::variable(x))`, read `.derivative().derivative()`).
- [x] 1.4 `gradient::<N>(f, &[R; N])` — for each coordinate `i`, build the seed array (coordinate `i` as `Dual::variable`, the rest `Dual::constant`), run `f`, collect `.derivative()` into `[R; N]`.
- [x] 1.5 `directional_derivative::<N>(f, &x, &dir)` — single pass seeding coordinate `i` as `Dual::new(x[i], dir[i])`, return `.derivative()`.
- [x] 1.6 `jacobian::<N, M>(f, &[R; N])` for `f: Fn(&[Dual<R>; N]) -> [Dual<R>; M]` → `[[R; N]; M]` (row `k` = gradient of output `k`; `jac[k][i] = ∂f_k/∂x_i`). Signature corrected from the proposal's `[[R; M]; N]` to the output-major convention in the spec scenario.

## 2. Bound relaxation (worked instance)

- [x] 2.1 `gitnexus_impact` on `solve_gm_analytical_kernel`: MEDIUM risk, 14 direct callers (1 wrapper + 13 tests) + 1 example, all `f64`; a bound widening breaks none of them.
- [x] 2.2 Relaxed `solve_gm_analytical_kernel` and `inv_r_effective` from `R: RealField` to `R: Real + Div<Output = R> + From<f64> + Debug`. **Also required (discovered during apply):** the kernel's input structs are `RealField`-bound, so `SpaceTimeCoordinate<R>` and `CentralBody<R>` (and their impls) were widened to `R: Real + Div<Output = R>` (a contained, source-compatible widening in `chronometric_quantities.rs`; field-needing impls keep their extra bounds). **Plus:** added `impl<T: Real + From<f64>> From<f64> for Dual<T>` (a real literal → a *constant* dual) so `Dual` satisfies the kernel's `From<f64>` bound without contaminating the `ε` channel.
- [x] 2.3 No other physics kernel changed; only the chronometric quantity types + `solve_gm` touched. Broader bound audit stays deferred to `causal-arrow-application`.

## 3. Tests (mirror `src/`, 100% of new code)

- [x] 3.1 Scalar: polynomials, `exp`/`sin`/`ln`, chain rule, product and quotient rules; `value_and_derivative` agreement; `second_derivative` on `x⁴` and `sin`. (9 tests.)
- [x] 3.2 Multi-input: `gradient` of `‖x‖²`, a trig field, a 3-input product; `directional_derivative` equals `∇f · dir`; `jacobian` of square / tall / wide maps. (10 tests.)
- [x] 3.3 Precision parameter: `f32` and `f64` tests across the surface prove it is precision-generic. (`Float106` follows the same generic path.)
- [x] 3.4 `solve_gm`: existing 13 `f64` tests unchanged; new `Dual<f64>` test recovers `GM` in the real part and `∂GM/∂clock_drift` in the `ε` part, matching a central finite difference. `From<f64> for Dual` covered by 3 dedicated tests (constant value, `.into()`, no `ε`-contamination).

## 4. Verification

- [x] 4.1 `cargo test -p deep_causality_num` (4298 + 186 doctests) and `cargo test -p deep_causality_physics` (1432) green.
- [x] 4.2 `cargo fmt` + `cargo clippy -p deep_causality_num -p deep_causality_physics --all-targets` — 0 warnings, no `#[allow(...)]`; no `dyn` / macros / external deps introduced.
- [x] 4.3 All error paths covered: the autodiff helpers and `From<f64>` are infallible (no `Err`/panic branches); the `solve_gm` error paths (non-positive radius, insufficient radial separation) stay covered by the existing `test_error_*` tests, which still pass under the widened bounds.
- [x] 4.4 Additive check: `Dual` arithmetic untouched (only an additive `From<f64>` conversion); `solve_gm` + chronometric-struct changes are bound widenings only (existing `f64` callers unaffected). Commit message prepared; owner commits.
