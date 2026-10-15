## ADDED Requirements

### Requirement: A teloid pair carries one relation, and a conflicting one is refused
`TeloidGraph` SHALL hold at most one relation per ordered pair of teloids, and `add_inheritance_edge` and `add_defeasance_edge` SHALL return `DeonticError::ConflictingRelation { from, to }` when the pair already carries the other relation, leaving the existing relation in place, and SHALL accept re-adding the relation the pair already carries.

A norm that both inherits from another norm and defeats it is a contradiction in the norm base; refusing the second relation surfaces it to the author instead of resolving it silently.

#### Scenario: Defeasance after inheritance is refused
- **WHEN** `add_inheritance_edge(0, 1)` succeeds and `add_defeasance_edge(0, 1)` is called
- **THEN** it returns `Err(DeonticError::ConflictingRelation { from: 0, to: 1 })` and the pair still carries `Inherits`

#### Scenario: Inheritance after defeasance is refused
- **WHEN** `add_defeasance_edge(0, 1)` succeeds and `add_inheritance_edge(0, 1)` is called
- **THEN** it returns `Err(DeonticError::ConflictingRelation { from: 0, to: 1 })` and the pair still carries `Defeats`

#### Scenario: Re-adding the same relation is accepted
- **WHEN** `add_inheritance_edge(0, 1)` is called twice
- **THEN** both calls succeed and `number_edges()` is 1

#### Scenario: The opposite direction is a different pair
- **WHEN** `add_inheritance_edge(0, 1)` and `add_defeasance_edge(1, 0)` are called
- **THEN** both succeed and `number_edges()` is 2

#### Scenario: EffectEthos reports the conflict
- **WHEN** `link_inheritance(a, b)` succeeds and `link_defeasance(a, b)` is called on an `EffectEthos`
- **THEN** it returns `Err(DeonticError::ConflictingRelation { .. })` naming the two graph indices
