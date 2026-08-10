# documentation-code-parity Specification

## Purpose
TBD - created by archiving change reconcile-cfd-docs-and-traceability. Update Purpose after archive.
## Requirements
### Requirement: A kernel docstring names the operator the kernel marches

A kernel's public docstring SHALL describe the operator the code actually evaluates, not an earlier or
intended form of it. Where the shipped operator differs from a textbook or historical form — because it
was stabilised, symmetrised, or otherwise transformed — the docstring SHALL name the shipped form and may
cite the change that introduced it.

The DEC Navier–Stokes rate kernel marches the skew-symmetrised convective operator
`conv' = ½[G_ω u − G*_ω u]` (`src/solvers/dec/dec_ns_rate.rs:621-652`), introduced to stabilise the
un-symmetrised advection. Its public docstrings still state `−i_u(du♭)` (`dec_ns_rate.rs:7,35,457`,
`src/solvers/dec/mod.rs:11-12,29-30`, `src/theories/incompressible_dec.rs:44`). The code is correct; the
prose describes a different operator.

#### Scenario: The convective docstring matches the marched operator

- **WHEN** the DEC rate kernel's module, type, and method docstrings are read against the implementation
- **THEN** the convective term they state is the skew-symmetrised operator the code evaluates, and the
  stabilisation that introduced it is cited

#### Scenario: A reader is not sent to distrust correct code

- **WHEN** a reader compares any kernel docstring to the operator the kernel marches
- **THEN** the two agree, so the reader has no false discrepancy to resolve

### Requirement: A comment does not contradict the code beneath it

A comment SHALL describe the code it annotates. A comment that states a different formula, eigenvalue,
sign, or algorithm than the code below it SHALL be corrected to match the code, where the code is
confirmed correct.

Two spectral-projector comments name the compact 5-point Laplacian eigenvalue `−(2−2cos(2πk/N))/Δ²` and a
sign/form the code does not use (`src/tensor_bridge/projection.rs:157,169`), while the code applies the
consistent `sin²(2πk/N)/dx²` (`:158-163`). The DEC solver's module prose describes a first-order Chorin
split (`src/solvers/dec/mod.rs:21-26`) where the code projects inside each RK4 stage with no splitting
error (`dec_ns_solver/step.rs:6-13`).

#### Scenario: The spectral-projector comments match the applied eigenvalue

- **WHEN** the projection comments are read against the operator the code applies
- **THEN** they describe `sin²(2πk/N)/dx²` and the sign the code uses, not the compact 5-point form

#### Scenario: The projection-placement prose matches the code

- **WHEN** the DEC solver's projection-placement prose is read against the step implementation
- **THEN** it describes the in-stage projection the code performs, not a first-order Chorin split

### Requirement: Prose describes the shipped code, not the intended design

Documentation prose SHALL describe what the code does. Where prose describes an intended design the code
implements only in part, it SHALL be marked as intent or corrected to the implemented subset. A prose
claim that a property holds "by construction" SHALL name the check that enforces it or be reconciled — and
where the property is both asserted and unenforced, the discrepancy SHALL be raised as a correctness
finding rather than resolved by softening the prose.

The audit catalogues ~86 actionable `doc-overclaim` rows in `../../audits/cfd_audit/ACTION-LIST.md`. A
minority are already closed by Phases 1–2 (the `blended.rs` fold claim, the `boundary_zone` hook, the
`penalization_heat_integral` rename); the remainder are reconciled here.

#### Scenario: A "by construction" claim names its check or is marked intent

- **WHEN** a doc asserts a property holds "by construction"
- **THEN** the doc names the check that enforces it, or the claim is marked as intended design, or —
  where the property is unenforced and the claim is false — it is raised as a correctness finding

#### Scenario: The catalogue is reconciled and the count recorded

- **WHEN** the `doc-overclaim` catalogue is worked
- **THEN** each surviving row is reconciled against the code, and the count reconciled and the count
  escalated are recorded against the catalogue so completeness is checkable

### Requirement: Every load-bearing public capability is documented where a user looks for it

A shipped public capability SHALL be documented where a user discovers capabilities — the crate README and
architecture docs — not only in rustdoc. A capability present in the code and absent from the user-facing
documentation misleads a reader into concluding it does not exist.

The audit lists capabilities present with rustdoc but absent from the crate README: `DuctMarchRun`
(`src/types/flow/duct_march_run.rs:56`), `IgnitionCorridor` (`src/types/flow/throttle_guidance.rs:107`),
snapshot/resume (`src/types/flow/state_snapshot.rs`), and `AcousticCoreInverse`/`2d`/`3d`
(`src/tensor_bridge/acoustic_inverse.rs:52,175,249`), among the `doc-gap` category.

#### Scenario: A shipped capability appears in the user-facing documentation

- **WHEN** a user reads the crate README for the capabilities the crate provides
- **THEN** the load-bearing public capabilities appear there, not only in rustdoc

### Requirement: A convergence claim states the order it holds in and the regime it holds over

A convergence claim SHALL state the order it holds in each independent variable and the regime over which
it holds. A claim of "n-th order convergence" that is n-th order in space and a lower order in time SHALL
state both, and SHALL document any error floor and the maximum problem size over which the order is
observed.

The QTT Taylor–Green harness reports "clean 2nd-order convergence" (the "Measured" section of
`verification/qtt_taylor_green_verification/README.md`, `summary()` in
`verification/qtt_taylor_green_verification/print_utils.rs`, and the `qtt_taylor_green_verification`
section of `verification/README.md`) without qualifying it as second-order in space and first-order in
time, and without documenting the temporal-error floor (~1e-5 at fixed `dt`) or the ladder's maximum
usable length.

#### Scenario: The Taylor–Green order claim is qualified

- **WHEN** the QTT Taylor–Green convergence claim is read
- **THEN** it states second-order in space and first-order in time, and documents the measured temporal
  floor and the maximum usable ladder length

### Requirement: Headline accuracy claims are consistent across the documents that state them

An accuracy claim stated in more than one document SHALL use the same framing in each. A validation claim
SHALL name what it validates against and at what fidelity, and SHALL NOT read as a stronger claim in one
document than in another.

This requirement is satisfied by Phases 1–2 for the two claims the audit flagged — the RAM-C anchor is
framed as order-of-magnitude in both the crate README (`README.md:224`) and the verification README
(`verification/README.md:127`), and the lid-cavity summary row reports the 65² default (RMSE 0.0617,
`verification/README.md:90`). It is captured here so a regression is caught rather than silently
reintroduced.

#### Scenario: The RAM-C claim reads the same in both READMEs

- **WHEN** the RAM-C anchor claim is read in the crate README and the verification README
- **THEN** both frame it as an order-of-magnitude prediction naming the ±0.70-decade pinned band, neither
  as a per-point accuracy claim

#### Scenario: A quantitative summary row reports the configuration it measures

- **WHEN** a summary row reports a measured accuracy figure
- **THEN** it names the configuration that produced it, and that configuration is the default the reader
  would run

### Requirement: The crate ships no dead, duplicate gate API

A public API exported as the mechanism a program uses SHALL be the mechanism that program uses, or SHALL
be retired. A gate-reporting type documented as the block every self-verifying program prints, but
constructed by none of them, SHALL be adopted across those programs or removed, and a gate harness SHALL
NOT report success for an empty gate set.

`Gates` was exported from `lib.rs` and documented as the `[PASS]`/`[FAIL]` block every self-verifying
program prints, yet `Gates::new` was constructed only in its own unit test — the programs used `GateSeq`
or `Verdict`'s `Display`. It held the only five `println!` in `src/`, and `Gates::finish()` returned
success for an empty gate set. `2026-07-26-reconcile-cfd-docs-and-traceability` retired the type on an
owner decision: `src/types/flow/gates.rs` and its tests are gone, the `lib.rs` re-export with them, `src/`
now holds no `println!`, and `GateSeq` is the mechanism the self-verifying programs use. The evidence is
historical; the requirement stands as the contract that keeps the duplicate API from returning.

#### Scenario: The empty gate set does not report success

- **WHEN** a gate harness finishes having registered no gate
- **THEN** it does not report pass

#### Scenario: Documentation names the gate mechanism the programs use

- **WHEN** a document describes how a self-verifying program reports its gates
- **THEN** it names the type the program actually uses, not a parallel API no program constructs

