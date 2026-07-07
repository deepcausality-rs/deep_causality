## 1. Remove the leaky merge surface

- [x] 1.1 Remove `join_fn`/`context_join_fn` fields from `Causaloid`, the `new_join`/`new_with_context_join` constructors, and the getters.
- [x] 1.2 Remove the `JoinFn`/`ContextualJoinFn` aliases.
- [x] 1.3 Remove the `ParentEffects` type/module and the `LinearJoin`/`join_kernels` module; drop the exports.
- [x] 1.4 Drop the `deep_causality_num` dependency (Cargo + Bazel) added only for the kernel. (The `Dual: Default` impl stays in `deep_causality_num` — it is a correct addition there.)

## 2. Engine: principled sequencing + loud reconvergence error

- [x] 2.1 `evaluate_subgraph_from_cause`: wire-slot resolution (fired/inactive), reachability pre-pass from the start, ascending-node-index canonical schedule, acyclicity gate (`has_cycle`).
- [x] 2.2 Single fired parent = identity pass-through. Two or more fired parents = descriptive `CausalityError` naming the node + fired parents + the pending `∇` merge extension.
- [x] 2.3 `RelayTo` as sequential round composition (single-level relay; state/log threaded).
- [x] 2.4 Mirror in `graph_reasoning/stateful.rs` (same loud error message).

## 3. Lean: remove the deferred-join formalization

- [x] 3.1 Remove `Core/GraphJoin.lean`, its import in `DeepCausalityFormal.lean`, the `core.graph_join.*` `THEOREM_MAP.md` rows, and the Rust witness (`tests/formalization_lean/graph_join_tests.rs`) + its registration + Bazel suite. (The merge formalization moves to the deferred extension change.)

## 4. Tests

- [x] 4.1 Golden: linear/tree graphs bit-identical vs the previous engine (existing suites pass unchanged).
- [x] 4.2 New: root-start diamond errors loudly (message names the node + `∇`); single-fired reconvergence is the identity; stateful diamond errors loudly.
- [x] 4.3 Remove the join/kernel/parent-effects tests and their Bazel suites.

## 5. Specs & notes

- [x] 5.1 Rescope this change's proposal/design/spec to Path A (principled sequencing + loud error; `∇` deferred).
- [x] 5.2 Re-scope `formalize-main-crate` graph-reasoning spec: drop the reconvergence-join theorems; record `∇` as deferred to the symmetric-monoidal extension.
- [x] 5.3 Record the ruling in `algebraic-causaloid-assumptions.md` #2 (sequencing fixed + loud error; merge semantics still OPEN).

## 6. Verify

- [ ] 6.1 `bazel test //...` green; `make format && make fix` clean; bare-`lean` on the remaining core files exit 0.
- [ ] 6.2 Prepare a commit message; do not commit (await user).
