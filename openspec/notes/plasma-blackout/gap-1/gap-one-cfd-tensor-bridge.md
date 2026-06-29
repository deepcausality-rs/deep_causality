<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Gap 1 — closing the CFD ↔ tensor-network bridge

**What this is.** A focused, literature-grounded plan for closing **Gap 1** of the
[plasma-blackout gap analysis](../gap-analysis.md): the missing layer that connects `deep_causality_cfd`'s
flowfield to a quantized-tensor-train (QTT/MPS) representation and operators, so chain step [4] of the
[flagship](../plasma-blackout-corridor.md) — *MPS flowfield → heat flux + drag + electron density* —
becomes real rather than aspirational.

Gap 1 is the critical path: the tensor-train *primitives* now exist in `deep_causality_tensor`, but
nothing in `deep_causality_cfd` uses them. This note names the SOTA method to follow, maps each step
onto the primitives we have, and lists exactly what must be built and where.

Honesty convention: **[holds]**, **[holds under precondition]**, **[open]**, **[speculative]**.

---

## 1. The method to follow (SOTA)

The "quantum-inspired CFD" lineage is now mature enough to copy rather than invent. The closest match to
our crate — incompressible Navier–Stokes **around immersed bodies**, which we already support via cut
cells — is:

- **Peddinti, Pisoni, Marini, Lott, Argentieri, Tiunov, Aolita — "A quantum-inspired framework for
  computational fluid dynamics," *Communications Physics* 7, 135 (2024).** MPS-encoded incompressible NS
  around immersed objects, with runtime and memory scaling **poly-logarithmically in the mesh size**.
  <https://www.nature.com/articles/s42005-024-01623-8>

Supporting / foundational:

- **Gourianov et al. — "A quantum-inspired approach to exploit turbulence structures," *Nat. Comput.
  Sci.* 2 (2022)**, and the 2025 *Sci. Adv.* turbulence-PDF follow-up. The original MPS-CFD demonstration.
- **Kiffner & Jaksch — tensor-network reduced-order models for wall-bounded flows (2023).**
- **Kornev et al. — "TetraFEM: numerical solution of the incompressible Navier–Stokes equations via
  quantum-inspired Tensor-Train Finite Element Method," arXiv:2305.10784 (2023).** TT-FEM on non-trivial
  geometries (T-mixer); FEM matrices and fields compressed to TT, solved with AMEn/DMRG/ALS.
- **Kazeev & Khoromskij — "Low-Rank Explicit QTT Representation of the Laplace Operator and Its
  Inverse."** The canonical construction: finite-difference operators as MPOs built from **grid-shift
  operators** (binary adders) at small bond dimension; the Laplacian *and its inverse* have explicit
  low-rank QTT forms — directly relevant to the pressure-Poisson solve.
- **Comparative MPS/QTT algorithm study — arXiv:2303.09430**; **pseudospectral PDE solving with MPS —
  arXiv:2409.02916** (FFT-in-MPS operator route); **interpolative DLRA time integration of QTT —
  arXiv:2512.15703**; **TN space-time spectral collocation for nonlinear convection-diffusion —
  arXiv:2406.02505**.
- **Tensor cross interpolation (TCI / `xfac`) — Fernández et al., "Learning tensor networks with tensor
  cross interpolation."** The standard tool for encoding functions, initial/boundary conditions, source
  terms, and nonlinear element-wise maps into TT form.
- Reference library to mirror behaviour against: **SeeMPS (arXiv:2601.16734)**.

The reacting/ionized extension (Gap 2's physics, same bridge) is **Pinkston et al. — "Matrix Product
State Simulation of Reacting Shear Flows," arXiv:2512.13661 (2025)** — species + Arrhenius source terms
carried in MPS form via TCI.

---

## 2. The method, step by step, mapped onto our primitives

The QTT-NS rollout is six repeating operations. Every one maps onto an existing
`deep_causality_tensor` primitive — the gap is the CFD-side glue, not the math.

| QTT-NS step (Peddinti / Kazeev) | Operation | Primitive in `deep_causality_tensor` | Have? |
|---|---|---|---|
| **Encode** field `u(x)` on a `2^L` grid | quantized binary reshape → MPS | `CausalTensorTrain::from_dense` + QTT reshape | ✓ |
| **Build operators** (shift `S₊`, then stencils) | hand-build cores; combine `(S₊ − S₋)/2Δx`, `(S₊ + S₋ − 2I)/Δx²` | `from_cores` + `add`/`sub`/`neg`/`scale` (added) + `round` | ✓ (algebra complete; `S₊` cores built in the bridge) |
| **Differentiate** (∇, ∇·, ∇²) | apply finite-difference **MPO** to the field | `CausalTensorTrainOperator::{apply, compose}` | ✓ |
| **Convect** `u·∇u` (nonlinear) | Hadamard product + round, or TT-cross | `hadamard` / `round`; `cross` / `apply_nonlinear` | ✓ |
| **Project** (pressure Poisson `∇²p = ∇·u`) | TT linear solve | `solve::linear` (AMEn), `solve::eigen` (DMRG), `solve::fit` (ALS) | ✓ |
| **Advance in time** | explicit MPO-apply + round, or TDVP | `apply` + `round`; `solve::tdvp` | ✓ |
| **Recompress** every step | TT rounding (rank control) | `round` (+ randomized, + NaN-robust SVD/QR) | ✓ (just hardened) |
| **Read observables** (drag, heat flux, `n_e`) | contraction with weight train / boundary fiber | `inner`, `integrate`, `marginalize` | ✓ |

**The hardened SVD/QR matters here.** A QTT-NS rollout calls `round` after *every* operator application;
rank-revealing recompression on near-low-rank cores is the hot path. The recent overflow-safe Jacobi and
noise-floor QR fixes (and randomized rounding) are precisely the robustness this loop needs at
`f64`/`Float106`.

---

## 3. What must be built (in `deep_causality_cfd`)

Five concrete pieces. None is blocked on missing tensor mathematics.

### 3.1 QTT codec — lattice field ⇄ MPS

A bidirectional codec between a CFD lattice field (`CausalTensor<R>` over a `2^L` grid, or per-axis
`2^{L_x} × 2^{L_y}`) and a `CausalTensorTrain<R>` in **quantized** (binary, bit-interleaved) layout.
`from_dense` gives the TT; the quantization is an index reshape/permutation choosing the bit ordering
(serial vs. interleaved across axes — interleaved is standard for isotropic multiscale structure).
Dequantize = `to_dense` + inverse reshape. **[holds: small, self-contained]**

### 3.2 Operator MPO assembly — shift operators and the differential stencils

Build the **grid-shift MPO** `S₊` (a binary incrementer; explicit small-bond cores, ~bond 2–3) and
assemble the finite-difference operators as linear combinations:
`∂ₓ ≈ (S₊ − S₋)/(2Δx)`, `∂²ₓ ≈ (S₊ + S₋ − 2I)/Δx²`, and the Laplacian as their sum (Kazeev–Khoromskij).

**Verified — the core-level constructor exists and is public.**
`CausalTensorTrainOperator::from_cores(cores: Vec<CausalTensor<T>>)` takes an explicit chain of rank-4
cores `[r_k, n_out_k, n_in_k, r_{k+1}]`, validates the bond structure (boundary bonds = 1, matching
shared bonds, all 4-D, no zero dims), and builds the MPO — exactly what hand-building `S₊` needs (each
interior core `[2, 2, 2, 2]`, the carry bit as the rank-2 bond; boundary cores `[1,2,2,2]`/`[2,2,2,1]`).
`identity(dims)` and `from_dense(dense, out, in, trunc)` are also available, and the
`TensorTrainOperator` trait supplies `apply` / `compose` / `round` / `transpose` / `to_dense`.

**Done — the operator algebra is now complete.** `CausalTensorTrainOperator` previously had `compose`
(operator product) and `identity` (multiplicative one) but no additive structure. Added (public,
delegating to the existing combined-train machinery; tested at `f64`/`Float106`):
  - `add(&self, &Self) -> Result<Self>` — sum; bonds add, `round` afterwards,
  - `scale(&self, T) -> Self` — scalar multiple (rank-preserving),
  - `neg(&self) -> Self` — additive inverse,
  - `sub(&self, &Self) -> Result<Self>` — difference, for `(S₊ − S₋)`.

So the FD stencils assemble directly: `grad = sp.sub(&sm)?.scale(half_inv_dx)`,
`lap = sp.add(&sm)?.sub(&id.scale(two))?.scale(inv_dx2)?.round(&trunc)?`. Option 2 (closed-form Laplacian
cores via `from_cores`) is kept only as a later micro-optimization if the add-then-round bond is ever a
measured bottleneck.

**[holds: `from_cores` public; operator `add`/`scale`/`sub`/`neg` implemented and tested — §3.2 unblocked]**

*Alternative (periodic boxes):* a **pseudospectral** route — apply derivatives in Fourier space using the
crate's existing FFT/DCT (`fft-dct`) lifted to an MPS QFT operator (arXiv:2409.02916). Reuses
`spectral_diffusion` thinking; defer unless the shift-MPO route shows rank trouble.

### 3.3 The QTT-NS rollout — a new compressed `FluidTheory` / solver

A reduced rollout engine that drives step [4]: encode → (convect via Hadamard/cross + round) → (project
via `solve::linear`) → (advance via MPO-apply + round) → read observables. Lands as a new solver behind
the existing **`FluidTheory<R>` / `Marcher`** seam (a `QttIncompressible` sibling to `DecIncompressible`),
so it composes with the `CfdFlow` DSL and `PhysicsStage` coupling unchanged. Tier-A target is **quasi-1D/2D**
(corridor §7). **[holds under precondition: 3.1 + 3.2 done]**

### 3.4 Boundary / immersed-body encoding in QTT

Encode the immersed-body mask and boundary values as TT/MPO operands (a mask MPS multiplied in via
`hadamard`, or a penalization MPO), reproducing Peddinti's immersed-object treatment. The crate already
computes cut-cell geometry; the work is expressing the mask/BC in QTT. Boundary conditions in QTT are the
**fiddliest** part of the literature — budget for it. **[holds under precondition; rank-sensitive]**

### 3.5 Observable extraction

Drag/lift/heat-flux/electron-density as **contractions** of the field MPS with weight/boundary trains:
surface-force integrals via `integrate`/`inner`, marginal profiles via `marginalize`. Mirrors the
existing `surface_force.rs` diagnostics, in TT form. **[holds]**

---

## 4. Architecture fit and honesty seams

- **Where it lands.** A new `solvers/qtt/` module exposing a `QttIncompressible` `FluidTheory`, plus a
  `tensor_bridge/` (codec + MPO assembly). The `CfdFlow` DSL, `Coupling`/`PhysicsStage`, and counterfactual
  `continue_with` are unchanged — the QTT solver is just another `FluidTheory`. This keeps the EPP role
  (compose/gate/audit) intact and puts the heavy compute behind the causaloid boundary.
- **Quantization needs power-of-two grids.** QTT encodes `2^L` grids; the lattice meshes must be sized
  accordingly (or padded). A real constraint, not a blocker. **[holds under precondition]**
- **Rank growth is the central risk.** Nonlinear convection and the chemical source (Gap 2) inflate bond
  dimension; the whole method lives or dies on `round` keeping ranks bounded. This is why the SVD/QR
  robustness work was a prerequisite, and why TT-cross (which builds at controlled rank) is preferred over
  Hadamard-then-round for the source terms. **[open: rank control must be demonstrated, not assumed]**
- **DEC alignment, not reuse.** The existing solver is DEC-native; its `d`/Hodge operators are banded and
  have natural low-rank QTT forms on uniform lattices, but graded/cut-cell Hodge stars do not. The QTT
  solver is a *sibling*, not a re-expression of the DEC one — do not conflate them. **[holds]**
- **EPP is the macroscope.** The QTT rollout is the compressed inner solve; the value narrative remains
  orchestration + auditable safety + counterfactuals, per corridor §6.

---

## 5. Validation anchors

The crate already verifies the classical benchmarks the QTT literature uses — so the bridge can be
validated by **reproducing the existing DEC results at a compressed bond dimension**:

- **2D decaying / lid-driven cavity** (Ghia) and **cylinder wake** (Williamson St, C_d) — already verified
  for the DEC solver; re-run through the QTT solver and compare error vs. bond dimension (the Peddinti /
  Gourianov accuracy-vs-rank curve).
- **Taylor–Green** energy decay — existing 3D invariant gate.
- **MPS compression ratio** — memory vs. dense at fixed error, the headline metric in every reference.

Cross-check primitive behaviour against **SeeMPS** on a shared toy problem before trusting the rollout.

---

## 6. Staged plan (Tier-A first)

1. **[DONE] QTT codec** (`tensor_bridge::quantize`/`dequantize`) + round-trip / compression / guard tests.
2. **[DONE] shift-MPO + gradient/Laplacian** (`tensor_bridge::shift_plus`/`gradient`/`laplacian`, hand-built
   `S₊` via `from_cores`, stencils via the operator algebra) + tests against the periodic FD stencils.
3. **[DONE] quasi-1D linear advection–diffusion** QTT rollout (`solvers::QttLinear1d`, a `Marcher`) —
   encode → MPO-apply → round — validated against the analytic diffusion solution, with bounded-rank and
   mean-conservation tests. (OpenSpec change `add-cfd-qtt-tensor-bridge`.)
4. **[DONE] projection → 2-D incompressible** — `QttProjector2d` (spectral Poisson, consistent
   `grad∘grad` eigenvalues, checkerboard/Nyquist null modes zeroed) + `QttIncompressible2d` marcher;
   validated against the analytic Taylor–Green vortex (divergence-free, bounded rank).
5. **[DONE] nonlinear convection** — `u·∇u` via the fused `hadamard_rounded` inside `QttIncompressible2d`
   (the same rollout the Gap-2 ionization/reacting surrogate will ride; TT-cross is the escape hatch).
   (OpenSpec change `add-cfd-qtt-incompressible-2d`.)
6. **[DONE] CfdFlow wiring + observable extraction** — `CfdFlow::qtt_march` (a parallel, geometry-free
   pipeline, sibling of `CfdFlow::march`), a `QttMarchConfig`/`QttMarchConfigBuilder` config layer, TT-native
   observables (`kinetic_energy`/`divergence_residual`/`max_bond` on the trains, `max_speed` via dequantize),
   and a per-step `QttStepView` hook — all reusing the owned `Report`/`MarchStop`. Validated bit-for-bit
   against the direct `QttIncompressible2d::run` driver. (OpenSpec change `add-cfd-qtt-flow-observe`.)
7. **[DONE] Immersed body + surface observables** — a Brinkman volume-penalization body (`QttImmersed2d`,
   a smoothed mask MPS, no cut cells), drag/lift as the penalization-force tensor-train contraction, a
   neutral wall heat-flux via a penalized passive scalar, the `CfdFlow::qtt_march` body wiring, and a
   self-verifying `qtt_cylinder_verification` (no-slip + accuracy-vs-bond convergence). (OpenSpec change
   `add-cfd-qtt-immersed-body`.) Then the Gap-2 ionization/reacting surrogate (electron density `n_e`,
   reacting heat flux) and the hand-off to the flagship's step [4].

**Steps 1–7 are done — Gap 1 is CLOSED:** a 2-D incompressible Navier–Stokes flowfield now lives in, and
evolves as, a tensor train on `deep_causality_tensor`, with a spectral projection keeping it
divergence-free, rounding keeping the rank bounded, an immersed body by volume penalization, and the
surface observables (drag/lift, neutral wall heat flux) the flagship's step [4] reads — all driven through
the `CfdFlow` DSL and verified (2nd-order Taylor–Green; no-slip + accuracy-vs-bond cylinder). The headline
numerical risks (singular Poisson, nonlinear rank growth, mask rank) were resolved and verified in code.
The only outstanding flagship physics is **Gap 2** (Park-2T ionization → `n_e`, reacting heat flux) — the
neutral thermal observable here is the seam it plugs into.

---

## 7. Sources

- Peddinti et al., *Commun. Phys.* 7, 135 (2024) — <https://www.nature.com/articles/s42005-024-01623-8>
- Gourianov et al., *Nat. Comput. Sci.* 2 (2022); *Sci. Adv.* 11 (2025) —
  <https://inspirehep.net/files/0ee2a95339cde99c2435a51ad0c6344a>
- Kornev et al. (TetraFEM), arXiv:2305.10784 — <https://arxiv.org/abs/2305.10784>
- Kazeev & Khoromskij — Low-Rank Explicit QTT Laplace operator and its inverse.
- Comparative MPS/QTT algorithms — arXiv:2303.09430 — <https://arxiv.org/pdf/2303.09430>
- Pseudospectral PDEs with MPS — arXiv:2409.02916 — <https://arxiv.org/pdf/2409.02916>
- Interpolative DLRA time integration of QTT — arXiv:2512.15703 — <https://arxiv.org/html/2512.15703>
- TN space-time spectral collocation (nonlinear convection–diffusion) — arXiv:2406.02505 —
  <https://arxiv.org/pdf/2406.02505>
- Reacting MPS (Gap-2 tie-in) — Pinkston et al., arXiv:2512.13661 — <https://arxiv.org/abs/2512.13661>
- SeeMPS reference library — arXiv:2601.16734 — <https://arxiv.org/pdf/2601.16734>

---

## 8. Related

- [`gap-analysis.md`](../gap-analysis.md) — the four-gap analysis this note drills into.
- [`../plasma-blackout-corridor.md`](../plasma-blackout-corridor.md) — the flagship; step [4] is what this
  bridge powers.
- `deep_causality_tensor` tensor-network layer — the primitives mapped in §2.
- `deep_causality_cfd` `FluidTheory` / `CfdFlow` / `PhysicsStage` — the seams the QTT solver plugs into.
