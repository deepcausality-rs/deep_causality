# lawful-effect-channel Specification (delta)

## MODIFIED Requirements

### Requirement: Value-XOR-error is structurally unrepresentable as anything else
The carrier `CausalEffectPropagationProcess<Value, State, Context, Error, Log>` SHALL encode its
outcome as a single field `Result<CausalEffect<Value>, Error>` =
`Except E (Free CausalCommand (Maybe V))`, where
`CausalEffect<V> = Free<CausalCommandWitness, Option<V>>` is the free-monad success channel unifying
value / absence / command (`Pure(Some v)` / `Pure(None)` / `Suspend(RelayTo(..))`). The W-invariant
SHALL hold (a carrier holding both a value and an error SHALL NOT be constructible), and a command SHALL
be carried inside the `Ok` effect (not a separate field), so value/command are mutually exclusive by the
`Free` structure. All fields SHALL be private, and every constructor SHALL be total over this
representation.

#### Scenario: Invalid state cannot be constructed
- **WHEN** any code outside `deep_causality_core`'s carrier module attempts to construct a carrier
  holding both a value and an error
- **THEN** the program does not compile (no public field access; error XOR effect is the `Result`)

#### Scenario: Total constructor accepts every representable state
- **WHEN** `new(outcome, state, context, logs)` is called with any `Result<CausalEffect<Value>, Error>`
- **THEN** a well-formed carrier is produced (every representable outcome is valid — no validation,
  cannot fail)

#### Scenario: A command is an effect, not a value
- **WHEN** a causaloid emits a `RelayTo` command
- **THEN** the carrier holds `Ok(CausalEffect)` whose effect is a command, `value()` returns `None`,
  `command_target()` returns `Some(target)`, and `error()` returns `None`

### Requirement: Accessor surface
The carrier SHALL expose read access exclusively through getters:
`outcome()` returning a reference to `Result<CausalEffect<Value>, Error>`,
`value() -> Option<&Value>` (the carried scalar — `Some` only for a value effect, `None` on an errored,
`None`-effect, or command carrier),
`value_cloned() -> Option<Value>` (where `Value: Clone`) and `into_value(self) -> Option<Value>`
(the owned, borrowing/consuming counterparts of `value()`),
`effect() -> Option<&CausalEffect<Value>>` (the whole effect, for discriminating value / none / command
via `CausalEffect::is_command` etc.),
`command_target() -> Option<usize>` (the jump target on a command carrier, else `None`),
`error() -> Option<&Error>`, `state() -> &State`, `context() -> &Option<Context>`,
`logs() -> &Log`, and the predicates `is_ok()` / `is_err()` generalized to all `State` and `Context`
parameters.

#### Scenario: Errored carrier exposes no value
- **WHEN** `value()` is called on a carrier constructed with `from_error(e)`
- **THEN** it returns `None`, `error()` returns `Some(&e)`, and `command_target()` returns `None`

#### Scenario: Valued carrier exposes its scalar
- **WHEN** `value()` is called on a carrier constructed with `pure(v)`
- **THEN** it returns `Some(&v)`, `effect()` returns `Some(&CausalEffect)` (a value effect), `error()`
  returns `None`, and `command_target()` returns `None`

#### Scenario: Command carrier exposes its target
- **WHEN** `command_target()` is called on a carrier holding a `RelayTo(target, sub)` command
- **THEN** it returns `Some(target)`, while `value()` and `error()` return `None`
