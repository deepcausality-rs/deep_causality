## Why

Two demands meet at the graph-reasoning engine.

First, a latent correctness bug: the BFS engine visits each node once, so the first parent to reach a node (in BFS order) enqueues it and marks it visited (`graph_reasoning/mod.rs:199-203`). A node with multiple parents — a reconvergence / diamond — silently consumes whichever single parent arrived first and drops the rest. The suite already exercises this (`causality_graph_reasoning_tests.rs` `logic_graph`: node `idx2` has parents `idx1` and `idx4`).

Second, the formalization roadmap requires one substrate to carry both Pearl's causal calculus (do-surgery over specific edges, pinned interventions, deterministic evaluation) and quantum causal models. Verification against the three `ctx/papers/` sources (Lorenz 2022 Defs 3.3/3.4/4.1; Barrett–Lorenz–Oreshkov Defs 1–6, Thms 1–2, Hypothesis 1; Barrett–Lorenz Eq. 14) fixes the design space:

- **Parent sets are labeled and generally overlapping.** Every QCM mechanism is one channel `ρ_{A|Pa(A)}` over its labeled parent product (Lorenz Fig. 3 treats the diamond as `ρ_{A₄|A₂A₃}` — a single mechanism over both labeled parents). An unlabeled merge of parent values cannot express this structure, and do-surgery on one specific edge has nothing to attach to after a symmetric collapse.
- **Commutativity appears only as a checkable predicate** (`PairwiseCommute`) on a model's mechanisms — never as a property of a value-combining operation.
- **Fan-out duplication exists only classically.** The quantum counterpart is commuting access to a shared parent output space, resolving into direct-sum sectors (no-cloning).
- **Cyclic admissibility is the `ValidProcess` condition**, not an evaluation order.

This change therefore delivers a **structure/interpretation split**: the substrate reifies labeled wires (keyed by parent index) and one mechanism per node; the classical evaluator (fixed here) and the QCM assembler (formalize-main-crate, deferred) are two folds over that one substrate. `do()` is one surgery on the substrate — cut a wire key, pin a mechanism — that both folds respect. The reconvergence join `∇_G` dissolves: fan-in is the labeled product `⊗_{p∈Pa(n)} X_p` already implicit in `Pa(n)`, consumed by the node's own mechanism; there is no engine-chosen combine operation.

## What Changes

- **BREAKING (only for reconvergent nodes with ≥2 simultaneously fired parents):** first-parent-wins is replaced by labeled fan-in. The engine resolves every in-wire of a node to `Fired(effect)` or `Inactive`; a node fires when all wires are resolved and at least one is `Fired`. With exactly one fired parent the node receives that effect unchanged (join of one is identity — conditional/`RelayTo`-style graphs where only one branch activates are unaffected). With two or more fired parents the node's **declared join mechanism** consumes the parent effects keyed by parent index; any function is permitted (asymmetric mechanisms are the norm). A multi-fired node without a declared join yields a descriptive `CausalityError`.
- **Engine:** wire-slot resolution replaces the `visited` array. Reachability pre-pass marks wires from non-descendants of `start_index` as `Inactive`; a `RelayTo` ends the current resolution round and starts a fresh round at the target (sequential composition, preserving today's follow-the-relay-exclusively semantics) with the abandoned cone resolved `Inactive`. Ready nodes are processed in ascending node index (canonical schedule). No deadlock, no waiting on parents that cannot fire. Frozen + acyclic remains required for the classical run (`ultragraph::has_cycle`).
- **Causaloid API: additive only.** A `join_fn: Option<JoinFn<I, O>>` field plus `new_join*` constructors, matching the existing `Option<CausalFn>` static-dispatch pattern. Existing constructors and all in-degree ≤ 1 behavior are unchanged (golden-tested bit-identical).
- **Channel rules (engine-owned):** error = short-circuit left-zero in ascending parent-index order; log = ascending parent-index concatenation. Value (and state, on the stateful path) belong to the join mechanism.
- **Substrate neutrality:** fan-out is out-wire structure; the copy law is stated as a law of the *classical interpreter only* (no-cloning compatibility). `Pa(n)` is exposed labeled, giving do-surgery and QCM factorization one attachment point. Acyclicity is classical-run admissibility, not a substrate axiom.
- **Precision tiers:** the engine stays algebra-free over `V`; provided join kernels are generic over `Scalar` (dual-number compatible, so `Dual<R>` runs yield intervention sensitivity); complex-field kernels are deferred to the QCM change.
- **Lean model** (`Core/GraphJoin.lean`): unique-valuation (well-founded induction, no algebraic hypotheses on mechanisms), schedule invariance for command-free runs, disjoint-key union lemmas, and the classical-scoped copy/discard laws — bound to Rust witnesses (`tests/formalization_lean/`) and `THEOREM_MAP.md` rows; bare-`lean` typecheck.

## Capabilities

### New Capabilities
- `comonoid-graph-join`: labeled fan-in for the reasoning engine over a substrate that serves both Pearl do-surgery and QCM factorization — wire-slot resolution (`Fired`/`Inactive`), per-node join mechanisms over parent-indexed effects, Lean-proved unique valuation and schedule invariance, with the copy law scoped to the classical interpreter.

### Modified Capabilities
<!-- No existing capability spec covers graph-reasoning behaviour (core-formalization is the monad layer). The behaviour change is captured under the new capability; the follow-on `formalize-main-crate` change is re-scoped in ITS artifacts to depend on this one. -->

## Impact

- **Rust engine:** `deep_causality/src/traits/causable_graph/graph_reasoning/{mod.rs,stateful.rs}` — wire-slot evaluation, reachability pre-pass, canonical scheduling, relay rounds.
- **Causaloid:** additive `join_fn` field + constructors in `deep_causality/src/types/causal_types/causaloid/`; new `ParentEffects<V>` type (main crate).
- **New Lean:** `Core/GraphJoin.lean`; registered in `DeepCausalityFormal.lean`; `THEOREM_MAP.md` + `LEAN_CORE.md` rows (advances `Formalization.md` #2/#11).
- **Tests (blast radius = reconvergent graphs with ≥2 fired parents):** diamond tests declare joins and assert labeled-join results; linear/tree graphs golden-tested bit-identical; new liveness tests (relay mid-diamond, mid-graph start) and a permuted-layout determinism witness.
- **Downstream:** `formalize-main-crate` is re-scoped: its D2 comonoid framing is narrowed (copy law classical-only; QCM predicates attach to the labeled wires; cyclic admissibility = `ValidProcess`); its intervention capability gains the surgery attachment point (wire-key cut + mechanism pin) defined here.
- **No new dependencies;** `unsafe_code = "forbid"` and the zero-macro/zero-dep ethos untouched. The QCM assembler, cyclic evaluation, and complex-field kernels are out of scope.
