<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `qtt_acoustic_precond` — does the split preconditioner de-risk the implicit step? (Res 6 / D10)

```bash
cargo run --release -p deep_causality_cfd --example qtt_acoustic_precond
```

**What it tests.** The second Tier-B make-or-break. The implicit-acoustic operator is
`A = I − Δt²·c(x)²·∂²`, backward Euler on the fast pressure mode, SPD because `−∂²` is positive.
Resolution 6 splits it `A = A₀ + A₁`, where `A₀ = I − Δt²·c̄²·∂²` is the **constant-coefficient**
core with a known low-rank inverse and `A₁ = −Δt²·(c²(x) − c̄²)·∂²` is the variable remainder. Two
claims follow: `A₀⁻¹` is **low-rank and resolution-stable**, and on a **smooth** sound-speed field
`‖A₀⁻¹A₁‖ < 1`, so the preconditioned solve `A₀⁻¹A = I + A₀⁻¹A₁` contracts. Together they convert
an unbounded question, "does AMEn converge?", into a measurable perturbation bound. Stiffness is
`s = Δt²·c̄²/Δx² = 8`, above 1, which is the acoustic CFL the IMEX step exists to remove.

**Findings (gated, exit nonzero on regression).**

| gate | measurement | result |
|---|---|---|
| **AC-A** | `A₀⁻¹·b` bond, smooth RHS | L=8 → **8** (res 2.0e-11); L=10 → **8** (res 1.1e-10) |
| **AC-B** | `ρ(A₀⁻¹A₁)`, smooth interior | **0.590** |
| **AC-C** | `ρ(A₀⁻¹A₁)`, captured `c`-jump | **0.872** |

- **The core inverts cheaply and stays cheap.** Bond 8 at both L=8 and L=10 is bounded and
  resolution-stable, so `A₀⁻¹` is a usable preconditioner with no AMEn-convergence gamble on the
  core itself.
- **On a smooth interior the preconditioned operator contracts.** `ρ = 0.59 < 1` by a comfortable
  margin, so the implicit step converges geometrically. That is the Res-6 claim, measured.
- **The captured jump is the hard part.** Across a sharp `c` jump `ρ` rises to **0.872**, 1.5×
  worse and heading toward the divergence threshold at 1.

**Conclusion.** The split plus closed-form core de-risks the implicit acoustic step. The AC-C
degradation is the honest counterpart rather than a failure: it is precisely why shock-**fitting**
(Res 5) pays twice. By keeping the interior smooth, fitting is what keeps the implicit solve cheap.
Analysis: `openspec/notes/plasma-blackout/gap-2/`.

**Caveats.** `ρ` is measured by power iteration on a model coefficient field, not on a live marched
solution. A value below 1 gives geometric convergence but not a rate that survives an arbitrary
shock strength; the 5× jump measured here is one point, and a stronger jump can cross 1. The AC-A
residuals are the AMEn solve tolerance, not an independent accuracy check.

See `output.txt` for the recorded reference output.
