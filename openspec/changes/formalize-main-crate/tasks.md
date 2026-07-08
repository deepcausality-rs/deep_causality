## 1. Groundwork & reconciliation

- [ ] 1.1 Verify the F-1 caveat is closed: confirm `error ⇒ value=None` holds by construction on the `Result<CausalEffect<V>, E>` carrier; record the finding.
- [ ] 1.2 Reconcile `openspec/notes/causal-algebra/Causaloid-Formalization.md` from `EffectValue`/`ContextualLink`/`Map` to the current `CausalEffect` model; flag each changed claim (F-3 = command-input-errors).
- [ ] 1.3 Reconcile `openspec/notes/causal-algebra/CausalMonadProptest.md` to the `CausalEffect` model and note the timestamp-equality fix already landed.
- [ ] 1.4 Add a `sorry`/obligation CI guard: fail if a `sorry` appears outside the whitelisted `Quantum/*` obligation slots.

## 2. Causaloid layer (`Core/Causaloid.lean`, `Core/Collection.lean`)

- [ ] 2.1 `Core/Causaloid.lean`: model singleton = context-parameterized Kleisli arrow `I → CausalEffect<O>`; prove identity/compose/error-left-zero by reduction to `Core/CausalArrow.lean`; bare-`lean` typecheck.
- [ ] 2.2 Prove `evaluate` is the value-fragment extension of the arrow; state command-input-errors (F-3) and unconditional right identity (no F-1 side-condition).
- [ ] 2.3 `Core/Collection.lean`: model Collection aggregate as a commutative-monoid fold over the verdict carrier; prove permutation-invariance for `AggregateLogic {All, Any, None, Some(k)}`.
- [ ] 2.4 Rust witnesses for 2.1–2.3 under `deep_causality/tests/formalization_lean/`; add `THEOREM_MAP.md` rows.

## 3. Graph-reasoning engine (`Core/GraphReasoning.lean`)

- [ ] 3.1 Model the engine as `Free::fold` over the canonical topological linearization — a sequential, single-hole program whose reconvergent sharing lives in the keyed valuation (let-environment), never in duplicated subterms; define the `jump` algebra (`fold(Suspend(RelayTo(t,k))) = jump(t, fold(k))`); reduce to `Haft/FreeMonad.lean` fold laws.
- [ ] 3.2 Compose with the prerequisite `comonoid-graph-join` theorems (`unique_valuation`, `schedule_invariance`, disjoint-key union lemmas) rather than re-deriving fan-in; state the copy/discard laws as laws of the classical interpreter only (interpreter-neutral substrate — no substrate-level duplication law a quantum instantiation would violate).
- [ ] 3.3 Prove local `jump` correctness (state/context/log threading) matching the single-level relay; state the nested-relay-folds-structurally property.
- [ ] 3.4 Rust witnesses for the fold/jump on the linear+relay fragment; `THEOREM_MAP.md` rows.
- [ ] 3.5 Confirm `comonoid-graph-join` has landed (engine is comonoid-correct) before finalizing this group; no engine-gap remains to flag.

## 4. Context hypergraph (`Core/ContextGraph.lean`)

- [ ] 4.1 Model the contextoid hypergraph with parent-set map `Pa` keyed by parent index (the same labeled-wire surface `comonoid-graph-join` exposes) and the hyperedge-threading = `bind` correspondence; encapsulation-equals-flat via `core.causal_monad.assoc`.
- [ ] 4.2 Model acyclicity as a separable constraint; map the acyclic case to `ultragraph::has_cycle`/`freeze`; show the cyclic case reuses the same definitions.
- [ ] 4.3 Rust witnesses (parent-set threading; freeze acyclicity gate); `THEOREM_MAP.md` rows.

## 5. Intervention / do-operator (`Core/Intervention.lean`)

- [ ] 5.1 Define `do(X=x)` as total graph surgery on the reified hypergraph, built from the `comonoid-graph-join` D10 primitive: delete every in-wire key of `X`, pin `X`'s mechanism to the constant `x`; keep the single-edge cut (`delete (P1, X)`, keep `(P2, X)`) as the expressible finer operation.
- [ ] 5.2 Prove acyclic surgery stays acyclic (maps to `has_cycle` acceptance at freeze).
- [ ] 5.3 Define the intervention handler as an alternate `Free::fold` algebra over the `RelayTo` program.
- [ ] 5.4 Prove intervention commutes with encapsulation (functoriality; via `core.causal_monad.assoc`).
- [ ] 5.5 Rust witnesses (surgery cut/pin; nest-then-intervene = intervene-then-nest); `THEOREM_MAP.md` rows.

## 6. QCM predicate/obligation layer (`Quantum/CJOp.lean`, `Quantum/QCM.lean`)

- [ ] 6.1 `Quantum/CJOp.lean`: define `CJOp` over tagged `H ⊗ H*` leg-sets; `embed`/pad and `traceOut` (partial trace); bare-`lean` typecheck (Mathlib operator API permitted here).
- [ ] 6.2 Prove padding/marginalization basics (suppressed-identity multiplication; `ρ_{B|A} = Tr_C ρ_{BC|A}`).
- [ ] 6.3 `Quantum/QCM.lean`: define `NoInfluence`/`DirectCause`/`Pa` (Def 1 / Def 4.1) and `Factorizes` / `PairwiseCommute` / `IsMarkov`.
- [ ] 6.4 Prove the `n=2` commutativity lemma (from Hermiticity); state `n ≥ 3` commutativity as an axiom/obligation.
- [ ] 6.5 Define `ValidProcess` as a SEPARATE predicate; show factorize+commute ⇒ valid in the acyclic/unitary case, independent in the cyclic case.
- [ ] 6.6 State `Compatible G σ` and the theorem slot `Compatible → IsMarkov`; record the converse as an open `Hypothesis`.
- [ ] 6.7 State the `traceOut_preserves_commute` obligation (Layer-D) as a named obligation; no hidden `sorry`.
- [ ] 6.8 Keep acyclicity a parameter throughout so the cyclic QCM case (quantum switch) reuses 6.3–6.7 unchanged.
- [ ] 6.9 Operator-valued-state Rust witness: exercise the causal-monad law-3 property test with a matrix-valued `State` payload; `THEOREM_MAP.md` rows for all QCM obligations (status `obligation`/`open`).

## 7. Registration & verification

- [ ] 7.1 Register all new Lean files in `lean/DeepCausalityFormal.lean`; update `lean/THEOREM_MAP.md` and `deep_causality_core/LEAN_CORE.md` (advance work-plan items #2/#11 graph-join, #12 handler).
- [ ] 7.2 Bare-`lean` typecheck every new file standalone (`lean DeepCausalityFormal/<file>.lean` → exit 0).
- [ ] 7.3 `bazel test //...` green (all Rust witnesses pass); `make format && make fix` clean; clippy clean.
- [ ] 7.4 Confirm no new runtime dependency, `unsafe_code = "forbid"` intact, and every deferred proof is an explicit obligation (grep guard from 1.4 passes).
