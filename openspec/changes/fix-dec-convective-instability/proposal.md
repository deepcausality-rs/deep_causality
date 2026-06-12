# Fix the DEC Navier–Stokes long-horizon instability

## Why

The TGV Re-1600 resolution ladder (2026-06-12, see
`openspec/notes/cfd/error-per-cost-vs-spectral.md`) found that the
unforced viscous march **gains kinetic energy** in the under-resolved
turbulent phase and eventually aborts on the CFL guard: onset t* ≈ 8 at
32³ (E grows 0.114 → 0.164, dissipation negative), abort t* ≈ 8.7 at
48³, abort t* ≈ 13.8 at 64³ (max|u| → 4.6). Energy growth without
forcing is unphysical: some discrete operator in the marched rate is
injecting energy. This gates every long-horizon claim the solver makes —
the dissipation-curve benchmark, the error-per-cost comparison, the
challenge entry, and long cavity runs.

## What Changes

A three-stage, evidence-gated process — diagnosis decides the fix, and
the change closes **only** if the verification criterion holds:

1. **Root cause.** Instrument the energy budget of the marched rate
   (per-term M-inner products against the state) and localize which
   discrete term injects energy on a destabilizing trajectory.
2. **Fix.** Apply the minimal correction the diagnosis names, preserving
   the equivalence-gate discipline (generic path is the oracle; every
   existing ladder rung must still pass).
3. **Verify.** The stability criterion of the new `dec-ns-stability`
   capability: monotone energy decay through the turbulent phase at
   every ladder resolution, no CFL aborts. If verification fails, the
   change does not close — the diagnosis loop reopens.

## Impact

- Affected specs: ADDED `dec-ns-stability`; the fix stage may touch
  `dec-ns-rate` / `dec-exterior-algebra` / `dec-stencil-operators`
  deltas once the diagnosis names the term (amended at that point, not
  speculatively now).
- Affected code: `deep_causality_physics` (rate instrumentation,
  possibly the fused assembly), `deep_causality_topology` (possibly the
  wedge/interior-product or stencil compilation), the TGV example
  (streaming CSV so failed runs keep their partial curves).
- Out of scope: upwinding/limiting (sacrifices the structure thesis),
  dealiasing-by-truncation (no spectral representation to truncate),
  IMEX time integration.
