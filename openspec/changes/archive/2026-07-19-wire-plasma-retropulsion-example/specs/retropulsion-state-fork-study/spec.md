## ADDED Requirements

### Requirement: The centerpiece forks the marched, plume-coupled state

The counterfactual centerpiece SHALL be the event form of the study grammar — `fork(&pause)` →
`branch(f)` → `continue_for(steps)` — taken at a pause placed *inside* the burn, so that every
branch resumes the shared, plume-imprinted marched state copy-on-write rather than re-flying a
freshly seeded world. Each branch world MUST differ from its siblings by exactly one published
intervention, its `"commanded_throttle"`, so that each branch's intervention feeds back into its own
plume geometry and its own drag through the model, step by step. The in-flight **drag authority in
every branch is the cited A0 Jarvinen–Adams correlation** carried by `PlumeObstruction`; the marched
imprint is state realism and is never the drag closure. This is the depth the M1 verdict's decision
table supports in both directions: the verdict pivoted the drag authority to A0 because a field
imprint does not carry a faithful decrement at this fidelity, and separately measured the state-fork
mechanics green.

#### Scenario: Branches share the paused state and diverge on their own intervention

- **WHEN** the burn march is paused and forked into the throttle roster
- **THEN** each branch resumes from the shared paused state, publishes only its own commanded
  throttle, and its flow observables and drag decrement follow that throttle rather than the trunk's

#### Scenario: The drag decrement in every branch comes from the correlation

- **WHEN** a branch's preserved-drag fraction is read for scoring
- **THEN** it is the value `PlumeObstruction` derived from the cited correlation at that branch's
  thrust coefficient, not a quantity contracted from the branch's evolved field

### Requirement: The throttle roster is small, purposeful, and on-axis

The branch roster SHALL consist of a coast branch at zero throttle, two or three candidates
straddling the drag sign-flip band, one nominal branch, and one engine-degraded branch at a fixed
fractional thrust. Every branch MUST stay **on-axis and inside the Cordell–Braun validated
envelope**: no angle of attack, no thrust vectoring. Off-axis operation leaves the model's validated
envelope, and a surprising result there would be unattributable — real physics and model
extrapolation would be indistinguishable — so the study stays where the physics is trustworthy and
names off-axis SRP as an upgrade rather than scope. Scoring MUST be trajectory-derived (terminal
miss to a shared aim point, propellant consumed, peak loads), with the committed rule pinned as a
constant, and MUST commit no finer than the navigated state supports.

#### Scenario: The roster straddles the sign-flip band

- **WHEN** the branch roster is built
- **THEN** it contains a zero-throttle coast branch and candidates on both sides of the correlation's
  predicted sign-flip band

#### Scenario: No branch leaves the validated envelope

- **WHEN** any branch world is constructed
- **THEN** its intervention is a throttle magnitude only, with no angle-of-attack or thrust-vector
  term

### Requirement: Three witnesses gate the coupling, each stating its own authority

The study SHALL gate three coupling witnesses. Gate **(4a) flow spread** requires the per-branch
flow observables to spread across branches beyond a pinned threshold, with the corridor's
branch-invariant flow columns as the explicit foil. Gate **(4b) sign-flip found** requires net
deceleration versus throttle to be non-monotone across the roster, with the band located within
tolerance of the correlation's prediction — and its rendered detail MUST state what it tests under
the A0 depth: that the correlation's non-monotonicity **survives trajectory integration** into
decision-relevant outcomes rather than being washed out by the thrust term and the mass depletion.
It MUST NOT be worded as an independent flowfield reproducing Jarvinen–Adams, because under this
depth the trajectory outcome is downstream of the correlation. Gate **(4c) coupling load-bearing**
requires each branch's trajectory divergence to differ measurably from a frozen-drag prediction —
the same thrust schedule with the drag held at its fork value — which is the witness that isolates
whether the throttle-to-drag coupling is load-bearing or the flow was along for the ride.

#### Scenario: The branch flow columns are not invariant

- **WHEN** the per-branch flow observables are compared across the roster
- **THEN** they differ beyond the pinned threshold, unlike the corridor's bank branches whose flow
  columns agreed to three digits

#### Scenario: The sign-flip gate reports what it tested

- **WHEN** gate (4b) renders its detail line
- **THEN** the line names the A0 correlation as the source of the non-monotonicity and states that
  the gate tests its survival through trajectory integration

#### Scenario: The frozen-drag foil separates

- **WHEN** each branch's divergence is compared against a same-thrust-schedule prediction with drag
  frozen at the fork value
- **THEN** the two differ beyond the pinned threshold

### Requirement: Fork economics regress the M1 measured bands

Gate **(4d) fork economics** SHALL regress M1's O(1) finding on the plume-coupled state rather than
re-deriving it: every forked branch MUST be witnessed as having entered by reference, sharing the
paused fluid state and coupled field, and each share MUST be witnessed as genuinely shared rather
than solely owned. The post-fork peak bond dimension MUST stay under the cap through every branch
continuation (M1 measured 16, flat, under a 32 ceiling), which gate (7) carries. A regression is a
measured finding to report, not a band to widen.

The witness is a **typed record on the branch report**, not a call on a fork handle. M1 measured the
structure directly on a `CarrierFork`, but the study grammar lowers `branch` onto
`CarrierPause::continue_with`, which never builds one — so `shares_fluid_with` / `shares_field_with`
are unreachable from a study, and gating through them would be impossible rather than merely
awkward. The carrier therefore records the facts off the shares actually handed to each branch and
attaches them to the `Report`; a report from an unforked march carries no record, which fails the
gate rather than defaulting true.

A live reference count above one is the load-bearing half of the witness. Sharing flags alone would
still hold for a branch that owned the only copy, so the count is what distinguishes a share from a
handoff.

**Not carried here: the per-branch continuation cost ratio.** M1's committed artifact records
0.67–1.04× the unforked trunk against a 2.0× band, and that band stays with M1's study, which times
each continuation at the caller. This example cannot: the study runs its branches internally and
`reduce` sees a `Report`, which carries no timing, and the carrier crate is `no_std` and cannot time
internally. Gate (9) bounds the whole descent's wall clock instead. Regressing the ratio would need
per-branch timing plumbed through the study grammar — a design change, recorded here rather than
silently dropped.

#### Scenario: The fork shares rather than copies

- **WHEN** the burn pause is forked across the roster
- **THEN** every branch report records that the paused fluid state and coupled field entered by
  reference at a live reference count above one, and the marched tensor train is not deep-copied

#### Scenario: An unforked report cannot pass the gate

- **WHEN** a report that did not come from a fork is scored as a branch
- **THEN** it carries no fork-economics record and fails gate (4d) rather than defaulting to true

#### Scenario: Post-fork rank stays under the cap

- **WHEN** every branch has run its continuation
- **THEN** the peak bond dimension observed through the continuations stays under the configured cap

### Requirement: Branch continuations run in parallel through the existing seam

Branch continuations SHALL run concurrently through the existing fork-join seam that
`continue_for` already lowers onto — `continue_branches`, which is `scoped_map` over the branch
worlds — and MUST NOT introduce a bespoke parallel path. Results MUST be bit-identical to a serial
run: the branches are data-independent by construction, sharing the trunk read-only after the O(1)
fork and never writing to one another, and each branch's march remains its own serial operation
sequence, so per-branch determinism and the earned bands are untouched. Wall time for the roster
therefore falls from the sum of the branches to roughly the slowest branch.

#### Scenario: Parallel and serial agree

- **WHEN** the roster is continued with the crate's `parallel` feature enabled and disabled
- **THEN** the per-branch reports and every gated witness are identical

### Requirement: Every branch carries the alternation marker, read from the report

Gate **(4e) audit trail** SHALL require every forked branch and every belief-counterfactual world to
carry the `!!ContextAlternation!!` provenance marker naming the world it is a counterfactual of,
with the baseline binding unmarked when it appears in the case axis. The gate MUST read the marker
from each run's `report.effect_log()` and MUST NOT read it from disk files: the event-fork path has
no audit-sink plumbing today — the campaign's `save_log` is consumed only by the origin-form path,
and the continued-segment driver never flushes to a sink — so branches produced by
`fork → branch → continue_for` write no on-disk log.

#### Scenario: Every branch names its baseline

- **WHEN** the branch reports are reduced
- **THEN** each carries the alternation marker naming the world it replaced, and the gate passes on
  their presence

#### Scenario: The audit is read from the report, not the filesystem

- **WHEN** the audit-trail gate runs
- **THEN** it inspects the effect log carried on each report and does not require any branch log file
  to exist
