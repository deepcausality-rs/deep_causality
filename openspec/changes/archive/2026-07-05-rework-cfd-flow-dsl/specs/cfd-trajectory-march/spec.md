## ADDED Requirements

### Requirement: One march verb over any marchable configuration

`deep_causality_cfd` SHALL expose the trajectory level through `CfdFlow::march(config)`, generic
over `C: Marchable`, replacing the five family-specific entries. `Marchable` SHALL carry a GAT
pipeline associated type so the family-specific pipeline stays hidden behind the facade, and a
`march()` producing a `Report`. The uncoupled config families SHALL implement `Marchable`
directly; the coupled path SHALL produce a `Coupled` wrapper (see the coupling requirement) that
itself implements `Marchable`, so one verb covers both.

#### Scenario: Different config families march through one verb

- **WHEN** a duct config and a 2-D compressible march config are each marched
- **THEN** both go through `CfdFlow::march`, and the caller never names the family pipeline type

#### Scenario: The retired entries no longer exist

- **WHEN** a program references `qtt_march`, `compressible_march`, `duct_march`, or
  `uncertain_march`
- **THEN** the program fails to compile, because those entries are retired in favor of `march`

### Requirement: Named-stage march builder

The march builder SHALL replace the positional `run_until(coupling, field, trigger, t0, predicate)`
with named stages: an optional `alternate(world)`, an optional `save_log(path)`, a `couple(stack)`
for coupled families, a `from(state)`, and a terminal `until(event)` (yielding a pause), `run()`,
or `run_for(steps)` (yielding a report). The blackout trigger SHALL fold into the coupling
description and the start clock into the march state.

#### Scenario: A leg reads as named stages

- **WHEN** a coupled leg is written
- **THEN** it reads `CfdFlow::march(&world).couple(stack).from(state).until(event)` with no
  positional argument list

### Requirement: One march state, two transports

`deep_causality_cfd` SHALL define `MarchState` as the single type that a pause exports
(`pause.state()`), that `from(state)` accepts to resume, and that the checksummed snapshot
stores and loads. A round trip through either transport SHALL be bit-identical at every
supported scalar: resuming from an in-memory state and resuming from a saved-then-loaded state
SHALL produce identical reports.

#### Scenario: Disk resume equals in-memory resume

- **WHEN** a march is paused, its state saved and reloaded, and continued; and separately the
  same pause is continued from its in-memory state
- **THEN** the two continued reports are identical bit-for-bit

### Requirement: The fork is O(1) and history-sharing

A pause SHALL provide `fork()` yielding a branch that shares the paused state and field by
reference (copy-on-write), a singular `continue_with(world, steps)` producing one continued
report, and a batch `continue_branches(worlds, steps)` running concurrently and returning
reports in world order. Branches SHALL share history with their siblings bit-identically up to
the pause.

#### Scenario: Forked branches share history to the pause

- **WHEN** two branch worlds are continued from one fork
- **THEN** each branch's state up to the fork point is identical to the pause's, and divergence
  occurs only after it
