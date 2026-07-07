/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Core — CausalCommand: the control operation functor of the adaptive-reasoning free monad.

Rust source: `deep_causality_core/src/types/causal_command/mod.rs` (`CausalCommand<K>`,
`CausalCommandWitness`) and `causal_command/hkt.rs` (the `Functor` witness).

Layering: the *base* is the free monad on a functor, proved in `Haft/FreeMonad.lean`
(`haft.free_monad.left_id`/`right_id`/`assoc`) for the representative single-hole functor
`f a = E × a`. `CausalCommand<K> = RelayTo(usize, K)` **is** that functor (tag `E = usize`, one hole
`K`), so the reasoning program `CausalEffect<V> = Free<CausalCommandWitness, Option<V>>` is exactly
`Haft/FreeMonad.lean` instantiated at this operation — nothing new about the free construction is
assumed here. This file (1) proves the single-hole functor laws `CausalCommandWitness` must satisfy
to *be* a `Free` operation functor (`fmap` maps the one hole, leaving the `target` index as
structure), and (2) transcribes the free monad over it to exhibit the three monad laws at this
concrete operation and to pin the **structural `RelayTo`-tree equality** (`program_eq` in
`causal_effect/mod.rs`) as a lawful *congruent* equivalence — the replacement for the removed,
non-reflexive `Map` partial-equivalence relation.

References: Swierstra, *Data Types à la Carte*, JFP 18(4), 2008; Plotkin & Power 2003 (algebraic
effects); Awodey, *Category Theory* 2nd ed. §10.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witnesses: `deep_causality_core/tests/formalization_lean/causal_command_tests.rs`.
-/

namespace DeepCausalityFormal.Core.CausalCommand

variable {A B C : Type}

/-- The control operation functor `CausalCommand<K>`: a single operation `RelayTo(target, hole)`.
    `target : Nat` is the causaloid index (structure, not a hole); `K` is the one sub-program hole. -/
inductive Command (K : Type) where
  | relayTo : Nat → K → Command K

/-- The witness `fmap` (`causal_command/hkt.rs`): map the single sub-program hole, leaving the
    `target` index untouched. -/
def cmap (f : A → B) : Command A → Command B
  | .relayTo t k => .relayTo t (f k)

/-- Functor identity: `fmap id = id` — the precondition `Free` requires of its operation functor.

    THEOREM_MAP: `core.causal_command.functor_laws` -/
theorem cmap_id (m : Command A) : cmap (fun a => a) m = m := by
  cases m; rfl

/-- Functor composition: `fmap (g ∘ f) = fmap g ∘ fmap f`.

    THEOREM_MAP: `core.causal_command.functor_laws` -/
theorem cmap_comp (f : A → B) (g : B → C) (m : Command A) :
    cmap (fun a => g (f a)) m = cmap g (cmap f m) := by
  cases m; rfl

-- ------------------------------------------------------------------
-- The free monad over `Command`, i.e. `Haft/FreeMonad.lean` instantiated at this operation.
-- `Free A = Pure a | relay target sub` — `relay` is `Suspend (RelayTo target sub)` flattened
-- (the single-hole node), exactly as `Haft/FreeMonad.lean` flattens `Suspend (e, m)`.
-- ------------------------------------------------------------------

/-- `CausalEffect`'s program: `Pure` leaves and `RelayTo` control nodes. -/
inductive Free (A : Type) where
  | pure  : A → Free A
  | relay : Nat → Free A → Free A

/-- `bind` — sequence: on a leaf apply the continuation; on a `RelayTo` node push the bind under the
    single hole (functorially, via `cmap`). -/
def bind (m : Free A) (k : A → Free B) : Free B :=
  match m with
  | .pure a    => k a
  | .relay t x => .relay t (bind x k)

/-- Left identity: `bind (pure a) k = k a` (holds definitionally). Instantiates
    `haft.free_monad.left_id`.

    THEOREM_MAP: `core.causal_command.functor_laws` -/
theorem bind_left_id (a : A) (k : A → Free B) : bind (.pure a) k = k a := rfl

/-- Right identity: `bind m pure = m` — by induction; the `RelayTo` case needs `Command`'s functor
    identity (`cmap_id`, here `rfl` under the single hole). Instantiates `haft.free_monad.right_id`. -/
theorem bind_right_id (m : Free A) : bind m (fun a => .pure a) = m := by
  induction m with
  | pure a => rfl
  | relay t x ih => simp [bind, ih]

/-- Associativity: `bind (bind m f) g = bind m (fun a => bind (f a) g)` — by induction; the
    `RelayTo` case needs `Command`'s functor composition (`cmap_comp`). Instantiates
    `haft.free_monad.assoc`. -/
theorem bind_assoc (m : Free A) (f : A → Free B) (g : B → Free C) :
    bind (bind m f) g = bind m (fun a => bind (f a) g) := by
  induction m with
  | pure a => rfl
  | relay t x ih => simp [bind, ih]

/-- Structural `RelayTo`-tree equality is a lawful **congruence**: equal targets and equal
    sub-programs give equal programs (and, being propositional `=`, it is reflexive, symmetric, and
    transitive). This is the equivalence `program_eq` implements — the honest replacement for the
    removed `Map` PER, which failed reflexivity. -/
theorem relay_congr {t₁ t₂ : Nat} {k₁ k₂ : Free A}
    (ht : t₁ = t₂) (hk : k₁ = k₂) : Free.relay t₁ k₁ = Free.relay t₂ k₂ := by
  rw [ht, hk]

end DeepCausalityFormal.Core.CausalCommand
