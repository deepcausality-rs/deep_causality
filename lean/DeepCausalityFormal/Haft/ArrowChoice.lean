/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — the choice fragment `⊕` (ArrowChoice): routing over the coproduct.

Rust source: `deep_causality_haft/src/arrow/choice.rs` (`Left`, `Right`, `Choice`, `Fanin` and the
`Arrow::{left, right, choice, fanin}` methods).

Textbook definition. An **ArrowChoice** (John Hughes, "Generalising Monads to Arrows," *Sci.
Comput. Program.* 37(1–3), 2000, §5) extends an arrow with routing over a sum type: `left f` acts
on the left summand and passes the right through, subject to the choice laws — `left (arr f) =
arr (f ⊕ id)`, functoriality `left (f >>> g) = left f >>> left g`, the exchange law with a
right-summand map, and the injection unit law `arr inl >>> left f = f >>> arr inl`. `f ||| g`
(fanin) is the **coproduct elimination** — the universal map of the coproduct (Awodey, *Category
Theory* 2nd ed., §3.1; the crate's `haft.either.coproduct_universal`). The `⊗`-over-`⊕`
distributivity `α × (β ⊕ γ) ≅ (α × β) ⊕ (α × γ)` is the rig-category coherence that causally
faithful direct-sum decompositions rely on (R. Lorenz & J. Barrett, "Causal and compositional
structure of unitary transformations," *Quantum* 5, 511 (2021), §3–4).

This file proves, in the eager (function-category) model that mirrors the Rust value-level
combinators:
  * the ArrowChoice laws on `left`/`right`/`choice` (`+++`),
  * `fanin` (`|||`) as the coproduct elimination — computation rules and uniqueness,
  * the `⊗`-over-`⊕` distributivity equations the crate uses: `distl`/`undistl` are mutually
    inverse and `distl` is natural in all three components.

DEVIATION NOTES.
  1. The eager model interprets an arrow as a plain function (the function category), matching the
     Rust `Arrow::run`; `arr` is therefore the identity embedding and composition is `∘`. The laws
     for an *effectful* arrow target are inherited through the interpreter
     (`haft.interpreter.choice_preserved`).
  2. The sum is Lean's `Sum α β`, mirroring the Rust `Either<L, R>` one-to-one
     (`inl ↔ Either::Left`, `inr ↔ Either::Right`).
  3. Full rig-category coherence (all coherence diagrams for `⊗`/`⊕`) is deliberately deferred:
     only the equations the crate uses are stated and proved (the roadmap's Stage-2b scope note).

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/formalization_lean/arrow_choice_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.ArrowChoice

variable {α β γ δ ε ζ : Type}

/-- `left f` — act on the left summand, pass a right injection through. Mirrors `Left<F, C>`. -/
def left (f : α → β) : Sum α γ → Sum β γ
  | .inl a => .inl (f a)
  | .inr c => .inr c

/-- `right f` — act on the right summand, pass a left injection through. Mirrors `Right<F, C>`. -/
def right (f : α → β) : Sum γ α → Sum γ β
  | .inl c => .inl c
  | .inr a => .inr (f a)

/-- `f +++ g` — route each summand to its own arm. Mirrors `Choice<F, G>`; this is also the
    functorial action `f ⊕ g` of the coproduct. -/
def choice (f : α → β) (g : γ → δ) : Sum α γ → Sum β δ
  | .inl a => .inl (f a)
  | .inr c => .inr (g c)

/-- `f ||| g` — the coproduct elimination: both arms converge on one output type. Mirrors
    `Fanin<F, G>` (`haft.either.coproduct_universal` as an arrow). -/
def fanin (f : α → β) (g : γ → β) : Sum α γ → β
  | .inl a => f a
  | .inr c => g c

-- ------------------------------------------------------------------
-- The ArrowChoice laws (Hughes 2000 §5), in the eager model.
-- ------------------------------------------------------------------

/-- `left (arr f) = arr (f ⊕ id)`: on pure arrows, `left` IS the functorial sum action with the
    identity on the passive summand.

    THEOREM_MAP: `haft.arrow_choice.laws` -/
theorem left_arr (f : α → β) :
    ∀ x : Sum α γ, left f x = choice f (fun c => c) x := by
  intro x; cases x <;> rfl

/-- Functoriality: `left (f >>> g) = left f >>> left g` (composition is preserved on the active
    summand).

    THEOREM_MAP: `haft.arrow_choice.laws` -/
theorem left_comp (f : α → β) (g : β → δ) :
    ∀ x : Sum α γ, left (γ := γ) (fun a => g (f a)) x = left g (left f x) := by
  intro x; cases x <;> rfl

/-- The exchange law: `left f >>> arr (id ⊕ g) = arr (id ⊕ g) >>> left f` — a pure map on the
    passive (right) summand commutes with `left f`.

    THEOREM_MAP: `haft.arrow_choice.laws` -/
theorem left_exchange (f : α → β) (g : γ → δ) :
    ∀ x : Sum α γ, choice (fun b => b) g (left f x) = left f (choice (fun a => a) g x) := by
  intro x; cases x <;> rfl

/-- The injection unit law: `f >>> arr inl = arr inl >>> left f` — injecting then routing equals
    running then injecting.

    THEOREM_MAP: `haft.arrow_choice.laws` -/
theorem left_unit (f : α → β) (a : α) :
    left (γ := γ) f (.inl a) = .inl (f a) := rfl

/-- `right` is `left` conjugated by the symmetry (swap): the two liftings are the same content on
    mirrored sums.

    THEOREM_MAP: `haft.arrow_choice.laws` -/
theorem right_is_swapped_left (f : α → β) :
    ∀ x : Sum γ α,
      right f x = (fun y : Sum β γ => match y with | .inl b => .inr b | .inr c => .inl c)
        (left f (match x with | .inl c => .inr c | .inr a => .inl a)) := by
  intro x; cases x <;> rfl

/-- Functoriality of `+++`: `(f >>> f') +++ (g >>> g') = (f +++ g) >>> (f' +++ g')`.

    THEOREM_MAP: `haft.arrow_choice.laws` -/
theorem choice_comp (f : α → β) (f' : β → ε) (g : γ → δ) (g' : δ → ζ) :
    ∀ x : Sum α γ,
      choice (fun a => f' (f a)) (fun c => g' (g c)) x = choice f' g' (choice f g x) := by
  intro x; cases x <;> rfl

-- ------------------------------------------------------------------
-- Fanin: the coproduct elimination (computation + uniqueness).
-- ------------------------------------------------------------------

/-- Fanin computation, left arm: `(f ||| g) ∘ inl = f`.

    THEOREM_MAP: `haft.arrow_choice.laws` -/
theorem fanin_inl (f : α → β) (g : γ → β) (a : α) : fanin f g (.inl a) = f a := rfl

/-- Fanin computation, right arm: `(f ||| g) ∘ inr = g`.

    THEOREM_MAP: `haft.arrow_choice.laws` -/
theorem fanin_inr (f : α → β) (g : γ → β) (c : γ) : fanin f g (.inr c) = g c := rfl

/-- Fanin uniqueness — the universal property of the coproduct (extends
    `haft.either.coproduct_universal` to the arrow fragment): any map agreeing with `f` on `inl`
    and `g` on `inr` IS `fanin f g`.

    THEOREM_MAP: `haft.arrow_choice.laws` -/
theorem fanin_unique (f : α → β) (g : γ → β) (h : Sum α γ → β)
    (hl : ∀ a, h (.inl a) = f a) (hr : ∀ c, h (.inr c) = g c) :
    ∀ x, h x = fanin f g x := by
  intro x
  cases x with
  | inl a => rw [hl a]; rfl
  | inr c => rw [hr c]; rfl

/-- Fanin absorbs choice: `(f +++ g) >>> (h ||| k) = (f >>> h) ||| (g >>> k)` — routing then
    eliminating equals eliminating the composed arms.

    THEOREM_MAP: `haft.arrow_choice.laws` -/
theorem fanin_choice (f : α → β) (g : γ → δ) (h : β → ε) (k : δ → ε) :
    ∀ x : Sum α γ,
      fanin h k (choice f g x) = fanin (fun a => h (f a)) (fun c => k (g c)) x := by
  intro x; cases x <;> rfl

-- ------------------------------------------------------------------
-- The ⊗-over-⊕ distributivity equations used (full rig coherence deferred — deviation note 3).
-- ------------------------------------------------------------------

/-- Left distributivity `distl : α × (β ⊕ γ) → (α × β) ⊕ (α × γ)`. -/
def distl : α × Sum β γ → Sum (α × β) (α × γ)
  | (a, .inl b) => .inl (a, b)
  | (a, .inr c) => .inr (a, c)

/-- The inverse `undistl : (α × β) ⊕ (α × γ) → α × (β ⊕ γ)`. -/
def undistl : Sum (α × β) (α × γ) → α × Sum β γ
  | .inl (a, b) => (a, .inl b)
  | .inr (a, c) => (a, .inr c)

/-- `distl` and `undistl` are mutually inverse (one direction): `undistl ∘ distl = id`.

    THEOREM_MAP: `haft.arrow_choice.laws` -/
theorem undistl_distl (x : α × Sum β γ) : undistl (distl x) = x := by
  obtain ⟨a, s⟩ := x
  cases s <;> rfl

/-- `distl` and `undistl` are mutually inverse (other direction): `distl ∘ undistl = id`.

    THEOREM_MAP: `haft.arrow_choice.laws` -/
theorem distl_undistl (y : Sum (α × β) (α × γ)) : distl (undistl y) = y := by
  cases y with
  | inl p => obtain ⟨a, b⟩ := p; rfl
  | inr p => obtain ⟨a, c⟩ := p; rfl

/-- Naturality of `distl`: it commutes with `f ⊗ (g ⊕ h)` — the pairs-distribute-over-sums
    equation faithful direct-sum decompositions rely on (Lorenz & Barrett 2021 §4).

    THEOREM_MAP: `haft.arrow_choice.laws` -/
theorem distl_natural (f : α → δ) (g : β → ε) (h : γ → ζ) :
    ∀ x : α × Sum β γ,
      distl ((fun p : α × Sum β γ => (f p.1, choice g h p.2)) x)
        = choice (fun p : α × β => (f p.1, g p.2)) (fun p : α × γ => (f p.1, h p.2)) (distl x) := by
  intro x
  obtain ⟨a, s⟩ := x
  cases s <;> rfl

end DeepCausalityFormal.Haft.ArrowChoice
