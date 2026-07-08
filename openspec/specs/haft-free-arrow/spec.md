# haft-free-arrow Specification

## Purpose
TBD - created by archiving change haft-categorical-machinery. Update Purpose after archive.
## Requirements
### Requirement: A reified free Arrow (ArrowTerm)

`deep_causality_haft` SHALL provide a reified free Arrow — an `ArrowTerm` reifying the existing `id`/`lift`/`compose`/`split`/`fanout`/`first`/`second` combinators as data — so that an arrow can be *interpreted* rather than eagerly `run`, and so a free object exists for the universal property. It SHALL introduce no `dyn`, no `unsafe`, and no macros in `/src` (the core is an enum of the defunctionalized combinators). Because Rust has no GADTs, the design SHALL be **typed-by-construction, erased-core**: a typed builder API guarantees well-typed `In`/`Out` wiring at build time and lowers into an untyped core term for storage and interpretation.

#### Scenario: A term interprets to the same function as its combinators

- **WHEN** an `ArrowTerm` is built and interpreted (`run`)
- **THEN** the result equals the composition of the corresponding eager `Arrow` combinators

#### Scenario: Mistyped wiring is rejected at build time

- **WHEN** a builder attempts to compose arrows whose `Out`/`In` types do not match
- **THEN** it fails to compile (typed-by-construction), even though the stored core term is erased

### Requirement: Free-Arrow interpretation soundness is tested and proved in Lean

The interpretation-soundness law (interpreting a term equals composing its generators) and the free-object property (interpretation determined by the generator interpretation) SHALL be exercised by Rust tests (including a compile-fail test for mistyped wiring) and proved in Lean under `DeepCausalityFormal/Haft/ArrowTerm.lean` (bare-`lean`), bound by `THEOREM_MAP.md` ids (`haft.arrow_term.interpret_sound`, `haft.arrow_term.free`) with Rust witnesses. The claim "the type system rejects every nonsensical graph" SHALL be reworded to "well-typed by construction at build time, executed from an erased core" (assumption #3).

#### Scenario: The reified arrow has both bridge sides

- **WHEN** `THEOREM_MAP.md` is checked
- **THEN** the `haft.arrow_term.*` ids have `proved` Lean locations and passing Rust witnesses, and `Haft/ArrowTerm.lean` typechecks standalone with bare `lean`

