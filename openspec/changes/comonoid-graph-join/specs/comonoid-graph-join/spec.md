## ADDED Requirements

### Requirement: A node fires only when all parents are ready

The engine SHALL evaluate a node only after every one of its parents has produced an effect, using a topological (Kahn-style) order over the frozen acyclic graph. It SHALL NOT evaluate a reconvergence node on the first parent to reach it. The graph MUST be acyclic (enforced via `ultragraph::has_cycle`/`freeze`); a cyclic graph is rejected and the cyclic join is out of scope.

#### Scenario: Reconvergence node waits for all parents

- **WHEN** a node has parents `P1` and `P2` and the engine reaches it via `P1` first
- **THEN** the node is not evaluated until `P2` has also produced its effect

#### Scenario: Linear and tree graphs are unaffected

- **WHEN** every node has at most one parent
- **THEN** evaluation results are identical to the previous engine (a join of one parent is identity)

### Requirement: Fan-in combines parents via an associative-commutative join

At a reconvergence, the engine SHALL reduce the parent effects to a single effect using a declared **associative-commutative** join-combine, and feed that single effect to the node (the causaloid input signature is unchanged). The channels SHALL combine as: value via the declared commutative-associative monoid; log via deterministic canonical-order (node-index) concatenation; error via short-circuit (a left-zero); state/context via the declared combine.

#### Scenario: Value combine is order-independent

- **WHEN** the same set of parent effects is combined in two different orders
- **THEN** the resulting node input value is identical (the combine is a commutative-associative monoid)

#### Scenario: A parent error short-circuits the join

- **WHEN** any parent effect carries an error
- **THEN** the combined effect is an error (left-zero), and its logs are preserved in canonical order

### Requirement: Whole-graph fold is invariant under topological linearization

The formalization SHALL prove, in `Core/GraphJoin.lean`, that the result of folding the graph is independent of which valid topological linearization is chosen — i.e. reconvergence is deterministic. This reduces to commutative-monoid fold order-invariance plus the comonoid copy law for fan-out. The theorem MUST carry a `THEOREM_MAP.md` row and a Rust witness, and the file MUST typecheck standalone with bare `lean`.

#### Scenario: Two topological orders agree

- **WHEN** a diamond graph is folded under two distinct valid topological orders
- **THEN** the model proves the two results equal, and the Rust witness confirms it on the real engine

#### Scenario: Fan-out is comonoid copy

- **WHEN** a node's output feeds multiple children
- **THEN** each child receives the same effect (copy), matching the existing fan-out behaviour, and this is stated as the comonoid copy law

### Requirement: Reconvergent tests updated to joined semantics

Existing tests that build reconvergent (multi-parent) graphs SHALL be updated to assert the joined result rather than the previous first-parent-wins outcome. The change SHALL first scan the graph tests/examples to enumerate the affected reconvergent graphs.

#### Scenario: Diamond test asserts the join

- **WHEN** the `logic_graph` diamond (node with two parents) is evaluated
- **THEN** its test asserts the combined (joined) effect, and the assertion is derived from the declared join-combine, not from BFS arrival order
