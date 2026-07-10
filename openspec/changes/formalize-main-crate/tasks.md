<!--
Reconciled 2026-07-10 against the landed changes `haft-categorical-machinery` (2026-07-08) and
`causaloid-formalization-stages-2-5` (2026-07-10, roadmap Stages 1–5). Tasks delivered by those
changes are checked with a "(delivered by …)" note — the content landed under different theorem
names/files than this change originally planned. The intervention/do-operator and QCM groups were
REMOVED from this change's scope (2026-07-10, user ruling): `deep_causality_do_calculus` and
`deep_causality_quantum` will be spec'd separately (requirement material in
`causaloid-formalization-roadmap.md` §6–§7).
What this change still owns is the RESIDUE: doc reconciliation, the sorry-guard, the collection
permutation-invariance theorem, the F-3 command-input theorem, relay-round composition, and the
context hypergraph.
-->

## 1. Groundwork & reconciliation

- [x] 1.1 Verify the F-1 caveat is closed: confirm `error ⇒ value=None` holds by construction on the `Result<CausalEffect<V>, E>` carrier; record the finding. *(delivered: tracker #7 DECIDED 2026-07-07 — `core.causal_monad.right_id` unconditional, machine-checked; recorded in `algebraic-causaloid-assumptions.md` §7)*
- [x] 1.2 Reconcile `openspec/notes/causal-algebra/Causaloid-Formalization.md` from `EffectValue`/`ContextualLink`/`Map` to the current `CausalEffect` model; flag each changed claim (F-3 = command-input-errors). *(reconciliation banner + per-claim table keyed to F-0…F-5 + updated Status block; F-1 CLOSED, F-3 = stated theorem)*
- [x] 1.3 Reconcile `openspec/notes/causal-algebra/CausalMonadProptest.md` to the `CausalEffect` model and note the timestamp-equality fix already landed. *(reconciliation banner: right identity now unconditional, hand-built-invalid test dropped, §7 invariant CLOSED; §3 timestamp fix unchanged)*
- [x] 1.4 Add a `sorry`/obligation CI guard: fail if a `sorry` appears outside the whitelisted `Quantum/*` obligation slots. *(added `sorry-guard` job to `.github/workflows/formalization.yml`: word-boundaried token scoped to `lean/DeepCausalityFormal/` minus `Quantum/`; passes now (zero-`sorry`), trips on a real bare `sorry`, ignores `sorryAx`/`isSyntheticSorry`)*

## 2. Causaloid layer

- [x] 2.1 Model singleton = context-parameterized Kleisli arrow `I → CausalEffect<O>`; prove identity/compose/error-left-zero by reduction to `Core/CausalArrow.lean`. *(delivered: `core.causal_arrow.{category_laws, left_zero}` + the atom case of `core.causaloid.catamorphism_unique` and `core.causaloid.arrow_fragment` — `Core/{CausalArrow, Causaloid, Catamorphism}.lean`)*
- [x] 2.2 Prove `evaluate` is the value-fragment extension of the arrow; unconditional right identity (no F-1 side-condition). *(delivered: the atom equation of `catamorphism_unique` + #7's unconditional laws)* — **remaining sub-item moved to 2.5 (F-3).**
- [ ] 2.3 Extend `Core/VerdictClosure.lean` with the **permutation-invariance** theorem for the aggregation modes over the Verdict carrier: `All`/`Any` as commutative-monoid folds (the `fuse_perm` device from `Core/GraphAlgebra.lean` applies), `None` via the `Any` result, `Some(k)` via permutation-invariance of the firing count. *(what landed in Stage 3 is the CLOSURE theorem `core.verdict.closure`; the #1 scoped order-invariance theorem is decided but not yet Lean-proved on the collection path)*
- [ ] 2.5 State and prove the F-3 command-input theorem: `evaluate` applied to a command (`RelayTo`) on the input channel yields a specific error, never a silent `None` (the Rust behaviour in `Causaloid::evaluate` / `evaluate_stateful` is implemented and unit-tested; the Lean model and THEOREM_MAP row are missing).
- [x] 2.4 Rust witnesses + `THEOREM_MAP.md` rows for the delivered causaloid layer. *(delivered: `deep_causality/tests/formalization_lean/{causaloid, verdict_closure, catamorphism}_tests.rs` + rows; the main-crate witness mirror and the CI gate scope extension landed with Stage 2)*

## 3. Graph-reasoning engine

- [x] 3.1 Model the engine as a fold with a `jump` algebra reduced to the free-monad laws. *(delivered by equivalent: `core.causal_effect.fold_universal` — `CausalEffect::fold` is the UNIQUE handler; `core.causal_effect.relay_termination` — the fuel-bounded relay handler is total; `core.causaloid.graph_fold_order_invariant` — the engine as a schedule-invariant dataflow fold — `Core/{CausalEffect, GraphAlgebra}.lean`)*
- [x] 3.2 Fan-in / schedule theorems. *(delivered: `schedule_invariance` = `schedule_invariant`, `unique_valuation` = `exec_computes_val` in `Core/GraphAlgebra.lean`; SUPERSEDED SCOPE: the original "reconvergence fails loudly, ∇ deferred" framing is obsolete — the defined merge `∇ ∘ (Λ₁ ⊗ Λ₂)` with `∇ = Verdict::join` landed in Stage 4, corpus-gated)*
- [x] 3.3 Local `jump` correctness (state/context/log threading; nested relay folds structurally). *(delivered: relay threading with error hoisting in `core.causal_effect.transformer_stack`; structural folding in `fold_universal`; engine-level relay-cycle + threading regression tests on both classical and stateful paths)*
- [ ] 3.6 State and prove **relay-round composition** at the graph level: multi-round adaptive evaluation is the sequential (Kleisli) composition of rounds — round `n`'s relayed sub-program seeds round `n+1` — so the fuel bound composes and the whole adaptive run is one composite arrow. *(the per-round fold and the fuel bound are proved; the cross-round composition equation is pinned only by engine tests)*
- [x] 3.4 Rust witnesses + `THEOREM_MAP.md` rows for the delivered engine layer. *(delivered: `causal_effect_tests.rs`, `graph_algebra_tests.rs`, the #10 characterization corpus)*
- [x] 3.5 Confirm the engine's fan-in substrate landed before finalizing. *(confirmed: wire-slot sequencing 2026-07-07, defined merge Stage 4 2026-07-10)*

## 4. Context hypergraph (`Core/ContextGraph.lean`) — **the main remaining Lean surface**

- [ ] 4.1 Model the contextoid hypergraph with parent-set map `Pa` keyed by parent identity (the identity-keyed wire surface the Stage-4 engine exposes — `fired[child][parent]`, `LambdaEdges` `(source, target)` keys) and the hyperedge-threading = `bind` correspondence; encapsulation-equals-flat via `core.causal_monad.assoc` (the graph-side counterpart of `core.causaloid.encapsulation_flat`).
- [ ] 4.2 Model acyclicity as a separable constraint; map the acyclic case to `ultragraph::has_cycle`/`freeze_dag`/`freeze_verified`; show the cyclic case reuses the same definitions (per-interpretation admissibility, unchanged apparatus).
- [ ] 4.3 Rust witnesses (parent-set threading; freeze acyclicity gate); `THEOREM_MAP.md` rows.

## 5. Removed from scope (2026-07-10, user ruling)

<!-- The intervention/do-operator and QCM capability groups were REMOVED from this change:
`deep_causality_do_calculus` and `deep_causality_quantum` will be spec'd as their own separate
changes (requirement material: `causaloid-formalization-roadmap.md` §6–§7, tracker #4/#5, and
this change's git history). No tasks remain here. -->

## 7. Registration & verification (applies to the residue: 1.2–1.4, 2.3, 2.5, 3.6, group 4)

- [ ] 7.1 Register new Lean files in `lean/DeepCausalityFormal.lean`; update `lean/THEOREM_MAP.md` and `deep_causality_core/LEAN_CORE.md`.
- [ ] 7.2 Bare-`lean` typecheck every new file standalone (exit 0, zero `sorry`).
- [ ] 7.3 `bazel test //...` green; `make format && make fix` clean; clippy clean.
- [ ] 7.4 Confirm no new runtime dependency, `unsafe_code = "forbid"` intact, and every deferred proof is an explicit obligation.
