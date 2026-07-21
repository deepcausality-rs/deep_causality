## ADDED Requirements

### Requirement: Every gate reads a measurement that can differ from its threshold

A gate SHALL compare a value the run produced against a threshold the run does not determine, and each
gate MUST have a demonstrated failing input. A gate whose predicate is entailed by the construction of
its own input measures nothing, however plausible its message reads.

#### Scenario: Each gate has a demonstrated falsification

- **WHEN** the change is complete
- **THEN** every gate in both sequences has a test supplying an input on which it fails

#### Scenario: A gate message describes what the gate checks

- **WHEN** a gate reports its verdict
- **THEN** the message names only conditions the predicate evaluated, and claims no condition the
  predicate omits

### Requirement: The compression gate reads a measured bond

The compression gate SHALL read the recorded peak bond of the committed branch's final state. The
present gate compares the configured truncation cap against the range bounded by that same cap.

#### Scenario: A state exceeding the cap fails

- **WHEN** a branch's final state records a peak bond above the configured cap
- **THEN** the compression gate fails

### Requirement: The ignition gate checks the conditions the commit predicate does not guarantee

The ignition gate SHALL check the navigation mode and the navigated sigma against the table-sized
margin, and SHALL NOT re-assert the Mach band and dynamic-pressure window that the commit predicate
already requires. A commit entry can only exist for values inside those two intervals, so re-checking
them reduces the gate to the existence of the entry.

#### Scenario: A commit on a dead-reckoned state fails

- **WHEN** a commit is recorded on a navigation state that was not aided on that step
- **THEN** the ignition gate fails and names the navigation mode

### Requirement: The inheritance gate compares corridor witnesses

The corridor-inheritance gate SHALL compare the flown Acts 0 and 1 witnesses — blackout onset, exit,
dwell, dead-reckoning drift, and reacquisition — against the corridor example's recorded values. A
count of leg-boundary log lines is fixed by the number of march calls and carries no information about
inheritance.

#### Scenario: A changed corridor witness fails the gate

- **WHEN** an Act-1 witness differs from the corridor example's recorded value
- **THEN** the inheritance gate fails and names the differing witness

### Requirement: The integrity gate covers every leg

The integrity gate SHALL report the error state of all four legs, and each leg's error MUST be carried
into the gate rather than short-circuiting the run before the gates execute. The present arrangement
returns early on three legs, so the gate is live for one.

#### Scenario: A failing intermediate leg reaches the gate

- **WHEN** the supersonic burn leg captures a step error
- **THEN** the integrity gate fails and names that leg and its error text

### Requirement: The audit-trail gate reads the applied-alternation flag

The audit-trail gate SHALL read the typed applied-alternation flag from each branch report rather than
searching a rendered log for a marker substring, which also matches the carrier's refusal message.

#### Scenario: A refused alternation fails the gate

- **WHEN** a branch's alternation was recorded as not applied
- **THEN** the audit-trail gate fails

### Requirement: The touchdown gate is bounded on both sides

The touchdown gate SHALL bound the sensed descent rate above and below the commanded contact speed. A
one-sided upper bound admits an undershoot and admits a hover, which are the outcomes the gate's own
tracking claim says it detects.

#### Scenario: A hover at the sampled altitude fails

- **WHEN** the vehicle arrives at the touchdown floor at a descent rate far below the commanded
  contact speed
- **THEN** the touchdown gate fails

### Requirement: Every band records its measurement and whether it binds

Each pinned band SHALL carry the measured value it was earned from on the corrected run, and SHALL
state whether it binds. A band with an order of magnitude of headroom is a runaway detector, and its
docstring must say so rather than describing it as a regression gate.

#### Scenario: A superseded measurement is replaced, not annotated

- **WHEN** a band's cited measurement no longer matches the recorded run
- **THEN** the band is re-earned from the corrected run and the stale figure is removed
