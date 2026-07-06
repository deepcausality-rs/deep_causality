## ADDED Requirements

### Requirement: Value-XOR-error is structurally unrepresentable as anything else
The carrier `CausalEffectPropagationProcess<Value, State, Context, Error, Log>` SHALL encode
its value and error channels as a single field of type
`Result<EffectValue<Value>, Error>` (the `Either E (Maybe T)` encoding of Formalization.md
precondition P2). A carrier that simultaneously holds a value and an error SHALL NOT be
constructible by any means: all fields SHALL be private, and every constructor SHALL be
total over the new representation.

#### Scenario: Invalid state cannot be constructed
- **WHEN** any code outside `deep_causality_core`'s carrier module attempts to construct a
  carrier holding both a value and an error
- **THEN** the program does not compile (no public field access, no constructor accepts
  both channels)

#### Scenario: Total constructor accepts every representable state
- **WHEN** `new(outcome, state, context, logs)` is called with any
  `Result<EffectValue<Value>, Error>`
- **THEN** a well-formed carrier is produced (with one channel, every representable state is
  valid — the constructor needs no validation and cannot fail)

### Requirement: Constructor surface
The carrier SHALL provide the constructors `pure(value)`, `none()`, `from_error(error)`,
`from_effect_value(effect_value)`, `from_effect_value_with_log(effect_value, logs)`,
`from_value(value)`, `from_value_with_log(value, logs)`, `with_state(effect, state, context)`
and `new(outcome, state, context, logs)`, preserving their current semantics on the new
encoding.

#### Scenario: pure produces the monadic unit
- **WHEN** `pure(v)` is called
- **THEN** the carrier holds `Ok(EffectValue::Value(v))`, default state, no context, and an
  empty log

#### Scenario: from_error produces the raise generator
- **WHEN** `from_error(e)` is called
- **THEN** the carrier holds `Err(e)`, default state, no context, and an empty log

### Requirement: Accessor surface
The carrier SHALL expose read access exclusively through getters:
`outcome() -> &Result<EffectValue<Value>, Error>`,
`value() -> Option<&Value>` (the carried scalar — `Some` only for `EffectValue::Value`, `None`
on an errored, `None`-effect, or dispatch carrier),
`value_cloned() -> Option<Value>` (where `Value: Clone`) and `into_value(self) -> Option<Value>`
(the owned, borrowing/consuming counterparts of `value()`),
`effect() -> Option<&EffectValue<Value>>` (the full effect-value wrapper, for discriminating the
`None` / `ContextualLink` / `RelayTo` / `Map` variants),
`error() -> Option<&Error>`, `state() -> &State`, `context() -> &Option<Context>`,
`logs() -> &Log`, and the predicates `is_ok()` / `is_err()` generalized to all `State` and
`Context` parameters.

> Deviation from the original design (D3): `value()` was first specified as returning the
> `EffectValue<Value>` wrapper (`Option<&EffectValue<Value>>`). During the downstream mop-up this
> proved unusable — extracting the carried scalar took a five-term chain
> (`value().unwrap().clone().into_value().unwrap()…`). `value()` was repurposed to return the inner
> scalar directly; the wrapper moved to `effect()`, and `value_cloned()` / `into_value()` were added
> for owned access. This is the ergonomic accessor surface the rest of the requirement now reflects.

#### Scenario: Errored carrier exposes no value
- **WHEN** `value()` is called on a carrier constructed with `from_error(e)`
- **THEN** it returns `None`, and `error()` returns `Some(&e)`

#### Scenario: Valued carrier exposes its scalar
- **WHEN** `value()` is called on a carrier constructed with `pure(v)`
- **THEN** it returns `Some(&v)`, `effect()` returns `Some(&EffectValue::Value(v))`, and `error()` returns `None`

### Requirement: Monad laws hold unconditionally
`bind` and `pure` SHALL satisfy the three Kleisli-triple laws (Moggi 1991) for ALL carriers —
including errored ones — with no well-formedness precondition: left identity
`bind(pure(a), f) = f(a)`, right identity `bind(m, pure) = m`, and associativity
`bind(bind(m, f), g) = bind(m, |x| bind(f(x), g))`.

#### Scenario: Right identity preserves an errored carrier verbatim
- **WHEN** `bind(m, pure)` is evaluated on a carrier `m` holding `Err(e)` with state `s`,
  context `c`, and logs `l`
- **THEN** the result equals `m` exactly — error, state, context, and logs all preserved,
  and no value is fabricated or destroyed

#### Scenario: Associativity across an erroring continuation
- **WHEN** `f` returns an errored carrier and `g` is any continuation
- **THEN** `bind(bind(m, f), g)` equals `bind(m, |x| bind(f(x), g))`, with identical logs

### Requirement: Error short-circuit is a left zero
On an errored carrier, `bind`, `bind_or_error`, and `fmap` SHALL NOT invoke their
continuation or mapping function; they SHALL propagate the error while preserving state,
context, and accumulated logs.

#### Scenario: Continuation does not run under error
- **WHEN** `bind(from_error(e), f)` is evaluated where `f` sets an observable flag
- **THEN** the flag is not set and the result carries `Err(e)`

#### Scenario: Logs survive the short-circuit
- **WHEN** an errored carrier with a non-empty log is bound
- **THEN** the result carries the same log (the audit trail survives failure — the
  transformer-stack ordering `StateCC ∘ ExceptT E ∘ Writer L` of Formalization.md §1)

### Requirement: Machine-checked verification
The laws SHALL be verified at three layers, bound by shared `THEOREM_MAP` ids: Lean theorems
over a model whose `pure`/`bind` transcribe the Rust bodies
(`core.causal_monad.left_id`, `core.causal_monad.right_id`, `core.causal_monad.assoc` —
the latter two moving out of the "blocked on P2" section), Kani bounded harnesses over the
real Rust `bind` (no-panic, error short-circuit, log monotonicity; the former W-well-formedness
harness obligation is discharged by construction), and one Rust witness test per id.

#### Scenario: THEOREM_MAP consistency gate passes
- **WHEN** the formalization CI job runs after the change
- **THEN** `core.causal_monad.right_id` and `core.causal_monad.assoc` are proved in Lean,
  witnessed in Rust, listed in `THEOREM_MAP.md`, and the consistency gate exits 0
