## ADDED Requirements

### Requirement: Reasoning engine formalized as a Free::fold catamorphism

The formalization SHALL model the graph-reasoning engine as a catamorphism (`Free::fold`) over the reified `RelayTo` program, with the adaptive-jump semantics given by the recursive equation `fold(Suspend(RelayTo(t, k))) = jump(t, fold(k))`. `Core/GraphReasoning.lean` SHALL reduce engine correctness to (i) the free-monad fold laws already in `Haft/FreeMonad.lean` and (ii) the local correctness of the single `jump` algebra step (thread state, context, logs). Each theorem MUST carry a `THEOREM_MAP.md` row and a Rust witness, and the file MUST typecheck standalone with bare `lean`.

#### Scenario: Adaptive relay is a fold step

- **WHEN** `Core/GraphReasoning.lean` is typechecked
- **THEN** the `RelayTo` denotation is the `jump` algebra of the fold, and the bound Rust witness confirms the engine's single-level relay matches it

#### Scenario: Nested relay folds structurally, not across nodes

- **WHEN** a nested `RelayTo` program is folded
- **THEN** the catamorphism resolves it structurally (`CausalEffect::fold`), consistent with the engine inlining a single-level jump and feeding the sub-program as the next node's input

### Requirement: Reconvergence join is a copy/discard comonoid

The formalization SHALL model multi-node reconvergence `∇_G` as the copy/discard comonoid of a free Markov category / cPROP (Fritz 2020; JKZ 2019/2021), so the join is constrained by the comonoid laws rather than defined ad hoc. `Core/GraphReasoning.lean` SHALL state the comonoid laws the join must satisfy and prove the fold respects them on the branch/reconverge shape.

#### Scenario: Join obeys the comonoid laws

- **WHEN** a diamond (branch then reconverge) subgraph is folded
- **THEN** the join is the comonoid combine and the associativity/commutativity/counit laws it must satisfy are stated and proved in the model

#### Scenario: Engine join is supplied by the prerequisite change

- **WHEN** the fold's reconvergence join is needed
- **THEN** it composes the comonoid join delivered by the prerequisite `comonoid-graph-join` change (already implemented and order-invariance-proved), so the formalization describes real engine behaviour rather than an intended-but-unimplemented semantics
