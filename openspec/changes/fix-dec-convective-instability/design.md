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

If (1): symmetrize the convective operator in the M-inner product —
evaluate the skew part `½(C(u,·) − C*(u,·))` where `C*` is the
M-adjoint, folded into the compiled stencil tables the same way the cup
antisymmetrization already is, with the generic path corrected
identically so the equivalence gates stay meaningful. If (2): tighten
the per-stage tolerance floor for marches (config, not code). If
(3)/(4): the budget evidence dictates; amend this design before
implementing.

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
