## ADDED Requirements

### Requirement: Reasoning engine formalized as a Free::fold catamorphism over single-input arrows

The formalization SHALL model the graph-reasoning engine as a catamorphism (`Free::fold`) over the canonical topological linearization of the frozen graph, composing **single-input** causaloid arrows (`I → CausalEffect<O>`, `Core/CausalArrow.lean`) in sequence. The adaptive-jump semantics is the recursive equation `fold(Suspend(RelayTo(t, k))) = jump(t, fold(k))`. `Core/GraphReasoning.lean` SHALL reduce engine correctness to (i) the free-monad fold laws already in `Haft/FreeMonad.lean` and (ii) the local correctness of the single `jump` algebra step (thread state, context, logs). Each theorem MUST carry a `THEOREM_MAP.md` row and a Rust witness, and the file MUST typecheck standalone with bare `lean`.

#### Scenario: Adaptive relay is a fold step

- **WHEN** `Core/GraphReasoning.lean` is typechecked
- **THEN** the `RelayTo` denotation is the `jump` algebra of the fold, and the bound Rust witness confirms the engine's single-level relay matches it

#### Scenario: Nested relay folds structurally, not across nodes

- **WHEN** a nested `RelayTo` program is folded
- **THEN** the catamorphism resolves it structurally (`CausalEffect::fold`), consistent with the engine inlining a single-level jump and feeding the sub-program as the next node's input

#### Scenario: Linear/tree sequencing is deterministic

- **WHEN** a single-input (in-degree ≤ 1) graph is folded
- **THEN** the wire-slot topological schedule from the prerequisite `comonoid-graph-join` change computes it deterministically, and the bound Rust witness confirms it on the real engine

### Requirement: Reconvergence merge (∇) is out of scope — deferred to the symmetric-monoidal extension

This change SHALL model only the single-input composition of causaloids. The **reconvergence merge `∇`** — combining two or more converging effects at a multi-parent node — is a symmetric-monoidal generator (copy/discard comonoid Δ + merge ∇) over the effect monad, an extension of the single-input causaloid, and is **not formalized here**. Its semantics is a load-bearing OPEN question (`algebraic-causaloid-assumptions.md` #2) deferred to a dedicated symmetric-monoidal (PROP / free-Markov) extension change, where per-connection asymmetry (weighted influence, Hardy's Λ) will live on the edges. The engine this change formalizes fails loudly at a multi-parent reconvergence (per `comonoid-graph-join`), so there is no merge behaviour to model.

#### Scenario: Multi-parent merge is not claimed

- **WHEN** the graph-reasoning formalization is reviewed
- **THEN** it states no reconvergence-merge theorem (no `∇`, no labeled join, no fan-in commutative-monoid), and references the deferred symmetric-monoidal extension as the home for `∇`

#### Scenario: The engine's loud reconvergence failure is consistent with the model

- **WHEN** a diamond fires two parents into a node at run time
- **THEN** the real engine returns a `CausalityError` (merge undefined), matching the model's scope of single-input composition only
