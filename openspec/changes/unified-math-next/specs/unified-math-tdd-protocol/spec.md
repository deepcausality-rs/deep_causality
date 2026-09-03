<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: Every stage runs the same five-phase cycle in order

Each of the five stages SHALL pass through phases 1 to 5 in order, and SHALL NOT begin a phase before the previous phase's exit condition is met.

The five phases are: (1) a compiling API with unimplemented bodies; (2) the full test suite, written
against that API and observed failing; (3) an audit of the suite against deliberate defects and
against its own input variety; (4) implementation, judged finished only by the audited suite; (5)
mutation testing to harden what the suite still misses.

The order is the point. A test written after the code it checks tends to encode what the code does
rather than what it should do, and a suite that has never been shown to reject a wrong implementation
has not been shown to do anything. This generalises
[`linear-test-first-development`](../../../specs/linear-test-first-development/spec.md), which applied
the same cycle to one crate, to a programme of five stages — including the stages that port or move
existing code, where the temptation to skip straight to phase 4 is strongest.

#### Scenario: A stage's phase order is auditable after the fact
- **WHEN** the commit history for any stage is read
- **THEN** the API-only commit precedes the suite commit, and the suite commit precedes the first implementing commit
- **AND** no test for a behaviour appears in a commit later than the one implementing that behaviour

#### Scenario: A ported stage does not skip phase 1
- **WHEN** a stage moves or ports existing code (C1's Meek closure, C2's absorbed statistics, C4's replacements)
- **THEN** the destination API is first declared with unimplemented bodies
- **AND** the existing implementation is not pasted into the destination before its suite exists and fails

### Requirement: Phase 1 produces a compiling API whose every body is unimplemented

Each stage SHALL first land its complete public surface — every type, trait, function, error variant and signature — with `unimplemented!()` bodies, and SHALL NOT contain a working algorithm at that point.

Designing the surface before the implementation forces it to answer to its callers rather than to
whatever the implementation found convenient. It also makes the suite compilable before anything
works, which phase 2 depends on.

For a stage that ports existing behaviour, the surface is derived from the call sites it must serve,
not transcribed from the source being replaced. C1's destination signature is an inherent method on
`MixedGraph<T>`, not the free function it comes from; C2's entropy signature carries the base and
zero-policy parameters the three sources disagree on, which none of the three has.

#### Scenario: The API-only stage compiles
- **WHEN** the stage's API-only commit is built under both `cargo build` and `bazel build`
- **THEN** compilation succeeds with no error
- **AND** every declared item is reachable from its crate root

#### Scenario: Nothing works yet
- **WHEN** any public function of the API-only surface is called
- **THEN** it panics as unimplemented rather than returning a value

#### Scenario: The surface is complete before the suite is written
- **WHEN** the suite is authored in phase 2
- **THEN** it compiles against the phase-1 surface with no addition to that surface

### Requirement: Phase 2 writes the full suite against the unimplemented API and observes it fail

The complete test suite SHALL be written before any algorithm, SHALL cover every scenario of the stage's capabilities, SHALL be run against the unimplemented API, and SHALL be observed failing for the intended reason.

Observing the failure is what separates a test that will catch a defect from one that passes
vacuously. A test that fails with a compile error, a missing import, or a panic raised somewhere
other than the function under test has not been shown to exercise anything.

The scenarios are the source the suite is written from, not a list to be reconciled against it
afterwards. A scenario with no test means phase 2 is unfinished — that is a property of the suite,
carried by the suite, and it needs no second artifact tracking it.

#### Scenario: Every test fails for the intended reason
- **WHEN** the suite runs against the unimplemented API
- **THEN** every test fails
- **AND** each failure is the unimplemented panic, not a compile error, a missing import, or a panic from elsewhere

#### Scenario: The failure run is recorded
- **WHEN** the phase-2 exit is claimed
- **THEN** the failing run's output is recorded in the stage's task notes, with the test count

#### Scenario: Every scenario of the stage has a test
- **WHEN** the phase-2 exit is claimed
- **THEN** each scenario of the stage's capabilities is exercised by at least one test in the suite

### Requirement: Expected values come from a source independent of the implementation

Every assertion SHALL compare against a value obtained independently of the code under test, and SHALL NOT compute its expectation through the same code path, the same formula, or a helper that shares either.

This is the anti-circularity rule, and it is the one that most often fails silently. A test that
checks `entropy(p)` by recomputing `-Σ p log p` in the test body asserts only that the expression was
typed the same way twice. A test that checks a ported function against the function it was ported
from asserts only that the port was faithful, including to its defects.

An acceptable independent source is one of: a closed form evaluated by hand and written as a literal;
a value published in a cited reference; a value computed by a demonstrably different algorithm (a
naïve O(n³) oracle against an optimised path); an algebraic invariant the result must satisfy
(idempotence, a conservation law, an inverse round trip); or a property that holds for all inputs in
a generated family.

#### Scenario: A tautological assertion is rejected in review
- **WHEN** a test's expected value is produced by calling the function under test, by re-implementing its formula inline, or by a helper that either calls it or shares its expression
- **THEN** the test is rejected and rewritten against an independent source

#### Scenario: A ported behaviour is pinned to the mathematics, not to its origin
- **WHEN** a stage ports existing behaviour
- **THEN** at least one test per ported function asserts against a closed form, a published value, or an invariant
- **AND** agreement with the origin implementation is recorded as a separate, additional test, never as the only one

#### Scenario: The independent source is named
- **WHEN** any test asserts a numeric literal
- **THEN** a comment names where the literal came from — the closed form, the citation, or the oracle

### Requirement: The suite covers corner cases enumerated in advance

Each stage SHALL enumerate its degenerate and boundary inputs before its suite is judged complete, and SHALL cover every enumerated case.

Enumerating in advance is what distinguishes a corner case that was considered from one that was
discovered through a defect. `AGENTS.md`'s own worked example is the topological-charge
normalisation that was eight times too small: the line ran 513 times, every test used the identity
gauge field where the field strength is zero, and `q == 0` held for any constant whatsoever.

The per-stage minimum is stated in that stage's own capability. Across all stages the following
classes SHALL be enumerated wherever the stage's inputs admit them: the empty input; the
single-element input; the input where two distinct quantities coincide (the identity, a diagonal, a
symmetric matrix, a tie); the input at which an index expression degenerates (a 2×2 where
`a[i*n+j]` and `a[i*n-j]` coincide at `j = 0`); a value at or across each documented threshold; a
value that is zero, negative, or exactly at a domain boundary; a non-finite input where the type
admits one; and, for every precision-generic path, the same case at `f32`, `f64` and `Float106`.

#### Scenario: The enumeration precedes the suite
- **WHEN** a stage's phase-2 exit is claimed
- **THEN** its corner-case enumeration is committed
- **AND** each enumerated case names the test that covers it

#### Scenario: A quantity is never pinned only where it vanishes
- **WHEN** a test asserts that a computed quantity is zero
- **THEN** the suite also asserts that quantity at an input where it is non-zero and known
- **AND** the non-zero expectation comes from an independent source

#### Scenario: A degenerate input never panics unexpectedly
- **WHEN** any enumerated degenerate input is passed to any public function
- **THEN** the call returns a value or a typed error, and does not panic, overflow, or return a silently substituted default

### Requirement: Every error variant is constructed by at least one test

Each stage SHALL exercise every variant of every error type it introduces or extends, through the public API, and SHALL assert the variant rather than only that an error occurred.

An error variant no test constructs is a claim about failure that nobody has checked. Asserting only
`is_err()` is the same omission one level down: it passes when the wrong error is returned, which is
exactly the case a caller matching on the variant will get wrong.

#### Scenario: Each variant has a constructing test
- **WHEN** an error enum introduced or extended by a stage is enumerated
- **THEN** each variant names a test that provokes it through the public API
- **AND** that test asserts the specific variant, not merely that the result is an error

#### Scenario: An unreachable variant is justified
- **WHEN** a variant cannot be provoked through the public API
- **THEN** either it is removed, or its unreachability is recorded with the argument that makes it unreachable

### Requirement: Phase 3 audits the suite against deliberate defects before the suite is trusted

Each stage SHALL, before implementing, verify its suite by introducing deliberate defects into a throwaway implementation and confirming the suite rejects each one.

A suite that passes on correct code proves nothing by itself; it has to be shown to reject incorrect
code. This is phase 3's whole purpose, and it happens before phase 4 so that the suite the
implementation is written against has already been shown to have teeth.

The defect classes SHALL include, wherever the stage's mathematics admits them: an off-by-one in an
index or a loop bound; a flipped comparison or an inverted sign; a constant factor changed (halved,
doubled, or a normalisation dropped); a tolerance loosened; an early return that skips a case; a
guard removed; and a returned value replaced by a plausible neighbour rather than by a crash.

#### Scenario: Each defect class is rejected
- **WHEN** each enumerated defect is introduced one at a time into a throwaway implementation
- **THEN** at least one test fails for each
- **AND** the failing test is one whose subject is the defective behaviour, not an unrelated test failing incidentally

#### Scenario: A defect the suite misses widens the suite
- **WHEN** an introduced defect leaves the suite green
- **THEN** a test is added that rejects it, and the audit is repeated
- **AND** the throwaway implementation is discarded before phase 4 begins

#### Scenario: Input variety is audited
- **WHEN** the phase-3 exit is claimed
- **THEN** the suite's inputs are reviewed against the corner-case enumeration for coincidence — cases where several distinct quantities take the same value — and any such case is supplemented by one where they differ

### Requirement: Phase 4 implements against the audited suite and never edits it to pass

Implementation SHALL be judged complete by the audited suite turning green at the repository's coverage requirement, and the suite SHALL NOT be weakened to accommodate an implementation.

The repository's standing rules apply unchanged: full coverage of added or edited files, and a
failing test means the implementation or the API is wrong, never that the test is. A test's assertion
is weakened only when the API itself is agreed to have been specified wrongly, and that agreement is
recorded.

#### Scenario: Coverage meets the standing requirement
- **WHEN** coverage runs over a stage's added or edited files
- **THEN** every added file reports full line coverage, except lines recorded as demonstrably unreachable

#### Scenario: A failing test changes the code
- **WHEN** a test fails during implementation
- **THEN** the implementation or the API changes
- **AND** an assertion is weakened only alongside a recorded decision that the API was wrong

#### Scenario: Both build systems run the suite
- **WHEN** `cargo test` and `bazel test //...` are run for the stage
- **THEN** both execute the full suite, both pass, and the executed test counts agree

### Requirement: Phase 5 hardens the implementation with mutation testing

Each stage SHALL run `scripts/mutants.sh` over the code it added or edited, and SHALL resolve every surviving mutant as either a suite gap or a recorded equivalence.

Mutation testing answers the question coverage cannot: whether a test would notice if a line were
wrong. It is scoped per stage rather than run over the workspace, because every mutant costs a build
plus a test run.

Equivalent mutants belong in `.cargo/mutants.toml`, each carrying the measurement that settles it
rather than an assertion that it is fine. Those entries are regular expressions matched against text
full of regex metacharacters, and a wrong one fails silently in both directions — an unescaped `+`
or `*` matches nothing, an unescaped `||` matches everything. Each entry SHALL be escaped and
confirmed with the `comm` check the file carries, and SHALL match no more than it argues for.

#### Scenario: Mutation runs over the stage's own code
- **WHEN** a stage's phase-5 exit is claimed
- **THEN** `scripts/mutants.sh` has run over the files that stage added or edited
- **AND** its report is recorded in the stage's task notes

#### Scenario: A survivor is a gap until argued otherwise
- **WHEN** a mutant survives
- **THEN** either a test is added that kills it, or an entry is added to `.cargo/mutants.toml` carrying the measurement that establishes the mutation cannot change behaviour

#### Scenario: An equivalence entry is verified
- **WHEN** an entry is added to `.cargo/mutants.toml`
- **THEN** its metacharacters are escaped, the `comm` check confirms it matches what it claims, and it is confirmed to match no additional mutants

### Requirement: Test layout and helpers follow the repository's structure

Every stage's suite SHALL mirror its source tree file for file, SHALL register each test module upward through its `mod.rs` chain, SHALL declare every test directory to Bazel, and SHALL place shared helpers under `src/utils_tests/` rather than inside `tests/`.

A test file absent from its `mod.rs` compiles and never runs; a directory absent from Bazel is
invisible to the primary gate. Both failures are silent — the suite appears to pass. A helper placed
inside `tests/` builds under Cargo and fails under Bazel, which cannot reach it; placed under `src`
it is library code, so the coverage requirement applies and it needs its own tests.

#### Scenario: Structure mirrors the source
- **WHEN** a source file `src/a/b.rs` is added by a stage
- **THEN** its tests are at `tests/a/b_tests.rs`

#### Scenario: The executed count matches the authored count
- **WHEN** the suite runs
- **THEN** the executed test count matches the number of test functions in the tree under both build systems

#### Scenario: Helpers are reachable and tested
- **WHEN** a stage adds a shared helper
- **THEN** it is under `src/utils_tests/`, `bazel test` resolves it from a clean output base, and it carries its own tests
