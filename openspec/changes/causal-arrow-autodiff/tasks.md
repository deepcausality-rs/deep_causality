## 1. Autodiff surface in `deep_causality_num`

- [ ] 1.1 Add an `autodiff` module under `src/` (one concern per file, per crate convention), re-exported from `lib.rs`. No new type, no `unsafe`, no macros.
- [ ] 1.2 `derivative(f, x)` and `value_and_derivative(f, x)` over `R: Real + Div<Output = R>`, `F: Fn(Dual<R>) -> Dual<R>` — seed `Dual::variable(x)`, read `value`/`derivative`.
- [ ] 1.3 `second_derivative(f, x)` via `Dual<Dual<R>>` nesting (`f` over `Dual::variable(Dual::variable(x))`, read `.derivative().derivative()`).
- [ ] 1.4 `gradient::<N>(f, &[R; N])` — for each coordinate `i`, build the seed array (coordinate `i` as `Dual::variable`, the rest `Dual::constant`), run `f`, collect `.derivative()` into `[R; N]`.
- [ ] 1.5 `directional_derivative::<N>(f, &x, &dir)` — single pass seeding coordinate `i` as `Dual::new(x[i], dir[i])`, return `.derivative()`.
- [ ] 1.6 `jacobian::<N, M>(f, &[R; N])` for `f: Fn(&[Dual<R>; N]) -> [Dual<R>; M]` → `[[R; M]; N]` (row `i` = gradient of output `i`).

## 2. Bound relaxation (worked instance)

- [ ] 2.1 Run `gitnexus_impact({target: "solve_gm_analytical_kernel", direction: "upstream"})`; report blast radius before editing.
- [ ] 2.2 Relax `solve_gm_analytical_kernel` (and its private helper `inv_r_effective`) from `R: RealField` to `R: Real + Div<Output = R> + From<f64>`. Confirm only `+ − × ÷ .abs()` ordering / `From<f64>` are used; no `RealField`-only method remains.
- [ ] 2.3 Verify no other physics kernel is changed in this stage (broader audit deferred to `causal-arrow-application`).

## 3. Tests (mirror `src/`, 100% of new code)

- [ ] 3.1 Scalar: polynomials, `exp`/`sin`/`cos`/`ln`, chain rule, product and quotient rules; `value_and_derivative` agreement; `second_derivative` on `x⁴` and `sin`.
- [ ] 3.2 Multi-input: `gradient` of `‖x‖²` and a trig field; `directional_derivative` equals `∇f · dir`; `jacobian` of a known vector map.
- [ ] 3.3 Precision parameter: at least one test each over `f32` and `f64` (and `Float106` where cheap) to prove the surface is precision-generic.
- [ ] 3.4 `solve_gm`: existing `f64` results unchanged; new `Dual<f64>` run gives `∂GM/∂input` matching a central finite-difference within tolerance.

## 4. Verification

- [ ] 4.1 `cargo test -p deep_causality_num` and `cargo test -p deep_causality_physics` green.
- [ ] 4.2 `cargo fmt` + `cargo clippy -p deep_causality_num -p deep_causality_physics --all-targets` — 0 warnings, no `#[allow(...)]`; no `dyn` / macros / external deps introduced.
- [ ] 4.3 Additive check: `dual-numbers` arithmetic untouched; `solve_gm` change is a bound widening only. Commit message prepared; owner commits.
