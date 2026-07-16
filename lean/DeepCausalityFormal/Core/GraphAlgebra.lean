/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Core — the graph algebra: the topological fold with `∇ ∘ (Λ₁ ⊗ Λ₂)` at reconvergent joins is
invariant under every schedule consistent with the causal order (Stage 4 of the causaloid
formalization roadmap; assumption #2 Q1).

Textbook definition. A dataflow network over a DAG computes, per node, a function of its parents'
results only; its semantics is the **denotational value** defined by recursion on the topological
rank, and any execution order that respects the dependencies computes exactly that denotation —
the determinacy of dataflow (G. Kahn, "The Semantics of a Simple Language for Parallel
Programming," IFIP 1974). The join at a reconvergent node fuses the parents' contributions with a
**commutative, associative** `∇` after identity-keyed edge transforms `Λ` (Hardy's connection
data; the crate's `join = ∇ ∘ (Λ₁ ⊗ Λ₂)`), so the fused input is a function of the parent **set**
— arrival order cannot enter (the symmetric-monoidal merge, `haft.sym_monoidal.*`;
`∇ = Verdict::join` is the join-semilattice reduct of the Verdict carrier — a commutative,
associative operation per `algebra.verdict.lattice_laws` / `Core/VerdictClosure.lean`).

This file proves, in that model:
  * `fuse_perm` — the ∇-fuse of the contributions is invariant under permutation (bag semantics
    at the join), from commutativity + associativity of `∇`.
  * `exec_computes_val` — **the schedule theorem**: executing any schedule in which every node
    finds its parents already resolved (a consistent schedule) assigns every scheduled node its
    denotational value `val`, independently of the schedule.
  * `schedule_invariant` — corollary: any two consistent schedules agree on every node they both
    process. This is `core.causaloid.graph_fold_order_invariant`.

Rust source: `deep_causality/src/traits/causable_graph/graph_reasoning/mod.rs`
(`evaluate_subgraph_from_cause_with_lambda_edges` — the Kahn ready-set engine whose reconvergence
arm fuses Λ-transformed fired-parent values with `Verdict::join`; the ready-set's ascending-index
pop order is ONE consistent schedule among the many this theorem quantifies over).

DEVIATION NOTES.
  1. The DAG is a `parents` function plus a rank certificate (`rank p < rank n` for every parent)
     — the well-foundedness the frozen acyclic graph carries (Rust: `freeze_dag` /
     `freeze_verified`); no graph library needed. The denotation uses the Stage-1 fuel device
     (structural recursion on fuel + a stability lemma) instead of well-founded recursion.
  2. `elem` consumes the fused value only — state never merges at a join (single-writer, checked
     at freeze by `freeze_verified`; the classical engine's state is `()`), and the log channel
     is a multiset at the join (ascending-index representative in Rust) — both outside this
     value-channel model.
  3. A node with no contribution consumes the seed — matching the engine, where only the start
     node is seeded; `getD seed` for an unresolved parent is likewise irrelevant under
     consistency (every parent is resolved first).
  4. Commutativity + associativity of `∇` are hypotheses; the Rust `∇ = Verdict::join` satisfies
     them per carrier (Boolean proved; MV min/max witnessed in Rust).

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witnesses: `deep_causality/tests/formalization_lean/graph_algebra_tests.rs`.
-/

namespace DeepCausalityFormal.Core.GraphAlgebra

variable {V : Type}

-- ------------------------------------------------------------------
-- The ∇-fuse is a bag operation (`fuse_perm`).
-- ------------------------------------------------------------------

/-- One fuse step over an optional accumulator: the first contribution seeds, later ones merge
    with `∇`. -/
def fuseStep (nabla : V → V → V) (acc : Option V) (v : V) : Option V :=
  match acc with
  | none   => some v
  | some a => some (nabla a v)

/-- Fuse a list of contributions: `none` when there is none (absence propagates — the engine's
    all-absent case), otherwise the ∇-fold. -/
def fuse (nabla : V → V → V) (l : List V) : Option V :=
  l.foldl (fuseStep nabla) none

/-- Local permutation relation (swap-generated), as in `Core/Causaloid.lean`. -/
inductive Perm : List V → List V → Prop where
  | nil   : Perm [] []
  | cons  (v : V) {l l' : List V} : Perm l l' → Perm (v :: l) (v :: l')
  | swap  (a b : V) (l : List V) : Perm (a :: b :: l) (b :: a :: l)
  | trans {l₁ l₂ l₃ : List V} : Perm l₁ l₂ → Perm l₂ l₃ → Perm l₁ l₃

/-- Two consecutive fuse steps exchange, given `∇` commutative + associative. -/
theorem fuseStep_exchange (nabla : V → V → V)
    (hcomm : ∀ a b, nabla a b = nabla b a)
    (hassoc : ∀ a b c, nabla (nabla a b) c = nabla a (nabla b c))
    (acc : Option V) (a b : V) :
    fuseStep nabla (fuseStep nabla acc a) b = fuseStep nabla (fuseStep nabla acc b) a := by
  cases acc with
  | none =>
      show some (nabla a b) = some (nabla b a)
      rw [hcomm]
  | some c =>
      show some (nabla (nabla c a) b) = some (nabla (nabla c b) a)
      rw [hassoc, hcomm a b, ← hassoc]

/-- The fuse fold is permutation-invariant from any accumulator. -/
theorem foldl_fuse_perm (nabla : V → V → V)
    (hcomm : ∀ a b, nabla a b = nabla b a)
    (hassoc : ∀ a b c, nabla (nabla a b) c = nabla a (nabla b c))
    {l l' : List V} (h : Perm l l') :
    ∀ acc : Option V, l.foldl (fuseStep nabla) acc = l'.foldl (fuseStep nabla) acc := by
  induction h with
  | nil => intro _; rfl
  | cons v _ ih =>
      intro acc
      show _ = _
      exact ih (fuseStep nabla acc v)
  | swap a b l =>
      intro acc
      show l.foldl (fuseStep nabla) (fuseStep nabla (fuseStep nabla acc a) b)
          = l.foldl (fuseStep nabla) (fuseStep nabla (fuseStep nabla acc b) a)
      rw [fuseStep_exchange nabla hcomm hassoc]
  | trans _ _ ih₁ ih₂ => intro acc; exact (ih₁ acc).trans (ih₂ acc)

/-- **The join is a bag**: the ∇-fuse of the parent contributions is invariant under permutation.
    Which Λ applies to which contribution is keyed by edge identity *before* the fuse, so nothing
    order-dependent survives at a reconvergent join.

    THEOREM_MAP: `core.causaloid.graph_fold_order_invariant` -/
theorem fuse_perm (nabla : V → V → V)
    (hcomm : ∀ a b, nabla a b = nabla b a)
    (hassoc : ∀ a b c, nabla (nabla a b) c = nabla a (nabla b c))
    {l l' : List V} (h : Perm l l') : fuse nabla l = fuse nabla l' :=
  foldl_fuse_perm nabla hcomm hassoc h none

-- ------------------------------------------------------------------
-- The dataflow model: a ranked DAG, its denotational value, schedule execution.
-- ------------------------------------------------------------------

/-- A DAG presented by parent lists plus a rank certificate — the well-foundedness the frozen
    acyclic graph carries (deviation note 1). Nodes are `Nat` indices. -/
structure Dag where
  parents : Nat → List Nat
  rank : Nat → Nat
  wf : ∀ n p, p ∈ parents n → rank p < rank n

variable (G : Dag) (elem : Nat → V → V) (lam : Nat → Nat → V → V)
  (nabla : V → V → V) (seed : V)

/-- One node's fused input from a contribution assignment: Λ applied per identity-keyed edge,
    then the ∇-fuse; no contribution ⇒ the seed. -/
def fusedInput (contrib : Nat → V) (n : Nat) : V :=
  match fuse nabla ((G.parents n).map (fun p => lam p n (contrib p))) with
  | none   => seed
  | some v => v

/-- The fuel-indexed denotation (the Stage-1 device): with fuel to spare, a node's value is its
    element map applied to the ∇-fuse of its parents' Λ-transformed values. -/
def valFuel : Nat → Nat → V
  | 0,        n => elem n seed
  | fuel + 1, n => elem n (fusedInput G lam nabla seed (fun p => valFuel fuel p) n)

/-- The **denotational value** — schedule-free by construction. -/
def val (n : Nat) : V :=
  valFuel G elem lam nabla seed (G.rank n + 1) n

/-- Map congruence on a list (membership-wise). -/
theorem map_congr {α β : Type} {f g : α → β} :
    (l : List α) → (∀ a, a ∈ l → f a = g a) → l.map f = l.map g
  | [], _ => rfl
  | a :: l, h => by
      show f a :: l.map f = g a :: l.map g
      rw [h a (List.mem_cons_self), map_congr l (fun b hb => h b (List.mem_cons_of_mem a hb))]

/-- Fuel stability: any two sufficient fuels compute the same value — the denotation is
    well-defined (rank-founded, μ not ν). -/
theorem valFuel_stable :
    ∀ (f₁ : Nat), ∀ (f₂ n : Nat), G.rank n < f₁ → G.rank n < f₂ →
      valFuel G elem lam nabla seed f₁ n = valFuel G elem lam nabla seed f₂ n := by
  intro f₁
  induction f₁ with
  | zero => intro _ n h₁ _; exact absurd h₁ (Nat.not_lt_zero _)
  | succ a ih =>
      intro f₂ n _h₁ h₂
      cases f₂ with
      | zero => exact absurd h₂ (Nat.not_lt_zero _)
      | succ b =>
          show elem n (fusedInput G lam nabla seed (fun p => valFuel G elem lam nabla seed a p) n)
              = elem n (fusedInput G lam nabla seed (fun p => valFuel G elem lam nabla seed b p) n)
          have hmap :
              (G.parents n).map (fun p => lam p n (valFuel G elem lam nabla seed a p))
                = (G.parents n).map (fun p => lam p n (valFuel G elem lam nabla seed b p)) := by
            apply map_congr
            intro p hp
            have hpa : G.rank p < a := Nat.lt_of_lt_of_le (G.wf n p hp) (Nat.lt_succ_iff.mp _h₁)
            have hpb : G.rank p < b := Nat.lt_of_lt_of_le (G.wf n p hp) (Nat.lt_succ_iff.mp h₂)
            rw [ih b p hpa hpb]
          show elem n _ = elem n _
          unfold fusedInput
          rw [hmap]

/-- Unfolding: `val n = elem n (∇-fuse of Λ(p,n)(val p) over the parents)` — the defining
    equation, with the recursion resolved through the stable fuel. -/
theorem val_eq (n : Nat) :
    val G elem lam nabla seed n
      = elem n (fusedInput G lam nabla seed (val G elem lam nabla seed) n) := by
  show elem n (fusedInput G lam nabla seed
        (fun p => valFuel G elem lam nabla seed (G.rank n) p) n)
      = elem n (fusedInput G lam nabla seed (val G elem lam nabla seed) n)
  have hmap :
      (G.parents n).map (fun p => lam p n (valFuel G elem lam nabla seed (G.rank n) p))
        = (G.parents n).map (fun p => lam p n (val G elem lam nabla seed p)) := by
    apply map_congr
    intro p hp
    rw [valFuel_stable G elem lam nabla seed (G.rank n) (G.rank p + 1) p (G.wf n p hp)
        (Nat.lt_succ_self _)]
    rfl
  unfold fusedInput
  rw [hmap]

-- ------------------------------------------------------------------
-- Schedule execution over keyed wire slots.
-- ------------------------------------------------------------------

/-- Execute one node: read the parents' stored results (keyed wires), fuse, apply the element
    map, store the result. Mirrors the engine's `fired[child][parent]` wire slots. -/
def stepStore (store : Nat → Option V) (n : Nat) : Nat → Option V :=
  fun m =>
    if m = n then
      some (elem n (fusedInput G lam nabla seed (fun p => (store p).getD seed) n))
    else store m

/-- Execute a schedule left to right. -/
def exec (store : Nat → Option V) : List Nat → (Nat → Option V)
  | []          => store
  | n :: sched  => exec (stepStore G elem lam nabla seed store n) sched

/-- A schedule is **consistent** (with the causal order) when every node, at its turn, finds all
    its parents already resolved. -/
def Consistent (store : Nat → Option V) : List Nat → Prop
  | []          => True
  | n :: sched  =>
      (∀ p, p ∈ G.parents n → (store p).isSome)
        ∧ Consistent (stepStore G elem lam nabla seed store n) sched

/-- The store invariant: everything stored is the denotational value. -/
def Agrees (store : Nat → Option V) : Prop :=
  ∀ m v, store m = some v → v = val G elem lam nabla seed m

theorem agrees_step {store : Nat → Option V} (ha : Agrees G elem lam nabla seed store) (n : Nat)
    (hready : ∀ p, p ∈ G.parents n → (store p).isSome) :
    Agrees G elem lam nabla seed (stepStore G elem lam nabla seed store n) := by
  intro m v hv
  by_cases hm : m = n
  · subst hm
    unfold stepStore at hv
    rw [if_pos rfl] at hv
    injection hv with hv
    -- the fused input over the store equals the fused input over `val` (parents resolved+agree)
    have hmap :
        (G.parents m).map (fun p => lam p m ((store p).getD seed))
          = (G.parents m).map (fun p => lam p m (val G elem lam nabla seed p)) := by
      apply map_congr
      intro p hp
      cases hsp : store p with
      | none => exact absurd (hready p hp) (by rw [hsp]; intro h; exact (Bool.noConfusion h))
      | some w =>
          have : w = val G elem lam nabla seed p := ha p w hsp
          rw [Option.getD, this]
    rw [← hv, val_eq]
    show elem m _ = elem m _
    unfold fusedInput
    rw [hmap]
  · unfold stepStore at hv
    rw [if_neg hm] at hv
    exact ha m v hv

/-- Execution preserves the invariant on every consistent schedule. -/
theorem exec_agrees :
    ∀ (sched : List Nat) (store : Nat → Option V),
      Agrees G elem lam nabla seed store → Consistent G elem lam nabla seed store sched →
      Agrees G elem lam nabla seed (exec G elem lam nabla seed store sched)
  | [], _, ha, _ => ha
  | n :: sched, store, ha, hc =>
      exec_agrees sched (stepStore G elem lam nabla seed store n)
        (agrees_step G elem lam nabla seed ha n hc.1) hc.2

/-- Executing one node resolves it. -/
theorem stepStore_self (store : Nat → Option V) (n : Nat) :
    ((stepStore G elem lam nabla seed store n) n).isSome = true := by
  simp [stepStore]

/-- Execution never unsets a wire. -/
theorem exec_preserves_some :
    ∀ (sched : List Nat) (store : Nat → Option V) (m : Nat),
      (store m).isSome = true → ((exec G elem lam nabla seed store sched) m).isSome = true
  | [], _, _, h => h
  | n :: sched, store, m, h => by
      apply exec_preserves_some sched
      simp only [stepStore]
      by_cases hm : m = n
      · rw [if_pos hm]; rfl
      · rw [if_neg hm]; exact h

/-- Every scheduled node ends up resolved. -/
theorem exec_assigns :
    ∀ (sched : List Nat) (store : Nat → Option V) (n : Nat),
      n ∈ sched → ((exec G elem lam nabla seed store sched) n).isSome = true := by
  intro sched
  induction sched with
  | nil => intro _ n h; cases h
  | cons k sched ih =>
      intro store n h
      cases h with
      | head =>
          exact exec_preserves_some G elem lam nabla seed sched
            (stepStore G elem lam nabla seed store k) k
            (stepStore_self G elem lam nabla seed store k)
      | tail _ h => exact ih _ _ h

/-- **The schedule theorem**: on every consistent schedule, execution assigns every scheduled
    node its denotational value — the fold's result is a property of the graph, not of the
    schedule. The commutative ∇-fuse (`fuse_perm`) is what makes the per-node input a function of
    the parent set; the wire keying is what makes each Λ land on its own edge.

    THEOREM_MAP: `core.causaloid.graph_fold_order_invariant` -/
theorem exec_computes_val (sched : List Nat) (store : Nat → Option V)
    (ha : Agrees G elem lam nabla seed store)
    (hc : Consistent G elem lam nabla seed store sched)
    (n : Nat) (hn : n ∈ sched) :
    (exec G elem lam nabla seed store sched) n = some (val G elem lam nabla seed n) := by
  have hsome := exec_assigns G elem lam nabla seed sched store n hn
  cases hv : (exec G elem lam nabla seed store sched) n with
  | none => rw [hv] at hsome; exact Bool.noConfusion hsome
  | some v =>
      have := exec_agrees G elem lam nabla seed sched store ha hc n v hv
      rw [this]

/-- **Schedule invariance** (`core.causaloid.graph_fold_order_invariant`, the corollary the spec
    states): two consistent schedules from the empty store agree on every node they both process.

    THEOREM_MAP: `core.causaloid.graph_fold_order_invariant` -/
theorem schedule_invariant (s₁ s₂ : List Nat)
    (h₁ : Consistent G elem lam nabla seed (fun _ => none) s₁)
    (h₂ : Consistent G elem lam nabla seed (fun _ => none) s₂)
    (n : Nat) (hn₁ : n ∈ s₁) (hn₂ : n ∈ s₂) :
    (exec G elem lam nabla seed (fun _ => none) s₁) n
      = (exec G elem lam nabla seed (fun _ => none) s₂) n := by
  have hempty : Agrees G elem lam nabla seed (fun _ => none) := by
    intro m v h; nomatch h
  rw [exec_computes_val G elem lam nabla seed s₁ _ hempty h₁ n hn₁,
      exec_computes_val G elem lam nabla seed s₂ _ hempty h₂ n hn₂]

end DeepCausalityFormal.Core.GraphAlgebra
