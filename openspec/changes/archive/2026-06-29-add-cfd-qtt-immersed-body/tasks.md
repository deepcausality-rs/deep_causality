## 1. Body-mask tensor train (`tensor_bridge/`)

- [x] 1.1 `body_mask_2d(lx, ly, dx, dy, center, radius, smoothing, trunc)` → `CausalTensorTrain`: sample the smoothed volume fraction `χ = ½(1 − tanh(d/δ))` over the signed distance `d` to a cylinder, `quantize_2d`, `round`.
- [x] 1.2 A general `mask_from_fn(closure, …, trunc)` for any sampled smoothed indicator; both return the field + report its bond dimension.
- [x] 1.3 Tests (`mask_tests.rs`, f64): the cylinder mask round-trips within tolerance; bond is bounded and **grows as the smoothing width shrinks** (the rank/accuracy trade-off is explicit).

## 2. Brinkman-penalized marcher (`solvers/qtt/immersed_2d.rs`)

- [x] 2.1 `QttImmersed2d` (or an opt-in body on `QttIncompressible2d`): holds the mask `χ_body`, `u_body` (zero for a static wall), and `η`. Step: `rate += −(1/η)·χ_body ⊙ (u − u_body)` (fused `hadamard_rounded`), then the existing convection/diffusion + projection; recompress each step.
- [x] 2.2 `eta`/`dt` stability guard + doc of the explicit bound (`Δt ≲ η`); penalization applied **before** the projection. Body-free (`χ_body = 0`) path reduces to `QttIncompressible2d`.
- [x] 2.3 Tests (`immersed_2d_tests.rs`, f64): no-slip — `max|u|` inside the body falls toward zero over the run; divergence stays ≤ projection floor; bond bounded; the zero-mask run matches `QttIncompressible2d` within rounding tolerance (≤1e-10 — the extra add-zero-then-round of the vanished penalization is not bit-exact).

## 3. Surface observables (`solvers/qtt/observe.rs`)

- [x] 3.1 `drag_lift(mask, u, v, u_body, eta, dx, dy, u_ref, d_ref)` → `(C_d, C_l)` via `F = (1/η)·inner(χ_body, u_body − u)·cell_volume`, nondimensionalized — a tensor-train contraction, no surface reconstruction.
- [x] 3.2 Optional passive scalar `T` (advect–diffuse on the same operators) + `wall_heat_flux(mask, T, T_w, eta, …)` = `(1/η)·inner(χ_body, T_w − T)·cell_volume` — **neutral**, the Gap-2 seam.
- [x] 3.3 Extend `QttObserve` (`drag(u_ref, d_ref)`, `wall_heat_flux(...)`) and the `CfdFlow::qtt_march` config/run to carry an optional body + emit the drag/lift (and heat-flux) series into the `Report`.
- [x] 3.4 Tests (`observe_tests.rs`, f64): drag/lift are the mask–deficit contraction (cross-checked against a dense reference integral); the pipeline emits a drag series; the heat-flux observable responds to a wall-temperature difference.

## 4. Validation (`verification/qtt_cylinder_verification/`)

- [x] 4.1 A self-verifying example (config.rs/main.rs/print_utils.rs, mirroring `qtt_taylor_green_verification`): a cylinder in a periodic free-stream, penalized, marched to quasi-steady; report `C_d`, the no-slip interior `max|u|`, the bond, and the accuracy-vs-bond table.
- [x] 4.2 Gates (exit nonzero on break): interior `max|u|` at the penalization floor (no-slip); the drag coefficient **converges as the bond cap rises** (successive-cap change shrinks); the streamwise drag is **positive and finite** (the absolute magnitude is inflated by the smoothing skirt + blockage — the convergence trend is the result, not the number). The committed DEC cylinder `C_d` is **reported as a cross-reference**, disclaimed for periodic blockage (the periodic box is not the DEC inflow/outflow/far-field configuration — no absolute match claimed; running DEC inline is descoped for runtime).
- [x] 4.3 `baseline.txt` + README (human-readable report style); register the example in `Cargo.toml`; add the row + references to `verification/README.md`.

## 5. Finalize

- [x] 5.1 `cargo fmt`; clippy `--all-targets` clean (fix, don't suppress); `cargo test -p deep_causality_cfd` green. No `unsafe`/`dyn`/lib-code macros; lib float literals confined to mask/observable mapping (`from_f64`).
- [x] 5.2 Register new test modules in their `mod.rs` and the Bazel `tests/` target; import crate types from the crate root.
- [x] 5.3 `openspec validate add-cfd-qtt-immersed-body --strict` passes; update `gap-one-cfd-tensor-bridge.md` §6 and `gap-analysis.md` §4 — **Gap 1 closed** (solver core + immersed body + surface observables), residual flagship physics (electron density, reacting heat flux) handed to **Gap 2**.
