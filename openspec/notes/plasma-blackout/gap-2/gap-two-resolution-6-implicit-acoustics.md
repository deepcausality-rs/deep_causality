<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Gap 2, Resolution 6 — the implicit acoustic step without betting on AMEn convergence

**What this is.** A TRIZ/ARIZ resolution of the second **make-or-break** Tier-B assumption: that `solve::linear`
(AMEn) **converges** on the variable-coefficient compressible acoustic operator. The acoustic CFL at micrometre
cells is brutal and is *acoustic*, not source, stiffness — orthogonal to the Tier-A LER cure
([design D3](../../changes/add-cfd-compressible-qtt-marcher/design.md)). The plan makes the fast pressure mode
implicit via AMEn, but *"AMEn convergence on the variable-coefficient compressible operator is unproven"* — a
node the design itself flags as a gated risk. This note removes the gamble: it replaces the iterative solve of
unknown convergence with a **closed-form, low-rank operator inverse** — the spatial-acoustic analogue of how
LER replaced a stiff source *solve* with a closed-form *exponential*.

It is the conditioning facet of the **Feature-Adaptive Coordinate**
([Resolution 7](gap-two-resolution-7-feature-adaptive-coordinate.md)), and it leans directly on the fitting of
[Resolution 5](gap-two-resolution-5-dynamic-rank-lever.md).

Honesty convention: **[holds]**, **[holds under precondition]**, **[open]**.

---

## 0. Frame

- **Key problem (no solution words):** advance the fast acoustic waves stably at a **convection-limited** `Δt`
  without an iterative solve whose convergence is not guaranteed.
- **System / main function:** the implicit acoustic sub-step (`solve::linear` / AMEn on the variable-coefficient
  acoustic operator); *to advance the acoustic (pressure/divergence) coupling implicitly so `Δt` is set by
  convection, not by the acoustic CFL.*
- **The constraint treated as fixed — the lever:** that the implicit operator is the **full variable-coefficient
  compressible acoustic operator**. What we make implicit **need not be the full operator — only the stiff
  part.**

---

## A. Reformulate (the ARIZ spine)

**A2 — Technical contradictions, both ways:**
- **TC-1:** make the **full** coupled operator implicit → unconditional stability (good), but AMEn convergence
  is **fragile and expensive** on an indefinite, variable-coefficient operator (bad).
- **TC-2:** keep everything **explicit** → trivially "converges" (good), but the micrometre **acoustic CFL**
  makes `Δt` impractically small (bad).

**A3 — Intensify.** Push the sound speed `c → ∞` (the stiff acoustic limit). The full implicit operator is
worst-conditioned *exactly* there — but its **stiff part is the constant-coefficient Laplacian** `∇·∇p`, whose
inverse on a `2^L` QTT grid is a **known closed-form low-rank MPO**. The extreme isolates a part that has an
*exact* inverse; the trouble is only in the *remainder*.

**A5 — Resources already present (no new substance):**
- The stiffness is **acoustic**, and the acoustic operator at leading order is a **constant-/slowly-varying-
  coefficient Laplacian**.
- A **constant-coefficient Laplacian / Helmholtz inverse is a known low-rank QTT MPO** (standard result) —
  precompute once, apply each step.
- The **fitted map** (Resolution 5) puts the **worst coefficient jump at the interface** (handled by exact RH),
  leaving the *interior* coefficients smooth and slowly varying.
- `eigen` (DMRG), `fit` (ALS), `tdvp` are available if a richer solve is ever needed.

**A7 — Smart Little People.** Each step the agents must undo a fast pressure wave. Solving the full tangled
operator by iteration might never settle. Instead they apply a **precomputed "anti-wave" stencil** — the
constant-coefficient inverse they built once and for all — and whatever small variation is left, **one cheap
correction sweep** mops up.

**Physical contradiction:** the acoustic update must be **implicit** (stable at the fast scale) **and must not
require an unbounded iterative solve** (robust). **Resolve by segmentation / scale separation:** split the
operator; give the stiff constant-coefficient core a closed-form inverse; treat the remainder as a **bounded
perturbation.**

→ This is the **spatial-acoustic analogue of LER**: Resolution 1 replaced a stiff source *ODE solve* with a
closed-form *exponential*; here we replace a stiff acoustic *linear solve* with a closed-form *operator inverse*.

---

## B. Solve — split + closed-form constant-coefficient inverse

Split the acoustic operator `A = A₀ + A₁`:

```text
A₀ = constant-coefficient (reference ρ̄, c̄) Laplacian/Helmholtz  — stiff, fast        → closed-form low-rank inverse MPO
A₁ = A − A₀ = variable-coefficient remainder                     — non-stiff / small  → explicit, or one correction sweep
```

- **`A₀` implicit** via its **analytic low-rank inverse MPO** — precomputed once; **no convergence question.**
- **`A₁`** treated **explicitly**, or with **one defect-correction sweep**.
- **If AMEn is kept**, `A₀⁻¹` is the **exact self-preconditioner**: AMEn then solves
  `A₀⁻¹A = I + A₀⁻¹A₁`, a **small perturbation of the identity**, which converges geometrically (Richardson)
  whenever `‖A₀⁻¹A₁‖ < 1`. The open *"does AMEn converge on the compressible operator?"* becomes the
  **well-studied** *"does Richardson on `I + small` converge?"* — yes, under a measurable bound.

> **TRIZ principles used:** **separation by scale / condition**; **#1 Segmentation** (split the operator);
> **#35 Parameter change** (freeze coefficients to a reference state for the stiff part); **#24 Intermediary**
> (the constant-coefficient inverse as a removable preconditioner). **Effects database:** constant-coefficient
> elliptic operators have **explicit QTT inverses** — a known result, not an invention.

---

## C. Verify & harvest

- **Physical contradiction removed, not compromised?** Yes — a stable implicit step with **no unbounded
  iteration**; the closed-form inverse carries the stiffness, the remainder is a bounded correction.
- **Only A5 resources?** Yes — the constant-coefficient inverse MPO and the fitted-smooth interior. No new
  substance.
- **Discharges the gated risk?** Yes — "pray AMEn converges" becomes "build the known inverse; bound the
  remainder."

**New harm — and how fitting pays it off.** A large density/sound-speed **jump at the shock** makes
`‖A₀⁻¹A₁‖` large *locally* — which would break the perturbation bound. **But the shock is fitted**
(Resolution 5), so that jump sits at a **coordinate interface handled by exact RH**, not inside the interior
Laplacian inverse. **The mechanism that controls rank also conditions the solve.** Residual: if the *smooth*
interior still has strong coefficient variation (a strong expansion fan), one correction sweep may be
insufficient → fall to **preconditioned AMEn** (now well-conditioned), then to the spec's **explicit small
`Δt`** (correct, just slower). **[holds under precondition: bounded interior coefficient variation]**

**Generalized method.** *Never make the full stiff operator implicit. Split off the constant-coefficient core,
invert it in closed form (a low-rank MPO), and treat the remainder as a bounded perturbation (one correction, or
a now-well-conditioned AMEn). Trade an iterative solver of unknown convergence for a closed-form inverse of
known structure.* The temporal twin is LER's "integrate the increment, not the rate."

**Inverse / scaling.** As interior coefficient variation → 0, the closed-form inverse is **exact in one apply**;
as it → ∞ at the shock, fitting **moves the jump to the interface**; the fallback ladder (defect-correction →
preconditioned AMEn → explicit `Δt`) degrades gracefully at every rung.

---

## Measured (study `qtt_acoustic_precond`, 2026-06-30)

Probed before the build. The constant-coefficient core `A₀ = I − β∂²` inverts at **bond 8, flat from `L=8` to
`L=10`** (low-rank and resolution-stable, residual `~10⁻¹¹`). The perturbation spectral radius `ρ(A₀⁻¹A₁)` is
**0.59 on a smooth interior** (`< 1`, so `I + A₀⁻¹A₁` contracts and the preconditioned solve converges
geometrically, the Res-6 claim) and rises to **0.87 across a captured ×5 sound-speed jump** (toward the
divergence threshold at 1). The reading: the **jump is the hard part, not the bulk**, so the fitting of
[Res 5](gap-two-resolution-5-dynamic-rank-lever.md), by keeping the interior smooth, is what keeps the implicit
step cheap. **[holds: closed-form core low-rank; smooth-interior preconditioner contracts]**

## Implemented (closed-form inverse, 2026-06-30)

The closed-form core inverse is now **built**, not merely measured densely. `AcousticCoreInverse`
(`deep_causality_cfd/src/tensor_bridge/acoustic_inverse.rs`) realizes `A₀⁻¹` directly: `A₀ = I − β∂²` factors
exactly through the cyclic shift as `A₀ = (s/ρ)(I−ρS₊)(I−ρS₋)` with `ρ = (1+2s−√(1+4s))/(2s) ∈ (0,1)`, so
`A₀⁻¹ = (ρ/s)(I−ρS₋)⁻¹(I−ρS₊)⁻¹` and each resolvent is the binary-doubling product
`Σ_{k<2^l} ρ^k S₊^k = Π_{j<l}(I + ρ^{2^j} S₊^{2^j})` — `O(l)` shift-applies, no iterative solve. `S₊^{2^j}` is the
existing shift on the high `l−j` bits (`lift_leading(shift_plus(l−j), j)`), so no new operator. The prefactor
`ρ/s` exactly cancels the two `1/(1−ρ)` resolvent gains (`s(1−ρ)² = ρ`), making the inverse **free-stream-exact**
— the property an AMEn-per-step solve loses to its residual tolerance, and the reason the marcher waited for this
rather than swapping in the AMEn prototype.

- **Gate 1 [holds]:** `A₀A₀⁻¹ = I` to round-off (residual `< 1e-9`) at bounded, resolution-stable bond
  (`≤ 16`, flat `L=8 → L=10`) — `acoustic_inverse_tests.rs`.
- **`AcousticImex1d` (Stage 3)** now advances the core with this inverse instead of `solve::linear`; all Stage-3
  gates (free-stream-exact step, stability beyond the explicit acoustic-diffusion number, conservation,
  positivity) hold.
- **2-D marcher (Stage 5)** uses the ADI form `AcousticCoreInverse2d = (I−β∂ₓ²)⁻¹(I−β∂ᵧ²)⁻¹` as its implicit
  acoustic-dissipation step (explicit convection + implicit dissipation, the 2-D analogue of `AcousticImex1d`):
  free-stream-exact and bounded past the explicit limit. The hyperbolic all-Mach acoustic-*flux*-implicit Euler
  scheme (lifting the true acoustic CFL, vs. the acoustic-*diffusion* limit) remains the open Stage-6 remainder.

## Verification gates (what a spec/PR must prove)

1. **Closed-form inverse:** the precomputed `A₀⁻¹` MPO satisfies `A₀ A₀⁻¹ = I` to round-off at bounded bond.
   **[holds: standard constant-coefficient QTT result]**
2. **Stability beyond the acoustic CFL:** the split step stays bounded at `Δt` set by **convection**, where the
   fully-explicit control diverges.
3. **Preconditioned convergence:** the spectral radius of `A₀⁻¹A₁` (measured on the **fitted interior**, jump
   excluded) is `< 1`, and the correction / AMEn converges in `O(1)` sweeps.
4. **Fallback ladder is wired:** when the perturbation bound is not met, the marcher degrades to preconditioned
   AMEn and then explicit `Δt` without changing results (only speed).

---

## Related

- [`tier-b-compressible-marcher.md`](tier-b-compressible-marcher.md) — the acoustic-CFL / IMEX requirement.
- [`gap-two-resolution-1-stiff-source.md`](gap-two-resolution-1-stiff-source.md) — the **temporal** analogue:
  closed-form increment instead of an iterative/explicit integration.
- [`gap-two-resolution-5-dynamic-rank-lever.md`](gap-two-resolution-5-dynamic-rank-lever.md) — fitting removes
  the worst coefficient jump from the interior operator solved here.
- [`gap-two-resolution-7-feature-adaptive-coordinate.md`](gap-two-resolution-7-feature-adaptive-coordinate.md) —
  the unifying mechanism this is the conditioning-facet of.
- `add-cfd-compressible-qtt-marcher/design.md` — D3 (IMEX), D10 (this), Stage 3.
- `deep_causality_tensor` — `solve::linear` (AMEn), reused as the *preconditioned* solver, not the bare one.
