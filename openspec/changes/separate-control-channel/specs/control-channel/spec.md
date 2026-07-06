# control-channel Specification

## ADDED Requirements

### Requirement: The control operations form a dedicated functor
The control operations SHALL live in a dedicated type `CausalCommand<T>` with exactly the variants
`RelayTo(usize, Box<PropagatingEffect<T>>)` (a computed jump to a target causaloid) and
`Dispatch(HashMap<IdentificationValue, Box<PropagatingEffect<T>>>)` (a dispatch table; the former
`EffectValue::Map`). `CausalCommand` SHALL NOT appear inside `EffectValue`. It SHALL implement a
lawful `Functor` whose `fmap` is total (identity and composition hold on both variants).

#### Scenario: RelayTo and Dispatch are the only control variants
- **WHEN** `CausalCommand<T>` is inspected
- **THEN** it has exactly `RelayTo` and `Dispatch`, and `EffectValue<T>` has exactly `None`, `Value`,
  and `ContextualLink` (no control variant)

#### Scenario: Functor laws hold totally
- **WHEN** `fmap(id)` and `fmap(g ∘ f)` are applied to any `CausalCommand` value
- **THEN** `fmap(id) = id` and `fmap(g ∘ f) = fmap(g) ∘ fmap(f)`, with no panic and no variant excluded

### Requirement: Control equality is a lawful congruence
`CausalCommand` SHALL provide an equality that is a true equivalence relation and a congruence:
`RelayTo` SHALL compare BOTH its target index AND its boxed payload (recursively), and `Dispatch`
SHALL compare its map structurally (keys and recursively-compared values). The former partial-
equivalence relation on the fused type — `Map(_) == Map(_)` always `false`, `RelayTo` ignoring its
payload — SHALL NOT exist anywhere in the crate.

#### Scenario: RelayTo compares its full payload
- **WHEN** two `RelayTo(t, a)` and `RelayTo(t, b)` with `a != b` are compared
- **THEN** they are unequal (the payload is not ignored)

#### Scenario: Dispatch is reflexive
- **WHEN** a `Dispatch(m)` is compared with itself
- **THEN** it is equal (reflexivity holds — the non-reflexive `Map` behavior is gone)

### Requirement: EffectValue is a lawful pointed functor after separation
With the control variants removed, `EffectValue<T> = { None, Value(T), ContextualLink(id, id) }` SHALL
be a lawful pointed functor: `fmap` total over all three constructors, a **derived** congruent
`PartialEq` (`#[derive(PartialEq)]`, no hand-written variant carve-outs), and `into_value` the honest
`Maybe` projection (`Value(v) → Some(v)`; `None` / `ContextualLink → None`). The arity-5 witness
`fmap` SHALL NOT contain a panic path.

#### Scenario: into_value is the honest Maybe projection
- **WHEN** `into_value` is called on `Value(v)`, `None`, and `ContextualLink(a, b)`
- **THEN** it returns `Some(v)`, `None`, and `None` respectively — no "dispatch collapses to None"
  conflation remains

#### Scenario: No reachable panic in the functor witnesses
- **WHEN** any witness `fmap` (arity-5, effect, process) runs on any clean-functor carrier
- **THEN** it returns without panicking and agrees with the inherent `fmap`

### Requirement: Adaptive reasoning consumes control via the handler seam
The graph-reasoning engine and CSM evaluator SHALL read the control operation from the carrier's
`Control` arm (`control()` / `CausalCommand::RelayTo` / `CausalCommand::Dispatch`) rather than from an
`EffectValue` variant. The BFS that resolves a `RelayTo` jump SHALL be the handler that interprets the
operation functor; its observable behavior (jump to target index, pass the inner effect, stop on a
control carrier) SHALL be preserved.

#### Scenario: RelayTo still jumps to its target
- **WHEN** a causaloid returns `Control(CausalCommand::RelayTo(target, inner))` during graph reasoning
- **THEN** the relayed-to node at `target` receives `inner` as input, exactly as before the change
  (behavior preserved; only the carrier arm changed)

#### Scenario: Missing target is still an error
- **WHEN** a `RelayTo(target, _)` names an index absent from the graph
- **THEN** reasoning returns the same "target causaloid with index N not found" error as before
