## Why

Stage 1 of the causaloid formalization roadmap
(`openspec/notes/causal-algebra/causaloid-formalization-roadmap.md`) landed 2026-07-09: the full
carrier stack `Except E (Free CausalCommand (Maybe V))` is proven lawful, `fold` is proven the
unique handler, and relay termination is fuel-bounded and enforced in both engine loops. The
organizing theorem — **`Causaloid ≅ μX.F(X)` with `evaluate` the unique F-algebra catamorphism into
`Kleisli` of the causal monad** — remains unproven, and it is the foundation the two downstream
crates (`deep_causality_do_calculus`, `deep_causality_quantum`) inherit. This change executes
Stages 2–5: the fixpoint and the Hardy inversion with per-edge Λ decoration (Stage 2), the
coproduct/choice generator `⊕` at the haft layer (Stage 2b — required by Lorenz & Barrett 2021 §3
for causally faithful quantum reification, and by classical case-splitting), the Verdict closure of
`Collection` (Stage 3), the schedule-invariant graph fold `∇ ∘ (Λ ⊗ Λ)` (Stage 4), and the
catamorphism-uniqueness keystone (Stage 5). Stages 6–7 (the extensibility contract and the two
crates) follow as a dedicated change set.

## What Changes

- **Stage 2 — fixpoint + inversion + Λ-edges.** `Core/Causaloid.lean` defines the signature functor
  `F(X) = Atom + Coll(Bag X, AggLogic) + Graph(Hyper X, Λ-edges)` and proves `Causaloid ≅ μX.F(X)`
  well-founded (`core.causaloid.fixpoint`, closes assumption **#9**) and the inversion factorization
  — `evaluate` factors as (symmetric local data) ∘ (asymmetric wiring), no ordering asymmetry inside
  the element (`core.causaloid.inversion`). Rust: identity-keyed, order-free **per-edge Λ decoration
  slots** on hyperedges (the connection data of `join = ∇ ∘ (Λ₁ ⊗ Λ₂)`).
- **Stage 2b — the choice fragment `⊕` (ArrowChoice).** Eager `Left`/`Right`/`Choice`/`Fanin`
  combinators over the proven coproduct `Either`; the reified term language (`ArrowCore` /
  `ArrowVal` / typed `ArrowTerm` façade) and both interpreters extended with the choice generators;
  ArrowChoice laws, interpretation soundness, the extended free property, and choice preservation
  proved in Lean. The second confirmed wiring-generator extension after `∇` (assumption #11a).
- **Stage 3 — Verdict closure.** `core.verdict.closure`: `All/Any/None/Some(k)` are closed
  operations in the `Verdict` algebra, so `Coll : Causaloid → Causaloid` (closes assumption **#5**);
  `core.verdict.carriers` names the Boolean and MV/[0,1] instances behind the one trait.
  **BREAKING**: `Collection` aggregation requires `O: Verdict` in place of "any `O`".
- **Stage 4 — the graph algebra.** The topological fold with `∇ ∘ (Λ ⊗ Λ)` at reconvergent joins,
  proven invariant under every schedule consistent with the derived causal order
  (`core.causaloid.graph_fold_order_invariant`, closes assumption **#2 Q1**); the per-channel policy
  (value → `∇` CommutativeMonoid; log → multiset at joins; state → single-writer, checked at
  freeze); freeze checks (acyclicity + single-writer + level hooks). Gated on the **#10
  characterization-test corpus** built first — chains/trees stay bit-identical; **BREAKING**: the
  loud-fail diamond becomes the defined-merge diamond (documented behavior change).
- **Stage 5 — the keystone.** `core.causaloid.catamorphism_unique` — `evaluate` is the unique
  F-algebra homomorphism into `Kleisli<M>` per fixed carrier (closes **B2** and **#6**, scoped);
  `core.causaloid.encapsulation_flat` — nested fold = flat fold (catamorphism fusion);
  `core.causaloid.arrow_fragment` — the `Atom`/`compose`/`split` fragment ≅ `ArrowTerm` with
  `evaluate = interpret_kleisli` on it (closes **#8**), covering the ⊕-enlarged term.
- **The bridge extends to the main crate.** Causaloid-layer witnesses live in
  `deep_causality/tests/formalization_lean/`; the CI theorem-map gate adds `deep_causality` to its
  Rust-witness search scope.
- **Out of scope** (unchanged from the roadmap): `core.causaloid.hardy_correspondence` (open
  publication target, blocks neither crate), assumption #4 (atom registry), Stages 6–7.

## Capabilities

### New Capabilities
- `causaloid-fixpoint`: the signature functor `F`, the well-founded fixpoint `Causaloid ≅ μX.F(X)`,
  the Hardy-inversion factorization of `evaluate`, and per-edge Λ decoration slots (Stage 2).
- `haft-arrow-choice`: the eager ArrowChoice fragment (`Left`/`Right`/`Choice`/`Fanin`) over
  `Either`, with the ArrowChoice laws and the `⊗`-over-`⊕` distributivity equations used
  (Stage 2b).
- `causaloid-verdict-closure`: `Collection` aggregation closed over the `Verdict` algebra —
  `Coll : Causaloid → Causaloid` as a theorem, `O: Verdict` as the stated bound (Stage 3).
- `causaloid-graph-algebra`: the schedule-invariant topological fold `∇ ∘ (Λ ⊗ Λ)`, the per-channel
  join policy, freeze-time checks, and the #10 characterization corpus gate (Stage 4).
- `causaloid-catamorphism`: uniqueness of `evaluate` as the F-algebra catamorphism, encapsulation
  flatness, and the arrow-fragment correspondence (Stage 5).

### Modified Capabilities
- `haft-free-arrow`: the reified term language gains the choice generators (`Left`/`Right`/
  `Choice`/`Fanin` in `ArrowCore`, a sum node in `ArrowVal`, typed façade methods over `Either`)
  with build-time wiring safety preserved.
- `haft-arrow-interpreter`: both interpreters (`interpret`, `interpret_kleisli`) dispatch the choice
  generators; functoriality extends to choice preservation (`haft.interpreter.choice_preserved`).
- `core-formalization`: the witness mirror and the CI theorem-map gate extend to the main
  `deep_causality` crate for causaloid-layer Core Lean files.

## Impact

- **Lean:** new `Core/Causaloid.lean`, `Core/VerdictClosure.lean`, `Core/GraphAlgebra.lean`,
  `Core/Catamorphism.lean`, `Haft/ArrowChoice.lean`; extensions in scope of `Haft/ArrowTerm.lean` /
  `Haft/Interpreter.lean`; ~14 new THEOREM_MAP rows. All bare-`lean`, zero `sorry`, cited
  (Hughes 2000 §5; Lorenz & Barrett 2021 §3–4; Hardy gr-qc/0509120; standard initial-algebra
  semantics).
- **`deep_causality_haft`:** new choice combinators + `ArrowCore`/`ArrowVal`/`ArrowTerm` variants +
  interpreter arms; witness tests incl. a `compile_fail` doctest.
- **`deep_causality` (main crate):** per-edge Λ slots on hyperedges; `Collection` gains the
  `O: Verdict` bound (**BREAKING**); the graph-reasoning fold rewritten to the schedule-invariant
  `∇ ∘ (Λ ⊗ Λ)` algebra behind the #10 characterization corpus (**BREAKING** on the diamond case
  only, documented); new `deep_causality/tests/formalization_lean/` witness mirror.
- **`deep_causality_algebra`:** consumed as-is (`Verdict`, `CommutativeMonoid` already landed).
- **CI:** `.github/workflows/formalization.yml` witness-search scope gains `deep_causality`.
- **Assumption tracker closures:** #9, #5, #2 Q1 (+ #1 applied), #6 (scoped), #8, B2; the `⊕`
  generator requirement of the quantum roadmap. **Unblocks** Stages 6–7
  (`deep_causality_do_calculus`, `deep_causality_quantum`).
- **Constraints preserved:** `unsafe_code = "forbid"`, no `dyn`, no crate-defined macros,
  clippy `-D warnings`, Bazel-registered tests, `bazel test //...` green per stage.
