## ADDED Requirements

### Requirement: Reasoning engine formalized as a Free::fold catamorphism with keyed-valuation sharing

The formalization SHALL model the graph-reasoning engine as a catamorphism (`Free::fold`) over the canonical topological linearization of the frozen graph — a sequential, single-hole program of the form "run node `nᵢ`'s mechanism against the valuation restricted to `Pa(nᵢ)`, extend the valuation, continue" — with reconvergent sharing carried by the keyed valuation (the let-environment), never by duplicated subterms (a tree-shaped `Free` cannot carry reconvergence: `bind` threads the continuation through every hole). The adaptive-jump semantics is the recursive equation `fold(Suspend(RelayTo(t, k))) = jump(t, fold(k))`. `Core/GraphReasoning.lean` SHALL reduce engine correctness to (i) the free-monad fold laws already in `Haft/FreeMonad.lean`, (ii) the `unique_valuation`/`schedule_invariance` theorems from the prerequisite `comonoid-graph-join` change, and (iii) the local correctness of the single `jump` algebra step (thread state, context, logs). Each theorem MUST carry a `THEOREM_MAP.md` row and a Rust witness, and the file MUST typecheck standalone with bare `lean`.

#### Scenario: Adaptive relay is a fold step

- **WHEN** `Core/GraphReasoning.lean` is typechecked
- **THEN** the `RelayTo` denotation is the `jump` algebra of the fold, and the bound Rust witness confirms the engine's single-level relay matches it

#### Scenario: Nested relay folds structurally, not across nodes

- **WHEN** a nested `RelayTo` program is folded
- **THEN** the catamorphism resolves it structurally (`CausalEffect::fold`), consistent with the engine inlining a single-level jump and feeding the sub-program as the next node's input

#### Scenario: Sharing lives in the environment

- **WHEN** a diamond (branch then reconverge) subgraph is modeled
- **THEN** the shared ancestor appears once in the linearized program and its output is read from the keyed valuation by each dependent, with no duplicated sub-program

### Requirement: Fan-in composed from the prerequisite; copy/discard scoped to the classical interpreter

The formalization SHALL compose the fold with the labeled fan-in delivered by the prerequisite `comonoid-graph-join` change — wire-slot resolution (`Fired`/`Inactive`), per-node join mechanisms over parent-indexed effects, and the proved `unique_valuation` + `schedule_invariance` (command-free) theorems — rather than re-deriving a join. The copy law (every out-wire of `n` carries `σ(n)`) and the discard law (all-`Inactive` parents resolve the node `Inactive`) SHALL be stated as laws of the **classical interpreter's algebra**, not of the substrate, so that the quantum instantiation — where fan-out is commuting access to a shared parent output (`PairwiseCommute`), per the verified `ctx/papers/` sources — folds the same structure without violating any substrate law. No commutativity, associativity, or symmetry obligation SHALL be imposed on join mechanisms by the model.

#### Scenario: Engine fan-in is supplied by the prerequisite

- **WHEN** the fold reaches a node with multiple fired parents
- **THEN** it consumes the `comonoid-graph-join` labeled join (parent-indexed effects into the node's declared mechanism), and the model cites that change's theorems for determinism, so the formalization describes real engine behaviour

#### Scenario: Copy law is classical-scoped

- **WHEN** the fan-out law is stated in `Core/GraphReasoning.lean`
- **THEN** it is a theorem about the classical evaluation algebra, and the substrate carries no duplication law — the linearization-invariance role is played by `schedule_invariance` classically and by the `PairwiseCommute` predicate quantumly (same theorem shape, stated in the file as a remark linking the two capabilities)

#### Scenario: Asymmetric mechanisms are admitted by the model

- **WHEN** a join mechanism is an arbitrary non-commutative function of its labeled parents
- **THEN** the fold's determinism theorems apply unchanged, because they carry no algebraic hypotheses on mechanisms
