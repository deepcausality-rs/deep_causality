## 1. Stage 2 — Fixpoint + inversion + Λ-edges (`causaloid-fixpoint`)

- [x] 1.1 Write `lean/DeepCausalityFormal/Core/Causaloid.lean`: the signature functor `F` as a
      three-constructor inductive (`atom`/`coll`/`graph`, design D1), `Bag` as `List` + `Perm`
      lemmas (D2); header cites initial-algebra semantics + Hardy gr-qc/0509120 with deviation
      notes; bare-`lean` exit 0, zero `sorry`
- [x] 1.2 Prove `core.causaloid.fixpoint` (initiality/well-foundedness of the inductive; the
      three-summand ↔ three-sealed-forms correspondence stated in the header)
- [x] 1.3 Prove `core.causaloid.inversion` (`evaluate = wiring ∘ element`; the element takes no
      order/position argument — D3)
- [x] 1.4 Rust: add identity-keyed per-edge Λ decoration slots to the hypergraph (fn-pointer slot,
      `None` = identity, D4); undecorated graphs evaluate bit-identically (test)
- [x] 1.5 Rust: reconvergent-join test with two distinct Λ decorations — lookup by edge identity,
      enumeration-order-free
- [x] 1.6 Create the main-crate witness mirror `deep_causality/tests/formalization_lean/`
      (`mod.rs`, `BUILD.bazel`) with `causaloid_tests.rs` witnessing both Stage-2 ids
- [x] 1.7 Extend the CI gate: add `deep_causality` to the Rust-witness search scope in
      `.github/workflows/formalization.yml` (D10)
- [x] 1.8 Add THEOREM_MAP rows for `core.causaloid.{fixpoint,inversion}`; traceability + clippy
      `-D warnings` + `lake build` + `bazel test //...` green; prepare the stage commit message

## 2. Stage 2b — The choice fragment ⊕ (`haft-arrow-choice` + haft deltas)

- [x] 2.1 Eager combinators in `deep_causality_haft`: `Left`/`Right`/`Choice` (`+++`)/`Fanin`
      (`|||`) as defunctionalized `Arrow` structs over `Either` (no `dyn`/`unsafe`/macros)
- [x] 2.2 Extend `ArrowCore<G>` with `Left`/`Right`/`Choice`/`Fanin` variants and `ArrowVal<V>`
      with the sum node (`InL`/`InR`); revisit every existing `ArrowVal` match with explicit arms
      (design risk list)
- [x] 2.3 Extend the `ArrowTerm<In, Out, G>` typed façade: `left::<C>`, `right::<C>`, `choice`,
      `fanin` typed over `Either<_, _>`; add the `compile_fail` doctest for mistyped branch wiring
- [x] 2.4 Extend `ArrowCore::interpret` and `ArrowCore::interpret_kleisli`: `InL`/`InR` dispatch,
      `Fanin` merge
- [x] 2.5 Write `lean/DeepCausalityFormal/Haft/ArrowChoice.lean` (cites Hughes 2000 §5, Lorenz &
      Barrett 2021 §4): prove `haft.arrow_choice.laws` (`left (arr f) = arr (f ⊕ id)`,
      composition/exchange, `fanin` as coproduct elimination) and the used `⊗`-over-`⊕`
      distributivity equations (full rig coherence deferred, noted)
- [x] 2.6 Prove the term-language extensions: `haft.arrow_term.choice_interpret_sound` and
      `haft.arrow_term.choice_free` (extends `Haft/ArrowTerm.lean` scope)
- [x] 2.7 Prove `haft.interpreter.choice_preserved` (extends `Haft/Interpreter.lean` scope)
- [x] 2.8 Rust witnesses for all four id groups in `deep_causality_haft/tests/**`
      (Bazel-registered); THEOREM_MAP rows; traceability + clippy + `lake build` +
      `bazel test //...` green; prepare the stage commit message

## 3. Stage 3 — Verdict closure (`causaloid-verdict-closure`)

- [x] 3.1 Write `lean/DeepCausalityFormal/Core/VerdictClosure.lean`: prove `core.verdict.closure`
      (`All`/`Any`/`None`/`Some(k)` closed in the Verdict algebra; `None` = `Any` ∘ `complement`;
      `Some(k)` via the `Count` monoid + boundary comparison — D6), citing `num.verdict.*` as base
- [x] 3.2 Prove the instantiation at the fixpoint: `Coll : Causaloid → Causaloid` in the model
- [x] 3.3 State `core.verdict.carriers` (bool Boolean + Prob MV behind the one trait, citing the
      landed instances)
- [x] 3.4 Rust **BREAKING**: require `O: Verdict` on the collection-aggregation path (constructors
      + `AggregateLogic` evaluation, not the `Causaloid` struct — D6); survey and migrate all
      in-repo usages (examples, tests, benches); changelog entry with migration text
      *(deviation: CHANGELOG.md is release-plz-generated — migration text lives in the
      `Aggregatable` rustdoc and the stage commit message instead, per user instruction)*
- [x] 3.5 Compile-fail coverage: a non-`Verdict` output type on collection aggregation fails with
      the missing-bound error
- [x] 3.6 Witnesses in `deep_causality/tests/formalization_lean/verdict_closure_tests.rs`;
      THEOREM_MAP rows for `core.verdict.{closure,carriers}`; all gates green; prepare the stage
      commit message

## 4. Stage 4 — The graph algebra (`causaloid-graph-algebra`)

- [x] 4.1 **Gate first:** build the #10 characterization corpus — snapshot current outputs (all
      five channels) for chains, trees, fan-outs, and the loud-fail diamond; commit it green
      against the pre-change engine (D7)
- [x] 4.2 Implement the single-writer freeze check (at most one state-writing branch per join;
      violation = freeze error naming the join — D8) alongside the existing `freeze_dag`, with a
      level-specific hook extension point
- [x] 4.3 Rewrite the graph fold: topological fold with `∇ ∘ (Λ₁ ⊗ Λ₂)` at reconvergent joins
      (value via `CommutativeMonoid` `∇` after per-edge Λ; log as multiset at joins, ordered
      within branches; state under single-writer); `RelayTo` handling and `MAX_RELAY_ROUNDS`
      unchanged
- [x] 4.4 Corpus acceptance: chains/trees bit-identical; the diamond expectation updated to the
      defined merge in the same commit with inline rationale + changelog **BREAKING** note
- [x] 4.5 Write `lean/DeepCausalityFormal/Core/GraphAlgebra.lean`: prove
      `core.causaloid.graph_fold_order_invariant` (invariance under every schedule consistent
      with the derived order), citing `CommutativeMonoid` + `haft.sym_monoidal.*` as base
- [x] 4.6 Witness in `deep_causality/tests/formalization_lean/graph_algebra_tests.rs` exercising
      schedule permutation on a real reconvergent graph; two-writer-diamond freeze-rejection test;
      THEOREM_MAP row; all gates green; prepare the stage commit message

## 5. Stage 5 — The keystone (`causaloid-catamorphism`)

- [x] 5.1 Write `lean/DeepCausalityFormal/Core/Catamorphism.lean`: prove
      `core.causaloid.catamorphism_unique` — hypothesis interpreter + three case equations ⇒
      equals `evaluate`, by induction on the fixpoint; carrier an explicit fixed parameter,
      cross-carrier non-claim in the header (D9)
- [x] 5.2 Prove `core.causaloid.encapsulation_flat` (nested fold = flat fold, catamorphism fusion)
- [x] 5.3 Prove `core.causaloid.arrow_fragment`: the `Atom`/`compose`/`split` fragment (including
      the ⊕-enlarged generators) ≅ `ArrowTerm`; `evaluate = interpret_kleisli` on it; the `T` vs
      `T/≈` quotient factoring as its own lemma
- [x] 5.4 Witnesses in `deep_causality/tests/formalization_lean/catamorphism_tests.rs`: a by-hand
      algebra-respecting interpreter equals `evaluate` (spot-check), wrapped-subgraph vs flattened
      equality, arrow-fragment agreement incl. a `choice`/`fanin` term
- [x] 5.5 THEOREM_MAP rows for the three keystone ids; all gates green; prepare the stage commit
      message

## 6. Close-out

- [x] 6.1 Update `openspec/notes/causal-algebra/algebraic-causaloid-assumptions.md`: close #9, #5,
      #2 Q1 (+ #1 applied), #6 (scoped), #8, B2 with resolution-log entries; update the status
      board
- [x] 6.2 Update `causaloid-formalization-roadmap.md`: Stages 2–5 → LANDED; refresh the closure
      map and dependency spine; note Stages 6–7 as the next change set
- [x] 6.3 Final full gates: traceability (all ids, fail=0), clippy `-D warnings` (workspace),
      bare-`lean` on every new file + `lake build`, `bazel test //...`; prepare the final commit
      message set for user review
