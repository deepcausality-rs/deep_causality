## ADDED Requirements

### Requirement: Immersed-cylinder validation constrains the reported drag

The `qtt_cylinder_verification` gate set SHALL include at least one gate that constrains the reported drag
coefficient against a parameter it actually depends on. No gate in the present set does: one cannot fail,
one is provably invariant under the parameter that dominates the answer, and one carries eleven orders of
margin.

Specifically the harness SHALL gate a **smoothing-width ladder** and an **η ladder** as first-class checks
alongside the existing bond ladder, and SHALL tighten `CONVERGENCE_BOUND` to the scale of the difference it
measures.

Measured behaviour that motivates this, recorded so the gates are interpretable:

- `C_d` moves **6.1×** with the mask smoothing width — 7.70, 12.33, 23.76, 35.81, 47.27 at 0.5, 1, 2, 3, 4
  cells — while the no-slip gate passes identically across that entire range.
- `C_d` is **non-monotone** in the penalization parameter — 17.39, 24.02, 26.25, 23.76, 21.40 at
  η = 0.128, 0.064, 0.032, 0.016, 0.008 — and is still drifting at the finest η, so no η → 0 limit is
  demonstrated. Establishing that limit is what would license calling the penalization integral a drag
  (Angot, Bruneau & Fabrie 1999).
- `CONVERGENCE_BOUND = 0.10` gates a measured successive difference of `1.89e-11`.

Where a ladder does not converge, the harness SHALL report the non-convergence as its result rather than
passing silently. Establishing the physical η envelope is out of scope here and is handled by the Phase 2
remediation; this requirement makes the harness capable of *detecting* the condition.

#### Scenario: Smoothing-width ladder is gated

- **WHEN** the harness runs its smoothing-width ladder
- **THEN** it reports `C_d` at each width and gates the trend, so a result that scales with a purely
  numerical parameter cannot pass unremarked

#### Scenario: Penalization ladder is gated

- **WHEN** the harness runs its η ladder
- **THEN** it reports `C_d` and the interior slip at each η and gates whether the sequence is converging,
  failing or explicitly reporting non-convergence when it is not

#### Scenario: Bond-convergence bound matches the phenomenon

- **WHEN** the bond ladder completes
- **THEN** the successive-difference bound is set at the scale of the measured differences rather than
  orders of magnitude above them, so a solver that had not saturated in bond would fail

#### Scenario: A parameter-dependent result cannot pass silently

- **WHEN** the reported `C_d` changes materially under a parameter the gate set covers
- **THEN** at least one gate responds to that change, and the harness output records which parameter moved
  the answer

#### Scenario: Cross-references state their configuration

- **WHEN** the harness prints the DEC isolated-cylinder cross-reference
- **THEN** the Reynolds number of both cases is shown — the QTT case runs at `Re = 37.7` against a DEC
  reference at `Re = 100` — and the value is marked as a disclaimed cross-reference, not a gate
