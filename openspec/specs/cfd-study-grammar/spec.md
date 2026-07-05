# cfd-study-grammar Specification

## Purpose
TBD - created by archiving change rework-cfd-flow-dsl. Update Purpose after archive.
## Requirements
### Requirement: One study entry point

`deep_causality_cfd` SHALL expose the campaign level through a single entry, `CfdFlow::study(title)`,
returning a `StudyDef` whose case-axis verbs (`read`, `matrix`, `cases`) open the typed case
axis. There SHALL be no second campaign facade type and no free-standing campaign constructors.

#### Scenario: A study opens through CfdFlow::study

- **WHEN** a campaign is written
- **THEN** it begins `CfdFlow::study("...")` and no other type constructs a campaign

### Requirement: Phase-typed verbs forbid mis-ordered programs

The campaign SHALL be a family of typestate phases (`Cases`, `Prepared`, `Counterfactual`,
`ForkStudy`, `Configured`, `Branched`, `Marched`, `Swept`, `Judged`), each exposing only the
verbs meaningful at that phase, such that a mis-ordered program fails to compile. In particular:
`record` and `gates` SHALL NOT exist before rows exist; `verdict` SHALL NOT exist before at
least one gating sequence is applied; `reduce` SHALL NOT exist before a march; `march` SHALL NOT
exist before a case binder.

#### Scenario: verdict before gates does not compile

- **WHEN** a program calls `verdict()` on a swept-but-ungated study
- **THEN** the program fails to compile because `verdict` is defined only after judgment begins

#### Scenario: record before a sweep does not compile

- **WHEN** a program calls `record(path)` before any rows are produced
- **THEN** the program fails to compile because the pre-sweep phases carry no `record` verb

### Requirement: The effect carrier threads errors and warnings invisibly

Every phase SHALL ride inside a `StudyEffect` carrier that short-circuits on the first error
(tagged with the failing verb) and accumulates non-fatal warnings, both travelling without
appearing in any user-facing signature. The carrier's monadic composition SHALL be provided by
a hidden haft HKT witness. `verdict()` SHALL surface the first error as `Err(StudyError)` and
otherwise return a `Verdict` whose report includes the accumulated warnings.

#### Scenario: A non-fatal warning reaches the verdict

- **WHEN** a stage records a non-fatal warning (for example a `force_load` snapshot override)
  and the study then completes
- **THEN** the returned `Verdict` renders that warning, and the warning does not affect
  `passed()`

#### Scenario: A stage failure names its verb

- **WHEN** a case binder or sweep fails
- **THEN** `verdict()` returns `Err(StudyError)` naming the verb that failed

### Requirement: The case binders and the sweep

`Cases` SHALL provide `prepare` (build a shared apparatus once), `case` (bind one owned solver
config per case), and `sweep` (one case, one row). The sweep SHALL be order-preserving and
first-error-wins, running concurrently under the `parallel` feature and inline otherwise, with
results identical in both modes. A study that reaches `Swept` SHALL therefore have produced a
result for every scheduled case by construction.

#### Scenario: The sweep preserves order and stops at the first error

- **WHEN** a sweep over N cases is run and case k returns an error
- **THEN** the study short-circuits with case k's error, and a successful sweep yields rows in
  case order identical under the parallel and serial builds

### Requirement: Reductions from reports to rows

`Marched` SHALL provide `reduce` (one report to one row), `reduce_all` (all reports at once to
one row per case in order, for cross-case references), and `reduce_ensemble` (one case and its
draw set to one row). Each reduction SHALL yield a `Swept` of typed rows.

#### Scenario: reduce_all sees every report

- **WHEN** a reduction needs a value derived from one case to score the others (a shared aim
  point)
- **THEN** `reduce_all` receives every case's run together and returns one row per case in order

### Requirement: Record then refine

`Swept` SHALL provide `record(path)`, writing the rows through the typed table writer with the
schema taken from the row type and the precision from its `TableScalar`, and `refine(f)`,
deriving a new case set from the rows so far while carrying the rig forward and retaining all
prior rounds for the gates. Recording SHALL precede judgment, so the table exists even when the
gates then fail.

#### Scenario: A recorded table survives a later gate failure

- **WHEN** a study records its rows and a subsequent gate fails
- **THEN** the table file has been written, and the study still reports the gate failure

### Requirement: Gating sequences are reusable typed values

The gate builder SHALL produce a `GateSeq<Row>` value built once with `new(title)` and `gate(label, check)`,
where each `check` is a higher-ranked `fn` over a borrowed study view
(`for<'a> fn(&StudyView<'a, Row>) -> (bool, String)`). `Swept::gates` and `Judged::gates` SHALL
accept a `GateSeq<Row>` whole; a sequence built for a different row type SHALL NOT compile into
a study. `GateSeq::check(subject)` SHALL run a sequence against any subject and return a
`Verdict`, so trajectory-level programs reuse the same sequences.

#### Scenario: A whole gating sequence is inserted by value

- **WHEN** a named `GateSeq<Row>` from `model.rs` is placed with `.gates(seq)`
- **THEN** the study applies every gate in the sequence and the sequence is defined once as a
  reusable value

#### Scenario: A wrong-row gating sequence does not compile

- **WHEN** a `GateSeq<OtherRow>` is inserted into a study whose rows are `Row`
- **THEN** the program fails to compile

### Requirement: The verdict is a value, not a process effect

`verdict()` SHALL return `Result<Verdict, StudyError>`; `Verdict` SHALL provide `passed()`,
`Display` rendering, `warnings()`, and `merge()` composing two verdicts into one. The grammar
SHALL NOT print to stdout/stderr, exit, or otherwise touch the process.

#### Scenario: A mixed program ends in one verdict

- **WHEN** a trajectory program with an embedded campaign produces a campaign verdict and a
  trajectory (leg) verdict
- **THEN** `merge` composes them into one `Verdict`, and the caller alone maps `passed()` to an
  exit code

