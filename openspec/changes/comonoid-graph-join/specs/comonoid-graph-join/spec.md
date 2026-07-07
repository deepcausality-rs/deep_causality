## ADDED Requirements

### Requirement: Wire-slot topological sequencing replaces first-parent-wins

The engine SHALL evaluate the reachable sub-DAG by resolving each node's in-wires and firing a node only after all of its in-wires are resolved, using a reachability pre-pass from the start node and an ascending-node-index canonical schedule. It SHALL NOT evaluate a node on the first parent to reach it (BFS arrival order). The classical run REQUIRES a frozen acyclic graph (`ultragraph::has_cycle`); a cyclic graph is rejected. A wire from a non-descendant of the start node is resolved inactive by the reachability pre-pass and never gates a node.

#### Scenario: Linear and tree graphs are bit-identical

- **WHEN** every node has at most one parent
- **THEN** evaluation results are identical to the previous engine

#### Scenario: Mid-graph start does not deadlock

- **WHEN** evaluation starts at a node `S` and a downstream node has a parent that is not a descendant of `S`
- **THEN** that parent's wire is resolved inactive at initialization and the node fires from its reachable ancestors alone

#### Scenario: Cyclic graph is rejected

- **WHEN** a frozen graph contains a directed cycle
- **THEN** evaluation returns a `CausalityError` rather than silently skipping the cycle's nodes

### Requirement: A single fired parent is the identity

A node reached at run time by exactly one fired parent SHALL consume that parent's effect unchanged (no merge is involved) and evaluate normally.

#### Scenario: Single-fired reconvergence evaluates

- **WHEN** a structurally reconvergent node is reached from a start such that only one of its parents is a descendant of the start
- **THEN** the node evaluates that single parent's effect as-is and returns a result

### Requirement: Multi-parent reconvergence fails loudly (merge ∇ deferred)

When two or more parents fire into a node, the engine SHALL return a descriptive `CausalityError` that names the node and its fired parents and states that the reconvergence merge (∇) is not yet defined. It SHALL NOT silently select one parent, drop the others, or apply an undeclared combine. The merge — a symmetric-monoidal generator (copy/discard comonoid + merge) over the effect monad, an extension of the single-input causaloid — is deferred to a dedicated change (see `algebraic-causaloid-assumptions.md` #2). No `join_fn`, `ParentEffects`, or join kernel is exposed on the user-facing API.

#### Scenario: Root-start diamond errors loudly

- **WHEN** `evaluate_subgraph_from_cause` starts at the root of a diamond `root → {A, B} → C`, so both `A` and `B` fire into `C`
- **THEN** the result is a `CausalityError` identifying node `C` and its fired parents, and stating the reconvergence merge (∇) is undefined

#### Scenario: No multi-input merge surface leaks to the user

- **WHEN** a user constructs a `Causaloid`
- **THEN** there is no `join_fn`/`context_join_fn` field, no `new_join*` constructor, and no `ParentEffects`/kernel type to key a merge by parent identity or graph position

### Requirement: RelayTo is sequential composition of rounds

A `RelayTo` command SHALL end the current evaluation round (its abandoned cone resolved inactive) and start a fresh round at the relay target with the command's sub-program as the new seed, threading state, context, and logs from the relaying node. Single-level relay semantics are preserved.

#### Scenario: Relay redirects without deadlock

- **WHEN** a node emits `RelayTo(target, sub)` mid-evaluation
- **THEN** evaluation continues at `target` with `sub`, and nodes on the abandoned branch are not evaluated
