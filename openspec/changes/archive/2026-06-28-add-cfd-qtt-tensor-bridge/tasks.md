## 1. QTT field codec (`tensor_bridge/`)

- [x] 1.1 New module `deep_causality_cfd/src/tensor_bridge/` (registered in `lib.rs`); typed `PhysicsError::DimensionMismatch` for non-power-of-two length.
- [x] 1.2 `quantize(field, trunc) -> Result<CausalTensorTrain<R>, _>` — length `2^L` → `L` binary modes (MSB-first = natural row-major reshape), via `from_dense`. Bound `R: CfdScalar + ConjugateScalar<Real = R>`.
- [x] 1.3 `dequantize(train) -> CausalTensor<R>` — `to_dense` + inverse reshape.
- [x] 1.4 Tests (f64): round-trip to tolerance; smooth sine compresses (`bond ≪ 2^L`); non-power-of-two rejected.

## 2. Finite-difference operators (`tensor_bridge/`)

- [x] 2.1 Periodic shift MPO `S₊` (bond-2 carry bit, MSB-first ripple-carry) via `CausalTensorTrainOperator::from_cores`; `S₋ = S₊.transpose()`.
- [x] 2.2 `gradient(l, dx) = (S₋ − S₊)/(2Δx)` and `laplacian(l, dx) = (S₊ + S₋ − 2·I)/Δx²`, each `round`ed. (Gradient sign: with `(S₊·u)[k] = u[k−1]` the forward centered difference is `(S₋ − S₊)`, corrected during implementation.)
- [x] 2.3 Tests (f64): `S₊` matches the cyclic-shift action; `S₋∘S₊ = I`; `laplacian`/`gradient` match the periodic FD stencils functionally; gradient annihilates a constant.

## 3. Quasi-1D linear rollout (`solvers/qtt/`)

- [x] 3.1 `QttLinear1d` advancing `∂u/∂t = −c·∂ₓu + ν·∂²ₓu`: `u ← round(u + Δt·(−c·grad + ν·lap)·u)`; state is `CausalTensorTrain<R>`; exposes the round policy; `run()` end-to-end driver with a grid-size guard.
- [x] 3.2 Implements **`Marcher`** directly (not the Rk4-based `FluidTheory`) — tensor-train stages must round between operations, which the generic Rk4 path does not. Noted in code + design.
- [x] 3.3 Tests (f64): matches the analytic periodic diffusion of a sine within discretization+truncation error; bond stays bounded (≤ 8) over 300 steps; pure advection (`ν = 0`) conserves the discrete mean.

## 4. Finalize

- [x] 4.1 `cargo fmt`; clippy `--all-targets` clean; `cargo test -p deep_causality_cfd` green (288 + 10 new). No `unsafe`, no `dyn`, no lib-code macros; the only float literals are in `f64` test code, not lib; `[lints] workspace = true` already set. Bazel `tensor_bridge` test target added (qtt rides the `solvers/**` glob).
- [~] 4.2 Verification gate-binary — **deferred.** The three rollout integration tests already validate the analytic advection–diffusion behaviour, bounded rank, and mean conservation (the exact content a gate-binary would assert); a full `verification/` example (config + main + baseline + print_utils + `[[example]]` + Bazel) is redundant presentation scaffolding, deferred to keep scope minimal.
- [x] 4.3 `openspec validate add-cfd-qtt-tensor-bridge --strict` passes; `gap-one-cfd-tensor-bridge.md` §6 updated (steps 1–3 done).
