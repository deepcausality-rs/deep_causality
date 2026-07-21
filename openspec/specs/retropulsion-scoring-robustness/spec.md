# retropulsion-scoring-robustness Specification

## Purpose
Failure behaviour of retropulsion branch scoring when a witness is absent, negative, or non-finite.

## Requirements

### Requirement: An absent measurement fails rather than defaulting

Reading a branch witness SHALL return an error when the series is absent, and the error MUST travel
through the reduce closure's existing error channel. A missing preserved-drag fraction currently reads
as zero, which makes the drag-collapse gate pass; a missing mass currently floors at one kilogram,
which inflates every deceleration and makes the coupling gate pass harder.

#### Scenario: A missing fraction fails the run

- **WHEN** a branch report carries no preserved-drag fraction series
- **THEN** scoring returns an error naming the missing series, and no gate reports a verdict on a
  substituted value

#### Scenario: A missing or non-positive mass fails the run

- **WHEN** a branch report carries no mass series, or a mass at or below zero
- **THEN** scoring returns an error rather than substituting a floor

### Requirement: Single-cell scalars are read sign-preserving

A single-cell scalar SHALL be read at its first cell rather than reduced by a peak folded from zero,
so a negative value survives. The SRP kernel documents a negative preserved-drag fraction past a
thrust coefficient near two as the measured wake-type forebody force, and that negative branch is the
sign-flip physics the counterfactual exists to find.

#### Scenario: A negative preserved-drag fraction reaches the gates

- **WHEN** a branch flies at a thrust coefficient where the correlation returns a negative fraction
- **THEN** the recorded fraction is negative, and the drag gate evaluates it as such

#### Scenario: Field quantities keep the peak reduction

- **WHEN** a witness is a per-cell marched quantity rather than a single-cell scalar
- **THEN** it is reduced by peak over cells, since the inflow strip is held identical across branches

### Requirement: Non-finite scalars are rejected at the read

A non-finite witness SHALL be rejected where it is read, so no comparison downstream can panic on it.
The branch selection orders rows by a floating-point comparison whose unwrap panics on a
not-a-number, and the value it orders derives from an unchecked report series.

#### Scenario: A non-finite witness fails before selection

- **WHEN** a branch witness is not finite
- **THEN** scoring returns an error, and the branch selection never compares it

#### Scenario: Gate views are read without indexing

- **WHEN** a leg gate reads its row from a study view
- **THEN** it uses a checked first-element read, so an empty view yields a failed gate rather than a
  panic
