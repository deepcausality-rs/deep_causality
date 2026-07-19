## ADDED Requirements

### Requirement: Rank of the plume-imprinted layer, Cartesian vs blend metric

A self-verifying study, `studies/qtt_rank_plume/`, SHALL march the plume-imprinted compressible
layer and record its peak bond dimension (`max_bond`) across a thrust-coefficient sweep, in
Cartesian coordinates and under the blend-metric coordinate dial (the `qtt_blend_metric`
lineage), gating that the imprinted layer stays under the run's bond cap in at least one
coordinate. The study MUST print the per-point bond series for both coordinates so the
colliding-shock rank behavior — unmeasured until now — is recorded even when it passes.

#### Scenario: The imprinted layer's rank is measured, not assumed

- **WHEN** the study sweeps C_T and records `max_bond` per point in both coordinates
- **THEN** the output carries both bond series, and the gate passes only if at least one
  coordinate holds the imprinted layer under the cap across the sweep

### Requirement: Fork economics on the plume-coupled state

The study SHALL pause the plume-imprinted march mid-run, fork it, and continue a small
purposeful throttle roster — coast (zero), two candidates straddling the sign-flip band, one
nominal, one high — each branch's intervention published as `"commanded_throttle"` into its
branch world so each branch's forcing derives from its own throttle. It MUST record three
measurements: (a) **fork structure** — every branch shares the paused fluid and field by
reference at fork time (the O(1) copy-on-write contract), a hard gate; (b) **per-branch step
cost** — the wall-clock ratio of a branch continuation step to an unforked trunk continuation
of the same length, recorded and gated against a band pinned at the first measured run; (c)
**post-fork bond growth** — every branch's `max_bond` through the continuation stays under the
run's cap, a hard gate. Branch flow observables MUST spread across the roster (branches with
different throttles produce different imprinted fields), the foil to the corridor's
branch-invariant flow columns.

#### Scenario: The fork is O(1) and shared at the pause

- **WHEN** the paused plume-coupled march is forked for the roster
- **THEN** every branch shares the paused state and field by reference before its first step,
  and the fork setup performs no tensor copy

#### Scenario: Branches stay under the bond cap

- **WHEN** every branch continues from the shared fork under its own throttle
- **THEN** no branch's bond dimension exceeds the run's truncation cap through the
  continuation, and the per-branch step-cost ratios land inside the pinned band

#### Scenario: The intervention feeds back into the flow

- **WHEN** the coast branch and a high-throttle branch complete their continuations
- **THEN** their imprinted-layer observables (contracted axial force, bond series) differ
  measurably — the intervention is coupled to the flow, not along for the ride

### Requirement: Either outcome is a recorded result

The study SHALL exit nonzero only on hard-gate failures (fork sharing broken, bond cap
exceeded in both coordinates, roster branch errored); a degraded-but-measured outcome (e.g.
step-cost ratio outside a comfortable band, rank viable only under the blend metric) SHALL be
recorded in the printed output as a finding for the verdict, not masked as a pass or inflated
into a failure. The design intent is measurement: if the plume-coupled fork degrades, the
trace says where, on a case cheap enough to diagnose.

#### Scenario: A degraded measurement still completes

- **WHEN** the fork economics measure poorly but no structural invariant breaks
- **THEN** the study completes with the degraded numbers printed and flagged, and the verdict
  note — not the study binary — makes the go/no-go call
