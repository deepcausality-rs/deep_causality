/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Core — GraphJoin: the labeled reconvergence fan-in of the graph-reasoning engine.

Rust source: `deep_causality/src/traits/causable_graph/graph_reasoning/mod.rs`
(`evaluate_subgraph_from_cause`), `deep_causality/src/types/causal_types/parent_effects/`
(`ParentEffects`), and the join surface on `Causaloid` (`join_fn` / `context_join_fn`).

Layering: the engine folds a frozen acyclic graph. At a reconvergence node `n` the engine
delivers the effects of the parents that fired, keyed by parent node index, to `n`'s declared
join mechanism `f n`, producing the single effect `n` consumes. This file models that as the
acyclic *labeled equation system* `σ(n) = f_n(σ|Pa(n))` and proves:

  * `unique_valuation`   — the system has exactly ONE solution, by well-founded induction on a
                           topological rank, with NO algebraic hypothesis on the mechanisms
                           (asymmetric mechanisms are admissible). Determinism is structural.
  * `schedule_invariance`— any two schedules that each produce a solution agree (a corollary:
                           the evaluation order does not change the result). Command-free scope.
  * `unionMap_comm` /
    `unionMap_assoc`     — the disjoint-key parent-map union is commutative and associative *by
                           construction* — the structural monoid the retired "declared
                           commutative-combine" design tried to impose as a user axiom.
  * `classical_copy`     — fan-out delivers the SAME value `σ(n)` along every out-wire (the copy
                           law of the *classical* interpreter; the quantum instantiation replaces
                           copy with commuting access, so this is scoped, not a substrate law).

Notes on the engine that this model reflects:
  * Dead-path pruning is at the WIRE level: a wire from a non-descendant of the start is never
    counted, so every reachable non-start node has a firing parent — there is no "node resolves
    Inactive" outcome to model (it is an unreachable engine guard).
  * `RelayTo` is sequential composition of rounds; each round is one instance of this acyclic
    system, so the per-round theorems compose. That composition is definitional and not re-proved.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witnesses: `deep_causality/tests/formalization_lean/graph_join_tests.rs`.
-/

namespace DeepCausalityFormal.Core.GraphJoin

variable {V : Type}

/-- A finite acyclic **labeled** causal system over `Nat`-indexed nodes.

    * `parents n` — the labeled parent set of `n` (order irrelevant; identity carried by the index).
    * `mech n`    — `n`'s mechanism: a function of the whole valuation that depends only on the
                    values at `parents n` (`mech_local`). No commutativity / symmetry is assumed.
    * `rank`      — a topological rank witnessing acyclicity (`acyclic`): every parent ranks below. -/
structure LabeledDag (V : Type) where
  parents : Nat → List Nat
  mech : Nat → (Nat → V) → V
  rank : Nat → Nat
  acyclic : ∀ n, ∀ p, p ∈ parents n → rank p < rank n
  mech_local : ∀ n (σ τ : Nat → V), (∀ p, p ∈ parents n → σ p = τ p) → mech n σ = mech n τ

/-- A valuation `σ` **solves** the system iff every node equals its mechanism applied to `σ`
    (the fixpoint `σ(n) = f_n(σ|Pa(n))`). The engine's completed round produces such a `σ`. -/
def Solves (G : LabeledDag V) (σ : Nat → V) : Prop :=
  ∀ n, σ n = G.mech n σ

/-- **Unique valuation.** Any two solutions of the acyclic labeled system agree everywhere.
    Proved by well-founded induction on the topological rank; the mechanisms are arbitrary
    (no algebraic hypothesis), so determinism comes from the labeled structure alone.

    THEOREM_MAP: `core.graph_join.unique_valuation` -/
theorem unique_valuation (G : LabeledDag V) (σ τ : Nat → V)
    (hσ : Solves G σ) (hτ : Solves G τ) : ∀ n, σ n = τ n := by
  -- `key k`: all nodes of rank `< k` agree. Induct on `k` (plain `Nat.rec`, bare-lean).
  have key : ∀ k, ∀ m, G.rank m < k → σ m = τ m := by
    intro k
    induction k with
    | zero => intro m h; omega
    | succ k ih =>
      intro m hm
      rw [hσ m, hτ m]
      apply G.mech_local
      intro p hp
      have hlt : G.rank p < G.rank m := G.acyclic m p hp
      exact ih p (by omega)
  intro n
  exact key (G.rank n + 1) n (by omega)

/-- **Schedule invariance** (command-free). Two schedules that each compute a solution of the
    round's system agree — the choice of topological linearization does not change the result.
    This is exactly `unique_valuation`, framed for the "two schedules" reading; the engine's
    canonical ascending-index schedule is one such schedule.

    THEOREM_MAP: `core.graph_join.schedule_invariance` -/
theorem schedule_invariance (G : LabeledDag V) (σ τ : Nat → V)
    (hσ : Solves G σ) (hτ : Solves G τ) (n : Nat) : σ n = τ n :=
  unique_valuation G σ τ hσ hτ n

/-- The fired-parent map as a partial valuation `Nat → Option V` (a key present iff that parent
    fired). Disjoint union prefers the left operand, falling back to the right. -/
def unionMap (m₁ m₂ : Nat → Option V) : Nat → Option V :=
  fun k => (m₁ k).orElse (fun _ => m₂ k)

/-- **Disjoint-key union is commutative.** When the two parent-maps have disjoint support (a fan-in
    keys each fired parent exactly once), merging them in either order yields the same map — the
    order in which parents fired is irrelevant. This is the structural monoid the engine relies on,
    established by construction rather than assumed of the mechanism.

    THEOREM_MAP: `core.graph_join.union_comm` -/
theorem unionMap_comm (m₁ m₂ : Nat → Option V)
    (hd : ∀ k, m₁ k = none ∨ m₂ k = none) : unionMap m₁ m₂ = unionMap m₂ m₁ := by
  funext k
  simp only [unionMap]
  rcases hd k with h | h
  · rw [h]; cases m₂ k <;> rfl
  · rw [h]; cases m₁ k <;> rfl

/-- **Disjoint-key union is associative.** Grouping three pairwise-disjoint parent-maps either way
    yields the same map.

    THEOREM_MAP: `core.graph_join.union_assoc` -/
theorem unionMap_assoc (m₁ m₂ m₃ : Nat → Option V) :
    unionMap (unionMap m₁ m₂) m₃ = unionMap m₁ (unionMap m₂ m₃) := by
  funext k
  simp only [unionMap]
  cases m₁ k <;> rfl

/-- The value delivered along an out-wire of `n` to a child: the classical interpreter copies the
    node's resolved value `σ n` to every child. -/
def deliver (σ : Nat → V) (n : Nat) (_child : Nat) : V := σ n

/-- **Classical copy law.** Every out-wire of `n` delivers the same value `σ n`; fan-out is a copy,
    not a recomputation. Stated for the classical interpreter — the quantum instantiation replaces
    this copy with commuting access to a shared output (no-cloning), so it is a scoped law, not a
    substrate law.

    THEOREM_MAP: `core.graph_join.classical_copy` -/
theorem classical_copy (σ : Nat → V) (n c₁ c₂ : Nat) :
    deliver σ n c₁ = deliver σ n c₂ := rfl

/-- **`LinearJoin` surgery locality** (the kernel-level shadow of do-surgery / "opening a
    mechanism"). Writing the linear combine as `bias + termₚ + rest` — where `termₚ = weights[p]·v_p`
    is parent `p`'s contribution and `rest` is the sum of the others — cutting `p`'s wire removes
    exactly `termₚ`: the full result minus the cut result equals `termₚ`, with no redefinition of the
    kernel. Stated over `Int` (the terms are opaque atoms), so it is the pure cancellation fact the
    `linear_join` Rust witness checks concretely on `f64`.

    THEOREM_MAP: `core.graph_join.linear_surgery_locality` -/
theorem linear_surgery_locality (bias termP rest : Int) :
    (bias + termP + rest) - (bias + rest) = termP := by omega

end DeepCausalityFormal.Core.GraphJoin
