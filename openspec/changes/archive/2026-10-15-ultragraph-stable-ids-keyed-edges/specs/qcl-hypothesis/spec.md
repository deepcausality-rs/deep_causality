## MODIFIED Requirements

### Requirement: A hypothesis is admitted only when its supports validate against its factors

Construction of a structural `Hypothesis` SHALL run `FactorSupports::validate(&factors)` and SHALL
return the shipped `QuantumError` on failure, and construction from a graph SHALL reject a graph that
is not frozen.

`validate` already checks that every factor is a square, non-empty matrix whose dimension equals the
product of its declared support leg dimensions, and that no factor is keyed on a node with no
declared support. `FactorSupports::from_graph` already rejects an unfrozen graph and a factor keyed
on an index that holds no causaloid. `Hypothesis` calls these rather than repeating them.

#### Scenario: A factor whose dimension disagrees with its support is rejected

- **WHEN** node 2 carries a 4×4 factor while its declared support is the single leg `[2]` of
  dimension 2
- **THEN** construction fails with `QuantumError::DimensionMismatch` reporting that the factor at
  node 2 has dim 4 while its support implies 2, and no `Hypothesis` value is produced

#### Scenario: An unfrozen graph yields no hypothesis

- **WHEN** a structural candidate is built from a graph that has not been frozen
- **THEN** `FactorSupports::from_graph` returns `QuantumError::CalculationError` stating that a
  frozen graph is required, and the failure propagates out of construction

#### Scenario: A factor keyed outside the graph is rejected

- **WHEN** the factor store carries a factor keyed by node 9 and the frozen graph holds no causaloid
  at index 9
- **THEN** construction fails with `QuantumError::CalculationError` naming node 9 as holding no
  causaloid, rather than declaring node 9 as a lone qubit detached from the graph

#### Scenario: A factor above number_nodes on a live index is accepted

- **WHEN** index 1 of a five-causaloid graph is removed, the graph is frozen, and the factor store
  carries a factor keyed by node 4
- **THEN** construction derives node 4's support from its parents in the graph, although 4 is not
  below `number_nodes()`, which is 4
