## ADDED Requirements

### Requirement: Every shipped gate is falsifiable

Every acceptance gate SHALL be falsifiable. This covers `deep_causality_cfd/verification/`,
`deep_causality_cfd/studies/`, and the CFD examples under `examples/avionics_examples/cfd/`: for each gate
there MUST exist an input, parameter value, or code mutation that makes it fail, and that breaking
condition MUST be recorded next to the gate as a comment. A gate whose predicate is an identity of the
implementation under test, a comparison between two compile-time constants, or a fold over a hardcoded
literal SHALL be repaired or removed.

The following gates are known-unfalsifiable and are in scope: `qtt_park2t_blackout` gates (ii) and (iv);
the `qtt_taylor_green_verification` convection-amplitude gate; the weather example's "(0) table integrity"
gate; the corridor "(4f) fine sweep refines the coarse winner" gate; `qtt_rank_dynamic` gate G2; the
compressible MMS `continuity_error` gate; and the energy-budget test gate.

#### Scenario: A repaired gate fails on its documented breaking input

- **WHEN** a gate's recorded breaking condition is applied to the harness
- **THEN** that gate reports FAIL, names itself, and the program exits non-zero

#### Scenario: No gate compares a value against itself

- **WHEN** the gate inventory is reviewed
- **THEN** no gate predicate re-derives its reference from the same function, constant, or code path it is
  gating, and no gate's operands are both compile-time constants

#### Scenario: A gate that cannot be made falsifiable is removed

- **WHEN** a gate carries no input that could make it fail and no meaningful check can replace it
- **THEN** the gate is deleted rather than retained, and the count of gates the harness advertises is
  corrected to match

### Requirement: Every gate bound declares its evidence class

Every numeric gate bound SHALL be labelled with one of exactly two evidence classes, in the source where
the bound is defined and in the harness's printed output: **reference** — the bound comes from an analytic
solution or a published external value, which MUST be cited; or **tripwire** — the bound is pinned from
this code's own prior measurement and detects regression only, carrying no claim of external accuracy.

A tripwire bound SHALL NOT be presented as validation against an external reference in the harness output,
in `verification/README.md`, or in the crate README.

#### Scenario: Printed output distinguishes the two classes

- **WHEN** a harness prints its gate block
- **THEN** each gate line carries its evidence class, so a reader can tell a reference comparison from a
  regression tripwire without consulting the source

#### Scenario: A pinned bound is not described as external validation

- **WHEN** a bound was derived from this code's own output — for example the `qtt_ramc_stagline` ±0.70-decade
  band, or the lid-cavity RMSE bounds carrying headroom from their pinning run
- **THEN** it is labelled `tripwire` everywhere it appears, and any prose describing it as agreement with
  flight data or a published table is corrected

#### Scenario: A reference bound cites its source

- **WHEN** a bound is labelled `reference`
- **THEN** the citation (paper, table, or closed-form solution) is recorded at the definition site

### Requirement: CI executes the verification suite

Continuous integration SHALL execute the `deep_causality_cfd/verification/` binaries and fail the build on
any non-zero exit. The fast harnesses — those completing in about five seconds or less, currently ten of
the thirteen — SHALL run on every pull request. The slow harnesses — currently `dec_cylinder_verification`
(~510 s), `dec_cylinder_wake_verification` (~155 s), and `dec_lid_cavity_re1000_verification` — SHALL run
on a nightly schedule.

Compiling a verification binary SHALL NOT be treated as executing it; `cargo test` compiles the examples
without running them, which is the gap this requirement closes.

#### Scenario: A failing gate fails the build

- **WHEN** any verification harness exits non-zero in CI
- **THEN** the job fails and names the harness that broke

#### Scenario: Fast suite runs per pull request

- **WHEN** a pull request touching the workspace is opened
- **THEN** the fast verification harnesses run to completion and their exit codes gate the merge

#### Scenario: Slow suite runs nightly

- **WHEN** the nightly schedule fires
- **THEN** the three slow harnesses run and their exit codes gate the nightly result, reported separately
  from the pull-request suite

#### Scenario: A newly added harness is executed

- **WHEN** a new binary is added under `verification/`
- **THEN** it is picked up by one of the two CI cadences rather than silently never running

### Requirement: Committed baseline artifacts are complete runs

Every committed `baseline.txt` under `deep_causality_cfd/verification/` SHALL be the output of a run that
reached its terminal summary — carrying the harness's reported quantities and its verdict line. A
truncated or aborted run SHALL NOT be committed as a baseline, because it silently removes the reference
a reader compares against.

#### Scenario: A baseline carries its verdict

- **WHEN** any committed baseline artifact is read
- **THEN** it contains the harness's final reported quantities and its verdict line, not a partial
  progress trace

#### Scenario: The baseline matches the configuration it documents

- **WHEN** a baseline is regenerated
- **THEN** the grid, horizon and step count in its header match the configuration whose numbers the
  harness and `verification/README.md` report
