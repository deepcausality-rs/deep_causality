<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# ARIZ resolution — the two open risks of the QTT 2-D incompressible solver

**What this is.** The record of working the two flagged risks of OpenSpec change
`add-cfd-qtt-incompressible-2d` (the QTT 2-D incompressible Navier–Stokes solver) through the
ARIZ/TRIZ template (`ctx/ariz-template.txt`) **before** implementation. Both dissolved at reformulation
— ARIZ's "~50% reformulation" maxim held: neither needed the contradiction matrix.

Honesty convention: **[resolved]**, **[controlled]**, **[open]**.

---

## Risk 1 — AMEn Poisson convergence + the singular periodic Laplacian → **[resolved]**

**Solution-neutral problem.** The projection needs the pressure that makes the velocity
divergence-free, but the periodic Laplacian is singular (constant null space), so a direct solve is
ill-posed.

**Technical contradiction.**
- TC-1: solve the singular operator directly → correct projection, BUT non-unique / may not converge.
- TC-2: regularize with `ε·I` → converges, BUT perturbs the physics and adds a tuning knob.

**The reformulation that dissolves it (A7 / IFR).** The projection needs **`∇p`, not `p`**. The null
space of the periodic Laplacian is the *constant*, whose gradient is **zero** — so `∇p` is unique even
though `p` is not. The singular operator is a non-problem for the projection. (Kills TC-2: no
regularization needed.)

**Resources → Effects lookup (A5 / B6).** Two free resources are already present:
1. the constant null mode is known exactly (a bond-1 train in QTT), and `∇·u*` is automatically
   mean-zero (divergence of a periodic field) — so the system is **consistent** (RHS ⊥ null space);
2. the **periodic-grid spectral effect**: the Laplacian is **diagonal in the Fourier basis** with known
   eigenvalues `λ_k = −(2 − 2cos(2πk/N))/Δx²`.

**Resolution (Principle 28, "mechanics substitution," + Effects).** Replace the iterative AMEn solve
with an **exact spectral Poisson solve** for the periodic case: transform → per-mode division
`p̂_k = div̂_k / λ_k` with `p̂_0 = 0` (null space pinned **by construction**) → inverse-transform. No
iteration, no regularization, no convergence risk. Staging: Tier-A small grids dequantize →
eigen-solve → requantize (trivially correct); the scalable form is a **QFT-MPO** that keeps it in QTT.
`solve::linear` (AMEn) is retained for the *future* wall-bounded (non-diagonal) case — not used here.

**Verify (C1).** Physical contradiction *removed*, not compromised; uses only present resources (the
known null mode + the spectral effect); no new harm. The only residual is the QFT-MPO construction
*effort* (not a risk).

---

## Risk 2 — Nonlinear rank growth → **[controlled]**

**Solution-neutral problem.** The convective term `u·∇u`, formed by Hadamard products of trains,
inflates the bond `r → r²`; over many steps the rank can grow until the compression advantage is lost.

**Technical contradiction.**
- TC-1: keep rank high → accuracy preserved, BUT cost/memory blows up (compression defeated).
- TC-2: truncate hard → cheap, BUT truncation error injects energy / destabilizes.

**Intensify → IFR (A3 / A7).** rank→∞ is exact-but-useless; rank→1 is cheap-but-garbage. IFR: *the field
carries exactly the rank its physics needs, no more — truncation removes only noise, not physical
content.*

**Resources — the key one is the physics itself (A5).** Resolved/laminar flow is genuinely low-rank
(the whole premise, per Gourianov); the `r²` from Hadamard is *mostly spurious* — the product's true
rank is bounded by the physical rank, which `round` reveals. Free resources already built: the hardened
`round` (randomized + NaN-robust SVD/QR); the **fused `hadamard_rounded`** (compresses as it builds, so
`r²` is never materialized); and **diffusion** — Principle 22 "blessing in disguise": the `ν·∇²` term we
are already solving damps exactly the high-rank small scales.

**Resolution (Separation by scale + Principles 19/22).**
1. Round after every bilinear op via the fused `hadamard_rounded` — the `r²` blow-up stays transient and
   is never allocated.
2. Tie the round tolerance to the discretization error (no point resolving below the spatial-truncation
   floor) with a `max_bond` backstop — separation by scale.
3. Scope-bound the residual: for the Tier-A demonstrator (smooth, resolved Taylor–Green) the physical
   rank *is* low; the validation sweeps error-vs-bond to confirm the rank **plateaus**.
4. Escape hatch: **TT-cross** (`apply_nonlinear`) builds the term *at* a capped rank by sampling, never
   forming `r²`, if Hadamard+round is insufficient.

**The irreducible part → [open / Tier-B].** Genuinely unbounded rank only bites at high-Re / turbulence
(adaptive rank, non-MPS geometries) — explicitly out of scope for the demonstrator.

**Verify (C1).** Contradiction resolved by reformulation (growth is spurious + physically self-limiting),
resources-only (fused round, diffusion, tolerance), and the irreducible part correctly fenced to Tier-B.

---

## Harvest (C2 / C3)

- **Generalized win.** The spectral/QFT Poisson is a **reusable primitive** — the natural route for *any*
  periodic elliptic solve in the QTT layer, generalizing the "diagonalize in a transform basis" pattern
  the tensor crate's FFT already supports.
- **Method, stated.** *When an operator is singular but only a derived (gauge-invariant) quantity is
  needed, solve in the basis that diagonalizes it and zero the null mode — the singularity never
  materializes.* Its inverse — *regularize to remove the singularity* — is the worse path (adds bias and
  a knob), and ARIZ's resource-only check is what flags it as such.
- **Lesson (where the path diverged from the template).** Both problems were solved in Part A
  (reformulation); the matrix/principles in Part B were only a confirming lookup. The decisive move each
  time was naming the *constraint tempted to treat as fixed* — "we must solve for `p`" and "the Hadamard
  rank is what it is" — and discarding it.

## Related
- [`gap-one-cfd-tensor-bridge.md`](gap-one-cfd-tensor-bridge.md) — the Gap-1 plan this change advances.
- OpenSpec change `add-cfd-qtt-incompressible-2d` — design §Decisions/§Risks and the `qtt-projection`
  spec carry these resolutions.
- `ctx/ariz-template.txt` — the working template.
