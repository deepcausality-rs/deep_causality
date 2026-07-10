## MODIFIED Requirements

### Requirement: A reified free Arrow (ArrowTerm)

`deep_causality_haft` SHALL provide a reified free Arrow — an `ArrowTerm` reifying the existing `id`/`lift`/`compose`/`split`/`fanout`/`first`/`second` combinators **and the choice generators `left`/`right`/`choice`/`fanin`** as data — so that an arrow can be *interpreted* rather than eagerly `run`, and so a free object exists for the universal property. It SHALL introduce no `dyn`, no `unsafe`, and no macros in `/src` (the core is an enum of the defunctionalized combinators). Because Rust has no GADTs, the design SHALL be **typed-by-construction, erased-core**: a typed builder API guarantees well-typed `In`/`Out` wiring at build time and lowers into an untyped core term for storage and interpretation. For the choice fragment this means: `ArrowCore<G>` gains `Left(f)`, `Right(f)`, `Choice(f, h)`, and `Fanin(f, h)` variants; `ArrowVal<V>` gains a sum node (`InL`/`InR`) alongside `Leaf`/`Pair`; and the `ArrowTerm<In, Out, G>` typed façade gains `left::<C>`, `right::<C>`, `choice`, and `fanin`, typed over `Either<_, _>`, so mistyped branch wiring fails to compile.

#### Scenario: A term interprets to the same function as its combinators

- **WHEN** an `ArrowTerm` is built and interpreted (`run`)
- **THEN** the result equals the composition of the corresponding eager `Arrow` combinators

#### Scenario: Mistyped wiring is rejected at build time

- **WHEN** a builder attempts to compose arrows whose `Out`/`In` types do not match
- **THEN** it fails to compile (typed-by-construction), even though the stored core term is erased

#### Scenario: Mistyped branch wiring is rejected at build time

- **WHEN** a builder attempts `choice`/`fanin` with branches whose summand types do not match the
  `Either<_, _>` wire
- **THEN** it fails to compile, demonstrated by a `compile_fail` doctest

### Requirement: Free-Arrow interpretation soundness is tested and proved in Lean

The interpretation-soundness law (interpreting a term equals composing its generators) and the free-object property (interpretation determined by the generator interpretation) SHALL be exercised by Rust tests (including a compile-fail test for mistyped wiring) and proved in Lean under `DeepCausalityFormal/Haft/ArrowTerm.lean` (bare-`lean`), bound by `THEOREM_MAP.md` ids (`haft.arrow_term.interpret_sound`, `haft.arrow_term.free`) with Rust witnesses. **Both properties SHALL extend to the choice-enlarged generator set**: interpreting the choice generators agrees with the eager ArrowChoice combinators (`haft.arrow_term.choice_interpret_sound`), and two interpretations that agree on all generators — including `left`/`right`/`choice`/`fanin` — agree on every term (`haft.arrow_term.choice_free`). The claim "the type system rejects every nonsensical graph" SHALL be reworded to "well-typed by construction at build time, executed from an erased core" (assumption #3).

#### Scenario: The reified arrow has both bridge sides

- **WHEN** `THEOREM_MAP.md` is checked
- **THEN** the `haft.arrow_term.*` ids — including `choice_interpret_sound` and `choice_free` — have `proved` Lean locations and passing Rust witnesses, and `Haft/ArrowTerm.lean` typechecks standalone with bare `lean`
