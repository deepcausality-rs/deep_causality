# lawful-effect-channel Specification (delta)

## MODIFIED Requirements

### Requirement: Value-XOR-error is structurally unrepresentable as anything else
The carrier `CausalEffectPropagationProcess<Value, State, Context, Error, Log>` SHALL encode its
outcome as a single field of a **3-way sum** — `Value(EffectValue<Value>) | Error(Error) |
Control(CausalCommand<Value>)` — where `EffectValue` is the pure value functor `{ None, Value,
ContextualLink }` and `CausalCommand` is the control operation functor. The W-invariant SHALL hold on
the value/error arms (a carrier holding both a value and an error SHALL NOT be constructible), and the
control operation SHALL occupy its own arm rather than re-widening the value carrier. All fields SHALL
be private, and every constructor SHALL be total over this representation.

#### Scenario: Invalid state cannot be constructed
- **WHEN** any code outside `deep_causality_core`'s carrier module attempts to construct a carrier
  holding both a value and an error, or a value simultaneously with a control operation
- **THEN** the program does not compile (no public field access; the outcome is one 3-way field, so
  the invalid combinations are unrepresentable)

#### Scenario: Total constructor accepts every representable state
- **WHEN** `new(outcome, state, context, logs)` is called with any value of the 3-way outcome sum
- **THEN** a well-formed carrier is produced (every representable outcome is valid — no validation,
  cannot fail)

#### Scenario: Control is a distinct arm
- **WHEN** a causaloid emits a `RelayTo` or `Dispatch` control operation
- **THEN** the carrier holds `Control(CausalCommand::RelayTo(..))` / `Control(CausalCommand::Dispatch(..))`,
  `value()` returns `None`, and `error()` returns `None`

### Requirement: Accessor surface
The carrier SHALL expose read access exclusively through getters:
`outcome()` returning a reference to the 3-way outcome sum,
`value() -> Option<&Value>` (the carried scalar — `Some` only for `EffectValue::Value`, `None` on an
errored, `None`-effect, `ContextualLink`, or control carrier),
`value_cloned() -> Option<Value>` (where `Value: Clone`) and `into_value(self) -> Option<Value>`
(the owned, borrowing/consuming counterparts of `value()`),
`effect() -> Option<&EffectValue<Value>>` (the value wrapper, for discriminating the `None` /
`Value` / `ContextualLink` variants only),
`control() -> Option<&CausalCommand<Value>>` (the control operation, `Some` only on a `Control`
carrier — for the reasoning handler to dispatch `RelayTo`/`Dispatch`),
`error() -> Option<&Error>`, `state() -> &State`, `context() -> &Option<Context>`,
`logs() -> &Log`, and the predicates `is_ok()` / `is_err()` generalized to all `State` and `Context`
parameters.

#### Scenario: Errored carrier exposes no value
- **WHEN** `value()` is called on a carrier constructed with `from_error(e)`
- **THEN** it returns `None`, `error()` returns `Some(&e)`, and `control()` returns `None`

#### Scenario: Valued carrier exposes its scalar
- **WHEN** `value()` is called on a carrier constructed with `pure(v)`
- **THEN** it returns `Some(&v)`, `effect()` returns `Some(&EffectValue::Value(v))`, `error()` returns
  `None`, and `control()` returns `None`

#### Scenario: Control carrier exposes its operation
- **WHEN** `control()` is called on a carrier holding `Control(CausalCommand::RelayTo(target, inner))`
- **THEN** it returns `Some(&CausalCommand::RelayTo(target, inner))`, while `value()`, `effect()`, and
  `error()` all return `None`
