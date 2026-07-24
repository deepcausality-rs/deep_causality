# haft-arrow-choice Specification

## Purpose
TBD - created by archiving change causaloid-formalization-stages-2-5. Update Purpose after archive.
## Requirements
### Requirement: An eager ArrowChoice fragment over the proven coproduct

`deep_causality_haft` SHALL provide the value-level ArrowChoice fragment over `Either` — `Left(f)`,
`Right(f)`, `Choice(f, h)` (`+++`), and `Fanin(f, h)` (`|||`) as defunctionalized `Arrow` structs —
with no `dyn`, no `unsafe`, and no crate-defined macros. `Fanin` SHALL be the coproduct elimination
(both branches converge on one output type), building on the proven universal property
(`haft.either.coproduct_universal`). Two forcings motivate the generator and SHALL be cited in the
docs: causally faithful quantum decomposition requires direct-sum structure (Lorenz & Barrett 2021
§3–4), and classical case-splitting/regime selection is a coproduct elimination (Hughes 2000 §5).

#### Scenario: Left acts on the left summand only

- **WHEN** `Left(f)` is applied to `Either::Left(a)` and to `Either::Right(c)`
- **THEN** the results are `Either::Left(f(a))` and `Either::Right(c)` respectively, matching
  `arr (f ⊕ id)` when `f` is pure

#### Scenario: Fanin eliminates the coproduct

- **WHEN** `Fanin(f, h)` is applied to `Either::Left(a)` and to `Either::Right(b)`
- **THEN** the results are `f(a)` and `h(b)` — one output type, the coproduct elimination

### Requirement: The distributivity equations used are stated and proved

The `⊗`-over-`⊕` interaction SHALL be stated, and the specific equations the crate uses SHALL be
proved — pairs distribute over sums, the rig-category coherence that faithful direct-sum
decompositions rely on. The full coherence-diagram machinery SHALL be explicitly deferred with a
scope note.

#### Scenario: Used equations are closed, full coherence is deferred

- **WHEN** `Haft/ArrowChoice.lean` is checked
- **THEN** the stated distributivity equations are closed theorems, and the deferral of full rig
  coherence is recorded in the proof header's deviation notes

### Requirement: ArrowChoice laws are tested and proved in Lean

The ArrowChoice equations on the eager fragment SHALL be exercised by Rust law-tests
(Bazel-registered) and proved in Lean under `DeepCausalityFormal/Haft/ArrowChoice.lean`
(bare-`lean`, zero `sorry`). The equations are `left (arr f) = arr (f ⊕ id)`, the
composition/exchange laws, and `fanin` as the coproduct elimination; the proofs cite Hughes 2000 §5
and Lorenz & Barrett 2021 §4 with deviation notes, bound by `THEOREM_MAP.md` ids
(`haft.arrow_choice.laws`) with Rust witnesses.

#### Scenario: Both bridge sides exist for the choice fragment

- **WHEN** `THEOREM_MAP.md` is checked
- **THEN** the `haft.arrow_choice.*` ids have proved Lean locations and passing Rust witnesses, and
  `Haft/ArrowChoice.lean` typechecks standalone with bare `lean`
