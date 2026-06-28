## 1. QTT field codec (`tensor_bridge/`)

- [ ] 1.1 New module `deep_causality_cfd/src/tensor_bridge/` (register in `lib.rs`); typed error for non-power-of-two length.
- [ ] 1.2 `quantize(field: &CausalTensor<R>) -> Result<CausalTensorTrain<R>, _>` — length `2^L` → `L` binary modes (MSB-first), via `from_dense`. Bound `R: CfdScalar + ConjugateScalar<Real = R>`.
- [ ] 1.3 `dequantize(train: &CausalTensorTrain<R>) -> CausalTensor<R>` — inverse reshape via `to_dense`.
- [ ] 1.4 Tests (f64/Float106): round-trip to tolerance; smooth field compresses (`bond ≪ 2^L`); non-power-of-two rejected.

## 2. Finite-difference operators (`tensor_bridge/`)

- [ ] 2.1 Build the periodic shift MPO `S₊` (bond-2, carry bit) via `CausalTensorTrainOperator::from_cores`; `S₋ = S₊.transpose()`.
- [ ] 2.2 `gradient(l, dx)` = `(S₊.sub(&S₋))?.scale(1/2dx).round(trunc)`; `laplacian(l, dx)` = `(S₊.add(&S₋)?.sub(&id.scale(2)))?.scale(1/dx²).round(trunc)`.
- [ ] 2.3 Tests (f64/Float106): `S₊` matches the dense cyclic-shift matrix; `S₋∘S₊ = I`; densified `laplacian`/`gradient` match the periodic FD matrices; gradient annihilates a constant and differentiates a smooth profile within discretization error.

## 3. Quasi-1D linear rollout (`solvers/qtt/`)

- [ ] 3.1 `QttLinear1d` advancing `∂u/∂t = −c·∂ₓu + ν·∂²ₓu`: `u ← round(u + Δt·(−c·grad + ν·lap)·u)`; state wraps `CausalTensorTrain<R>`; exposes the round policy.
- [ ] 3.2 Implement `FluidTheory` / `Marcher` for it (state has `Add` + scalar `Mul` already).
- [ ] 3.3 Tests (f64): matches the analytic periodic advection–diffusion of a smooth profile within discretization + truncation error; bond stays bounded across many steps and error falls as the round tolerance tightens; pure advection (`ν = 0`) conserves the mean.

## 4. Finalize

- [ ] 4.1 `make format && make fix`; clippy `--all-targets` clean; `cargo test -p deep_causality_cfd` green. No `unsafe`, no `dyn`, no lib-code macros, no concrete float literals in lib code; `[lints] workspace = true` already set.
- [ ] 4.2 Add a runnable verification example (analytic advection–diffusion error vs. grid `L` and bond), matching the crate's `verification/` gate-binary style.
- [ ] 4.3 `openspec validate add-cfd-qtt-tensor-bridge --strict`; update `gap-one-cfd-tensor-bridge.md` §6 to mark steps 1–3 done.
