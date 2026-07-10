<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Causaloid Formalization Roadmap

**What this is.** The staged program that formalizes the causaloid in `deep_causality` so that the
four-level causal stack of [`../quantum/full-stack.md`](../../quantum/full-stack.md) rests on one proven
foundation, and the two downstream crates — **`deep_causality_do_calculus`** (Pearl / do-calculus)
and **`deep_causality_quantum`** (QCM / Lorenz, ICO / Hardy) — can be implemented *and formalized*
against that foundation instead of re-establishing it.

**Goals.**
- **A.** Formalize the `deep_causality` crate (the causaloid, its evaluation, its graph algebra).
- **B.** Express Pearl's do-calculus through the formalism (`deep_causality_do_calculus`).
- **C.** Express the quantum causal model (Lorenz/Barrett/Oreshkov) through the formalism
  (`deep_causality_quantum`).
- **Bonus.** Express indefinite causal structure (ICO / Hardy) through the formalism.

**Companions.** [`Causaloid-structure.md`](Causaloid-structure.md) (the structure and the Hardy
inversion) · [`../quantum/full-stack.md`](../../quantum/full-stack.md) (the four levels, HAVE/GAP) ·
[`algebraic-causaloid-assumptions.md`](algebraic-causaloid-assumptions.md) (the assumption tracker
this roadmap closes against) · [`../quantum/QCM-on-EPP.md`](../../quantum/QCM-on-EPP.md) (the level-4
reconstruction) · [`../quantum/quantum-epp.md`](../../quantum/quantum-epp.md) (the QPU-as-effect
corollary).

Honesty convention: **[holds]**, **[holds under precondition]**, **[open]**, **[planned]**,
**[speculative]**.

---

## 1. The organizing theorem

Everything hangs off one statement, proven once, parametric in everything the levels vary:

> **`Causaloid ≅ μX.F(X)`** with
> `F(X) = Atom(I →ᴹ O)  +  Coll(Bag X, AggLogic)  +  Graph(Hyper X, Λ-edges)`,
> and **`evaluate` is the unique F-algebra homomorphism (catamorphism) from `Causaloid` into the
> Kleisli category of the causal monad** — parametric over `(I, O, STATE, CTX)` and over the
> verdict carrier.

The "three shapes, one trait surface, isomorphic-recursive" structure of
[`Causaloid-structure.md`](Causaloid-structure.md) *is* this fixpoint; the formalization turns the
design note into a theorem. Every stack level then instantiates parameters — a carrier, a fan-in
monoid, an order source, a freeze check — and never touches the core again. This is the extensible
`F` that assumption #11a concludes must exist.

## 2. Standing on: what is already proven

- **Causal monad lawful, unconditional** — `core.causal_monad.{left_id, right_id, assoc, left_zero,
  lawful}`; Kleisli category threading state+context — `core.causal_arrow.{category_laws,
  left_zero}` (assumption #7 DECIDED). **[holds]**
- **The haft categorical machinery** (change `haft-categorical-machinery`, 2026-07-08):
  `Foldable::fold_map` over the generic `Monoid` tower; `Category`/`Fun`/`Kleisli<M>`; the reified
  free Arrow `ArrowTerm`/`ArrowCore` (typed construction, erased storage — assumption #3 DECIDED);
  the one-way interpreter `interpret_kleisli` + `NaturalTransformation`; the symmetric-monoidal
  PROP `SymMonoidal` (copy `Δ`/`ε`, merge `∇`/`η`, symmetry `σ`). All Lean-proven with Rust
  witnesses. **[holds]**
- **The coproduct** `Either` with `haft.either.coproduct_universal` — the classical direct-sum
  generator (needed in §8; see the Lorenz–Barrett faithfulness result). **[holds]**
- **Collection order-invariance, scoped** — value-only / stateless / all-success / up-to-log-
  permutation (assumption #1 DECIDED). **[holds]**
- **Free-monad laws** — `haft.free_monad.{left_id, right_id, assoc, lift_bind, map_id}`. **[holds]**

## 3. Three leverage facts that shape the program

1. **The outcome channel is compositional, already-proven algebra.** The carrier is
   `Result<CausalEffect<V>, E>` = `Except E (Free CausalCommand (Maybe V))` — a transformer stack of
   three monads whose laws are each in the Lean corpus (Except via the effect system, `Free` via
   `haft.free_monad.*`, `Maybe` via the functor/monad laws), with the composite value-fragment
   already proven (`core.causal_monad.lawful`). Stage 1 is assembly, not invention.
2. **`RelayTo` is an algebraic effect.** `CausalEffect<V> = Free<CausalCommandWitness, Option<V>>`;
   `RelayTo` is a `Suspend` node — an *operation* of the free monad — and the reasoning engine is
   the **handler** (`CausalEffect::fold`). The "computed goto outside any static diagram" framing in
   assumption #2 Q3 dissolves into: *the handler is a fuel-bounded free-monad catamorphism*, a
   standard formal object. Termination = fuel; status = named operation functor.
3. **Do-calculus requires reification — which landed.** `do(X = x)` is graph surgery: cut the
   incoming hyperedges of `X`, pin its value. Surgery is a syntactic rewrite, and rewrites need a
   *term*, not an opaque closure. `ArrowCore` + the reified hypergraph are the surgical substrate;
   H3 of `haft-categorical-machinery` was the hidden prerequisite for Goal B.

## 4. The Hardy inversion, precisely (the physics anchor)

Primary source: Hardy, *Probability Theories with Dynamic Causal Structure* (arXiv:gr-qc/0509120)
— [`deep_causality/papers/causaloid.pdf`](../../../../deep_causality/papers/causaloid.pdf).

- Hardy's causaloid product (Eq. 2, p. 4) composes regions by **union**:
  `r(R₁ ∪ R₂) = r(R₁) ⊗^Λ r(R₂)` — symmetric, all elementary regions on an equal footing. The
  theory-specific **Λ matrices** are the connection data that "break the symmetry between elementary
  regions" (p. 4). The product's purpose (§2, p. 3) is to **unify** quantum theory's two products —
  spacelike `Â ⊗ B̂` and timelike sequential `B̂Â` — so causal structure need not be known in advance.
- DeepCausality realizes the same content with the opposite factorization
  ([`Causaloid-structure.md`](Causaloid-structure.md)): the **element** (causal function) stays
  symmetric — it sees values with intrinsic identity, never spacetime position — and the asymmetry
  lives in the **composition**: two modes, `bind` (sequential/timelike) and the commutative merge
  `∇` (simultaneous/spacelike), selected by order *derived from channel data* (state/context
  invariants; see `event_horizon_probe`). Hardy: one symmetric product, Λ decides the regime.
  DeepCausality: two composition modes, derived order decides the regime. Neither puts before/after
  inside the element.
- **Per-edge Λ decoration** reconciles the two: a reconvergent join is
  `join = ∇ ∘ (Λ₁ ⊗ Λ₂)` — each incoming connection may carry its own asymmetric transform Λᵢ,
  keyed by intrinsic edge identity (never by order), and the fuse `∇` stays commutative. Connection
  asymmetry à la Hardy, order-freedom à la the spacelike fan-in, in one formula. This un-defers the
  "Hardy's Λ on edges" item from the tracker's 2026-07-07 resolution-log entry.

**Correspondence target** (bonus goal, publication-grade): DeepCausality's `{bind, ∇ ∘ (Λ ⊗ Λ)}`
pair reconstructs Hardy's ⊗^Λ regimes — timelike Λ ↦ `bind`, spacelike Λ ↦ plain `∇`, general Λ ↦
decorated merge. Hardy's "well-defined probability" condition (Eq. 3, p. 4: `prob = |v|/|u|` iff
`v ∥ u`) maps into the freeze-time global-check family. **[open — target]**

## 5. The stages (core crate)

Each stage ends with the established loop: Lean file(s) with textbook citation + deviation notes →
Rust witness tests (Bazel-registered, `lean-test` tag) → `THEOREM_MAP.md` rows → bare-`lean` exit 0
→ traceability green → `bazel test //deep_causality/...` green.

### Stage 1 — The carrier stack (assembly)

Formalize the full `CausalEffectPropagationProcess`, channel by channel.

| Proposed id | Statement | Closes |
|---|---|---|
| `core.causal_effect.transformer_stack` | `Except E (Free CausalCommand (Maybe V))` is a lawful monad as a composite of the three proven layers | — (assembly) |
| `core.causal_effect.fold_universal` | the handler `fold` is the unique interpreter of the `CausalCommand` operation functor (free-monad catamorphism) | — |
| `core.causal_effect.relay_termination` | the fuel-bounded handler is total (relay bound) | **#2 Q3** (RelayTo status + termination) |
| channel laws | W-invariant by construction (already); log = non-commutative Writer monoid (order-significant, per #1); state Markovian threading; context = Reader, **optional** (state channel can carry spacetime — `event_horizon_probe`) | #11b via the immutable-context constructor (write methods unreachable) |

**Status: LANDED 2026-07-09.** All three theorem groups proved in `Core/CausalEffect.lean`
(bare-`lean`), witnessed in `deep_causality_core/tests/formalization_lean/causal_effect_tests.rs`,
THEOREM_MAP rows added. Rust: `CausalEffect::{and_then, try_and_then}` realize the stack bind;
relay fuel (`MAX_RELAY_ROUNDS`) added to both graph-reasoning loops with classical + stateful
relay-cycle regression tests — **#2 Q3's termination item closed**. The "immutable-context
constructor" resolved by **verification instead of new code**: no context type carries interior
mutability and `Arc` forbids `&mut`, so the referent is immutable within a pass while the slot is
per-branch threaded data — **#11b DECIDED** by scoping (see the tracker). **[holds]**

### Stage 2 — The element and the inversion

`Core/Causaloid.lean`: define `F`, the fixpoint, and the symmetry factorization.

| Proposed id | Statement | Closes |
|---|---|---|
| `core.causaloid.fixpoint` | `Causaloid ≅ μX.F(X)`, well-founded (nesting guard or fuel — μX, not νX) | **#9** |
| `core.causaloid.inversion` | the element carries no ordering asymmetry: `evaluate` factors as (symmetric local data) ∘ (asymmetric wiring) | the Hardy-inversion thesis, formal |
| `core.causaloid.hardy_correspondence` | `{bind, ∇ ∘ (Λ ⊗ Λ)}` reconstructs the ⊗^Λ regimes (§4) | bonus goal **[open — target]** |

Rust refactor: per-edge Λ decoration slots on hyperedges (identity-keyed, order-free).

**Status: LANDED 2026-07-10** (change `causaloid-formalization-stages-2-5`). `Core/Causaloid.lean`
proves `core.causaloid.fixpoint` (the `roll`/`unroll` Lambek isomorphism, `size` as the μ-witness,
three summands ↔ the three sealed forms — closes **#9**) and `core.causaloid.inversion`
(`eval = wiring ∘ element-map`, element bag-symmetric via `mapL_perm`). Rust: `LambdaEdges`
(identity-keyed fn-pointer slots, absent = identity) + `from_causal_graph_with_lambda_edges`;
main-crate witness mirror `deep_causality/tests/formalization_lean/` created; CI theorem-map gate
scope extended to `deep_causality`. `hardy_correspondence` remains **[open — target]**. **[holds]**

### Stage 2b — The choice fragment: a coproduct/direct-sum generator `⊕` (ArrowChoice)

**Requirement.** The reified wiring language MUST carry a coproduct/direct-sum generator beyond
`compose`/`split`. Two independent forcings:

- **Quantum faithfulness (Lorenz & Barrett 2021, §3–4).** Sequential + tensor composition cannot
  make all no-influence relations of a unitary simultaneously evident; causally faithful
  decomposition requires **direct-sum** structure (their extended circuit diagrams). A quantum term
  language without `⊕` is provably unable to reify the causal structure of general unitaries.
- **Classical case-splitting.** Contextual switches, regime selection, and counterfactual branching
  are coproduct eliminations; do-calculus models with context-dependent mechanisms want the same
  generator. The classical shadow is Hughes' **ArrowChoice** (Hughes 2000, §5) over the proven
  coproduct `Either` (`haft.either.coproduct_universal`).

**Deliverables (haft change, `haft-arrow-choice`-shaped, mirroring `haft-categorical-machinery`):**

- Eager combinators: `Left`/`Right`/`Choice` (`+++`)/`Fanin` (`|||`) as defunctionalized `Arrow`
  structs over `Either` (the value-level ArrowChoice fragment).
- Reified term: extend `ArrowCore<G>` with `Left(f)`, `Right(f)`, `Choice(f, h)`, `Fanin(f, h)`;
  extend `ArrowVal<V>` with a sum node (`InL`/`InR`) alongside `Leaf`/`Pair`; extend the
  `ArrowTerm<In, Out, G>` typed façade with `left::<C>`, `right::<C>`, `choice`, `fanin` typed over
  `Either<_, _>` (mistyped branch wiring fails to compile, `compile_fail` doctest as in H3).
- Interpreters: extend `ArrowCore::interpret` and `ArrowCore::interpret_kleisli` — `InL`/`InR`
  dispatch; `Fanin` merges the branches into one output.
- Distributivity note: state the `⊗`-over-`⊕` interaction (pairs distribute over sums — the rig-
  category coherence faithful decompositions lean on); prove the equations used, defer full
  coherence-diagram machinery.

| Proposed id | Statement |
|---|---|
| `haft.arrow_choice.laws` | the ArrowChoice equations on the eager fragment (`left (arr f) = arr (f ⊕ id)`, composition/exchange laws, `fanin` as the coproduct elimination) |
| `haft.arrow_term.choice_interpret_sound` | interpreting the choice generators agrees with the eager combinators (extends `haft.arrow_term.interpret_sound`) |
| `haft.arrow_term.choice_free` | the free/universal property extends to the enlarged generator set (agree on generators ⇒ agree on every term) |
| `haft.interpreter.choice_preserved` | `interpret_kleisli` preserves the choice generators (extends `preserves_id`/`preserves_compose`) |

Lean: `Haft/ArrowChoice.lean` (+ extensions to `ArrowTerm.lean`/`Interpreter.lean` scope), citing
Hughes 2000 §5 and Lorenz & Barrett 2021 §4, deviation notes as usual. Downstream:
`deep_causality_quantum` instantiates `⊕` as the Hilbert-space direct sum (sectorized wires);
`deep_causality_do_calculus` uses it for context-dependent mechanisms. `⊕` is the **second
confirmed generator extension** after `∇` (assumption #11a).

**Status: LANDED 2026-07-10** (change `causaloid-formalization-stages-2-5`, Stage 2b). All four
deliverables shipped exactly as specified: eager `Left`/`Right`/`Choice`/`Fanin` over `Either`
with `Arrow::{left, right, choice, fanin}`; `ArrowCore`/`ArrowVal` (sum node `InL`/`InR`) and the
typed `ArrowTerm` façade extended (`compile_fail` guard for mistyped branch wiring); both
interpreters dispatch the sum (effect only on the taken branch; `Fanin` eliminates);
`Haft/ArrowChoice.lean` + `ArrowTerm`/`Interpreter` extensions prove all four id groups, with the
used `⊗`-over-`⊕` distributivity (full rig coherence deferred, noted). **[holds]**

### Stage 3 — Verdict carrier + Collection closure

| Proposed id | Statement | Closes |
|---|---|---|
| `core.verdict.closure` | `All/Any/None/Some(k)` are closed operations in the `Verdict` algebra (bounded lattice + complement + threshold) ⇒ `Coll : Causaloid → Causaloid` | **#5** — the "a collection always outputs another causaloid" claim, made rigorous |
| `core.verdict.carriers` | Boolean and MV/[0,1] instances behind one `Verdict` trait | names #6's two carriers |

Already landed at the algebra layer: the `Verdict` trait (`bottom`/`top`/`meet`/`join`/`complement`)
with the `bool` (Boolean) and `Prob` (MV, `[0,1]`) instances, `Algebra/Verdict.lean`, and the
`AggregateLogic` carrier monoids (`Conjunction`, `Disjunction`, `Count`, `Prob`) with
`CommutativeMonoid: Monoid + Commutative` enforced as a type bound. **[holds]**

Remaining Rust refactor: `Collection` requires `O: Verdict` instead of "any `O`", plus the closure
theorem itself. This is the level-1 carrier prerequisite.

**Status: LANDED 2026-07-10** (change `causaloid-formalization-stages-2-5`, Stage 3).
`Core/VerdictClosure.lean` proves `core.verdict.closure` (all four modes closed; `None` = `Any` ∘
`complement`; `Some(k)` = Count + boundary decision; `coll_closure` at the fixpoint — closes
**#5**) and `core.verdict.carriers` (Boolean proved, MV Rust-witnessed; orthomodular projection
lattice planned for quantum, general effects excluded — the scope guard in the Stage-3 spec).
Rust: `Aggregatable: Verdict` (BREAKING, `compile_fail`-pinned); new instances `f64` (algebra
crate, MV), `UncertainBool`/`UncertainF64` (uncertain crate, pointwise; `ArithmeticOperator`
gained `Min`/`Max`). **[holds]**

### Stage 4 — The graph algebra (the last load-bearing gap)

| Proposed id | Statement | Closes |
|---|---|---|
| `core.causaloid.graph_fold_order_invariant` | the topological fold with `∇ ∘ (Λ ⊗ Λ)` at reconvergent joins is invariant under every schedule consistent with the derived causal order | **#2 Q1** — the join semantics, decided as commutative-`∇`-per-input-type |
| per-channel policy | value → `∇` (CommutativeMonoid); log → multiset at the join (#1's "up to log permutation", applied); state → **single-writer invariant** at joins, checked at freeze | the per-channel ruling |
| freeze checks | acyclicity (opt-in `freeze_dag`, existing) + single-writer + level-specific hooks | the enforcement point (QCM-on-EPP's freeze model) |

Gate: the **#10 characterization-test corpus first** — the fold must reproduce current behaviour
bit-identically on chains/trees; the loud-fail diamond becomes the defined-merge diamond as a
documented change.

**Status: LANDED 2026-07-10** (change `causaloid-formalization-stages-2-5`, Stage 4). Corpus
committed against the pre-change engine, then the reconvergence arm became the defined join
`∇ ∘ (Λ₁ ⊗ Λ₂)` with `∇ = Verdict::join` (`V: Verdict` on the reasoning trait; new
`evaluate_subgraph_from_cause_with_lambda_edges` consumes the Stage-2 Λ slots). Per-channel policy
as specified; single-writer checked at freeze (`freeze_verified` with declared writers +
level-specific hook; pre-fork writers cannot conflict); stateful engine keeps its loud failure
behind the guard. `Core/GraphAlgebra.lean` proves `core.causaloid.graph_fold_order_invariant`
(fuse is a bag; consistent schedules compute the schedule-free denotation and agree) — closes
**#2 Q1** (+ #1 applied, #10 applied). **[holds]**

### Stage 5 — The keystone

| Proposed id | Statement | Closes |
|---|---|---|
| `core.causaloid.catamorphism_unique` | `evaluate` is the **unique** F-algebra homomorphism into `Kleisli<M>`, **per fixed carrier** | **B2**; **#6** (uniqueness, correctly scoped per semantic algebra) |
| `core.causaloid.encapsulation_flat` | nested fold = flat fold (catamorphism fusion; inherited from monad law-3) | QCM-on-EPP Layer B, generalized to the whole causaloid |
| `core.causaloid.arrow_fragment` | the `Atom`/`compose`/`split` fragment ≅ `ArrowTerm`, and `evaluate` = `interpret_kleisli` on it | **#8** — `T` (free term) vs `T/≈` (quotient by the proven Arrow laws); the interpreter factors through `T/≈` |

This is the theorem the downstream crates inherit.

**Status: LANDED 2026-07-10** (change `causaloid-formalization-stages-2-5`, Stage 5).
`Core/Catamorphism.lean` proves all three: `catamorphism_unique` (initiality per fixed carrier —
closes **B2** and **#6**, correctly scoped), `encapsulation_flat` (wrapper transparency +
continuation fusion), `arrow_fragment` + `interp_respects_category_laws` (fragment ≅ `ArrowTerm`,
⊕-enlarged set covered; interpretation factors through `T/≈` — closes **#8**). Witnessed on the
real engine (by-hand interpreter agreement, one-pass vs two-stage chains, `Choice`/`Fanin` term).
**[holds]**

### Stage 6 — The extensibility contract

Publish, as the crate-authoring spec, the five things a downstream crate supplies to inherit the
Stage-5 guarantees:

1. a **lawful carrier** (a `Verdict`/operator algebra instance + its laws),
2. a **lawful fan-in** (a `CommutativeMonoid` `∇`, optionally with per-edge Λ's),
3. an **order derivation** (or the trivial/topological one),
4. its **freeze-time global check** (the one level-specific structural condition),
5. its **level-specific theorems** (beyond the inherited catamorphism laws).

Uniqueness, encapsulation-flatness, and schedule-invariance transfer by the universal property.
**[planned]**

## 6. `deep_causality_do_calculus` (Goal B)

Carrier: the probability `Verdict` (Stage 3). Wiring: the reified hypergraph (Stage 5's
`arrow_fragment` + the frozen graph).

| Proposed id | Statement |
|---|---|
| `docalc.intervention_surgery` | `do(X = x)` = the term rewrite cutting `X`'s incoming hyperedges and pinning its value — an endofunctor on `Causaloid` |
| `docalc.truncated_factorization` | the surgered model's distribution is Pearl's truncated product |
| `docalc.d_separation` | d-separation over the hypergraph ⟺ conditional independence under the fold semantics |
| `docalc.rule1` / `rule2` / `rule3` | Pearl's three rules of do-calculus, sound against `d_separation` |
| `docalc.counterfactual_twin` | counterfactuals via the twin-network construction (two surgered copies sharing exogenous nodes) |

References: `openspec/notes/causal-do/` (Pearl, *Introduction to Do-Calculus*; *The Do-Calculus
Revisited*). Everything classical and verifiable; the crate is thin because Stages 3–5 did the
work. **[planned]**

## 7. `deep_causality_quantum` (Goal C + bonus)

Carrier: operator-valued CJ state on the arity-5 **state channel** (`Float106` complex matrices),
per [`../quantum/QCM-on-EPP.md`](../../quantum/QCM-on-EPP.md). Primary sources: Lorenz 2022 (Synthese
200:424) and Lorenz & Barrett 2021 (arXiv:2001.07774), both in `openspec/notes/quantum/`.

| Proposed id | Statement | Source |
|---|---|---|
| `quantum.markov_commutativity` | the freeze-time Layer-D walk: `σ` is Markov for `G` iff it factorizes into **pairwise-commuting** CJ operators `ρ_{Aᵢ|Pa(Aᵢ)}` (the commutativity clause is the load-bearing content at ≥3 factors) | Lorenz 2022, Def 3.3 |
| `quantum.unitary_factorization` | for **unitary** channels the commuting factorization holds automatically — `ρ^U = ∏ⱼ ρ_{Bⱼ|Pa(Bⱼ)}`, `[ρ_{Bⱼ|Pa(Bⱼ)}, ρ_{Bₘ|Pa(Bₘ)}] = 0` — so the freeze check targets general (open-system) process operators and is a *theorem* on the unitary fragment | Lorenz & Barrett 2021, Thm 1 |
| `quantum.no_influence` | the causal relation: `A ⇸ D` iff `Tr_B[ρ^U_{BD|AC}] = ρ^M_{D|C} ⊗ 1_{A*}` (no signalling from `A` to `D`); causal structure = the parent-set family `{Pa(Bⱼ)}` | Lorenz & Barrett 2021, Defs 1–2 |
| `quantum.partial_trace_preservation` | **the one hard open theorem**: encapsulation (partial trace over an interior subgraph) preserves pairwise commutation with the new neighbours — prove under identifiable conditions (single-node interface; shared supports on the boundary) or produce the counterexample; discover conditions empirically via the instrumented freeze first | QCM-on-EPP, item 2 |
| `quantum.classical_embedding` | classical causal models are the diagonal-`σ` special case — the point where this crate and `deep_causality_do_calculus` provably meet | Lorenz 2022, §4.1 |
| `quantum.cyclic_support` | cyclic QCM rides the native non-DAG hypergraph | Barrett–Lorenz–Oreshkov 2021 (2002.12157) |

**Faithful reification (from Lorenz & Barrett 2021, §3).** Sequential + tensor composition alone
cannot make all no-influence relations of a unitary simultaneously evident; causally faithful
decomposition requires **direct-sum** structure (their extended circuit diagrams). The required
**coproduct/direct-sum generator `⊕`** is a first-class requirement of this roadmap — **Stage 2b**
formalizes and implements it at the haft layer (ArrowChoice over `Either`, reified `Left`/`Right`/
`Choice`/`Fanin`, extended interpreters); this crate instantiates it as the Hilbert-space direct
sum. The *general* faithfulness claim remains Lorenz & Barrett's open hypothesis, so the crate
scopes its faithfulness claims to their proven classes (§8). **[planned — Stage 2b]**

Plus the **logical/physical bridge**: cloud-QPU adapters (real quantum sampling as a monadic
effect) in the **emergent / unverifiable** modality, kept type-distinct from the verifiable
simulated-CJ modality (`full-stack.md` §7.1). **[planned]**

**Verdict carriers at the quantum level (2026-07-10).** The `Verdict` trait (bounded lattice +
complement) admits a third algebra class here: the **projection lattice** of a Hilbert space
(Birkhoff–von Neumann quantum logic — `bottom = 0`, `top = I`, orthocomplement `I − P`, meet/join
on ranges), an **orthomodular** lattice that fails distributivity the way `Prob` fails excluded
middle. If this crate wants quantum propositions as first-class aggregation outputs, the instance
is a dedicated newtype over a commuting projection family, with an orthomodular law note —
Boolean → MV → orthomodular behind the one trait. Two guards: (1) general effects (`0 ≤ E ≤ I`)
form an **effect algebra** whose meet/join are *partial* (undefined for non-commuting pairs), so
no lawful blanket `Verdict` impl for a general tensor/operator type exists; (2) a **process
matrix / process operator** is not a verdict at all — it is state-channel data whose causal
content is its commuting factorization (checked at freeze), and verdicts are *extracted* from it
at the measurement boundary: generalized Born rule → `Prob`, propositions → the projection
lattice. Collection aggregation over quantum causaloids therefore aggregates extracted verdicts,
never the operators themselves. **[planned — scope guard in the Stage-3 spec]**

**Relativistic (level 2)** needs no crate: the metric is channel data applied coordinate-free
inside `f` (working: `event_horizon_probe`); the optional invariant→order scheduler and
frame-covariance freeze checks are a module over the core (`full-stack.md` §7). **[partially holds]**

## 8. What remains genuinely open

1. **`quantum.partial_trace_preservation`** — the single genuinely-quantum proof obligation;
   possibly conditional. **[open]**
2. **`core.causaloid.hardy_correspondence`** — the ⊗^Λ reconstruction; publication-grade, blocks
   neither crate. **[open — target]**
3. **Causally faithful quantum reification in general** — the `⊕` generator itself is planned work
   (Stage 2b); what stays open is Lorenz & Barrett's own hypothesis (every finite-dimensional
   unitary has a causally faithful extended circuit decomposition); scope the crate's claims to the
   proven (n≤3 / k≤3 and listed (4,4)) classes. **[open — upstream]**
4. **The #10 behaviour-preservation gate** for the Stage-4 engine rewrite — engineering discipline
   (characterization corpus), not mathematics. **[planned]**

Everything else is the established loop (Lean + witness + THEOREM_MAP + bare-`lean` + Bazel), run
per stage.

## 9. Dependency spine

```
Stage 1 (carrier stack)
   → Stage 2 (fixpoint + inversion + Λ-edges)
      ∥  Stage 2b (choice fragment ⊕ — haft-level, parallelizable from now)
      → Stage 3 (Verdict closure)            [level-1 carrier]
         → Stage 4 (graph algebra, ∇∘(Λ⊗Λ))  [gated on #10 corpus]
            → Stage 5 (catamorphism_unique)   [the keystone; arrow_fragment
                                               covers the ⊕-enlarged term]
               → Stage 6 (extensibility contract)
                  → deep_causality_do_calculus  ∥  deep_causality_quantum
                     (independent of each other; meet at quantum.classical_embedding;
                      quantum's faithful reification requires Stage 2b)
```

Assumption-tracker closure map: Stage 1 → #2-Q3, #11b (**landed 2026-07-09**) · Stage 2 → #9
(**landed 2026-07-10**; Hardy correspondence stays [open — target]) · Stage 2b → the `⊕` wiring
generator (**landed 2026-07-10**; consumed by the quantum crate) · Stage 3 → #5 (**landed
2026-07-10**) · Stage 4 → #2-Q1 + #1 applied + #10 via the corpus gate (**landed 2026-07-10**) ·
Stage 5 → #6 (scoped), #8, B2 (**landed 2026-07-10**) · already closed by prior work: #1, #3, #7,
and **#11a (DECIDED 2026-07-10: three forms, sealed trait — `traits/causable/sealed.rs`; ∇/⊕ are
wiring generators over the fixed `F`, not new forms — the closed-world premise Stage 5's
uniqueness argument needs)**. **Stages 2–5 all landed with change
`causaloid-formalization-stages-2-5` (2026-07-10). Remaining open in the tracker: #4
(generation/atom registry — out of scope) and the open targets of §8. Next: Stage 6 (the
extensibility contract) and the two crates (`deep_causality_do_calculus`,
`deep_causality_quantum`) as a dedicated follow-up change set.**

## 10. References

- Hardy, *Probability Theories with Dynamic Causal Structure: A New Framework for Quantum Gravity*,
  arXiv:gr-qc/0509120 — [`deep_causality/papers/causaloid.pdf`](../../../../deep_causality/papers/causaloid.pdf)
  (⊗^Λ Eq. 2 p. 4; Λ matrices p. 4; product unification §2 p. 3; `|v|/|u|` Eq. 3 p. 4).
- Lorenz, *Quantum causal models: the merits of the spirit of Reichenbach's principle…*, Synthese
  200:424 (2022) — `../../quantum/Quantum causal models-lorenz2022.pdf` (QCCP; process operator;
  quantum node; Def 3.3 quantum Markov — pairwise-commuting factorization; Def 3.4 QCM; causal
  relation via unitary influence §4.3; classical containment §4.1).
- Lorenz & Barrett, *Causal and compositional structure of unitary transformations*, Quantum 5, 511
  (2021), arXiv:2001.07774 —
  `../../quantum/Causal and compositional structure of unitary transformations-2001.07774v2.pdf`
  (Def 1 no-influence; Thm 1 unitary commuting factorization; Def 2 causal structure = parent sets;
  §3 sequential+tensor insufficiency; extended circuit diagrams / direct sums; the causal-
  faithfulness hypothesis).
- Barrett, Lorenz & Oreshkov, *Cyclic Quantum Causal Models*, arXiv:2002.12157 —
  `../../quantum/Cyclic Quantum Causal Models-2002.12157v3.pdf`.
- Pearl, *Introduction to Do-Calculus* and *The Do-Calculus Revisited* — `../../causal-do/`.
- [`../quantum/full-stack.md`](../../quantum/full-stack.md) · [`../quantum/QCM-on-EPP.md`](../../quantum/QCM-on-EPP.md)
  · [`../quantum/quantum-epp.md`](../../quantum/quantum-epp.md) · [`Causaloid-structure.md`](Causaloid-structure.md)
  · [`algebraic-causaloid-assumptions.md`](algebraic-causaloid-assumptions.md).
