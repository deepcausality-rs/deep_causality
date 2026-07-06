# lawful-effect-channel Specification

## Purpose
TBD - created by archiving change enforce-w-invariant. Update Purpose after archive.
## Requirements
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

