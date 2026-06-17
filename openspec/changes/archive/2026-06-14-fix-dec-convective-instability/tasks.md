# Tasks

Closure is conditional: the change does not close while any 3.x
verification task fails — a failed verification reopens stage 1 with
the new budget evidence.

## 1. Root cause

- [x] 1.1 Implement the energy-budget diagnostic (per-term M-inner
      products of the state against the rate's terms and the projected
      rate), evaluable on both assemblies; budget-sums-to-dE/dt test
- [x] 1.2 Stream the TGV example's CSV incrementally so aborted runs
      keep their partial curves (dec-ns-stability, evidence requirement)
- [x] 1.3 Run the budget on the destabilizing 32³ Re-1600 trajectory;
      record which term injects energy; discriminate the ranked
      hypotheses (convective skew-residue / projection tolerance A-B at
      1e-10 vs 1e-13 / viscous sign / dt-halving RK4 check)
- [x] 1.4 Write the diagnosis into design.md (amend D2 with the named
      term and the localized stage); if the diagnosis is not (1),
      re-rank before any fix work

## 2. Fix

- [x] 2.1 Implement the minimal correction the diagnosis names, in both
      the generic operator path and the compiled stencil tables
      (equivalence gates stay meaningful)
- [x] 2.2 Re-run the stencil-vs-generic equivalence battery and the
      existing validation ladder rungs; re-derive any shifted
      expectations per the audit discipline (never loosen tolerances)

## 3. Verify (closure gate)

- [x] 3.1 Stability ladder: Re-1600 TGV to t* = 14 at 32³, 48³, 64³ —
      no CFL aborts, kinetic energy non-increasing to solve tolerance
      at every step
- [x] 3.2 Budget re-run on the previously destabilizing 32³ trajectory:
      no term cumulatively positive beyond solve tolerance
- [x] 3.3 Full regression: validation ladder + wall/cavity suites green
      in both feature configurations; fmt/clippy clean
- [x] 3.4 Update `openspec/notes/cfd/error-per-cost-vs-spectral.md`
      with the diagnosis, the fix, and the post-fix ladder numbers;
      prepare the commit message and ask the user to commit
