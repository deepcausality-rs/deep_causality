## Why

The QTT bridge (`add-cfd-qtt-tensor-bridge`) gave a 1-D linear rollout; the flagship's step [4] needs a
real **incompressible** flowfield. The missing leap is **2-D encoding + pressure projection** — a
divergence, a pressure-Poisson solve in tensor-train form (AMEn), and Leray projection — plus
**nonlinear convection**: the two expensive axes a quantum-inspired CFD method (Peddinti et al., 2024)
compresses. With these, a 2-D incompressible Navier–Stokes field lives in, and evolves entirely as, a
tensor train. Context: `openspec/notes/plasma-blackout/gap-one-cfd-tensor-bridge.md` (steps 4–5).

## What Changes

- **2-D codec + axis operators** in `tensor_bridge`: `quantize_2d` / `dequantize_2d` (a `2^Lx × 2^Ly`
  field as `Lx + Ly` serial binary modes), and the periodic axis operators `gradient_x` / `gradient_y` /
  `laplacian_2d` / `divergence` built by lifting the 1-D shift stencils with identity blocks
  (`∂ₓ = ∂ₓ ⊗ I_y`) and summing via the MPO algebra.
- **Projection** (`qtt-projection`): `divergence(u, v)`, a pressure-Poisson solve via `solve::linear`
  (AMEn) on the 2-D Laplacian MPO — with the periodic constant null-space pinned — and the Leray
  `project(u*, v*) → (u, v)`.
- **`QttIncompressible2d`** (`solvers/qtt`): a `Marcher` advancing periodic 2-D incompressible NS —
  nonlinear convection `u·∇u` (Hadamard products + round) + viscous diffusion, then projection —
  recompressing every step. State is the `(u, v)` velocity train pair.
- **Validation** against the periodic 2-D Taylor–Green vortex (analytic decay) — field error vs. bond,
  and post-projection divergence ~ 0.
- Bound: real `R: CfdScalar + ConjugateScalar<Real = R>`. Purely additive; no default changes.

### Non-Goals (explicit follow-on)
CfdFlow DSL wiring; observable extraction (drag / heat flux / electron density); immersed-body boundary
conditions in QTT; the Gap-2 ionization / reacting-flow surrogate physics; 3-D. This change is steps 4–5
of the gap-one plan.

## Capabilities

### New Capabilities
- `qtt-codec-2d`: bidirectional quantized encoding of a 2-D lattice field and lifted axis operators.
- `qtt-projection`: divergence, AMEn pressure-Poisson solve, and Leray projection in tensor-train form.
- `qtt-incompressible-2d`: a validated periodic 2-D incompressible Navier–Stokes tensor-train marcher.
