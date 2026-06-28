<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Plasma Blackout Corridor — gap analysis after the tensor-train layer

**What this is.** A capability-vs-SOTA gap analysis for the [Plasma Blackout Corridor flagship](../plasma-blackout-corridor.md),
written immediately after the tensor-train (MPS/MPO) layer landed in `deep_causality_tensor`. It answers
one question: *with the tensor train added, what stands between us and a buildable example using current
state-of-the-art methods?*

---

## 1. Verdict

The tensor-train addition **closes the tensor-network *primitive* gap** — the MPS flowfield-compression
lever (chain step [4]) is now buildable, and the recently-hardened SVD/QR (overflow-safe Jacobi,
noise-floor rank revealing) plus randomized rounding directly serve the per-step recompression that
QTT/MPS solvers depend on. **It did not, by itself, unblock the example.** *(Update: **Gap 1 is now closed** — the CFD-side encoding,
the immersed-body solver, and the surface observables have since been built and verified across the
`add-cfd-qtt-*` change series; see §4. The remaining gaps are 2–4.)* At the time of writing, the largest
gap was that **nothing in `deep_causality_cfd` touched a tensor network**: the generic primitives lived in
`deep_causality_tensor`; the CFD-side encoding did not exist.

For **Tier A** (the buildable demonstrator with surrogate physics) the remaining work is bounded and no
longer blocked on missing mathematics. For **Tier B** genuine open research remains (validated coupled
reacting-plasma CFD; the Bars-2T-exact-gravity + perturbative-aero coupling).

---

## 2. State of the art (mapped to the four axes)

### Axis 1 — Tensor-network flowfield (step [4])

The field has moved onto the flagship's thesis since the original note was written:

- **MPS Simulation of Reacting Shear Flows** — Pinkston et al., arXiv:2512.13661 (Dec 2025). The direct
  precedent: matrix-product-state / tensor-train for a *reacting* flow, not just turbulence.
  <https://arxiv.org/abs/2512.13661>
- **Tensor networks for turbulence probability distributions** — Gourianov et al., *Sci. Adv.* 11 (2025);
  ~10⁶ memory and ~10³ compute reduction on a 5+1D reactive-flow PDF.
  <https://inspirehep.net/files/0ee2a95339cde99c2435a51ad0c6344a>
- The quantized tensor-train (QTT) incompressible Navier–Stokes lineage (Gourianov; Kiffner–Jaksch) is
  the building-block method: **QTT field + MPO operators + TT-cross for nonlinear terms + TT-rounding
  each step.**

### Axis 2 — Plasma / blackout (steps [2], [3])

- **Numerical simulation of air ionization in the RAM-C-II flight experiment** — *Fluid Dynamics* (Springer,
  2022); Park-2T electron density vs flight data. A modern, citable companion to the RAM-C anchor.
  <https://link.springer.com/article/10.1134/S0015462822100639>
- **Vibrational–electron heating: a thermodynamically consistent model** — arXiv:2506.11457 (2025), and
  **Impact of ion mobility on electron density and temperature in hypersonic flows** — arXiv:2410.12760
  (2024). Current refinements of the `T_ve` / electron-density physics that drives blackout onset.
- **Data-driven lookup-table reduction for hypersonic chemical nonequilibrium** — arXiv:2210.04269. The
  surrogate-table route that Tier A explicitly permits.

### Axis 2′ — Tensor networks *for* plasma kinetics (bonus, not in the original note)

The same tensor-train layer could later carry a kinetic plasma closure, not just neutral flow:

- **Quantized tensor networks for the Vlasov–Maxwell equations** — Ye & Loureiro, arXiv:2311.07756 (2024).
- **Dynamical tensor-train approximation for kinetic (Boltzmann) equations** — arXiv:2512.14950 (2025).
- **Tensor-network compression for fully spectral Vlasov–Poisson** — arXiv:2602.13092.

### Axis 3 — Trajectory / relativity

No single 2024 "GNSS-denied relativistic INS" paper; the relativistic-GNSS modelling is mature
(Schwarzschild-frame GNSS, arXiv:1003.5836) and the IERS clock/EOM terms are textbook — confirming the
§2 framing that this axis is **bias-correction engineering, not open physics**.

---

## 3. What the tensor train provides vs. what the SOTA method needs

The reacting-MPS / QTT-NS method depends on exactly this primitive set — all now present:

| SOTA method needs | In `deep_causality_tensor` now | Status |
|---|---|---|
| QTT / MPS field encoding | `CausalTensorTrain`, `from_dense`, QTT reshape | ✓ |
| MPO spatial operators | `CausalTensorTrainOperator`: `from_cores` (hand-build), `from_dense`, `identity`, `apply`, `compose`, `transpose`, `round` | ✓ |
| MPO operator algebra (assemble stencils) | `add` / `sub` / `neg` / `scale` — **added this session** (completes the algebra) | ✓ (new) |
| Nonlinear convective + chemical source | `hadamard`/`round`, `cross`, `apply_nonlinear` | ✓ |
| Recompression every step | `round` (+ randomized, + NaN-robust SVD/QR) | ✓ (just hardened) |
| Implicit step / linear solve | `solve::linear` (AMEn), `fit` (ALS), `eigen` (DMRG), `tdvp` | ✓ |

**Conclusion:** the primitive layer is **sufficient** for a Tier-A MPS flowfield. The operator algebra
was the one genuine hole found while drilling into Gap 1 — the MPO type had `compose`/`identity` but no
additive structure, so differential stencils like `(S₊ − S₋)/2Δx` could not be assembled. That hole is
now closed (`add`/`sub`/`neg`/`scale`, tested f64/Float106; spec change `add-tensor-operator-algebra`).
The shift operator `S₊` itself is hand-built via `from_cores` and lives in the CFD bridge, not the tensor
crate (see [`gap-one-cfd-tensor-bridge.md`](gap-one-cfd-tensor-bridge.md) §3.2).

---

## 4. Remaining gaps (the actual answer)

### Gap 1 — CFD ↔ tensor-network bridge — **CLOSED**

*(Original state: `deep_causality_cfd` used `CausalTensor` only as a flat `Vec`; zero MPS/MPO usage.)*

**Resolved across the `add-cfd-qtt-*` change series.** `deep_causality_cfd` now has: a QTT codec (1-D/2-D
field ⇄ MPS); finite-difference MPO assembly (hand-built shift operators + the stencil algebra); a
periodic 2-D incompressible Navier–Stokes tensor-train marcher (`QttIncompressible2d`) with spectral Leray
projection and nonlinear convection; an immersed body by Brinkman volume penalization (`QttImmersed2d`, a
smoothed mask MPS — no cut cells); the surface observables the flagship's step [4] reads — **drag/lift** as
the penalization-force tensor-train contraction and a **neutral wall heat flux** via a penalized passive
scalar; and the `CfdFlow::qtt_march` DSL wiring + TT-native diagnostics. Verified: 2nd-order convergence to
the analytic Taylor–Green vortex; no-slip + accuracy-vs-bond convergence on an immersed cylinder. The
headline numerical risks (singular periodic Poisson, nonlinear rank growth, mask rank) were resolved and
verified in code. **[CLOSED — solver core + immersed body + surface observables built and verified]**

The one remaining flagship deliverable that *touches* this bridge — **electron density** and a *reacting*
heat flux — is **Gap 2** physics, not Gap 1; the neutral thermal observable is the seam it plugs into.

### Gap 2 — reacting / ionized physics is absent in the CFD crate

No species transport, finite-rate chemistry, Park-2T vibrational–electron energy, ionization /
electron-density, or shock-capturing. The compressible Navier–Stokes *pointwise kernels exist but are not
integrated into the marcher*. Tier A's escape hatch (corridor note §7) is a **parametric Park-2T
ionization surrogate** → electron density → plasma frequency → blackout trigger — narrow and tractable,
but **not yet written**. The `PhysicsStage` coupling DSL is the right home (`IonizationStage`, `EosStage`)
and is already in place. **[holds under precondition: surrogate acceptable for Tier A]**

### Gap 3 — trajectory axis is a proof-of-concept skeleton (matches corridor seam §6)

`hypersonic_2t/model.rs` has a "simplified for demo" conformal embedding (`data[16] = sqrt(x²+y²+z²)`), a
hand-set generator, and a `correct()` that is a literal no-op stub — no 6D measurement update.
`grmhd/model.rs` uses hardcoded proxy curvature (`g_00 = -0.9`, `ricci = g·-0.1`). Both are honest
skeletons, not engines. Carrying the flagship needs the real conformal lift, a genuine 6D filter update,
a relativistic-timing causaloid (IERS terms), and the **2T-exact-gravity + perturbative-aero coupling**,
which is correctly named open research — *not* something the tensor train touches. **[open]**

### Gap 4 — EPP composition glue (substrate present, flagship wiring absent)

`CausalFlow` / `PropagatingEffect` (counterfactual `continue_with`), the `grmhd` metric-selection coupling
pattern, the CFD `Coupling` / `PhysicsStage` seam, and an Effect Ethos layer all exist as working
substrate. The regime classifier (Knudsen + ionization + GNSS state), the safety-gate wiring, and the
provenance log are composition work — not missing primitives. **[holds under precondition: example wired]**

---

## 5. Bottom line and smallest next step

- **Did the tensor train remove a gap?** Yes — the one that made step [4] aspirational. The
  flowfield-compression axis is now primitive-complete and the SOTA reacting-MPS method
  (arXiv:2512.13661) maps cleanly onto what we have.
- **Can the example be built now?** Closer. ~~(1) a CFD→QTT/MPO bridge + a small MPS rollout (Gap 1)~~ —
  **done** (the bridge, the immersed-body solver, and the drag/heat surface observables are built and
  verified). Tier A now needs: (2) a parametric Park-2T / ionization `PhysicsStage` surrogate (Gap 2), and
  (3) wiring the existing skeletons + Ethos gate + provenance (Gap 4). Neither is blocked on missing
  mathematics.
- **Tier B** retains genuine open research: validated coupled reacting-plasma CFD, and the
  Bars-2T-exact-gravity + perturbative-aero coupling — keep labelled **[open]**.

**Smallest honest slice that proves the thesis:** a Tier-A vertical slice — quasi-1D reacting flow as a
QTT/MPS rollout (new tensor train), a parametric ionization surrogate feeding a blackout trigger, 2–3
`continue_with` bank-angle branches, the Effect Ethos corridor gate, and provenance — leaving the
trajectory axis at its current skeleton fidelity and labelling it so.

Gap 1 is the critical path; its dedicated closing plan is in
[`gap-one-cfd-tensor-bridge.md`](gap-one-cfd-tensor-bridge.md).

---

## 6. Related

- [`../plasma-blackout-corridor.md`](../plasma-blackout-corridor.md) — the flagship specification this
  analysis measures against.
- [`gap-one-cfd-tensor-bridge.md`](gap-one-cfd-tensor-bridge.md) — SOTA methodologies for closing Gap 1.
- `deep_causality_tensor` tensor-network layer — the primitives Gap 1 builds on.
- `examples/avionics_examples/hypersonic_2t/`, `examples/physics_examples/grmhd/` — the skeletons of
  axes 3 and 4.
