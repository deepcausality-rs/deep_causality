/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Core — the Verdict closure: `All`/`Any`/`None`/`Some(k)` are closed operations in the Verdict
algebra, so a collection of causaloids is again a causaloid (Stage 3 of the causaloid
formalization roadmap; assumption #5 made rigorous).

Textbook definition. A **bounded lattice with complement** (the Verdict algebra: `bottom`, `top`,
`meet`, `join`, `complement`) is closed under its operations by definition; an aggregation of a
finite bag of verdicts by a fold of `meet` (All) or `join` (Any) therefore lands in the carrier,
`None` is `Any` post-composed with `complement`, and `Some(k)` is a counting threshold decided
into `top`/`bottom` (Davey & Priestley, *Introduction to Lattices and Order*, 2nd ed., ch. 4;
Birkhoff, *Lattice Theory*). Instantiated at the fixpoint `Causaloid ≅ μX.F(X)`
(`Core/Causaloid.lean`), closure gives `Coll : Causaloid → Causaloid` — the `Coll` summand's
output inhabits the same carrier the `Atom` summand requires.

Carrier classes behind the one trait (`core.verdict.carriers`):
  * `bool` — a **Boolean algebra** (distributive; proved concretely here and in
    `Algebra/Verdict.lean`, `algebra.verdict.{lattice_laws, complement}` — cited, not re-proved).
  * `Prob` / `f64` — an **MV algebra** on `[0,1]` (`meet = min`, `join = max`,
    `complement = 1 − p`); excluded middle fails. Witnessed on the Rust side (no reals in a
    bare-`lean` file — deviation note 2).
  * (planned, `deep_causality_quantum`) the **projection lattice** of a Hilbert space — an
    orthomodular lattice that fails distributivity; general effects (`0 ≤ E ≤ I`) form only an
    effect algebra with *partial* meet/join, so no blanket operator instance exists (the Stage-3
    spec's scope guard).

DEVIATION NOTES.
  1. The Verdict algebra is a `structure` of the five operations, not a typeclass with law
    fields: the closure statements are about totality and shape of the aggregations (and hold for
    every operation choice); the lattice/complement *laws* per carrier are the separate
    `algebra.verdict.*` theorems (Boolean, proved) and the Rust witnesses (MV).
  2. The MV carrier (`Prob`/`f64`) is not modelled here — bare Lean has no real (or rational
    literal) arithmetic without imports; the Boolean carrier is proved concretely and the MV
    instance is pinned by the Rust witness, following `Algebra/Verdict.lean`.
  3. `Some(k)` fires on a decidable per-verdict predicate (`fires : V → Bool` — Rust: `b == true`
    for `bool`, `p > threshold` for probabilities) and decides `top`/`bottom` by count — the
    `Count` monoid + boundary comparison of design D6.
  4. The bag is a `CList` (mutual list, structural recursion in bare Lean) as in
    `Core/Causaloid.lean`; aggregation folds it.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witnesses: `deep_causality/tests/formalization_lean/verdict_closure_tests.rs`.

Imports: keep to the exact minimum. Every Mathlib import pulls its whole transitive closure into
the build, so import the narrowest module that still type-checks -- `Mathlib.Analysis.Quaternion`
once reached 8,639 of Mathlib's 9,450 modules to supply four algebraic laws. Mirror any new import
into `cache_roots` in `//MODULE.bazel`: that list tree-shakes the Mathlib olean download to those
roots plus their closure, so a module absent from it is never fetched and the build fails on it.
-/

namespace DeepCausalityFormal.Core.VerdictClosure

/-- `AggregateLogic` (Rust `deep_causality/src/types/causal_types/aggregate_logic/`). -/
inductive AggLogic where
  | all   : AggLogic
  | any   : AggLogic
  | none  : AggLogic
  | someK : Nat → AggLogic

/-- The Verdict algebra: a carrier with the five operations of the Rust `Verdict` trait
    (`deep_causality_unified_math/deep_causality_algebra/src/algebra/verdict.rs`). Laws live per carrier
    (`algebra.verdict.*`); deviation note 1. -/
structure VerdictAlg (V : Type) where
  bottom : V
  top : V
  meet : V → V → V
  join : V → V → V
  compl : V → V

variable {V : Type}

/-- `All`: the meet-fold (identity `top`). -/
def aggAll (A : VerdictAlg V) : List V → V
  | []      => A.top
  | v :: vs => A.meet v (aggAll A vs)

/-- `Any`: the join-fold (identity `bottom`). -/
def aggAny (A : VerdictAlg V) : List V → V
  | []      => A.bottom
  | v :: vs => A.join v (aggAny A vs)

/-- `None`: `Any` post-composed with `complement`. -/
def aggNone (A : VerdictAlg V) (vs : List V) : V :=
  A.compl (aggAny A vs)

/-- The firing count of a bag under a decidable per-verdict predicate (the `Count` monoid). -/
def countFires (fires : V → Bool) : List V → Nat
  | []      => 0
  | v :: vs => (if fires v then 1 else 0) + countFires fires vs

/-- `Some(k)`: at least `k` members fire — the count decided into `top`/`bottom` at the boundary
    (design D6; deviation note 3). -/
def aggSome (A : VerdictAlg V) (fires : V → Bool) (k : Nat) (vs : List V) : V :=
  if k ≤ countFires fires vs then A.top else A.bottom

/-- The one dispatcher: every `AggregateLogic` mode, on every finite bag, is a **total, V-valued**
    operation — the closure of the Verdict algebra under aggregation. -/
def aggregate (A : VerdictAlg V) (fires : V → Bool) : AggLogic → List V → V
  | .all,     vs => aggAll A vs
  | .any,     vs => aggAny A vs
  | .none,    vs => aggNone A vs
  | .someK k, vs => aggSome A fires k vs

-- ------------------------------------------------------------------
-- `core.verdict.closure`: the four modes are closed operations.
-- ------------------------------------------------------------------

/-- `None` IS `Any` post-composed with `complement` — the spec's `None` characterization.

    THEOREM_MAP: `core.verdict.closure` -/
theorem none_is_any_complement (A : VerdictAlg V) (vs : List V) :
    aggregate A (fun _ => false) .none vs = A.compl (aggregate A (fun _ => false) .any vs) := rfl

/-- Closure under the fold step: each mode consumes a bag one member at a time entirely inside the
    algebra's operations — `All` steps by `meet`, `Any` by `join` (with their lattice identities on
    the empty bag), so the aggregate of verdicts is a verdict.

    THEOREM_MAP: `core.verdict.closure` -/
theorem closure_fold_step (A : VerdictAlg V) (v : V) (vs : List V) :
    aggAll A (v :: vs) = A.meet v (aggAll A vs)
    ∧ aggAny A (v :: vs) = A.join v (aggAny A vs)
    ∧ aggAll A [] = A.top
    ∧ aggAny A [] = A.bottom :=
  ⟨rfl, rfl, rfl, rfl⟩

/-- `Some(k)` decides by count: it returns `top` exactly when at least `k` members fire, `bottom`
    otherwise — a closed two-valued operation via the `Count` monoid.

    THEOREM_MAP: `core.verdict.closure` -/
theorem someK_decides (A : VerdictAlg V) (fires : V → Bool) (k : Nat) (vs : List V) :
    (k ≤ countFires fires vs → aggSome A fires k vs = A.top)
    ∧ (¬ k ≤ countFires fires vs → aggSome A fires k vs = A.bottom) := by
  constructor
  · intro h; simp [aggSome, h]
  · intro h; simp [aggSome, h]

-- ------------------------------------------------------------------
-- `Coll : Causaloid → Causaloid` — the closure at the fixpoint (task 3.2).
-- A verdict-evaluating causaloid model (the `Core/Causaloid.lean` shape, with the `coll` summand
-- given the aggregation semantics).
-- ------------------------------------------------------------------

mutual
  /-- A causaloid whose evaluation lands in the verdict carrier `V`: atoms carry an element map,
      `coll` carries a bag and an aggregation mode. -/
  inductive VCausaloid (V : Type) where
    | atom : (V → V) → VCausaloid V
    | coll : VCList V → AggLogic → VCausaloid V

  inductive VCList (V : Type) where
    | nil  : VCList V
    | cons : VCausaloid V → VCList V → VCList V
end

mutual
  /-- Evaluation: an atom applies its element map; a collection evaluates every member on the
      incoming verdict and aggregates through the Verdict algebra. -/
  def eval (A : VerdictAlg V) (fires : V → Bool) : VCausaloid V → V → V
    | .atom f,      v => f v
    | .coll cs lgc, v => aggregate A fires lgc (evalList A fires cs v)

  def evalList (A : VerdictAlg V) (fires : V → Bool) : VCList V → V → List V
    | .nil,       _ => []
    | .cons c cs, v => eval A fires c v :: evalList A fires cs v
end

/-- **`Coll : Causaloid → Causaloid`** — for every bag of causaloids and every aggregation mode
    there is a causaloid (the `coll` node) whose evaluation is exactly the aggregate of the
    members' evaluations. The collection does not leave the type: closure of the Verdict algebra
    lifts to closure of the causaloid fixpoint (assumption #5).

    THEOREM_MAP: `core.verdict.closure` -/
theorem coll_closure (A : VerdictAlg V) (fires : V → Bool) (cs : VCList V) (lgc : AggLogic) :
    ∃ c : VCausaloid V, ∀ v : V,
      eval A fires c v = aggregate A fires lgc (evalList A fires cs v) :=
  ⟨.coll cs lgc, fun _ => rfl⟩

-- ------------------------------------------------------------------
-- `core.verdict.perm_invariance`: aggregation is a BAG operation — the #1 scoped order-invariance
-- theorem on the collection path (roadmap #1). Extends the closure above: for every mode the
-- aggregate VALUE is invariant under permutation of the member bag. `All`/`Any` are the
-- commutative-associative meet-/join-folds (the `fuse_perm` device of `Core/GraphAlgebra.lean`,
-- here for the lattice reducts); `None` inherits from `Any`; `Some(k)` from the permutation-
-- invariant firing count. Commutativity + associativity of `meet`/`join` are the lattice-law
-- HYPOTHESES (the `algebra.verdict.{lattice_laws}` theorems supply them per carrier; deviation
-- note 1), exactly as `fuse_perm` takes `∇`'s laws as hypotheses.
--
-- SCOPE (the #1 ruling, stated honestly): the VALUE channel, on the stateless, all-success path;
-- the log channel is a multiset (invariant only up to permutation) and the stateful path are
-- OUTSIDE this statement.
-- ------------------------------------------------------------------

/-- Swap-generated permutation of a member list (self-contained, as in `Core/GraphAlgebra.lean`). -/
inductive Perm : List V → List V → Prop where
  | nil   : Perm [] []
  | cons  (v : V) {l l' : List V} : Perm l l' → Perm (v :: l) (v :: l')
  | swap  (a b : V) (l : List V) : Perm (a :: b :: l) (b :: a :: l)
  | trans {l₁ l₂ l₃ : List V} : Perm l₁ l₂ → Perm l₂ l₃ → Perm l₁ l₃

/-- `All` — the meet-fold — is permutation-invariant when `meet` is commutative + associative. -/
theorem aggAll_perm (A : VerdictAlg V)
    (hcomm : ∀ a b, A.meet a b = A.meet b a)
    (hassoc : ∀ a b c, A.meet (A.meet a b) c = A.meet a (A.meet b c))
    {l l' : List V} (h : Perm l l') : aggAll A l = aggAll A l' := by
  induction h with
  | nil => rfl
  | cons v _ ih => exact congrArg (A.meet v) ih
  | swap a b l =>
      show A.meet a (A.meet b (aggAll A l)) = A.meet b (A.meet a (aggAll A l))
      rw [← hassoc a b (aggAll A l), hcomm a b, hassoc b a (aggAll A l)]
  | trans _ _ ih₁ ih₂ => exact ih₁.trans ih₂

/-- `Any` — the join-fold — is permutation-invariant when `join` is commutative + associative. -/
theorem aggAny_perm (A : VerdictAlg V)
    (hcomm : ∀ a b, A.join a b = A.join b a)
    (hassoc : ∀ a b c, A.join (A.join a b) c = A.join a (A.join b c))
    {l l' : List V} (h : Perm l l') : aggAny A l = aggAny A l' := by
  induction h with
  | nil => rfl
  | cons v _ ih => exact congrArg (A.join v) ih
  | swap a b l =>
      show A.join a (A.join b (aggAny A l)) = A.join b (A.join a (aggAny A l))
      rw [← hassoc a b (aggAny A l), hcomm a b, hassoc b a (aggAny A l)]
  | trans _ _ ih₁ ih₂ => exact ih₁.trans ih₂

/-- The firing count is permutation-invariant (`Nat` addition is commutative + associative). -/
theorem countFires_perm (fires : V → Bool) {l l' : List V} (h : Perm l l') :
    countFires fires l = countFires fires l' := by
  induction h with
  | nil => rfl
  | cons v _ ih => exact congrArg (fun n => (if fires v then 1 else 0) + n) ih
  | swap a b l =>
      show (if fires a then 1 else 0) + ((if fires b then 1 else 0) + countFires fires l)
          = (if fires b then 1 else 0) + ((if fires a then 1 else 0) + countFires fires l)
      omega
  | trans _ _ ih₁ ih₂ => exact ih₁.trans ih₂

/-- `None` (= `Any` ∘ `complement`) is permutation-invariant, inheriting from `Any`. -/
theorem aggNone_perm (A : VerdictAlg V)
    (hcomm : ∀ a b, A.join a b = A.join b a)
    (hassoc : ∀ a b c, A.join (A.join a b) c = A.join a (A.join b c))
    {l l' : List V} (h : Perm l l') : aggNone A l = aggNone A l' := by
  show A.compl (aggAny A l) = A.compl (aggAny A l')
  rw [aggAny_perm A hcomm hassoc h]

/-- `Some(k)` is permutation-invariant, inheriting from the firing count. -/
theorem aggSome_perm (A : VerdictAlg V) (fires : V → Bool) (k : Nat)
    {l l' : List V} (h : Perm l l') : aggSome A fires k l = aggSome A fires k l' := by
  show (if k ≤ countFires fires l then A.top else A.bottom)
      = (if k ≤ countFires fires l' then A.top else A.bottom)
  rw [countFires_perm fires h]

/-- **Permutation-invariance for every mode**: on a permuted member bag the aggregate VALUE is
    unchanged — the #1 scoped order-invariance theorem on the aggregation itself. Requires the
    meet/join lattice laws (comm + assoc), supplied per carrier by `algebra.verdict.lattice_laws`.

    THEOREM_MAP: `core.verdict.perm_invariance` -/
theorem aggregate_perm (A : VerdictAlg V) (fires : V → Bool)
    (hmc : ∀ a b, A.meet a b = A.meet b a)
    (hma : ∀ a b c, A.meet (A.meet a b) c = A.meet a (A.meet b c))
    (hjc : ∀ a b, A.join a b = A.join b a)
    (hja : ∀ a b c, A.join (A.join a b) c = A.join a (A.join b c))
    (lgc : AggLogic) {l l' : List V} (h : Perm l l') :
    aggregate A fires lgc l = aggregate A fires lgc l' := by
  cases lgc with
  | all    => exact aggAll_perm A hmc hma h
  | any    => exact aggAny_perm A hjc hja h
  | none   => exact aggNone_perm A hjc hja h
  | someK k => exact aggSome_perm A fires k h

/-- Swap-generated permutation of a member bag at the fixpoint (`VCList`). -/
inductive PermC : VCList V → VCList V → Prop where
  | nil   : PermC .nil .nil
  | cons  (c : VCausaloid V) {cs cs' : VCList V} : PermC cs cs' → PermC (.cons c cs) (.cons c cs')
  | swap  (a b : VCausaloid V) (cs : VCList V) :
      PermC (.cons a (.cons b cs)) (.cons b (.cons a cs))
  | trans {cs₁ cs₂ cs₃ : VCList V} : PermC cs₁ cs₂ → PermC cs₂ cs₃ → PermC cs₁ cs₃

/-- Permuting the member bag permutes the evaluated verdict list (the element layer is pointwise). -/
theorem evalList_permC (A : VerdictAlg V) (fires : V → Bool) (v : V)
    {cs cs' : VCList V} (h : PermC cs cs') :
    Perm (evalList A fires cs v) (evalList A fires cs' v) := by
  induction h with
  | nil => exact .nil
  | cons c _ ih => exact .cons (eval A fires c v) ih
  | swap a b cs => exact .swap (eval A fires a v) (eval A fires b v) (evalList A fires cs v)
  | trans _ _ ih₁ ih₂ => exact .trans ih₁ ih₂

/-- **Collection aggregation is a bag operation at the fixpoint** (`Coll : Causaloid → Causaloid`):
    permuting the members of a collection causaloid leaves its verdict unchanged. The order-
    invariance of the aggregate (`aggregate_perm`) composed with the pointwise element layer
    (`evalList_permC`) — the #1 scoped order-invariance theorem, lifted to the `coll` node.

    THEOREM_MAP: `core.verdict.perm_invariance` -/
theorem coll_perm (A : VerdictAlg V) (fires : V → Bool)
    (hmc : ∀ a b, A.meet a b = A.meet b a)
    (hma : ∀ a b c, A.meet (A.meet a b) c = A.meet a (A.meet b c))
    (hjc : ∀ a b, A.join a b = A.join b a)
    (hja : ∀ a b c, A.join (A.join a b) c = A.join a (A.join b c))
    (lgc : AggLogic) {cs cs' : VCList V} (h : PermC cs cs') (v : V) :
    eval A fires (.coll cs lgc) v = eval A fires (.coll cs' lgc) v := by
  show aggregate A fires lgc (evalList A fires cs v)
      = aggregate A fires lgc (evalList A fires cs' v)
  exact aggregate_perm A fires hmc hma hjc hja lgc (evalList_permC A fires v h)

-- ------------------------------------------------------------------
-- `core.verdict.carriers`: the named carriers behind the one trait.
-- ------------------------------------------------------------------

/-- The Boolean carrier — mirrors `impl Verdict for bool`. -/
def boolAlg : VerdictAlg Bool :=
  ⟨false, true, and, or, not⟩

theorem aggAll_bool : (vs : List Bool) → aggAll boolAlg vs = vs.all (fun b => b)
  | [] => rfl
  | v :: vs => by
      show boolAlg.meet v (aggAll boolAlg vs) = (v :: vs).all (fun b => b)
      rw [aggAll_bool vs]
      cases v <;> rfl

theorem aggAny_bool : (vs : List Bool) → aggAny boolAlg vs = vs.any (fun b => b)
  | [] => rfl
  | v :: vs => by
      show boolAlg.join v (aggAny boolAlg vs) = (v :: vs).any (fun b => b)
      rw [aggAny_bool vs]
      cases v <;> rfl

/-- The Boolean carrier is the Boolean-algebra instance: `All` is the universal conjunction,
    `Any` the existential disjunction, `None` its complement — and it is distributive (the law the
    MV and orthomodular classes each fail in their own way; header). The MV carrier (`Prob`/`f64`)
    is pinned by the Rust witness (deviation note 2).

    THEOREM_MAP: `core.verdict.carriers` -/
theorem bool_carrier_characterization (vs : List Bool) :
    (aggAll boolAlg vs = vs.all (fun b => b))
    ∧ (aggAny boolAlg vs = vs.any (fun b => b))
    ∧ (aggNone boolAlg vs = !(vs.any (fun b => b))) := by
  refine ⟨aggAll_bool vs, aggAny_bool vs, ?_⟩
  show boolAlg.compl (aggAny boolAlg vs) = !(vs.any (fun b => b))
  rw [aggAny_bool vs]; rfl

/-- Distributivity on the Boolean carrier — the law that separates the Boolean class from MV
    (fails excluded middle) and orthomodular (fails distributivity itself).

    THEOREM_MAP: `core.verdict.carriers` -/
theorem bool_distributive (x y z : Bool) :
    boolAlg.meet x (boolAlg.join y z) = boolAlg.join (boolAlg.meet x y) (boolAlg.meet x z) := by
  cases x <;> cases y <;> cases z <;> rfl

end DeepCausalityFormal.Core.VerdictClosure
