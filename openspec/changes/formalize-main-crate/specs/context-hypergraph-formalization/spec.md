## ADDED Requirements

### Requirement: Context hypergraph formalized with parent-set semantics

The formalization SHALL model the contextoid hypergraph with explicit parent-set (hyperedge) semantics, so that a hyperedge `{Pa(A_i)} → A_i` names a node's parent set directly. `Core/ContextGraph.lean` SHALL define the parent-set map `Pa` and the hyperedge-threading = `bind` correspondence, providing the shared structural substrate that both intervention surgery and QCM factorization operate over. Each theorem MUST carry a `THEOREM_MAP.md` row and a Rust witness, and the file MUST typecheck standalone with bare `lean`.

#### Scenario: Hyperedge threading is bind

- **WHEN** `Core/ContextGraph.lean` is typechecked
- **THEN** the parent-set threading of a node's evaluation is the causal monad `bind`, so encapsulation-equals-flat is inherited from `core.causal_monad.assoc`

### Requirement: Acyclicity is a freeze-enforceable, relaxable parameter

The formalization SHALL treat acyclicity as a separable constraint on the graph, not a hard-wired assumption, so that the same parent-set / threading apparatus supports both the acyclic (DAG) case and the directed-cyclic case. The model SHALL record that acyclicity is enforceable in the implementation via the existing `ultragraph::has_cycle` freeze gate, and that relaxing the gate admits cyclic models (quantum switch / indefinite causal order).

#### Scenario: Acyclic case is freeze-enforced

- **WHEN** a graph is frozen with the acyclicity constraint active
- **THEN** the model corresponds to `ultragraph::has_cycle` rejecting a cyclic graph at freeze

#### Scenario: Cyclic case reuses the same apparatus

- **WHEN** the acyclicity constraint is relaxed
- **THEN** the parent-set factorization and threading definitions are unchanged (no separate machinery), enabling the deferred cyclic-QCM support
