## 1. Integration operator in `deep_causality_num`

- [x] 1.1 Add an `autointegration` module under `src/` (one concern per file), sitting beside the `autodiff` module, re-exported from `lib.rs`. No new dependency, no `unsafe`, no macros.
- [x] 1.2 `Integrator` trait: `step<S, R, F>(&self, &S, R, &F) -> S` with bounds `S: Clone + Add<Output = S> + Mul<R, Output = S>`, `R: RealField`, `F: Fn(&S) -> S`; provided `integrate(&self, S, R, usize, &F) -> S` folding `step`.
- [x] 1.3 `Euler` struct: `step` = `s + f(s)·dt`.
- [x] 1.4 `Rk4` struct: classical four-stage `step` = `s + (k1 + 2k2 + 2k3 + k4)·(dt/6)`, scalars built from `R::one()`. Uses only `Add` + scalar `Mul` (no state-type-specific ops).

## 2. Quadrature

- [x] 2.1 `quadrature(f, a, b, n) -> R` — composite Simpson over `n` panels, `R: Real + Div<Output = R>`, `F: Fn(R) -> R`. `n` normalised to an even value `≥ 2`; panel count built by accumulation (Real has no integer conversion). Infallible (`-> R`, no `Result`).

## 3. Tests (mirror `src/`, 100% of new code)

- [x] 3.1 Euler: `y' = y` → `exp`; first-order error ratio ≈ 2 across `dt`, `dt/2`; a non-scalar `Dual`-as-2-module state.
- [x] 3.2 Rk4: `y' = y` order check (error ratio ≈ 16 across `dt`, `dt/2`); RK4 vs Euler accuracy at fixed `dt`; **harmonic-oscillator energy near-conservation** over 1000 steps (SHM encoded in a `Dual`'s two channels).
- [x] 3.3 Accuracy-swap: one model run with both `Euler` and `Rk4`, identical call shape; single-step Euler-formula check.
- [x] 3.4 Quadrature: exact on `x³` (= 1/4) for several even `n`; odd-`n` and `n < 2` normalisation; convergent on `sin` over `[0, π]` (→ 2); precision-generic over `f32`.
- [x] 3.5 Leibniz: `quadrature(|x| (x·θ).sin(), 0, 1, n)` with `θ = Dual::variable` — real part = `∫`, `ε` part = `dI/dθ`, both matching analytic `(1−cos θ)/θ` and its derivative.

## 4. Verification

- [x] 4.1 `cargo test -p deep_causality_num` — 14 new integration tests + 3 new doctests; full suite 4311 + 189 doctests green.
- [x] 4.2 `cargo fmt` + `cargo clippy -p deep_causality_num --all-targets` — 0 warnings, no `#[allow(...)]`; no `dyn` / macros / external deps. (Two `assign_op_pattern` clippy hints fixed by rewriting to `+=`, not suppressed.)
- [x] 4.3 All error paths covered: the integrator and `quadrature` are infallible (no `Err`/panic branches); the only conditional path — panel-count normalisation (odd `n`, `n < 2`) — is exercised by dedicated tests.
- [x] 4.4 Additive check: no existing `num` API changed; the three existing per-crate integrators (geodesic RK4, gauge-flow Euler/RK3, Stokes integrate) left untouched. Commit message prepared; owner commits.
