# sp2r-constraint-projection Specification

## Purpose
TBD - created by archiving change add-plasma-blackout-corridor. Update Purpose after archive.
## Requirements

### Requirement: Projection onto the KS / Sp(2,R) constraint surface

`deep_causality_physics` SHALL provide a projection kernel returning the nearest constraint-satisfying state for
a regularized conformal/KS state that has left its gauge surface after a measurement update. It SHALL enforce
the KS bilinear constraint (equivalently the Sp(2,R) conditions `X·X = X·P = P·P = 0` in the Bars packaging),
realizing the "evolve freely, then project onto the constraint manifold" pattern the CFD Leray projection uses.
It SHALL be generic over the scalar (tested `f32`/`f64`/`Float106`), use `from_f64` literals, static dispatch,
and document + test a **fixed gauge** making the projection unique.

#### Scenario: Idempotent on an on-surface state
- **WHEN** a state already satisfying the constraint within tolerance is projected
- **THEN** it is returned unchanged to working precision

#### Scenario: Restores a perturbed state
- **WHEN** a small off-surface perturbation is added to a valid state and projected
- **THEN** the result satisfies the bilinear constraints within tolerance and is the nearest such state under the
  fixed gauge

#### Scenario: Constraint preservation across a predict + correct + denial cycle
- **WHEN** a state is advanced through predict → correct → GNSS-denial (predict-only) → fix-return, projecting
  after each correction
- **THEN** the constraint residuals stay within tolerance across the whole cycle
