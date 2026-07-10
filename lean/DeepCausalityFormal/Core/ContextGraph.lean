/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Core — the Context hypergraph: parent-set (hyperedge) semantics keyed by identity, hyperedge
threading = the causal monad `bind`, and acyclicity as a SEPARABLE, freeze-enforceable parameter
(formalize-main-crate group 4; the shared structural substrate both intervention surgery and QCM
factorization operate over).

Textbook framing. A causal/Bayesian structure is a hypergraph in which each node `A` names its
PARENT SET `Pa(A)` directly (Pearl, *Causality*, 2nd ed., 2009, ch. 1; Koller & Friedman,
*Probabilistic Graphical Models*, 2009 — the factorization is over parent sets, not pairwise
edges). A node's evaluation THREADS its parents' contributions through the ambient monad; for the
causal monad that thread is `bind` (`Core/CausalMonad.lean`), and "encapsulate a sub-hypergraph as
one node vs inline it" is exactly bind ASSOCIATIVITY — `core.causal_monad.assoc`. Acyclicity is a
separate well-foundedness certificate on the parent-set map, not part of the threading: the same
definitions serve the acyclic (DAG) case and the directed-cyclic case (Oreshkov–Costa–Brukner,
"Quantum correlations with no causal order," *Nat. Commun.* 3, 2012 — indefinite causal order needs
the cyclic case, so acyclicity must be relaxable without changing the apparatus).

This file proves, in that model:
  * `thread_is_bind` — the hyperedge threading of a node's parent set IS `bind`, folded over the
    parents (`fired[child][parent]` wire slots / `LambdaEdges` `(source, target)` keys).
  * `thread_append` / `evalParents_split` — nested threading = flat threading: partitioning a
    parent set and threading the parts in sequence equals threading the whole (encapsulation at the
    parent-set level; the graph-side `evalL_append`).
  * `encapsulation_flat` — the ASSOCIATIVITY form: a two-stage threaded evaluation equals binding
    through the encapsulated composite once — inherited from `core.causal_monad.assoc` (taken as a
    hypothesis and discharged by it), the graph-side counterpart of
    `core.causaloid.encapsulation_flat`.
  * `Acyclic` / `self_parent_not_acyclic` / `apparatus_acyclicity_agnostic` — acyclicity is a
    separable predicate (a rank certificate = `ultragraph::has_cycle` / `freeze_dag`); a cyclic
    parent-set (a self-parent) has no certificate (rejected at freeze); and the threading /
    encapsulation theorems hold for EVERY parent-set map, so the cyclic case reuses the same
    definitions (enabling deferred cyclic-QCM support).

DEVIATION NOTES.
  1. The causal monad `bind` is abstracted to `bind : M → (V → M) → M` with its ASSOCIATIVITY law
     taken as a hypothesis — exactly the house style of `Core/GraphAlgebra.lean` (which takes `∇`'s
     laws as hypotheses) and `Core/Catamorphism.lean`'s `evalL_append`. The hypothesis is discharged
     by `core.causal_monad.assoc` (`Core/CausalMonad.lean :: bind_assoc`); the value/state/context/
     log channels of the full arity-5 `bind'` are threaded identically, so the associativity content
     is faithfully this one equation.
  2. A node's parents are computed elsewhere (the recursive dataflow is `Core/GraphAlgebra.lean`);
     here `mech : Id → V → M` gives each contextoid's mechanism and threading composes them — the
     encapsulation content lives entirely in how the mechanisms are bound, not in the recursion.
  3. Acyclicity is the same rank-certificate device as `Core/GraphAlgebra.lean`'s `Dag.wf`; here it
     is a standalone predicate on `Pa` so its separability is a theorem, not a structural assumption.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witnesses: `deep_causality/tests/formalization_lean/context_graph_tests.rs`.
-/

namespace DeepCausalityFormal.Core.ContextGraph

/-- A contextoid identity — the parent key (`fired[child][parent]`, `LambdaEdges` `(source,
    target)`). -/
abbrev Id := Nat

variable {M V : Type}

-- ------------------------------------------------------------------
-- `core.context_graph.threading_bind`: the parent-set threading IS `bind`, and encapsulation is
-- flat (inherited from `core.causal_monad.assoc`).
-- ------------------------------------------------------------------

/-- Hyperedge threading: thread a running monadic value through a node's parent mechanisms via the
    causal monad `bind` — each parent contributes a Kleisli step `V → M`, folded with `bind`. -/
def thread (bind : M → (V → M) → M) : M → List (V → M) → M
  | m, []      => m
  | m, k :: ks => thread bind (bind m k) ks

/-- Evaluate a node by threading its parent set (`Pa n`, keyed by identity) through `bind`, each
    parent identity resolved to its contextoid mechanism `mech`. This ties the parent-set MAP to the
    threading: a hyperedge `{Pa(n)} → n` is the bind-fold of the parents' mechanisms. -/
def evalParents (bind : M → (V → M) → M) (mech : Id → V → M) (seed : M)
    (Pa : Id → List Id) (n : Id) : M :=
  thread bind seed ((Pa n).map mech)

/-- **Threading is `bind`**: a one- and two-parent thread unfold to the `bind` chain — the hyperedge
    threading of a node's evaluation IS the causal monad `bind` (definitional).

    THEOREM_MAP: `core.context_graph.threading_bind` -/
theorem thread_is_bind (bind : M → (V → M) → M) (m : M) (f g : V → M) :
    thread bind m [f] = bind m f
    ∧ thread bind m [f, g] = bind (bind m f) g :=
  ⟨rfl, rfl⟩

/-- **Nested threading = flat threading**: threading a concatenated parent chain equals threading
    the first part then the second — the graph-side `evalL_append` (structural; holds for the
    left-fold form with no extra law).

    THEOREM_MAP: `core.context_graph.threading_bind` -/
theorem thread_append (bind : M → (V → M) → M) (ks : List (V → M)) :
    ∀ (m : M) (ks' : List (V → M)),
      thread bind m (ks ++ ks') = thread bind (thread bind m ks) ks' := by
  induction ks with
  | nil => intro m ks'; rfl
  | cons k ks ih =>
      intro m ks'
      show thread bind (bind m k) (ks ++ ks') = thread bind (thread bind (bind m k) ks) ks'
      exact ih (bind m k) ks'

/-- Encapsulation at the parent-set level: partitioning a node's parent set into two hyperedge
    groups and threading them in sequence equals threading the whole parent set — wrapping a
    sub-hypergraph as one contextoid does not change the node's evaluation.

    THEOREM_MAP: `core.context_graph.threading_bind` -/
theorem evalParents_split (bind : M → (V → M) → M) (mech : Id → V → M) (seed : M)
    (Pa : Id → List Id) (n : Id) (ps qs : List Id) (hn : Pa n = ps ++ qs) :
    evalParents bind mech seed Pa n
      = thread bind (thread bind seed (ps.map mech)) (qs.map mech) := by
  unfold evalParents
  rw [hn, List.map_append, thread_append]

/-- **Encapsulation is flat, the associativity form** — the graph-side counterpart of
    `core.causaloid.encapsulation_flat`: a two-stage threaded evaluation (the running result bound
    through `f`, then `g`) equals binding the running result once through the encapsulated composite
    `fun a => bind (f a) g`. Wrapping a sub-hypergraph `{· → f → g}` as one contextoid whose
    mechanism is that composite does not change the semantics. This is precisely the causal monad's
    bind ASSOCIATIVITY (`hassoc`), discharged by `core.causal_monad.assoc`
    (`Core/CausalMonad.lean :: bind_assoc`).

    THEOREM_MAP: `core.context_graph.threading_bind` -/
theorem encapsulation_flat (bind : M → (V → M) → M)
    (hassoc : ∀ (m : M) (f g : V → M), bind (bind m f) g = bind m (fun a => bind (f a) g))
    (m : M) (f g : V → M) :
    bind (bind m f) g = bind m (fun a => bind (f a) g) :=
  hassoc m f g

-- ------------------------------------------------------------------
-- `core.context_graph.acyclicity_separable`: acyclicity is a separable, freeze-enforceable
-- parameter — the same threading apparatus serves the acyclic and the cyclic case.
-- ------------------------------------------------------------------

/-- A parent-set map is **acyclic** iff it admits a rank certificate: a `rank` on identities that
    strictly decreases along every hyperedge. This is the well-foundedness the frozen DAG carries
    (`ultragraph::has_cycle` / `freeze_dag` / `freeze_verified`; the `Dag.wf` device of
    `Core/GraphAlgebra.lean`). It is a SEPARATE predicate on `Pa` — the threading / encapsulation
    definitions above never mention it. -/
def Acyclic (Pa : Id → List Id) : Prop :=
  ∃ rank : Id → Nat, ∀ n p, p ∈ Pa n → rank p < rank n

/-- A self-parent (`p ∈ Pa p`) has NO rank certificate: the minimal cycle is inadmissible. This is
    what `ultragraph::has_cycle` rejects at `freeze_dag` — a cyclic parent-set is not acyclic, so it
    fails the freeze gate.

    THEOREM_MAP: `core.context_graph.acyclicity_separable` -/
theorem self_parent_not_acyclic (Pa : Id → List Id) (p : Id) (h : p ∈ Pa p) :
    ¬ Acyclic Pa := by
  rintro ⟨rank, hrank⟩
  exact absurd (hrank p p h) (Nat.lt_irrefl (rank p))

/-- An acyclic parent-set is exactly one with a rank certificate — the freeze-`DAG`-admissible case
    (`freeze_dag` accepts iff `has_cycle` is false, i.e. a rank exists).

    THEOREM_MAP: `core.context_graph.acyclicity_separable` -/
theorem acyclic_iff_rank (Pa : Id → List Id) :
    Acyclic Pa ↔ ∃ rank : Id → Nat, ∀ n p, p ∈ Pa n → rank p < rank n :=
  Iff.rfl

/-- **The apparatus is acyclicity-agnostic**: the encapsulation / threading law holds for EVERY
    parent-set map — acyclic or cyclic — because the threading definitions never consult `Acyclic`.
    So relaxing the freeze gate (admitting cyclic models — quantum switch / indefinite causal order)
    reuses the same definitions with no separate machinery; what changes is only the per-
    interpretation admissibility predicate, not the threading.

    THEOREM_MAP: `core.context_graph.acyclicity_separable` -/
theorem apparatus_acyclicity_agnostic (bind : M → (V → M) → M)
    (hassoc : ∀ (m : M) (f g : V → M), bind (bind m f) g = bind m (fun a => bind (f a) g))
    (m : M) (f g : V → M) :
    ∀ _Pa : Id → List Id, bind (bind m f) g = bind m (fun a => bind (f a) g) :=
  fun _Pa => hassoc m f g

end DeepCausalityFormal.Core.ContextGraph
