# control-channel Specification

## Purpose
TBD - created by archiving change separate-control-channel. Update Purpose after archive.
## Requirements
### Requirement: The dead `Map` and `ContextualLink` operations are removed
The system SHALL remove the `Map` variant of `EffectValue` and the `ContextualLink` concept on **both**
sides (the `EffectValue` output form and the `ActionParameterValue` input-form mirror) entirely —
together with their predicates, `Display`/`PartialEq` arms, and unit tests. They SHALL NOT be renamed or
relocated; usage verification found them constructed only in their own tests, never interpreted.

#### Scenario: No dead control surface remains
- **WHEN** the codebase is searched for `EffectValue::Map`, `is_map`, or `ContextualLink`
- **THEN** there are no matches in production, example, or reasoning code (only removed test history)

### Requirement: Control operations form the free-monad operation functor
The control operation SHALL live in a dedicated single-operation functor
`CausalCommand<K> = { RelayTo(usize, K) }`, where `K` is the sub-program hole. `CausalCommandWitness`
SHALL implement `HKT<Constraint = NoConstraint>` (`Type<K> = CausalCommand<K>`) and a lawful `Functor`
(`fmap` maps the `RelayTo` hole; identity and composition hold). `CausalCommand` SHALL NOT be a variant
of the value channel.

#### Scenario: CausalCommand is a lawful single-hole functor
- **WHEN** `fmap(id)` and `fmap(g ∘ f)` are applied to a `CausalCommand::RelayTo(t, k)`
- **THEN** `fmap(id) = id` and `fmap(g ∘ f) = fmap(g) ∘ fmap(f)`, mapping only the hole `k`

### Requirement: The adaptive-reasoning effect is a unified haft Free monad
The adaptive-reasoning success channel SHALL be the newtype
`CausalEffect<V> = deep_causality_haft::Free<CausalCommandWitness, Option<V>>` — reusing the haft free
monad, with `Option<V>` (`Maybe`) value leaves. It SHALL unify value, absence, and command as one type:
`Pure(None)` is the absence effect, `Pure(Some(v))` is a value, `Suspend(CausalCommand::RelayTo(t, k))`
is a command. `CausalEffect` SHALL provide a **total** functor `map` (mapping the `Option` leaves through
the `Free`, no panic and no error), a `fold` handler (the catamorphism the reasoning engine specializes),
and a congruent structural `PartialEq`/`Clone`/`Debug` (`Free` derives none). The free-monad laws SHALL
be cited from `haft.free_monad.*`, not re-proved.

#### Scenario: One type carries value, none, and command
- **WHEN** `CausalEffect::value(v)`, `CausalEffect::none()`, and `CausalEffect::relay_to(t, sub)` are
  built and discriminated
- **THEN** they report `is_value` / `is_none` / `is_command` respectively, `into_value` yields the scalar
  only for a value, and `command_target`/`into_command` yield the target and sub-program only for a command

#### Scenario: map is total (no panic, no error)
- **WHEN** `map(f)` is applied to a value, a `None`, or a command effect
- **THEN** it maps the value, passes `None` through, and maps the command's sub-program leaves — never
  panicking (the former arity-5 `.expect` bug, deviation D15, is gone) and never manufacturing an error

### Requirement: The carrier outcome is the free-monad effect XOR an error
The carrier `CausalEffectPropagationProcess` outcome SHALL be
`Result<CausalEffect<Value>, Error>` = `Except E (Free CausalCommand (Maybe V))` — a monad transformer
stack of three already-proven monads (`Except`, the free monad, `Maybe`). The W-invariant SHALL hold on
value/error (never both). A command SHALL be a **second left-zero** of the monad `bind` (like error): a
value-level `bind`/`map` on a command SHALL NOT retype it — it is interpreted by the reasoning engine's
`Free::fold` handler before any value-level step, so the value monad-law surface is the `Pure` fragment.
The carrier SHALL expose `effect() -> Option<&CausalEffect<Value>>`, `command_target() -> Option<usize>`,
and `into_command()`; `value()`/`into_value()` project the `Maybe`.

#### Scenario: Value fragment obeys the monad laws; command short-circuits
- **WHEN** the carrier holds a value/`None` effect
- **THEN** `bind`/`map` behave as the state-threading causal monad (`CausalMonad.lean`); **WHEN** it
  holds a command, a value-level step short-circuits it (unreachable-defensive — the engine folds it)

### Requirement: Adaptive reasoning consumes control via the fold handler
The graph-reasoning engine SHALL read the command from the carrier (`command_target()` / `into_command()`)
and interpret it via the `Free::fold` handler, not via a value-channel variant. Observable behavior SHALL
be preserved: jump to the target feeding it the command's folded sub-program value, thread/combine the
relaying node's state/context/logs, error on a missing target, and stop on a command carrier.

#### Scenario: RelayTo still jumps to its target
- **WHEN** a causaloid returns a `RelayTo(target, sub)` command during graph reasoning
- **THEN** the node at `target` receives the folded sub-program value as input and reasoning continues,
  exactly as before the change (behavior preserved; only the representation changed)

#### Scenario: Missing target is still an error
- **WHEN** a `RelayTo(target, _)` names an index absent from the graph
- **THEN** reasoning returns the same "target causaloid with index N not found" error as before

