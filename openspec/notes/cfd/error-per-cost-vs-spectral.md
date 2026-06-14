# Error-per-cost vs. pseudo-spectral — measured state and the instability that gates the claim

Date: 2026-06-12. Status: measurement note. The gating instability
(Finding 1) was diagnosed and fixed under the
`fix-dec-convective-instability` change — see the **Resolution** section
at the end; the error-per-cost benchmark is now unblocked.

Follow-up to an external review of the DEC solver's positioning against
pseudo-spectral codes.

## The reviewer's framing (accepted)

Wall-clock-per-run against a tuned pseudo-spectral TGV code is
structurally unwinnable and not the scorecard: a spectral projection is
one division by `k²` per stage; ours is a real solve buying exact
discrete divergence-freeness as an operator property. The defensible
claims are:

1. **Error-at-resolution**: projecting inside the RK stages removes the
   Chorin splitting bleed, so the `−dE*/dt*` curve at a given grid
   should track the DNS reference (van Rees et al. 2011, Re 1600,
   peak ≈ 0.0122 at t* ≈ 9) more faithfully than an equal-order
   split-step solver.
2. **Domain class**: after `add-walls-and-dec-stencils`, the same march
   runs no-slip cavities and channels — plain Fourier pseudo-spectral
   does not play there at all (Chebyshev/IB territory). The
   Ghia-validated cavity is as much a part of the answer as TGV.

One caveat to state before any reviewer does: DEC-on-cubical is
2nd-order; spectral converges exponentially on smooth fields. In the
TGV's smooth early phase, spectral wins error-per-DOF by construction.
The structure argument bites **at and after the dissipation peak**,
where under-resolution punishes non-conservative schemes — claim that
window, not the whole curve.

## Measured (2026-06-12, Apple Silicon, release, `--features parallel`, f64, dt = 0.2)

`dec_taylor_green_re1600 <grid> <t*>`:

| Grid | Horizon t* | Wall-clock | Outcome |
| --- | --- | --- | --- |
| 32³ | 10 | 4.5 s | completes; curve sane through t* ≈ 7 |
| 32³ | 14 | ~6 s | **destabilizes**: energy grows from t* ≈ 8 (E 0.1138 → 0.1636 by t* = 13.4, dissipation negative) |
| 48³ | 10 | 27.5 s (abort) | **CFL abort** at step 332 ≈ t* 8.7, max\|u\| = 4.5 |
| 64³ | 10 | 38.2 s | completes; −dE*/dt* = 0.0071 at t* = 10, still rising |
| 64³ | 14 | (abort) | **CFL abort** at step 705 ≈ t* 13.8, max\|u\| = 4.6 |

The 64³-to-t*-10 cost of 38 s makes the full ladder a coffee-break
artifact, not an overnight one (the stencil + spectral-projection +
star-memoization work of this change cycle).

## Finding 1 — the gating one: nonlinear instability in the turbulent phase

Every resolution eventually destabilizes at Re 1600, with onset time
increasing with resolution: 32³ at t* ≈ 8, 48³ at ≈ 8.7, 64³ between 11
and 13.8. Kinetic energy *grows* without forcing — the discrete
convective term is injecting energy once gradients pile up at grid
scale. This is the classic aliasing-type instability of
non-dealiased central/energy-inexact convection; the inviscid invariant
tests pass on smooth short horizons because the spatial energy residue
is small there — but it is **sign-indefinite**, and the under-resolved
turbulent regime finds the unstable sign.

Initially the 48³ failure looked like a Bluestein-path (non-power-of-two
rFFT) suspect; the onset-time pattern across 32/48/64 exonerates the
transform — it is the same regime-driven instability. (A targeted n = 48
spectral-projection equivalence test is still cheap insurance.)

**Consequence**: the error-at-resolution claim is gated on fixing this.
A solver that cannot march through the dissipation peak at 64³ cannot
claim the peak. Candidate fixes, in order of preference:

1. **Exactly M-skew-symmetric convection**: enforce
   `⟨u, P(i_u du)⟩_M = 0` *by construction* (symmetrize the compiled
   convective stencil against its M-adjoint — the same fold-the-fix
   pattern the cup antisymmetrization already uses). First measure the
   residue: evaluate `⟨u, conv(u)⟩_M` along a marched trajectory; if it
   correlates with the energy-growth onset, the diagnosis is confirmed
   and the symmetrization is the fix.
2. Time-step/integrator damping (RK4 is mildly dissipative but
   evidently insufficient) — weaker medicine, treats the symptom.
3. Upwinding/limiting — off the table; sacrifices the structure thesis.

## Finding 2 — minor: the TGV example loses the curve on failure

`dec_taylor_green_re1600` only emits its CSV after a successful run; a
CFL abort discards the partial history (the 64³ t* = 14 curve up to the
abort is lost). Stream the CSV per step (or buffer + flush on error) so
failed runs still produce evidence.

## Proposed experiments (the note's successor change set)

1. **Skew-residue diagnostic**: log `⟨u, conv(u)⟩_M` per step in the
   example; overlay against `dE/dt`. Confirms/refutes the aliasing
   diagnosis in one run.
2. **Convective symmetrization**: implement fix (1); re-run the ladder.
   Success criterion: 32³ marches to t* = 14 with monotone E decay and
   the 64³ curve through the peak.
3. **The reviewer's benchmark**: with stability fixed, overlay
   −dE*/dt* at 32³/64³/(96³) against the van Rees reference; report
   peak error and wall-clock per run — the error-per-cost figure.
   Verify the t*/ε normalization conventions against the reference
   data before claiming numbers (`references.md`).
4. **The splitting kill shot**: same grid, projected-rate march vs. a
   Chorin post-step variant, two curves on one plot — turns the
   "5–20 % inviscid bleed" development note into a figure.

## Posture for the challenge entry

Lead with the cavity (walls, Ghia-validated, exact no-slip ∩
divergence-free projection — a domain where pseudo-spectral defaults
don't exist), present TGV error-per-cost honestly *after* the
stability fix, and state the smooth-regime spectral advantage up front.

## Resolution (2026-06-12, `fix-dec-convective-instability`)

**Diagnosis.** The energy-budget diagnostic (per-term `⟨u, ·⟩_M` of the
marched rate) on the 32³ trajectory pinned the defect to the
**convective term**: its power is zero on the smooth initial state
(−9e-16) but turns positive as the spectrum fills (+0.59 at t* 3.1, +28
at 7.9) and overwhelms the always-negative viscous sink at t* ≈ 8.5.
The discrete `i_u(du)` was not energy-neutral — an aliasing-type
residue, sign-indefinite, amplified by the under-resolved regime.

**Fix.** Skew-symmetrize the convective term in the **vector slot**:
`conv'(u) = ½[G_ω u − G*_ω u]` with `ω = du`, `G_ω : x ↦ i_x ω`, and
`G*_ω` its exact M-adjoint (`M₁⁻¹ Wᵇᵀ Postᵀ M₁`, static transposed
stencil tables compiled once). `⟨u, conv'⟩_M = 0` identically. The
vector slot (not the form slot) is the correct one: the continuum
antisymmetry `ω(x, w) = −ω(w, x)` lives there, so the skew part is
full-strength consistent and **2nd order is preserved** (the form-slot
variant, tried first, halved the convergence order — caught by the
pointwise-oracle rung). The adjoint is pinned by the
`⟨G_ω x, w⟩_M = ⟨x, G*_ω w⟩_M` identity tests; the generic path carries
the same correction (assembled transpose) as the equivalence oracle.

**Verified.** The stability ladder (Re-1600 TGV → t* = 14, f64), the
exact configurations that failed above:

| Grid | Before | After (peak −dE*/dt*) |
| --- | --- | --- |
| 32³ | growth from t* ≈ 8 | clean, 0.0065 @ t* 10.6 |
| 48³ | CFL abort t* ≈ 8.7 | clean, 0.0086 @ t* 10.1 |
| 64³ | CFL abort t* ≈ 13.8 | clean, 0.0094 @ t* 9.5 |

Zero energy-growth steps at every resolution. The full validation
ladder, wall/cavity suites, and stencil-vs-generic equivalence gates
stay green in both feature configs (physics 1561, topology 1205); the
Couette/Poiseuille exactness results were **not** disturbed (the
correction is energy-neutral and the wall steady states held at their
existing tolerances). Finding 2 (streaming CSV) is also fixed: the TGV
example emits per step, so aborted runs keep their partial curves.

**Bonus — the error-per-cost figure is now real.** The dissipation peak
converges toward the van Rees reference (≈0.0122 at t* ≈ 9) in both
magnitude (0.0065 → 0.0086 → 0.0094) and timing (t* 10.6 → 10.1 → 9.5
→ 9.0) as resolution rises — the structure-preservation claim is
defensible because the solver now marches *through* the peak. The
remaining benchmark work (overlay vs DNS, the Chorin-splitting kill
shot) is unblocked and belongs to the successor change.
