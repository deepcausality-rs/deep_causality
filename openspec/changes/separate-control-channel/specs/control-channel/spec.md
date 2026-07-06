# control-channel Specification

## ADDED Requirements

### Requirement: The dead `Map` operation is removed
The `Map` variant of `EffectValue` SHALL be removed entirely, together with `is_map`, its `Display`
and `PartialEq` arms, and its unit tests. It SHALL NOT be renamed or relocated — usage verification
found it constructed only in its own tests, never in production, example, or reasoning code.

#### Scenario: No Map remains
- **WHEN** the codebase is searched for `EffectValue::Map` / `is_map`
- **THEN** there are no matches outside removed test history, and `EffectValue` has exactly `None`,
  `Value`, and `ContextualLink`

### Requirement: Control operations form the free-monad operation functor
The control operation SHALL live in a dedicated single-operation functor
`CausalCommand<K> = { RelayTo(usize, K) }`, where `K` is the sub-program hole. `CausalCommandWitness`
SHALL implement `HKT<Constraint = NoConstraint>` (`Type<K> = CausalCommand<K>`) and a lawful
`Functor` (`fmap` maps the `RelayTo` hole; identity and composition hold). `CausalCommand` SHALL NOT
appear inside `EffectValue`.

#### Scenario: CausalCommand is a lawful single-hole functor
- **WHEN** `fmap(id)` and `fmap(g ∘ f)` are applied to a `CausalCommand::RelayTo(t, k)`
- **THEN** `fmap(id) = id` and `fmap(g ∘ f) = fmap(g) ∘ fmap(f)`, mapping only the hole `k`

### Requirement: The adaptive-reasoning program is a haft Free monad
The adaptive-reasoning program SHALL be `deep_causality_haft::Free<CausalCommandWitness, EffectValue<V>>`
— `Pure(EffectValue<V>)` leaves and `Suspend(CausalCommand<Box<Free<…>>>)` branches — reusing the haft
free monad, not a bespoke recursion. The carrier's `Control` arm SHALL hold this program, and the graph
reasoning engine SHALL interpret it via `Free::fold` (the F-algebra handler): `pure_case` emits a leaf
effect, the `algebra` resolves `RelayTo(target, sub)`. The free-monad laws SHALL be cited from
`haft.free_monad.*` (not re-proved); this change SHALL prove only that `CausalCommandWitness` is a lawful
functor (the precondition `Free` requires).

#### Scenario: The reasoning handler is a fold over the Free program
- **WHEN** a causaloid emits a control program and reasoning interprets it
- **THEN** the interpretation is `Free::fold(pure_case, algebra)`, with `algebra` performing the
  jump-to-target, and the result equals the pre-change BFS on every jump/leaf

#### Scenario: Program equality by fold-canonicalization
- **WHEN** two control programs are compared for equality
- **THEN** equality is decided by folding each to a canonical value (as the haft free-monad witnesses
  do); the former partial-equivalence relation (`Map(_) == Map(_) = false`, target-only `RelayTo`) does
  not exist anywhere

### Requirement: EffectValue is a lawful pointed functor after separation
With control removed, `EffectValue<T> = { None, Value(T), ContextualLink(id, id) }` SHALL be a lawful
pointed functor: `fmap` total over all three constructors, a **derived** congruent `PartialEq`
(`#[derive(PartialEq)]`, no hand-written carve-outs), and `into_value` the honest `Maybe` projection
(`Value(v) → Some(v)`; `None` / `ContextualLink → None`). The arity-5 witness `fmap` SHALL NOT contain
a panic path.

#### Scenario: into_value is the honest Maybe projection
- **WHEN** `into_value` is called on `Value(v)`, `None`, and `ContextualLink(a, b)`
- **THEN** it returns `Some(v)`, `None`, and `None` respectively — no "dispatch collapses to None"
  conflation remains

#### Scenario: No reachable panic in the functor witnesses
- **WHEN** any witness `fmap` (arity-5, effect, process) runs on any clean-functor carrier
- **THEN** it returns without panicking and agrees with the inherent `fmap`

### Requirement: Adaptive reasoning consumes control via the fold handler
The graph-reasoning engine and CSM evaluator SHALL read the control program from the carrier's `Control`
arm (`control()`), not from an `EffectValue` variant, and interpret it via the `Free::fold` handler. The
observable behavior SHALL be preserved: jump to the target index feeding the sub-program, thread/combine
state/context/logs, error on a missing target, and stop on a control carrier.

#### Scenario: RelayTo still jumps to its target
- **WHEN** a causaloid returns a control program `RelayTo(target, sub)` during graph reasoning
- **THEN** the node at `target` receives `sub` as input and reasoning continues, exactly as before the
  change (behavior preserved; only the representation changed)

#### Scenario: Missing target is still an error
- **WHEN** a `RelayTo(target, _)` names an index absent from the graph
- **THEN** reasoning returns the same "target causaloid with index N not found" error as before
