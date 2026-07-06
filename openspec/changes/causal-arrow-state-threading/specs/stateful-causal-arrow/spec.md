# stateful-causal-arrow Specification

## ADDED Requirements

### Requirement: The Kleisli stage receives and threads state and context
The causal arrow stage SHALL have type `(A, S, Option<C>) -> CausalFlow<B, S, C>` — receiving the
incoming `(value, state, context)` and producing the next flow. The unit SHALL be
`η(a, s, c) = CausalFlow::from_parts(Ok(EffectValue::Value(a)), s, c, EffectLog::new())`, and
composition SHALL be `f >=> g = λ(a, s, c). bind_stateful(f(a, s, c), g)`, threading `f`'s output
`(b, s1, c1)` into `g`. Composition SHALL short-circuit on error (left zero) and propagate
`None`/`ContextualLink` lawfully.

#### Scenario: State threads across a composed pipeline
- **WHEN** two stages that read and update the accumulated state are composed and run from `s0`
- **THEN** the first stage sees `s0`, the second stage sees the first stage's output state `s1`, and
  the pipeline result carries `s2` (state is threaded `s0 → s1 → s2`, not discarded)

#### Scenario: Error short-circuits and preserves state
- **WHEN** the first stage returns an errored flow with state `s1`
- **THEN** the second stage is not invoked and the result carries the error with state `s1` and the
  accumulated logs (left zero)

### Requirement: The arrow satisfies the Kleisli category laws over arbitrary state
Over arbitrary `S` and `C`, the arrow SHALL satisfy left identity `η >=> f = f`, right identity
`f >=> η = f`, associativity `(f >=> g) >=> h = f >=> (g >=> h)`, and error left-zero. Right identity
SHALL hold on the value fragment unconditionally (the `and_then` `None`-collapse is corrected), and
fully unconditionally once the control channel is separated.

#### Scenario: Identity laws hold with state threaded
- **WHEN** `η >=> f` and `f >=> η` are evaluated on a stage `f` that updates state
- **THEN** both equal `f` — state/context are threaded identically on each side (no `S,C` erasure)

#### Scenario: Associativity holds across state updates
- **WHEN** three state-updating stages are composed as `(f >=> g) >=> h` and `f >=> (g >=> h)`
- **THEN** the two composites produce identical value, state, context, and logs

### Requirement: Value-only stages are retained as the lawful state-preserving sub-case
The system SHALL retain the value-only builder `causal_arrow(f)` + `.next(g)` and the value-only
`CausalFlow::and_then`/`next`, lifted as the state-preserving Kleisli arrow `|a, s, c| f(a).with_state(s, c)` —
a lawful morphism that transforms the value while preserving `(s, c)`. Existing value-only callers
SHALL compile and behave unchanged (zero regression); a `run_value(a)` convenience SHALL call
`run((a, (), None))` for the `S = C = ()` case.

#### Scenario: Existing value-only pipeline is unaffected
- **WHEN** a pre-existing `causal_arrow(|x| ..).next(|y| ..)` pipeline is run via `run_value`
- **THEN** it produces the same result as before the change, with state/context preserved through each
  stage

#### Scenario: State-preserving lift is a lawful sub-case
- **WHEN** a value-only stage `f` is composed with a state-updating stage `g`
- **THEN** `f`'s lift preserves the incoming `(s, c)` and only transforms the value, and the composite
  obeys the category laws

### Requirement: The state-threading arrow is machine-checked
The arrow laws SHALL be formalized in `lean/DeepCausalityFormal/Core/CausalArrow.lean` with the stage
`A → S → Option C → Process B S C E Λ` (identical to `CausalMonad.lean`'s continuation), reducing
`kcomp_left_id`/`kcomp_right_id`/`kcomp_assoc`/`kcomp_left_zero` to the already-proved monad theorems
(arrow-from-monad, Mac Lane CWM §VI.5), and witnessed in
`deep_causality_core/tests/formalization_lean/causal_arrow_tests.rs` under ids
`core.causal_arrow.category_laws` and `core.causal_arrow.left_zero`.

#### Scenario: Lean file typechecks and reduces to the monad laws
- **WHEN** `lean lean/DeepCausalityFormal/Core/CausalArrow.lean` is run
- **THEN** it typechecks with zero `sorry`, and the four arrow theorems are closed by reduction to
  `bind_left_id`/`bind_right_id`/`bind_assoc`/`bind_raise_left_zero`

#### Scenario: Witnesses pin the Rust arrow
- **WHEN** `causal_arrow_tests.rs` runs
- **THEN** a reusable stateful pipeline threads accumulated state; left/right identity, associativity,
  and error short-circuit-preserves-state each have a passing witness carrying the `core.causal_arrow.*`
  id
