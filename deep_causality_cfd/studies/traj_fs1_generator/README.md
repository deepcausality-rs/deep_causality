<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `traj_fs1_generator` — is the inverse-square core an exact constant-generator matrix exponential?

```bash
cargo run --release -p deep_causality_cfd --example traj_fs1_generator
```

**What it tests.** Gap-3 Resolution-3, de-risking Resolution-1's B1 "exact conformal core" (items
①/②). Resolution 1 asserts that the bound inverse-square (Kepler) trajectory equals
`ψ(s) = e^{G·s}·ψ(0)` for a **constant** generator `G` under a time reparametrisation, but it never
gives `G`. This study supplies the concrete, textbook realisation.

**Method.** The **eccentric-anomaly linearisation**, which is the 1-D essence of
Kustaanheimo–Stiefel / Levi-Civita regularisation (Stiefel & Scheifele, *Linear and Regular
Celestial Mechanics*, 1971). In eccentric anomaly `s = E`, the recentred perifocal coordinate
`Q = (a·cos E, b·sin E)` solves the unit-frequency harmonic oscillator `Q'' = −Q`, so the phase
state `ψ = (Q, Q')` advances by the **constant** 4×4 symplectic generator `Ω = [[0, I₂], [−I₂, 0]]`.
Physical time is the closed form `Δt = (M − M₀)/n` with `M = E − e·sin E`, Kepler's equation. Test
orbit: `a = 7038.0 km`, `e = 0.1323`, period 5876.0 s, `μ = EARTH_GM`.

**Findings (gated, exit nonzero on regression).**

| gate | measurement | result |
|---|---|---|
| **G1** | one-period closure `‖e^(Ω·2π) − I‖_max` | **2.665e-15**, exact periodicity |
| **G2** | generic matrix-exp vs closed form | **2.998e-15** |
| **G3** | max position error over a full orbit | **1.584e-8 m** = 2.251e-15·a |
| **G4** | semigroup law `‖e^(Ωs₁)e^(Ωs₂) − e^(Ω(s₁+s₂))‖_max` | **1.110e-16** |

- **The orbit closes to round-off.** The matrix exponential is an exact Kepler solver, not an
  approximation that happens to be accurate.
- **The "matrix exponential" is literal.** Generic scaling-and-squaring matches the closed-form
  cos/sin block, so nothing is hand-waved into the notation.
- **The trajectory matches an independent propagation.** Positions reconstructed from `e^{Ω·s}·ψ₀`
  agree with a Newton-solved orbital-element Kepler propagation to 2.3e-15·a over a full orbit.
- **One constant generator drives the whole flow.** The semigroup law holds to round-off, so `G` is
  genuinely `s`-independent.

**Conclusion.** The bound inverse-square core **is** an exact constant-generator matrix
exponential. Resolution-1's B1 holds, with the concrete generator `Ω` supplied. The production 3-D,
singularity-free, perturbation-ready form is **KS regularisation** (Stiefel–Scheifele), and the
heavier Bars `(4,2)` packaging is **optional, not required**, which simplifies Resolution 1.
Analysis: `openspec/notes/plasma-blackout/gap-3/gap-three-resolution-3-trajectory-axis.md`.

**Caveats.** 2-D perifocal, the planar essence, and bound orbits only. The parabolic, hyperbolic,
and full 3-D KS cases are the production generalisation and are not measured here. The independent
reference is an element propagation sharing the same `μ` and initial conditions, so G3 checks the
internal consistency of two formulations rather than agreement with an external ephemeris.

See `output.txt` for the recorded reference output.
