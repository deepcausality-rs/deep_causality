## ADDED Requirements

### Requirement: A lazy IO effect that runs only at the edge
`deep_causality_core` SHALL provide a lazy `Io<T>` effect that describes a file input/output
computation and performs **no** side effect until an explicit `run()` is called. Constructing an
`Io<T>` or composing it with combinators SHALL NOT touch the filesystem; only `run()` SHALL.

#### Scenario: Construction performs no side effect
- **WHEN** an `Io::write_text(path, contents)` (or any action) is constructed and composed but `run()` is not called
- **THEN** no file is created or modified

#### Scenario: run performs the described effects in order
- **WHEN** `run()` is called on a composed `Io` program
- **THEN** the described file effects execute in composition order and a `Result<T, CausalityError>` is returned

### Requirement: Monadic composition of IO actions
`Io<T>` SHALL compose without running through `pure`, `map`, `and_then`, and `sequence`. `Io::pure(v)`
SHALL lift a value with no effect; `map` SHALL transform the result; `and_then` SHALL chain a
dependent action; `sequence` SHALL run a list of `Io<()>` actions in order.

#### Scenario: Independent writes compose into one program
- **WHEN** two `write_csv` actions are combined with `sequence` (or `and_then`) and `run()` once
- **THEN** both files are written in order and the program yields a single result

#### Scenario: pure carries a value with no effect
- **WHEN** `Io::pure(value)` is run
- **THEN** it returns the value and performs no IO

### Requirement: A minimal file-action set
`Io` SHALL provide `write_text(path, contents)`, `write_csv(path, header, rows)`, and
`read_text(path)`. `write_csv` SHALL emit the header line followed by each pre-rendered row line, so
the exact bytes written are determined by the caller-supplied strings (enabling byte-for-byte
reproduction of existing output).

#### Scenario: write_csv reproduces caller-formatted bytes
- **WHEN** `write_csv` is given a header and rows pre-formatted by the caller and is run
- **THEN** the file contains exactly the header line and those row lines, byte-for-byte

#### Scenario: read_text round-trips a written file
- **WHEN** a file written by `write_text` is read by `read_text`
- **THEN** the read string equals the written contents

### Requirement: IO failures propagate as CausalityError
An `Io` action that fails SHALL surface the failure as a `CausalityError`, and `run()` SHALL
short-circuit the remaining actions on the first failure (no later effect runs). An `Io` result SHALL
be usable within a `CausalFlow` / `PropagatingEffect` pipeline without a new error type.

#### Scenario: A failing write short-circuits the program
- **WHEN** the first of two sequenced writes fails (e.g. an unwritable path)
- **THEN** `run()` returns `Err(CausalityError)` and the second write does not execute

### Requirement: File IO is std-gated; pure combinators are no_std-safe
The `Io<T>` type and its pure combinators (`pure`, `map`, `and_then`, `sequence`) SHALL be available
without `std`; the file actions (`write_text`, `write_csv`, `read_text`) and `run()`'s filesystem
interpretation SHALL be gated behind the `std` feature.

#### Scenario: Pure description compiles without std
- **WHEN** the crate is built without the `std` feature
- **THEN** `Io::pure`/`map`/`and_then`/`sequence` are available and the file actions are not compiled
