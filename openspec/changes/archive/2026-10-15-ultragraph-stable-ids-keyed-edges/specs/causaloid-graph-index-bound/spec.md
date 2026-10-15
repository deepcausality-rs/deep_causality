## ADDED Requirements

### Requirement: CausableGraph reports one past its highest live index
`CausableGraph` SHALL provide `index_bound()`, returning one past the highest index that holds a causaloid and 0 for a graph holding none.

#### Scenario: The bound of an empty graph
- **WHEN** `index_bound()` is called on a graph with no causaloid
- **THEN** it returns 0

#### Scenario: The bound skips a removed causaloid
- **WHEN** causaloids are added at indices 0, 1 and 2 and index 1 is removed
- **THEN** `index_bound()` returns 3 while `number_nodes()` returns 2

#### Scenario: The bound falls with a removed last causaloid
- **WHEN** causaloids are added at indices 0, 1 and 2 and index 2 is removed
- **THEN** `index_bound()` returns 2

### Requirement: Reasoning reaches every causaloid after a removal and a freeze
`MonadicCausableGraphReasoning` and `StatefulMonadicCausableGraphReasoning` SHALL size their per-node bookkeeping by `index_bound()`, and SHALL evaluate a frozen graph with removed causaloids without panicking, reaching every causaloid at the index `add_causaloid` returned.

#### Scenario: Evaluation crosses a removed index
- **WHEN** causaloids are added at 0, 1 and 2, edge 0 → 2 exists, index 1 is removed, the graph is frozen, and evaluation starts at 0 in each engine
- **THEN** each engine returns the effect causaloid 2 produces from causaloid 0's output, and does not panic

### Requirement: The single-writer check validates writers by presence
`check_single_writer` SHALL size its cones by `index_bound()`, SHALL inspect every join whose index holds a causaloid, and SHALL refuse a declared writer whose index holds no causaloid with a `CausalityGraphError` naming that index.

#### Scenario: A writer above number_nodes is accepted
- **WHEN** index 1 of a four-causaloid graph is removed and `freeze_verified(&[3])` is called on an acyclic graph with no conflicting join
- **THEN** it returns `Ok(())`

#### Scenario: A writer on a removed index is refused
- **WHEN** `freeze_verified(&[1])` is called after index 1 was removed
- **THEN** it returns a `CausalityGraphError` naming index 1, and the graph is left unfrozen

#### Scenario: A join above number_nodes is inspected
- **WHEN** index 0 is removed from a graph whose diamond join sits at index 4 with exclusive writers on both branches
- **THEN** `freeze_verified` returns the single-writer violation naming join 4

#### Scenario: The check examines a dynamic graph
- **WHEN** `check_single_writer` is called on an unfrozen graph holding a two-writer diamond
- **THEN** it returns the single-writer violation naming the join

### Requirement: Indices held by callers name the same causaloid after a freeze
An index taken from `add_causaloid` SHALL name the same causaloid after any removal of another causaloid and any freeze, so `LambdaEdges` keys, `freeze_verified` writer declarations and `RelayTo` targets keep their meaning.

#### Scenario: A decorated edge keeps its decoration
- **WHEN** edge 2 → 3 carries a Λ that doubles the value, index 1 is removed, the graph is frozen, and evaluation starts at 2
- **THEN** the value arriving at causaloid 3 is double the value causaloid 2 produced

### Requirement: A repeated edge between two causaloids updates its weight
`CausaloidGraph` SHALL hold at most one edge per ordered pair, so `add_edge(a, b)` followed by `add_edg_with_weight(a, b, w)` leaves one edge from `a` to `b` with weight `w`.

#### Scenario: One edge after two additions
- **WHEN** `add_edge(0, 1)` and then `add_edg_with_weight(0, 1, 7)` are called
- **THEN** `number_edges()` returns 1
