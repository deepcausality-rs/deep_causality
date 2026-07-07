## 1. Scan & decide

- [x] 1.1 Enumerate all reconvergent (≥2 in-edges) graphs in `deep_causality/tests` and `examples/`; classify each: multi-fired vs single-fired at run time, stateless vs stateful. Record the affected tests. **Done (design.md Blast-radius scan): all existing reconvergent graphs resolve single-fired at runtime; zero multi-fired cases exist → diamond multi-fired test coverage is MISSING and is added in group 5.**
- [x] 1.2 Confirm the D4 policy against the scan: descriptive `CausalityError` for undeclared multi-fired reconvergence (no silent default). **Confirmed — zero migration cost.**
- [x] 1.3 Kernel set is decided (D9: `LinearJoin<R: Scalar>`); confirm whether any stateful multi-parent graph forces a stateful join decision now. **No stateful multi-parent graph exists → stateful join kernel deferred; recorded in design.md.**

## 2. Types: labeled parent surface (additive API)

- [x] 2.1 `ParentEffects<V>` in `deep_causality/src/types/`: wrapper over `BTreeMap<usize, PropagatingEffect<V>>` (fired parents only, keyed by parent node index), with accessors; export from `lib.rs`.
- [x] 2.2 `JoinFn<I, O>` and `ContextualJoinFn<I, O, STATE, CTX>` type aliases following the `CausalFn`/`ContextualCausalFn` fn-pointer pattern; `join_fn` field on `Causaloid` + `new_join*` constructors (plain and with-context) + getters. Existing constructors set `None`; no existing signature changes.
- [x] 2.3a Prerequisite (investigated 2026-07-07): add `impl<T: Real> Default for Dual<T> { fn default() -> Self { Self::zero() } }` in `deep_causality_num/src/dual/dual_number/default.rs`, register in the `dual_number` mod, add a test under `tests/dual/dual_number/`. This is the sole missing bound for `Dual<S>` to serve as the graph value `V` (`Clone`/`Debug` derived; `Send`/`Sync`/`'static` auto; `Dual: Scalar` already holds). Without it the differentiability witness (5.4) cannot compile.
- [x] 2.3 Kernel per D9: `LinearJoin<R: Scalar> { weights: BTreeMap<usize, R>, bias: R }` config type + shipped `linear_join` fn (`bias + Σ weights[p]·v_p`, ascending key order; command → error, `Pure(None)`/missing weight → no contribution, missing config → error); coefficients via `FromPrimitive`; docstring cites Pearl (2009) and Lorenz (2022). After 2.3a, `Dual<f64>` meets the full engine `V` bound (compile-verified).

## 3. Engine: wire-slot evaluation (stateless)

- [x] 3.1 Replace the BFS `visited` array in `evaluate_subgraph_from_cause` with per-node wire slots (`Fired(effect) | Inactive` engine-local enum), reachability pre-pass from `start_index`, and a `BTreeSet` ready set processed in ascending node index.
- [x] 3.2 Firing rule: all wires resolved ∧ ≥1 `Fired` → evaluate; all `Inactive` → resolve node `Inactive` and propagate. Single fired parent = identity pass-through; ≥2 fired = engine channel rules (error left-zero, log concat, ascending key order) then the declared join, or `CausalityError` if undeclared.
- [x] 3.3 `RelayTo` as sequential composition: command ends the round (abandoned cone resolves `Inactive`), fresh round starts at the target with the sub-program; preserve single-level relay semantics and state/log threading.
- [x] 3.4 Golden-check: linear/tree graphs bit-identical vs the previous engine (record expected outputs before rewriting).

## 4. Engine: stateful path

- [ ] 4.1 Mirror the wire-slot evaluation in `graph_reasoning/stateful.rs`; the join mechanism returns a full carrier, so state/context combine is the mechanism's decision (per D5). Apply the 1.3 scan outcome (implement or document deferral).
- [ ] 4.2 Preserve state/context/log threading from the relaying node across rounds.

## 5. Tests updated to declared-join semantics

- [ ] 5.1 Update the `logic_graph` diamond and every multi-fired graph from 1.1: declare a join mechanism, assert the mechanism-derived result. Confirm single-fired reconvergent graphs pass unchanged.
- [ ] 5.2 New tests (the MISSING diamond multi-fired coverage, per scan): **root-start diamond** — `evaluate_subgraph_from_cause(0)` on `build_multi_cause_graph` with a declared join on C(3), asserting the joined result (both A and B fire); asymmetric join distinguishes parents; join-of-one identity; undeclared multi-fired error (root-start diamond, no join → `CausalityError`); parent-error short-circuit (canonical order); log canonical-order concat; mid-graph start with out-of-cone parent (no deadlock, single-fired); relay mid-diamond liveness; all-`Inactive` discard propagation.
- [ ] 5.4 Kernel tests: `LinearJoin` weighted diamond (`b + w1·x1 + w2·x4`); surgery locality (cut wire / `Inactive` parent drops exactly that term); `Dual<S>` sensitivity (`ε` channel = seeded parent's weight); degenerate-parent policy (command → error; `Pure(None)` and missing weight → no contribution; missing config → error).
- [ ] 5.3 Determinism test: same diamond model under two node-index assignments agrees modulo relabeling.

## 6. Lean model + witness

- [ ] 6.1 `Core/GraphJoin.lean` (bare-`lean`, self-contained): `unique_valuation` (well-founded induction over the acyclic labeled system, no algebraic hypotheses on mechanisms); `schedule_invariance` (command-free); disjoint-key union lemmas; `inactive_discard`; `classical_copy` stated as a law of the classical interpreter; relay composition definitional with determinism as a separate scoped result. Register in `DeepCausalityFormal.lean`.
- [ ] 6.2 Rust witnesses under `deep_causality/tests/formalization_lean/`: permuted-layout agreement on the real engine (6.1a/b), inactive-discard, and copy-law witnesses; `THEOREM_MAP.md` rows (state the command-free scope) + `LEAN_CORE.md` rows (advance `Formalization.md` #2/#11).
- [ ] 6.3 Per-kernel lemmas (scoped to the kernel, not the engine): `LinearJoin` key-order invariance via the proved num-layer monoid laws; surgery locality (`join over Pa∖{p}` = `join over Pa` minus `weights[p]·v_p` — the kernel-level shadow of mechanism opening), each with a Rust witness and `THEOREM_MAP.md` row.

## 7. Verify & hand off

- [ ] 7.1 `bazel test //...` green; `make format && make fix` clean; bare-`lean` on `Core/GraphJoin.lean` exit 0.
- [ ] 7.2 Re-scope `formalize-main-crate`: D2 narrowed (copy law classical-only; QCM predicates attach to labeled wires; cyclic admissibility = `ValidProcess`); its intervention capability references the D10 surgery attachment point; drop the "engine gap / gated follow-up" wording.
- [ ] 7.3 Prepare a commit message per completed task group; do not commit (await user).
