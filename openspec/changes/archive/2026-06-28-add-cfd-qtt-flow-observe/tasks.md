## 1. TT-native observable extraction (`solvers/qtt/observe.rs`)

- [x] 1.1 `kinetic_energy(u, v) -> R = ½(‖u‖² + ‖v‖²)` via `TensorTrain::norm` (square the norms); no dequantize.
- [x] 1.2 `divergence_residual(projector, u, v) -> R = ‖projector.divergence(u, v)‖` via the train norm; no dequantize.
- [x] 1.3 `max_bond(u, v) -> usize` = largest bond `c.shape()[2]` across both trains' cores.
- [x] 1.4 `max_speed(u, v, lx, ly) -> R` = `max √(u² + v²)` over the dequantized grid.
- [x] 1.5 Tests (`observe_tests.rs`, f64): TT-native energy/divergence match the dense-field reference within tolerance; `max_bond` matches the cores; `max_speed` matches the dense field.

## 2. QTT marching config (`flow_config/qtt_march_config.rs`)

- [x] 2.1 `QttMarchConfig<R>` (owned): grid `(lx, ly, dx, dy)`, solver `(dt, nu, trunc)`, seed `(u0, v0): (CausalTensor<R>, CausalTensor<R>)`, `stop: MarchStop<R>`, `observe: QttObserve`, `name`.
- [x] 2.2 `QttObserve` fluent set: `kinetic_energy()` / `divergence()` / `max_speed()` / `bond()` toggles; `Default` collects nothing.
- [x] 2.3 `QttMarchConfigBuilder`: set grid / solver / stop / observe / name; `seed_fn(Fn(R, R) -> (R, R))` materializes `(u0, v0)` over the grid, `taylor_green()` convenience; `build()` validates `2^Lx × 2^Ly` and seed-shape match (`PhysicsError::DimensionMismatch`).
- [x] 2.4 Tests (`qtt_march_config_tests.rs`, f64): seed closure materializes correctly; non-power-of-two grid and mismatched seed are rejected.

## 3. CfdFlow wiring (`flow/qtt_march_run.rs`, `flow/cfd_flow.rs`)

- [x] 3.1 `CfdFlow::qtt_march(&QttMarchConfig<R>) -> QttMarchRun<'_, R>` (parallel to `march`; no `.on()` — no borrowed geometry).
- [x] 3.2 `QttMarchRun`: build `QttIncompressible2d` from the config, quantize the seed, march under `MarchStop` (Fixed / Steady via the kinetic-energy delta), sample enabled observables each step into a `Series`, dequantize the final `(u, v)`, return an owned `Report<R>` (`u → set_final_field`, `v → "final_v"` series). Counterfactual overrides `seed_with` / `march_with` / `observe_with`.
- [x] 3.3 `run()` and `run_with(hook)` + `QttStepView` (step / time / `(u, v)` train refs / TT-native `kinetic_energy` / `divergence` / `max_bond`).
- [x] 3.4 Re-export `QttMarchConfig`, `QttMarchConfigBuilder`, `QttObserve`, `QttStepView` from the crate root beside the existing flow types.
- [x] 3.5 Tests (`qtt_march_run_tests.rs`, f64): the DSL run matches `QttIncompressible2d::run` bit-for-bit; the report carries one series per enabled observable + the final `(u, v)`; the kinetic-energy series matches the Taylor–Green decay; the steady stop terminates early on the plateau; the per-step hook fires once per step and leaves the report identical.

## 4. Finalize

- [x] 4.1 `cargo fmt`; clippy `--all-targets` clean (fix, don't suppress); `cargo test -p deep_causality_cfd` green. No `unsafe`/`dyn`/lib-code macros; lib float literals confined to seed/observable mapping (`from_f64`), not magic constants.
- [x] 4.2 Register the new test modules in their `mod.rs` and the Bazel `tests/` target; import all crate types from the crate root.
- [x] 4.3 `openspec validate add-cfd-qtt-flow-observe --strict` passes; `gap-one-cfd-tensor-bridge.md` §6 updated (step 6 done, immersed-body observables next).
