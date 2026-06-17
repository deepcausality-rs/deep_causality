## Context

Unforced viscous incompressible flow satisfies `dE/dt = −ν‖∇u‖² ≤ 0`.
The semi-discrete DEC march preserves this iff every term of the
projected rate is energy-consistent in the M-inner product:

- convective: `⟨u, P(i_u du)⟩_M = 0` (skew),
- viscous: `⟨u, −νΔ_dR u⟩_M ≤ 0` (M-symmetric negative),
- projection: M-orthogonal (removes energy, never adds),
- RK4: not exactly conservative, but dissipative-leaning at these CFL
  numbers — it cannot explain sustained growth on its own.

The measured behavior (growth onset that moves later with resolution,
blow-up through the CFL guard) is the signature of a **sign-indefinite
discrete residue that the under-resolved regime amplifies** — but which
term carries it is an empirical question, not an assumption.

## Goals / Non-Goals

**Goals:** localize the energy-injecting term with a per-term budget on
a real destabilizing trajectory; apply the minimal structural fix;
verify monotone energy decay through the turbulent phase on the full
ladder; keep the generic-vs-stencil equivalence gates green.

**Non-Goals:** upwinding/limiting, spectral dealiasing, IMEX, any
accuracy-vs-reference claims (that is the successor benchmark note's
job, unblocked by this change).

## Decisions

### D1: Diagnosis before fix — the budget decides

Stage 1 produces, per step on a 32³ Re-1600 trajectory marched into the
growth regime, the M-inner products of the state against each rate
term (convective, viscous, body-force = 0 here) plus `dE/dt` itself.
The term whose cumulative contribution turns positive and tracks the
energy growth is the defect. Ranked hypotheses, each with its
discriminating signature:

1. **Convective skew-residue** (most likely): the cup-product
   ½-antisymmetrization gives `⟨α∧β⟩` cancellation in the continuum
   limit but is not exactly M-skew on the lattice; signature:
   `⟨u, conv(u)⟩_M > 0` growing with gradient pile-up.
2. **Projection residue at finite tolerance**: stage rates are
   projected to CG tolerance, not exactly; signature: budget closes
   when solves run at 1e-13 but not at 1e-10 (cheap A/B).
3. **Viscous sign/symmetry defect at grade 1**: would show as
   `⟨u, −νΔu⟩_M > 0` on specific modes; considered unlikely (Group B
   pinned M-symmetry) but the budget covers it for free.
4. **RK4/CFL interaction**: signature: halving dt delays onset
   proportionally; rules-in/out time integration in one run.

### D2: The fix is the minimal correction the diagnosis names

**Diagnosis (2026-06-12, task 1.3 — budget probe on the 32³ Re-1600
trajectory): hypothesis (1) confirmed.** The convective power
`⟨u, −i_u(du)⟩_M` is exactly zero on the smooth single-mode initial
state (−9e-16), turns positive as the spectrum fills (+0.59 at t* 3.1,
+28 at 7.9), and overwhelms the viscous sink (always properly negative)
at t* ≈ 8.5 — energy growth follows. Hypothesis (2) was ruled out a
priori on the torus (the projection is spectral-exact); (3) is
exonerated by the always-negative viscous column; (4) is ruled out by
the budget's semi-discrete nature — the positive convective power is a
property of the spatial operator at the visited states, so no time
integrator can rescue an anti-dissipative rate.

**The fix: M-skew-symmetrize the convective operator.** With
`B_u(v) := i_u(dv)` (linear in `v` at fixed advecting field), march the
skew part

```text
conv'(u) = ½ [ B_u(u) − B_u^{*M}(u) ],   B_u^{*M} = δ₂ ∘ A_u,
A_u = M₂⁻¹ T(u)ᵀ M₁
```

where `T(u)` is the actual discrete grade-2→grade-1 convective map at
fixed `u` (transport ∘ star-wedge chain). Then
`⟨u, conv'(u)⟩_M = ½(⟨u, B_u u⟩ − ⟨B_u u, u⟩) = 0` **identically** —
using only the exact discrete adjointness `δ = d^{*M}` (diagonal
masses) and the true transpose. Both halves are consistent
discretizations of the same continuum operator (`i_u` and `u♭∧` are
mutually adjoint in the continuum, `δ = d*`), so the order of accuracy
is preserved.

Realization per assembly:
- **Stencil path**: the chain `T(u) = Post ∘ W(·, u) ∘ Pre` is built
  from static tables; compile **static transposed tables** for the
  three stages once per manifold (the transpose of a gather is a
  gather over the reorganized triples), so `A_u` applies with the same
  streaming pattern and parallel safety as the forward chain.
- **Generic path** (the oracle, test-scale only): assemble `T(u)`
  column-by-column through the public `interior_product` and transpose
  with the mass weights — mathematically the exact same object, slow
  but unambiguous; the equivalence gate then pins the static transposed
  tables against the assembled transpose.

Known consequence accepted up front: the Couette/Poiseuille **exactness
results** (steady profiles at rounding) relied on the uncorrected
convective term being annihilated by the projector on x-uniform shear;
the correction's `B*` half has only the continuum identity (`dw(u,u) =
0`) behind it, so those steady states may shift to discretization-order
accuracy. If measured so, the wall-validation expectations are
re-derived per the audit discipline (order-gated, never
tolerance-loosened) — long-horizon stability outranks an accidental
exactness.

### D3: Closure is conditional on verification

The change closes only when the `dec-ns-stability` scenarios pass. If
the fix fails verification, the diagnosis loop reopens with the new
budget data; the change stays open. No partial landing: an
instrumentation-only landing is permitted (the budget diagnostic is
additive and valuable on its own), but the capability spec ships only
with the passing verification.

## Risks / Trade-offs

- [Exact skew-symmetrization changes the convective truncation error] →
  the existing TG convergence tables and inviscid-invariant rungs are
  the regression net; observed orders must not degrade.
- [The defect is in topology's shared operators and the fix shifts
  archived expectations] → equivalence gates and the wall/cavity suites
  run both feature configs before closure; any re-derivation follows
  the audit discipline (correct expectations, never loosen).
- [Multiple defects compound] → the budget is per-term and re-run after
  the fix; closure requires the *budget* clean, not just the energy
  trace.

## Open Questions

- Whether the skew-residue, if confirmed, is in the wedge, the
  dual→primal transport averaging, or their composition — the budget
  localizes to the term; a second instrumented split inside
  `i_u(du)` localizes to the stage.
