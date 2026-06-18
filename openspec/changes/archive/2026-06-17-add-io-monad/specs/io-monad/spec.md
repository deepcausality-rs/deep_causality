## ADDED Requirements

### Requirement: A value-level, dyn-free IO effect interface

`deep_causality_haft` SHALL provide an `IoAction` trait describing a deferred input/output computation.
The trait SHALL expose an associated `Output`, an associated `Error`, and a `run(self) -> Result<Output,
Error>` method that is the ONLY operation permitted to perform a side effect. The IO abstraction SHALL
be realized with concrete combinator types (no `dyn`, no trait objects, no macros), mirroring the
`Arrow` algebra, so that composition is total and monomorphized.

#### Scenario: Construction and composition perform no side effect
- **WHEN** an `IoAction` is constructed and composed with `map` / `and_then` but `run` is not called
- **THEN** no side effect occurs (no file is read, created, or modified)

#### Scenario: run executes the described effect exactly once
- **WHEN** `run` is called on a composed `IoAction`
- **THEN** the described effects execute in composition order and a `Result<Output, Error>` is returned

#### Scenario: No dynamic dispatch is introduced
- **WHEN** the `io` module is compiled
- **THEN** it contains no `dyn`, no `Box<dyn ...>`, and no trait objects

### Requirement: Monadic composition of IO actions

`IoAction` SHALL compose without running through `pure`, `map`, and `and_then`. `pure(v)` SHALL lift a
value with no effect; `map` SHALL transform the successful result; `and_then` SHALL chain a dependent
action whose `Error` matches. `map_err` SHALL transform the error channel. Each combinator SHALL return
a new concrete type implementing `IoAction`. The monad laws (left identity, right identity,
associativity) SHALL hold.

#### Scenario: pure carries a value with no effect
- **WHEN** `pure(value)` is run
- **THEN** it returns `Ok(value)` and performs no IO

#### Scenario: and_then chains a dependent action
- **WHEN** `a.and_then(|x| b(x))` is composed and run
- **THEN** `a` runs first, its output feeds `b`, and the combined result is returned

#### Scenario: Left identity
- **WHEN** `pure(a).and_then(f)` and `f(a)` are each run
- **THEN** they produce equal results

#### Scenario: Associativity
- **WHEN** `m.and_then(f).and_then(g)` and `m.and_then(|x| f(x).and_then(g))` are each run
- **THEN** they produce equal results

### Requirement: Error generic in the abstraction, fixed in the specialization

The `IoAction` abstraction in `deep_causality_haft` SHALL be generic over its `Error` type so that haft
introduces no concrete error type. `deep_causality_core` SHALL specialize the file actions to
`Error = CausalityError`.

#### Scenario: haft names no concrete error
- **WHEN** the `deep_causality_haft` `io` module is compiled
- **THEN** it does not reference `CausalityError` or any concrete error type

### Requirement: A minimal file-action set in core

`deep_causality_core` SHALL provide `read_text(path)`, `write_text(path, contents)`,
`write_csv(path, header, rows)`, and `read_csv(path)`, each a concrete `IoAction` with
`Error = CausalityError`. `write_csv` SHALL emit the header line followed by each pre-rendered row line
(`','`-joined fields, `'\n'`-terminated), so the exact bytes written are determined by the
caller-supplied strings, enabling byte-for-byte reproduction of existing output.

#### Scenario: write_csv reproduces caller-formatted bytes
- **WHEN** `write_csv` is given a header and pre-formatted rows and is run
- **THEN** the file contains exactly the header line and those row lines, byte-for-byte

#### Scenario: read_text round-trips a written file
- **WHEN** a file written by `write_text` is read by `read_text`
- **THEN** the read string equals the written contents

### Requirement: IO failures propagate as CausalityError

A file action that fails SHALL surface the failure as a `CausalityError` carrying a new
`CausalityErrorEnum::IoError(String)` variant. `run` SHALL short-circuit on the first failure, leaving
later actions in the chain un-run.

#### Scenario: A failing write short-circuits the program
- **WHEN** the first of two `and_then`-sequenced writes fails (e.g. an unwritable path)
- **THEN** `run` returns `Err(CausalityError::IoError(..))` and the second write does not execute

### Requirement: IO integrates with the CausalFlow DSL via read/write verbs

`deep_causality_core` SHALL provide intent-named, format-qualified Flow verbs whose prepositions name a
file path, distinguishing the two IO directions by their effect on the carried value:

- **Reads are constructors** (`CausalFlow::read_text_from(path)`, `CausalFlow::read_csv_from(path)`):
  they run the read and the result becomes the flow's initial value (or the error channel on failure).
- **Writes are value-preserving steps** (`flow.write_text_to(path, |v| contents(v))`,
  `flow.write_csv_to(path, header, |v| rows(v))`): they run the write for its side effect, **pass the
  carried value through unchanged**, and append an `EffectLog` audit entry. A write step SHALL NOT
  replace the flow value with the action's `()` output.

The verbs SHALL be thin wrappers over the core file actions; the value-producing read and the
value-preserving write semantics SHALL be distinct so a write never collapses the flow to
`CausalFlow<()>`.

#### Scenario: A read constructor produces the initial value
- **WHEN** `CausalFlow::read_csv_from(path)` is run on a readable file
- **THEN** the flow's value is the parsed rows and no value was supplied by the caller

#### Scenario: A write step preserves the carried value
- **WHEN** `flow.write_csv_to(path, header, |v| rows(v))` is run on a successful `CausalFlow<V>`
- **THEN** the file is written, the flow is still `CausalFlow<V>` carrying the original value, and an `EffectLog` entry is recorded

#### Scenario: A failing read or write routes to the error channel
- **WHEN** a read constructor or a write step fails (e.g. an unreadable/unwritable path)
- **THEN** the flow short-circuits with `CausalityError::IoError(..)` and subsequent steps do not run

### Requirement: A generic IoAction bridge for composed actions

`deep_causality_core` SHALL also provide a generic, format-agnostic bridge for the case where an
`IoAction` is composed first and only then entered into a flow: `CausalFlow::source(io)` SHALL start a
flow from any `IoAction` (its `Output` becomes the value), and `flow.commit(|v| io)` SHALL run a
value-preserving `IoAction<Output = ()>` step (value passes through, audit entry appended). The
format-qualified verbs above SHALL be expressible in terms of this bridge.

#### Scenario: source enters a flow from a composed action
- **WHEN** `CausalFlow::source(read_text(path).map(parse))` is run
- **THEN** the composed action runs once and its parsed output is the flow value

#### Scenario: commit runs a value-preserving effect
- **WHEN** `flow.commit(|v| write_csv(path, header, rows(v)))` is run on a `CausalFlow<V>`
- **THEN** the write runs, the flow remains `CausalFlow<V>`, and an `EffectLog` entry is recorded

### Requirement: File IO is std-gated; the abstraction is no_std-safe

The `IoAction` trait and the pure combinators (`pure`, `fail`, `map`, `and_then`, `map_err`) SHALL be
available without `std`. The file actions (`read_text`, `write_text`, `write_csv`, `read_csv`) and the
filesystem effect of `run` SHALL be gated behind the `std` feature.

#### Scenario: The abstraction compiles without std
- **WHEN** `deep_causality_haft` is built without the `std` feature
- **THEN** `IoAction` and its pure combinators are available

#### Scenario: File actions are absent without std
- **WHEN** `deep_causality_core` is built without the `std` feature
- **THEN** the file actions are not compiled

### Requirement: CFD CSV application

`deep_causality_cfd` SHALL provide std-gated CSV helpers built on the core `write_csv` file action:
`Report::write_series_csv(path, labels)` (named observation columns) and a free `write_xy_csv(path,
header, series)` `(x, y)` writer. `dec_cylinder_wake` SHALL write its full wake-probe time series to a
CSV file by constructing a deferred `write_xy_csv` action and executing it with a single `run` at the
program edge (an IO failure surfacing as `CausalityError` through the example's `fail` path).

#### Scenario: The ported example writes a well-formed wake CSV
- **WHEN** the migrated `dec_cylinder_wake` is run
- **THEN** it writes a CSV file with a `t,v_probe` header and one row per march step, and the file is
  not created until the action's `run` executes
