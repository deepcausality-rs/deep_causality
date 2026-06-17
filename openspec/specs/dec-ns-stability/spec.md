# dec-ns-stability Specification

## Purpose
TBD - created by archiving change fix-dec-convective-instability. Update Purpose after archive.
## Requirements
### Requirement: Energy-budget diagnostic of the marched rate
The solver stack SHALL provide an energy-budget diagnostic that
evaluates, for a given state, the M-inner product of the state against
each term of the unprojected rate (convective, viscous, body force) and
against the projected rate, so that the energy contribution of every
discrete operator is observable per step along a march. The diagnostic
SHALL be evaluable through both the fused stencil assembly and the
generic compositional assembly.

#### Scenario: Budget terms sum to the energy derivative
- **WHEN** the budget is evaluated along a marched trajectory
- **THEN** the sum of the per-term contributions matches the measured discrete `dE/dt` to solve tolerance

#### Scenario: Budget localizes the instability
- **WHEN** the budget runs on a destabilizing under-resolved trajectory (32³ Re-1600 TGV marched past the growth onset)
- **THEN** it identifies which term's cumulative contribution is positive and tracks the energy growth

### Requirement: Long-horizon energy stability of the unforced march
The unforced viscous march SHALL be energy-stable through the
under-resolved turbulent phase: kinetic energy is non-increasing (to
solve tolerance per step) and the march completes without CFL abort.
The acceptance ladder is the Re-1600 Taylor–Green case at 32³, 48³, and
64³ marched to t* = 14 — the configurations measured unstable on
2026-06-12 (growth onset t* ≈ 8 at 32³; CFL aborts at t* ≈ 8.7 and
≈ 13.8 at 48³ and 64³). This requirement gates closure: the change
SHALL NOT close while any ladder rung fails it.

#### Scenario: The measured failures are gone
- **WHEN** the Re-1600 TGV marches to t* = 14 at 32³, 48³, and 64³ (f64, the example's configuration)
- **THEN** every run completes without CFL abort and kinetic energy is non-increasing to solve tolerance at every step

#### Scenario: The energy budget is clean after the fix
- **WHEN** the budget diagnostic re-runs on the previously destabilizing 32³ trajectory
- **THEN** no term's cumulative contribution is positive beyond solve tolerance

#### Scenario: Existing validation is preserved
- **WHEN** the full validation ladder (Taylor–Green convergence tables, inviscid invariants, double shear layer, Couette, Poiseuille, cavity rung) and the stencil-vs-generic equivalence gates run after the fix
- **THEN** every rung passes with observed orders and gates unchanged

### Requirement: Failed example runs keep their evidence
The TGV example SHALL emit its time-series output incrementally, so a
march that aborts (CFL or solver error) still leaves the partial curve
up to the failing step.

#### Scenario: Aborted run leaves a partial curve
- **WHEN** an example march aborts mid-run
- **THEN** the rows up to the failing step are on disk/stdout

