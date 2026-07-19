## ADDED Requirements

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
