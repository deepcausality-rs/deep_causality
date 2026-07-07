## Context

`evaluate_subgraph_from_cause` (and its stateful twin) is a BFS that marks a child visited when the *first* parent enqueues it (`graph_reasoning/mod.rs:199-203`), passing that one parent's effect; a reconvergence node consumes an arbitrary single parent and ignores the rest. The suite already contains such a graph (`logic_graph`: node `idx2` has parents `idx1` and `idx4`).

An earlier draft of this change resolved the fan-in with a user-declared associative-commutative monoid under Kahn wait-for-all-parents scheduling. Three findings retired that draft:

1. **Liveness.** Evaluation starts mid-graph by design, and `RelayTo` abandons the frontier at run time, so "wait for all parents" waits on parents that never fire.
2. **Expressiveness.** Causal mechanisms are functions of *labeled* parent tuples and are generally asymmetric; a commutative pre-merge erases parent identity — the identity that do-surgery on a specific edge and the QCM factorization `σ = ∏ ρ_{A|Pa(A)}` both require. No canonical commutative monoid exists on a generic `V`, so the "declared combine" also forces new trait bounds through every graph API.
3. **The papers** (`ctx/papers/`, read and verified): QCM parent sets are labeled and generally overlapping; commutativity appears only as the checkable `PairwiseCommute` predicate; fan-out duplication is classical-only (no-cloning; the unitary decomposition resolves shared access into direct-sum sectors); cyclic admissibility is `ValidProcess`, not an evaluation order. Classical causal models are the diagonal special case of QCM — the two frameworks share the labeled structure and one-mechanism-per-node, and nothing else.

The design is therefore a **structure/interpretation split**: the substrate reifies labeled wires and per-node mechanisms; the classical evaluator (built here) and the QCM assembler (formalize-main-crate) are two folds over the same substrate. `∇_G` dissolves — fan-in is the labeled product `⊗_{p∈Pa(n)} X_p` consumed by the node's own mechanism; no engine-chosen combine exists.

## Goals / Non-Goals

**Goals:**
- Replace first-parent-wins with labeled fan-in over the frozen acyclic graph: wire-slot resolution (`Fired`/`Inactive`), per-node join mechanisms over parent-indexed effects, canonical scheduling. No deadlock under `RelayTo` or mid-graph starts.
- Keep in-degree ≤ 1 evaluation bit-identical; keep the Causaloid API additive-only.
- Prove in Lean, bound to Rust witnesses: unique valuation, schedule invariance (command-free runs), disjoint-key union lemmas, and copy/discard laws scoped to the classical interpreter.
- Expose the labeled structure (`Pa(n)` keyed by parent index) so do-surgery and the QCM predicates attach to it without a second framework.

**Non-Goals:**
- The QCM assembler, `PairwiseCommute`/`ValidProcess` predicates, and complex-field kernels (formalize-main-crate and the QCM change).
- Cyclic evaluation (cyclic classical = future fixpoint semantics; cyclic quantum = `ValidProcess`; both deferred).
- The do-operator implementation — this change defines its attachment point (wire-key cut, mechanism pin), nothing more.
- Any breaking `Causaloid` signature change.

## Decisions

**D1 — Substrate = labeled structure; dynamics live in interpreters.** The substrate is the frozen graph read as: nodes with in/out interfaces, wires keyed by `(parent, child)`, one mechanism per node bound to its labeled parent set `Pa(n)`. It fixes no copy, no schedule, no admissibility. The classical evaluator and the QCM assembler are folds with different target algebras; the copy law belongs to the classical fold's target (where the diagonal exists), never to the substrate (no-cloning compatibility). *Rejected:* one engine serving both semantics — the papers show the shared content is structural only.

**D2 — Graph-as-program via the free monad, with sharing in the environment.** A tree-shaped `Free` program cannot carry reconvergence: `Free::bind` threads the continuation through every hole, so a shared sub-program would be duplicated per parent — the copy problem at the syntax level (`deep_causality_haft/src/monad/free_monad.rs`, `Fn + Clone` note). Instead, a topological linearization compiles the graph to a *sequential, single-hole* program — "run node `nᵢ`'s mechanism against the valuation restricted to `Pa(nᵢ)`, extend the valuation, continue" — matching the house single-hole pattern (`core.causal_command.functor_laws`). Sharing lives in the keyed valuation (the let-environment), not in subterms. Interpreters are `Free::fold`-shaped algebras over that sequence: classical = valuation-threading handler (built here); quantum = multiply CJ operators into `σ` (deferred), where linearization-irrelevance is supplied by `PairwiseCommute`. Classical schedule invariance and quantum commutation are the same theorem shape — "the linearization does not matter" — one proved structurally, one assumed as the Markov predicate. This reuses the proved `haft.free_monad.*` laws.

**D3 — Wire-slot evaluation (classical fold).** Per node: one slot per in-wire, each resolving to `Fired(effect)` or `Inactive`. A node fires when all slots are resolved and ≥ 1 is `Fired`; all-`Inactive` resolves the node `Inactive` without firing (discard / dead-path propagation). A reachability pre-pass from `start_index` resolves wires from non-descendants `Inactive` at initialization. Ready nodes are processed in ascending node index (canonical schedule; `BTreeSet` ready set). `Inactive` is an engine-local enum, not a public carrier variant. *Rejected:* Kahn wait-for-all (deadlocks); BFS re-visit/merge (order-fragile).

**D4 — Join = the node's own mechanism, additive API.** New field `join_fn: Option<JoinFn<I, O>>` on `Causaloid` plus `new_join*` constructors, matching the existing `Option<CausalFn>` static-dispatch fn-pointer pattern. Configured joins mirror the existing context-aware pattern: a `ContextualJoinFn<I, O, STATE, CTX>` variant receives `(&ParentEffects<I>, Option<&CTX>)`, so kernel parameters (weights, thresholds) ride the causaloid's `CTX` channel exactly as `ContextualCausalFn` config does — fn pointers stay stateless, no closures, no dyn. `JoinFn` consumes `&ParentEffects<I>` — a wrapper over `BTreeMap<usize, PropagatingEffect<I>>` holding the *fired* parents keyed by parent node index (BTreeMap gives canonical iteration order for free). Any function is permitted; asymmetric mechanisms are the norm; determinism comes from the keys, not from properties of the function. Exactly one fired parent = identity pass-through, no declaration needed — conditional graphs where reconvergence exists structurally but only one branch activates (the `RelayTo` pattern) are unaffected. ≥ 2 fired parents without a declared join = descriptive `CausalityError` naming the node and the fired parents. *Rejected:* an engine-default combine — any silent default is a semantic choice the engine has no right to make (the retired draft's trap); a loud error is reversible, a silent default is not.

**D5 — Channel rules.** Engine-owned, applied before the join runs: if any fired parent carries an error, short-circuit with the first error in ascending parent-index order (left-zero; the canonical key order is what makes "first" well-defined); logs concatenate in ascending parent-index order. The join mechanism owns the value channel (and state/context on the stateful path — the mechanism returns a full carrier). Stateless engine: `S = ()`, nothing to combine.

**D6 — `RelayTo` = sequential composition of resolution rounds.** A command result ends the current round (the abandoned cone resolves `Inactive`) and starts a fresh round at the target with the command's sub-program — preserving today's follow-the-relay-exclusively semantics as `eval(start, e) = round(start, e); if command ⇒ eval(target, sub)`. Determinism across rounds holds under the canonical schedule. The schedule-invariance theorem is scoped to command-free runs; relayed runs get determinism, stated separately and honestly.

**D7 — Acyclicity is classical-run admissibility, not a substrate axiom.** The classical fold requires frozen + acyclic (`ultragraph::has_cycle`). The substrate itself is digraph-agnostic. Admissibility per instantiation: acyclic classical = well-foundedness (this change); cyclic classical = fixpoint semantics (future); cyclic quantum = `ValidProcess` (QCM change).

**D8 — Lean: `Core/GraphJoin.lean`, bare-`lean`, self-contained.** Theorems: `unique_valuation` (the acyclic labeled equation system `σ(n) = f_n(σ|Pa(n)^Fired)` has exactly one total resolution; well-founded induction, no algebraic hypotheses on mechanisms); `schedule_invariance` (every admissible schedule computes it; command-free); disjoint-key union lemmas (commutative/associative/idempotent by construction — the structural monoid the retired draft tried to axiomatize); `classical_copy` (every out-wire of `n` carries `σ(n)` — stated as a law of the classical interpreter); `inactive_discard` (all-`Inactive` parents ⇒ node `Inactive`). Relay composition is definitional. Witness: the real engine on two index-permuted isomorphic diamond layouts agrees (two canonical schedules, same result modulo relabeling), plus liveness witnesses.

**D9 — The shipped multi-parent kernel: `LinearJoin<R: Scalar>` (Pearl linear SCM; the QCM shadow).** The engine is algebra-free over `V`; the change ships exactly one kernel, defined as:

- **Config type** `LinearJoin<R: Scalar> { weights: BTreeMap<usize, R>, bias: R }`, carried as the join causaloid's context (`CTX = LinearJoin<R>`), evaluated by the shipped fn-pointer `linear_join<R: Scalar>(parents: &ParentEffects<R>, ctx: Option<&LinearJoin<R>>) -> PropagatingEffect<R>` computing `v_n = bias + Σ_{p ∈ fired} weights[p] · v_p`, iterating in ascending key order (result independent of arrival order by construction; coefficients via `FromPrimitive`, the `quadrature` pattern).
- **Value policy per fired parent:** command on the value channel → `CausalityError` (F-3-consistent); `Pure(None)` → contributes nothing (the honest `Maybe`); fired parent without a weight entry → coefficient zero (contributes nothing). Missing config (`ctx = None`) → `CausalityError`.
- **Surgery locality (the do()-preparation lemma):** removing wire key `(p, n)` — or `p` resolving `Inactive` — changes the result by exactly `− weights[p] · v_p`; the kernel needs no redefinition after surgery. This is the kernel-level classical shadow of "opening a mechanism," proved as a per-kernel Lean lemma via the num-layer ring laws, with a Rust witness.
- **Precision / lift path (the QCM preparation):** one kernel shape at every tier — `R = f32/f64/Float106` (classical); `R = Dual<S>` (`∂output/∂parent_p = weights[p]` in the `ε` channel — the intervention-sensitivity witness); the complex tier (`field_complex.rs`) for the QCM change; operator-valued entries (`CJOp`) deferred. Multilinearity in the labeled parents is exactly the property that makes `LinearJoin` the diagonal classical shadow of a QCM channel factor `ρ_{A|Pa(A)}` — the kernel is the object the QCM change lifts, not a throwaway.
- **`Dual<R>`-as-graph-value gap (investigated, resolved as a prerequisite).** For the differentiable path the effect value type is `V = Dual<S>`, so `Dual<S>` must meet the engine bound `V: Default + Clone + Send + Sync + 'static + Debug` (`graph_reasoning/mod.rs:23`). Verified empirically (compile-check against `deep_causality_num`): `Dual<f64>` satisfies **all but `Default`** — `Clone`/`Debug` are derived, `Send`/`Sync`/`'static` are auto-traits on the two-field POD struct (the `MaybeParallel` blanket documents that all workspace scalars are `Send + Sync`), and `Dual<f64>: Scalar` holds (`Real` + `Div` + `FromPrimitive` all impl'd). The lone gap is that `Dual<T>` has no `Default` impl, and `Scalar` does not imply `Default`. The `Default` bound is structural on `Causaloid`/`CausalEffectPropagationProcess`, not vestigial in the engine, so relaxing it is out of scope; the fix is a one-liner in `deep_causality_num`: `impl<T: Real> Default for Dual<T> { fn default() -> Self { Self::zero() } }` (a new `dual/dual_number/default.rs` per the trait-per-file convention; `default = 0 + 0ε` is the additive identity / constant-zero-with-zero-derivative, matching the existing `Zero` impl and needing only `T: Real`). This lands as a prerequisite step (task 2.3a) before the differentiability witness; it is additive and has no blast radius.
- **Docstring citations** (papers-in-docstring convention): Pearl, *Causality* (2009), linear SCMs; Lorenz (2022) for the diagonal-shadow remark.

`RealField` is reserved for future kernels needing total division. Kernel order-properties (e.g. `LinearJoin` symmetry under equal weights) are per-kernel lemmas via the proved num-layer laws, never engine axioms.

**D10 — do()-readiness (stated, not built).** Surgery on the substrate: cut = delete wire key `(p, c)`; pin `do(X = x)` = replace `X`'s mechanism with a constant (equivalently pre-resolve its valuation slot) and resolve its in-wires `Inactive`. Both folds respect the same surgery (truncated factorization classically; the Lorenz–Tull opening quantumly). The commuting-with-encapsulation theorem lives in formalize-main-crate's intervention capability.

## Risks / Trade-offs

- **[Breaking for multi-fired reconvergence]** Undeclared joins now error loudly. → Task 1 enumerates affected graphs; tests declare joins with assertions derived from the declared mechanism (AGENTS.md: fix the API, then assert the correct value). Single-fired reconvergence keeps working, which the scan must confirm covers the `RelayTo`-conditional graphs.
- **[Engine rewrite drift on linear graphs]** → Golden tests: linear/tree graphs bit-identical before/after; only multi-fired diamonds may change.
- **[Stateful join underspecified]** → The join mechanism returns a full carrier, so state combine is the mechanism's decision, not a global rule; the scan decides whether any existing stateful multi-parent graph forces a kernel now or documentation suffices.
- **[Scope creep toward QCM]** → Only the labeled structure surface lands here; predicates, assembler, and complex kernels are hard Non-Goals with named homes.
- **[Two-layer theorem misread as full invariance]** → `THEOREM_MAP.md` rows state the command-free scope of `schedule_invariance` explicitly; the relay-determinism statement is a separate row.

## Migration Plan

Land before `formalize-main-crate`. Steps: (1) blast-radius scan; (2) `ParentEffects` + Causaloid join surface; (3) wire-slot engine in `graph_reasoning/{mod,stateful}.rs`; (4) kernels (per scan decision); (5) tests updated + liveness/golden/determinism suites; (6) `Core/GraphJoin.lean` + witnesses + `THEOREM_MAP`/`LEAN_CORE` rows; (7) `bazel test //...` green. Rollback = revert engine + additive API (no signature changed, so downstream compiles either way); linear graphs unaffected throughout.

## Blast-radius scan (task 1, completed 2026-07-07)

Reconvergent (≥2 in-edge) graphs fed to the fan-in engine (`evaluate_subgraph_from_cause`):

- `build_multi_cause_graph` (`utils_test/test_utils_graph.rs`): root(0)→{A(1),B(2)}→C(3); C has parents {1,2}. Exercised only via `evaluate_subgraph_from_cause(2)` (start B) — A is not reachable from B, so C is **single-fired** at runtime.
- `build_multi_layer_cause_graph`: E(5)←{A(1),B(2)}, F(6)←{B(2),C(3)}. Exercised via starts 3 and 2 — each reconvergence node has exactly one reachable parent → **single-fired**.
- `logic_graph` (`causality_graph_reasoning_tests.rs`): idx2←{idx1,idx4}. Exercised **only** via `evaluate_shortest_path_between_causes` (single path), which never enters the fan-in engine.
- Examples `rcm`/`dbn`: single linear edges. Stateful tests: all linear 3-node paths or two unconnected nodes.

**Result:** no existing test or example triggers *multi-fired* reconvergence at runtime; every reconvergent structure resolves to a single reachable parent (identity) or uses shortest-path. Breaking blast radius on the existing suite = **zero**; task 3.4 golden-checks this empirically. True multi-fired behaviour is introduced only by the new tests (5.2/5.4), e.g. starting `evaluate_subgraph_from_cause(0)` on `build_multi_cause_graph` with a declared join on C.

## Open Questions

- Error-vs-default for undeclared multi-fired reconvergence: **decided error (D4)** — confirmed against the scan: nothing currently produces undeclared multi-fired at runtime, so the loud-error policy has zero migration cost.
- Kernel set: **decided (D9)** — `LinearJoin<R: Scalar>` ships with the change as the defined multi-parent kernel and QCM lift target.
- Stateful multi-parent graph: **none exists** (scan). The stateful path (task 4) mirrors wire-slot scheduling and single-fired identity; a stateful ≥2-fired join returns the same descriptive error as the stateless path, and a stateful join *kernel* is deferred until a real stateful diamond exists.
