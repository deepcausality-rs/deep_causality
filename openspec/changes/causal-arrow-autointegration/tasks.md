## 1. Integration operator in `deep_causality_num`

- [ ] 1.1 Add an `integration` module under `src/` (one concern per file), re-exported from `lib.rs`. No new dependency, no `unsafe`, no macros.
- [ ] 1.2 `Integrator` trait: `step<S, R, F>(&self, &S, R, &F) -> S` with bounds `S: Clone + Add<Output = S> + Mul<R, Output = S>`, `R: RealField`, `F: Fn(&S) -> S`; provided `integrate(&self, S, R, usize, &F) -> S` folding `step`.
- [ ] 1.3 `Euler` struct: `step` = `s + f(s)·dt`.
- [ ] 1.4 `Rk4` struct: classical four-stage `step` = `s + (k1 + 2k2 + 2k3 + k4)·(dt/6)`, scalars via `R: RealField` / `From<f64>`. Confirm only `Add` + scalar `Mul` are used (no state-type-specific ops).

## 2. Quadrature

- [ ] 2.1 `quadrature(f, a, b, n) -> R` — composite Simpson over `n` panels, `R: Real + Div<Output = R>`, `F: Fn(R) -> R`. Validate `n` even (or handle odd by promoting).

## 3. Tests (mirror `src/`, 100% of new code)

- [ ] 3.1 Euler: `y' = y` → `exp`, error `O(dt)`; a 2-D rotation / harmonic state to exercise non-scalar `S`.
- [ ] 3.2 Rk4: `y' = y` order check (error ratio ≈ 16 across `dt`, `dt/2`); RK4 vs Euler accuracy at fixed `dt`; harmonic-oscillator energy near-conservation over many steps.
- [ ] 3.3 Accuracy-swap: one integration written against the trait, run with both `Euler` and `Rk4`, identical model and call shape.
- [ ] 3.4 Quadrature: exact on `x³` (= 1/4); convergent on `sin` over `[0, π]` (→ 2); precision-generic over `f32`/`f64`.
- [ ] 3.5 Leibniz: `quadrature(|x| (x*theta).sin(), 0, 1, n)` with `theta = Dual::variable` — real part = `∫`, infinitesimal part = `dI/dθ`, both matching analytic values.

## 4. Verification

- [ ] 4.1 `cargo test -p deep_causality_num` green.
- [ ] 4.2 `cargo fmt` + `cargo clippy -p deep_causality_num --all-targets` — 0 warnings, no `#[allow(...)]`; no `dyn` / macros / external deps.
- [ ] 4.3 Additive check: no existing `num` API changed; the three existing per-crate integrators left untouched. Commit message prepared; owner commits.
