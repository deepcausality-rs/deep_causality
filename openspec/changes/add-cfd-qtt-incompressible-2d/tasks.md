## 1. 2-D codec + lifted axis operators (`tensor_bridge/`)

- [ ] 1.1 `quantize_2d(field: &CausalTensor<R>(shape [Nx,Ny]), trunc) -> CausalTensorTrain<R>` — reshape to `[2;Lx, 2;Ly]` (x-modes then y-modes) → `from_dense`; `dequantize_2d` the inverse; power-of-two guard per axis.
- [ ] 1.2 `lift` helper on `CausalTensorTrainOperator`: concatenate a 1-D operator's cores with `m` identity cores (bond-1 join) → operator over the full mode set.
- [ ] 1.3 `gradient_x`/`gradient_y` (lift the 1-D `gradient` onto the x-/y-block), `laplacian_2d = lift_x(lap) + lift_y(lap)` (sum + round), `divergence` helper.
- [ ] 1.4 Tests (f64): 2-D round-trip; axis derivative hits the right axis; `laplacian_2d` matches the dense five-point stencil functionally.

## 2. Pressure projection (`tensor_bridge/`)

- [ ] 2.1 `divergence(u, v) = gradient_x·u + gradient_y·v` (apply + add + round).
- [ ] 2.2 Pressure-Poisson solve — **spectral/exact** (periodic): per-mode division by the known Laplacian eigenvalues with the `k=0` mode zeroed (null space pinned by construction; no AMEn, no regularization). Tier-A: dequantize → eigen-solve → requantize; scalable form via a QFT-MPO. (AMEn retained for the future wall-bounded case.)
- [ ] 2.3 `project(u*, v*) -> (u, v)`: `u = u* − gradient_x·p`, `v = v* − gradient_y·p` (apply + sub + round).
- [ ] 2.4 Tests (f64): projection drives divergence ≤ tolerance; idempotent on a divergence-free field; singular Poisson returns a finite pressure (no NaN).

## 3. `QttIncompressible2d` marcher (`solvers/qtt/`)

- [ ] 3.1 `QttIncompressible2d`: `Marcher` with `State = (CausalTensorTrain<R>, CausalTensorTrain<R>)`. Step: `rate = −(u·∇)u + ν·∇²u` per component (convection via the **fused `hadamard_rounded`** — never materializes the `r²` intermediate; round tolerance tied to the discretization floor + `max_bond` cap), `u* = round(u + Δt·rate)`, then `project`. TT-cross (`apply_nonlinear`) is the escape hatch if rank growth is excessive.
- [ ] 3.2 `run(u0, v0, steps)` driver (quantize → step loop → dequantize), with grid-size guards.
- [ ] 3.3 Tests (f64): matches the analytic Taylor–Green vortex decay within discretization+truncation error; divergence stays ≤ tolerance over many steps; bond stays bounded and error falls as the round tolerance tightens.

## 4. Finalize

- [ ] 4.1 `cargo fmt`; clippy `--all-targets` clean; `cargo test -p deep_causality_cfd` green. No `unsafe`/`dyn`/lib-code macros/lib float literals; Bazel test targets updated.
- [ ] 4.2 `openspec validate add-cfd-qtt-incompressible-2d --strict`; update `gap-one-cfd-tensor-bridge.md` §6 (steps 4–5 done).
