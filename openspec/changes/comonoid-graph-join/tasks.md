## 1. Scan & decide

- [ ] 1.1 Enumerate all reconvergent (multi-parent) graphs in `deep_causality/tests` and `examples/` (nodes with ≥2 in-edges); record the affected tests and whether any are stateful.
- [ ] 1.2 Decide where the join-combine is declared (graph-level default policy vs per-call vs `AggregateLogic` reuse) and the default value-combine monoid; record in design.md Open Questions and confirm before implementing.
- [ ] 1.3 Confirm whether a declared state/context combine is needed now (any stateful multi-parent graph) or can be deferred; scope accordingly.

## 2. Engine: topological evaluation + fan-in join (stateless)

- [ ] 2.1 Add a join-combine surface (per D4 decision) for the value channel (associative-commutative monoid), with canonical-order log concat and error short-circuit.
- [ ] 2.2 Rewrite `evaluate_subgraph_from_cause` to topological/Kahn scheduling: buffer parent effects; fire a node when all parents are processed; feed the joined effect. Require frozen + acyclic (reject via `has_cycle`).
- [ ] 2.3 Preserve `RelayTo` adaptive-jump semantics (single-level relay) within the topological order; a relay still resets/redirects as today.
- [ ] 2.4 Golden-check: linear/tree graphs produce bit-identical output vs the previous engine.

## 3. Engine: stateful path

- [ ] 3.1 Mirror the join in `graph_reasoning/stateful.rs` (or document deferral if 1.3 shows no stateful multi-parent usage), including the state/context combine rule.
- [ ] 3.2 Preserve state/context/log threading from the relaying node on `RelayTo`.

## 4. Tests updated to joined semantics

- [ ] 4.1 Update the `logic_graph` diamond (node `idx2`, parents `idx1`/`idx4`) and any other reconvergent test from 1.1 to assert the join-derived result (derive the expected value from the declared combine, not from current output).
- [ ] 4.2 Add tests: a diamond evaluated under two valid topological orders yields the same result; parent-error short-circuits the join; join-of-one is identity.

## 5. Lean model + witness

- [ ] 5.1 `Core/GraphJoin.lean`: model the fan-in join as a commutative monoid; prove topological-order invariance (reduce to commutative-monoid fold order-invariance + comonoid copy law); bare-`lean` typecheck.
- [ ] 5.2 State the comonoid copy law for fan-out and the join-monoid laws (assoc/comm/unit).
- [ ] 5.3 Rust witness under `deep_causality/tests/formalization_lean/`: run the real engine on a diamond under two topological orders and assert equality; add `THEOREM_MAP.md` + `LEAN_CORE.md` rows (advance `Formalization.md` #2/#11).

## 6. Verify & hand off

- [ ] 6.1 `bazel test //...` green; `make format && make fix` clean; clippy clean; bare-`lean` on `Core/GraphJoin.lean` exit 0.
- [ ] 6.2 Re-scope `formalize-main-crate`: its `graph-reasoning-formalization` spec drops the "engine gap / gated follow-up" (tasks 3.2/3.5 become "describe the now-real comonoid join"); add an explicit dependency note that `comonoid-graph-join` lands first.
- [ ] 6.3 Prepare a commit message per completed task group; do not commit (await user).
