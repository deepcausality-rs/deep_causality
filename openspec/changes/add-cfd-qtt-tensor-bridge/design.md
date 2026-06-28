## Context

The Gap-1 analysis (`openspec/notes/plasma-blackout/`) established that every tensor-train *primitive* the
quantum-inspired CFD method needs now exists in `deep_causality_tensor` (encode via `from_dense`, MPO
`apply`/`compose` + the just-completed operator algebra `add`/`sub`/`neg`/`scale`, `round`, `solve`), but
`deep_causality_cfd` has no code that uses them. This change writes the first such code, following the
Peddinti (MPS incompressible NS) / Kazeev–Khoromskij (QTT operators) blueprint.

## Goals / Non-Goals

Goals: a minimal, tested foundation proving a CFD field can be encoded as a QTT, differentiated by an
assembled finite-difference MPO, and marched in tensor-train form with bounded rank — validated against an
analytic solution. Steps 1–3 of the gap-one staged plan.

Non-Goals (next change): pressure projection / 2D incompressible, nonlinear convection (TT-cross),
immersed-body BCs in QTT, the full `QttIncompressible` solver, observable extraction, `CfdFlow` wiring.

## Decisions

- **Scalar bound.** Real `R: CfdScalar + ConjugateScalar<Real = R>` — `f32` / `f64` / `Float106`. The QTT
  layer is real-valued here; complex/AD are out of scope for the bridge.

- **Quantization layout.** A 1D field of length `N = 2^L` is reshaped to `L` binary modes (physical
  dim 2 each) and `from_dense` produces the MPS. Choose the **most-significant-bit-first** ordering
  (mode 0 = coarsest scale) so multiscale structure lands in low bonds; document it as the contract.
  Higher-D fields use per-axis bit blocks (serial), interleaving deferred to the follow-on.

- **Shift MPO from `from_cores`.** The periodic increment `S₊` (binary `+1` with carry) is a bond-2 MPO:
  the rank-2 bond carries the carry bit; each mode core is `[2, 2, 2, 2]` (boundary cores `[1,2,2,2]` /
  `[2,2,2,1]`). `S₋ = S₊.transpose()`. Hand-built once, tested against the dense cyclic-shift matrix.

- **Stencil assembly via the operator algebra.** `gradient = (S₊.sub(&S₋)).scale(1/2Δx)`,
  `laplacian = (S₊.add(&S₋).sub(&id.scale(2))).scale(1/Δx²).round(trunc)`. Validated by densifying and
  comparing to the standard periodic finite-difference matrices.

- **Rollout behind `FluidTheory`/`Marcher`.** `QttLinear1d` advances `∂u/∂t = −c·∂ₓu + ν·∂²ₓu` as
  `u ← round(u + Δt·(−c·grad + ν·lap)·u)` (explicit Euler/RK; one MPO-apply per operator, one round per
  step). Its `State` wraps `CausalTensorTrain<R>` (which already has `Add` + scalar `Mul`, satisfying the
  `FluidTheory::State` bound). Validated against the analytic advection–diffusion of a smooth initial
  profile (periodic), measuring error vs. grid `L` and vs. round tolerance / bond cap.

- **Module placement.** `tensor_bridge/` (codec + operator assembly) and `solvers/qtt/` (the rollout),
  mirroring the `solvers/dec/` layout. The rollout is a sibling `FluidTheory`, so it composes with the
  existing `Marcher`/`CfdFlow` machinery unchanged when later wired in.

## Risks / Trade-offs

- **Power-of-two grids.** QTT needs `N = 2^L`. A real constraint for the codec; documented, validated by a
  shape guard.
- **Rank growth & round tolerance.** The whole method depends on `round` keeping bonds bounded; the
  rollout exposes the round policy and the validation sweeps error-vs-bond, so the trade-off is measured,
  not assumed. The recently hardened SVD/QR (overflow-safe Jacobi, noise-floor rank revealing) is the
  robustness this loop relies on.
- **Scope discipline.** Linear, periodic, 1D only. Nonlinearity and projection are deliberately deferred
  so this change stays small and verifiable.
