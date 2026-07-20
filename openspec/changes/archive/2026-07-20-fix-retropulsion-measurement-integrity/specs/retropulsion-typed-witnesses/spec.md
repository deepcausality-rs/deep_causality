## ADDED Requirements

### Requirement: The ignition commit publishes typed witnesses

`ThrottleGuidance` SHALL publish the commit's step, sensed Mach, sensed dynamic pressure, and
navigated position sigma as field scalars at the moment it latches, in addition to the existing
provenance entry. A consumer must be able to read every commit witness without rendering or parsing a
log, and without depending on a message's wording or on a scalar's `Debug` formatting.

#### Scenario: A gate reads the commit without touching the log

- **WHEN** the corridor commits
- **THEN** the commit step, Mach, dynamic pressure and sigma are readable as field scalars carried
  across leg boundaries

#### Scenario: A message rewording does not change a gate's verdict

- **WHEN** the commit entry's wording changes
- **THEN** the published witnesses are unchanged and every gate reading them reports the same verdict

### Requirement: Leg re-seeds and regime transitions are counted, not grepped

A paused march SHALL expose the number of leg re-seeds it has accumulated, and the regime classifier
SHALL maintain a monotone transition counter as a field scalar. The classifier already computes
whether the regime key changed and currently discards that decision after logging it.

#### Scenario: The re-seed count is available from the pause

- **WHEN** a descent chains four march calls
- **THEN** the terminal pause reports three re-seeds through a typed accessor

#### Scenario: The transition counter increments once per genuine change

- **WHEN** consecutive steps cross a regime band and then remain inside the new band
- **THEN** the counter increments exactly once

### Requirement: An applied context alternation is distinguishable from a refused one

A continued branch's report SHALL carry a typed flag that is true only when the context alternation
was applied, and false on the carrier's refusal path. A substring search for the alternation marker
matches the refusal message as well, so a marker's presence does not establish that an alternation
took effect.

#### Scenario: A refused alternation fails the audit gate

- **WHEN** a fork is alternated on an errored run and the carrier records that the alternation was not
  applied
- **THEN** the report's applied flag is false and the audit-trail gate fails

### Requirement: The peak bond of a branch's final state is recorded

The carrier SHALL record the peak bond dimension of a report's final marched state, so a compression
gate reads a measured rank. A gate that compares the configured truncation cap against itself measures
nothing about the state.

#### Scenario: The compression gate reads a rank the run produced

- **WHEN** a branch's final state is re-quantized under the configured truncation
- **THEN** the recorded peak bond is that state's rank, and it can be below the cap

### Requirement: Fork economics are deterministic and complete

The recorded fork economics SHALL be reproducible across runs, and SHALL carry the trunk-relative
per-branch step cost and the post-fork bond growth alongside the sharing witnesses. A reference count
sampled inside concurrently running branches varies between runs, which makes the recorded artifact
undiffable.

#### Scenario: Two runs record identical fork economics

- **WHEN** a roster is forked and continued twice with the parallel feature enabled
- **THEN** the recorded economics columns are identical between the two runs

#### Scenario: The economics carry the cost measurements the study needs

- **WHEN** a branch continues from a fork
- **THEN** its report carries the step-cost ratio against the trunk and the bond growth over the
  continuation
