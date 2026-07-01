## Why

`deep_causality_cfd` uses `CausalTensor` only as a flat buffer; nothing connects a flowfield to a
quantized tensor train (QTT/MPS), so the Plasma Blackout Corridor flagship's compressed-flowfield
step [4] cannot be built. The tensor-train primitives now exist (encode, MPO operators + algebra,
round, solve); the CFD-side bridge does not. This change builds the **foundation**: a QTT codec,
finite-difference MPO assembly, and a quasi-1D linear rollout — the minimum that proves a CFD field can
live in, and evolve as, a tensor train. Context:
`openspec/notes/plasma-blackout/gap-one-cfd-tensor-bridge.md`.

## What Changes

- New `deep_causality_cfd/src/tensor_bridge/` module:
  - **QTT codec** — `quantize` / `dequantize` between a `2^L` periodic lattice field `CausalTensor<R>`
    and a `CausalTensorTrain<R>` (binary mode encoding), round-tripping within tolerance.
  - **Finite-difference operator assembly** — the grid-shift MPO `S₊` / `S₋` via
    `CausalTensorTrainOperator::from_cores`, and `gradient` / `laplacian` stencils via the operator
    algebra (`add` / `sub` / `scale`) + `round`, for periodic 1D grids (Kazeev–Khoromskij).
- New `solvers/qtt/` **quasi-1D linear advection–diffusion rollout** (`QttLinear1d`) behind the existing
  `FluidTheory` / `Marcher` seam: encode → MPO-apply → round per step, validated against an analytic
  solution.
- Scalar bound: real `R: CfdScalar + ConjugateScalar<Real = R>` (`f32` / `f64` / `Float106`).
- Purely additive: no change to the DEC solver, the `CfdFlow` DSL, or any default.

### Non-Goals (explicit follow-on change)
Pressure projection / 2D incompressible NS, nonlinear convection via TT-cross, immersed-body boundary
conditions in QTT, the full `QttIncompressible` `FluidTheory`, observable extraction, and `CfdFlow`
wiring. This change is steps 1–3 of the gap-one staged plan only.

## Capabilities

### New Capabilities
- `qtt-field-codec`: bidirectional quantized encoding between a lattice field and a tensor train.
- `qtt-differential-operators`: shift-operator MPOs and finite-difference stencil assembly.
- `qtt-linear-rollout`: a validated quasi-1D linear advection–diffusion tensor-train marcher.
