## 1. 2-D codec + lifted axis operators (`tensor_bridge/`)

- [x] 1.1 `quantize_2d` / `dequantize_2d` (`tensor_bridge/codec.rs`): `[Nx,Ny]` → `[2;Lx,2;Ly]` (x-modes then y-modes) → `from_dense`; per-axis power-of-two guard.
- [x] 1.2 `lift_leading` / `lift_trailing` (`operators.rs`): concatenate a 1-D operator's cores with `m` identity cores (bond-1 join) for `op ⊗ I` / `I ⊗ op`.
- [x] 1.3 `gradient_x` / `gradient_y` (lift the 1-D `gradient`), `laplacian_2d = lift_x(lap) + lift_y(lap)` (add + round).
- [x] 1.4 Tests (`operators_2d_tests.rs`, f64): 2-D round-trip; axis derivative hits the right axis; `laplacian_2d` matches the dense five-point stencil.

## 2. Pressure projection (`tensor_bridge/`)

- [x] 2.1 `QttProjector2d::divergence(u, v) = gradient_x·u + gradient_y·v` (apply + add + round).
- [x] 2.2 Pressure-Poisson solve — **spectral/exact** via `RfftPlanNd`. Key correctness point caught during implementation: the projection applies `grad∘grad` (centered-difference squared), so the Poisson must use that operator's eigenvalues `−sin²(2πk/N)/Δ²` (the *consistent* operator, **not** the compact five-point Laplacian) for `div(project(u))=0` to hold exactly; both the constant *and* the collocated-grid checkerboard/Nyquist null modes (`k ∈ {0, N/2}` per axis) are zeroed. No AMEn, no regularization.
- [x] 2.3 `QttProjector2d::project(u*, v*) -> (u, v)`: `u = u* − ∂ₓp`, `v = v* − ∂ᵧp` (apply + add(neg) + round).
- [x] 2.4 Tests (`projection_tests.rs`, f64): projection drives divergence ≤ 1e-8 (and reduces it); idempotent; projected field finite.

## 3. `QttIncompressible2d` marcher (`solvers/qtt/`)

- [x] 3.1 `QttIncompressible2d` (`solvers/qtt/incompressible_2d.rs`): `Marcher` with `State = (u_train, v_train)`. Step: `rate = −(u·∇)a + ν·∇²a` per component, convection via the **fused `hadamard_rounded`** (no `r²` materialization); `u* = round(u + Δt·rate)`, then `project`.
- [x] 3.2 `run(u0, v0, steps)` driver (quantize → step loop → dequantize) with grid-size guards.
- [x] 3.3 Tests (`incompressible_2d_tests.rs`, f64): matches the analytic Taylor–Green vortex decay `e^{−2νt}` (max err ≤ 2e-2 at N=16); stays divergence-free (≤ 1e-5) and bond-bounded (≤ 12) over 30 steps.

## 4. Finalize

- [x] 4.1 `cargo fmt`; clippy `--all-targets` clean; `cargo test -p deep_causality_cfd` green (296 + 1 ignored). No `unsafe`/`dyn`/lib-code macros; lib float literals confined to the RNG/eigenvalue mapping (from_f64), not magic constants. Bazel: `solvers/**` glob covers the qtt tests; the `tensor_bridge` target covers the bridge tests; the crate already deps `deep_causality_fft`.
- [x] 4.2 `openspec validate add-cfd-qtt-incompressible-2d --strict` passes; `gap-one-cfd-tensor-bridge.md` §6 updated (steps 4–5 done).
