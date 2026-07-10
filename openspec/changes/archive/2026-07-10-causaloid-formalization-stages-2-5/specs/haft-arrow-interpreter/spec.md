## MODIFIED Requirements

### Requirement: A one-way interpreter from the free Arrow into Kleisli

`deep_causality_haft` SHALL provide a one-way `NaturalTransformation<F, G>` (naturality: commutes with `fmap`) and/or an `ArrowInterpreter<A, M: Monad>` mapping the free Arrow (`ArrowTerm`) into `Kleisli<M>`. Because the interpretation is not invertible (the syntactic term cannot be recovered from a composed Kleisli arrow), the bidirectional `NaturalIso` SHALL NOT be used for it. This is the substrate for expressing `evaluate` as a unique interpretation functor. **Both interpreters (`ArrowCore::interpret` and `ArrowCore::interpret_kleisli`) SHALL cover the choice generators**: `InL`/`InR` values dispatch to the corresponding branch of `Choice`, and `Fanin` merges the branches into one output.

#### Scenario: The interpreter is a functor

- **WHEN** an `ArrowTerm` is interpreted into `Kleisli<M>`
- **THEN** the interpretation preserves identity (`interpret(id) = Kleisli::id`) and composition (`interpret(compose f g) = compose (interpret f) (interpret g)`)

#### Scenario: One-way, not an iso

- **WHEN** the interpretation is applied
- **THEN** it is a `NaturalTransformation`/`ArrowInterpreter` with no inverse (no `to_source`), reflecting that syntax is not recoverable from semantics

#### Scenario: Choice generators dispatch on the sum

- **WHEN** a term containing `choice(f, h)` is interpreted and applied to an `InL`/`InR` value
- **THEN** the `InL` value flows through the interpretation of `f` and the `InR` value through the interpretation of `h`, and `fanin(f, h)` yields the merged single-typed output

### Requirement: Interpreter functoriality is tested and proved in Lean

The functoriality laws (`preserves id`, `preserves compose`) and naturality SHALL be exercised by Rust law-tests (Bazel-registered) and proved in Lean under `DeepCausalityFormal/Haft/Interpreter.lean` (bare-`lean`), citing the arrow laws (`haft.arrow.*`), bound by `THEOREM_MAP.md` ids (`haft.interpreter.preserves_id`, `haft.interpreter.preserves_compose`, `haft.interpreter.naturality`) with Rust witnesses. **Preservation SHALL extend to the choice generators**: `interpret_kleisli` maps `left`/`right`/`choice`/`fanin` to their Kleisli counterparts (`haft.interpreter.choice_preserved`), extending `preserves_id`/`preserves_compose` to the enlarged generator set.

#### Scenario: Both bridge sides exist for the interpreter

- **WHEN** `THEOREM_MAP.md` is checked
- **THEN** the `haft.interpreter.*` ids — including `choice_preserved` — have `proved` Lean locations and passing Rust witnesses, and `Haft/Interpreter.lean` typechecks standalone with bare `lean`
