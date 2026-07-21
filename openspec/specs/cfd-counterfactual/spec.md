# cfd-counterfactual Specification

## Purpose
TBD - created by archiving change rework-cfd-flow-dsl. Update Purpose after archive.
## Requirements
### Requirement: Two counterfactual forms, both marked

The grammar SHALL provide both counterfactual forms. The origin form: `Cases::baseline(origin)`
declares a validated baseline world, and `Counterfactual::alternate(f)` binds each case to a
world alternated from that baseline, flown from scratch. The event form: `Cases::fork(&pause)`
declares a shared fork point, and `ForkStudy::branch(f)` binds each case to a branch world
resuming the pause bit-identically, continued by `continue_for(steps)`. In both forms every
bound counterfactual world SHALL carry the `!!ContextAlternation!!` provenance marker naming
what it is a counterfactual of; the baseline itself, when present in the case axis, SHALL bind
unmarked.

#### Scenario: An origin counterfactual carries its marker

- **WHEN** a weather condition is alternated from the standard-day baseline
- **THEN** its world carries the alternation marker naming the baseline, and a gate can demand
  the marker's presence

#### Scenario: A forked branch resumes the pause bit-identically

- **WHEN** a bank-angle branch is forked from the blackout-onset pause
- **THEN** the branch resumes the pause's state bit-identically and carries the alternation
  marker for its commanded world

### Requirement: alternate and branch are phase-gated

`alternate` SHALL exist only after `baseline` (on `Counterfactual`), and `branch` SHALL exist
only after `fork` (on `ForkStudy`). `continue_for` SHALL exist only on the event-form branched
phase; origin-form worlds SHALL march, not continue. A program mixing these does not compile.

#### Scenario: branch without a declared fork does not compile

- **WHEN** a program calls `branch(f)` without first calling `fork(&pause)`
- **THEN** the program fails to compile

### Requirement: Ensemble realizations

`Cases::ensemble(draws)` SHALL raise the sweep multiplicity so each case flies `draws` times
with the draw index threaded to the coupling, and `reduce_ensemble` SHALL receive each case's
whole draw set to compute means, scatter, and worst-case rows where the data is.
`reduce_ensemble` SHALL be available only on a study that declared `ensemble`.

#### Scenario: A draw set reduces to error bars

- **WHEN** a condition is flown with `ensemble(n)` deterministic receiver-noise draws and
  reduced with `reduce_ensemble`
- **THEN** the resulting row carries the mean and scatter over the `n` draws

### Requirement: The coupling seam is the couple verb

The grammar SHALL attach the multiphysics stack (flow, reacting plasma, regime classification,
navigation, envelope control, composed as data) through `couple(f)` where `f` receives the case
and the draw index, at both the campaign level (on `Configured`) and the trajectory level (on
the march builder). The draw index SHALL be 0 unless `ensemble` raised the multiplicity.

#### Scenario: The coupling sees case and draw

- **WHEN** a coupled ensemble study is run
- **THEN** each run's coupling stack is built from its case and its draw index, and no run
  shares another's coupling

### Requirement: A continued branch records what its fork cost

Every counterfactual branch the carrier continues SHALL carry a typed **fork-economics record** on
its `Report`: whether the paused marched state and coupled field entered the branch by reference,
and how many live references shared each at the moment the branch was set up. A report from a plain
march MUST carry no record, because nothing was forked and there is nothing to claim.

The state fork's whole claim is that a branch is **O(1)** — it shares the paused tensor and field
through `Arc` and takes a single copy-on-write clone at its first write, so a roster of N branches
costs one paused state rather than N copies. The record is that claim made checkable from a branch
report, so a study can regress it instead of trusting it.

The record MUST be taken from the shares **actually handed to the branch**, not asserted about them.
A path that ever deep-copies instead of sharing must therefore report that it did, and a study
gating on the record fails rather than passing on a stale claim.

Both continuation paths MUST record it: `CarrierPause::continue_with` — which the fan-out
`continue_branches` and the study grammar's `branch` both lower onto — and
`CarrierFork::continue_march`. The existing `shares_fluid_with` / `shares_field_with` methods on a
fork handle remain, but they are unreachable from a study, which never builds a `CarrierFork`; the
record is what makes the property observable from where studies actually run.

A live reference count above one is the load-bearing half. The sharing flags alone would still hold
for a branch that owned the only copy, so the count is what distinguishes a share from a handoff.

#### Scenario: A continued branch reports its fork as O(1)

- **WHEN** a branch is continued from a pause through either continuation path
- **THEN** its report carries a fork-economics record showing both halves of the paused state
  entered by reference at a live reference count above one

#### Scenario: A fan-out shares one paused state across every branch

- **WHEN** a roster of branch worlds is continued from one pause
- **THEN** every branch report records an O(1) fork, so the roster costs one paused state rather
  than one copy per branch

#### Scenario: A plain march claims nothing

- **WHEN** a report is produced by a march that was never forked
- **THEN** it carries no fork-economics record

