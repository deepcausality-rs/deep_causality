## ADDED Requirements

### Requirement: The audit log is an opt-in verb, console-unchanged by default

`deep_causality_cfd` SHALL provide an `AuditLog` wrapping the effect log with an optional disk
sink (`LogSink`), and a `save_log(path)` verb at both the campaign level (on `StudyDef`, before
the cases) and the trajectory level (on the march builder). Without `save_log` the effect log
SHALL behave exactly as today (in-memory, console rendering); `save_log` SHALL attach the sink
and change nothing else about the run.

#### Scenario: A study without save_log is unchanged

- **WHEN** a study omits `save_log`
- **THEN** its console output and effect log behave exactly as before this change

### Requirement: Stepwise flushing with the abort-tail guarantee

When a sink is attached, every effect-log entry SHALL be appended and flushed to disk the moment
it is recorded, not batched at the end. If the process dies for any reason, the file SHALL end
at the last entry recorded before the death, so its tail identifies the failure point. A sink
write error SHALL fail the run at that point with the IO error; an audited run that can no longer
be audited SHALL NOT continue silently. Power-loss durability (`fsync` per entry) MAY be enabled
by an option and SHALL be off by default.

#### Scenario: A killed run leaves a tail

- **WHEN** an audited run is killed mid-march
- **THEN** its log file exists and ends at the last entry recorded before the kill

#### Scenario: A completed run's file equals its in-memory log

- **WHEN** an audited run completes
- **THEN** the file renders identically to the in-memory effect log, closed by the verdict summary

### Requirement: One thread, one file under fan-out

The main log file SHALL be the single source of truth up to any concurrent fan-out (fork, sweep,
ensemble). Under fan-out, each concurrently running branch SHALL write its own file, exclusively,
named from the main path plus the numbered fan-out round and the case name. No file SHALL be
written by two threads and no entries SHALL interleave. The main file SHALL record the fan-out
spawn naming every branch file and the rejoin naming each branch's outcome. The abort-tail
guarantee SHALL hold per file: a branch that dies leaves its own file ending at its last entry,
and the main file names which branch died.

#### Scenario: A fork writes one file per branch

- **WHEN** an audited counterfactual study forks N branches
- **THEN** N per-branch files are written, each by one thread, each named by round and case, and
  the main file names every spawn and every rejoin outcome

#### Scenario: A dead branch is isolated

- **WHEN** one branch of an audited fan-out dies mid-march
- **THEN** that branch's file ends at its last entry, the sibling files are complete, and the
  main file names the casualty
