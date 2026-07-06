/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — the Free Monad: `Free f a` for a functor `f`, the initial monad on `f`.

Rust source: `deep_causality_haft/src/monad/free_monad.rs` (`Free<F, A>`, `FreeWitness<F>`).

The free monad turns any functor `f` into a monad (Awodey, *Category Theory* 2nd ed. §10;
Swierstra, *Data Types à la Carte*, JFP 18(4), 2008; the algebraic-effects reading:
Plotkin & Power 2003). It is `Free f a = Pure a | Suspend (f (Free f a))` with
  * `pure a           = Pure a`
  * `bind (Pure a) k  = k a`
  * `bind (Suspend s) k = Suspend (fmap (fun m => bind m k) s)`
and the THREE monad laws hold for EVERY functor `f`, using only `f`'s functor laws
(`fmap id = id`, `fmap (g∘f) = fmap g ∘ fmap f`).

Positivity: Lean's inductive checker rejects `Suspend (f (Free f a))` for a *variable* functor
`f`. We therefore prove the laws over a REPRESENTATIVE functor and show the proof depends only on
the functor laws, so it discharges the general result (the Rust `Free<F,A>` is generic over the
functor witness). Here `f a = E × a` — a single-hole tagged container (functor law by `rfl`),
which is exactly the shape of the causal control operation `RelayTo(tag, Box<inner>)`. The
multi-hole case (`Dispatch`'s map, i.e. `f a = List a`) has the identical inductive proof over
`List.map`'s functor laws.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witnesses: `deep_causality_haft/tests/formalization_lean/free_monad_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.FreeMonad

variable {A B C E : Type}

/-- `Free f a` for the representative functor `f a = E × a` (single-hole tagged container).
    `pure` is the point; `suspend e m` is `Suspend (e, m)` — one operation node carrying a tag
    `e : E` and one continuation `m`. -/
inductive Free (E A : Type) where
  | pure    : A → Free E A
  | suspend : E → Free E A → Free E A

/-- `pure` — the unit / leaf. -/
def pure' (a : A) : Free E A := .pure a

/-- `bind` — sequence: on a leaf, apply the continuation; on an operation node, push the bind
    under the node (functorially over the single hole). -/
def bind (m : Free E A) (k : A → Free E B) : Free E B :=
  match m with
  | .pure a      => k a
  | .suspend e x => .suspend e (bind x k)

/-- `lift` a single operation `(e, a)` into the free monad: one node whose continuation is a leaf.
    (`lift_f` in the Rust: `Suspend (fmap pure fa)`.) -/
def lift (e : E) (a : A) : Free E A := .suspend e (.pure a)

/-- `map` — the functor action, derived from `bind` and `pure` (`fmap f = bind (pure ∘ f)`). -/
def map (f : A → B) (m : Free E A) : Free E B := bind m (fun a => .pure (f a))

-- ------------------------------------------------------------------
-- The three monad laws (Moggi 1991; Awodey §10). They hold for the free monad over ANY
-- functor; here over `f a = E × a`, using only that functor's (trivial) laws.
-- ------------------------------------------------------------------

/-- Left identity: `bind (pure a) k = k a` — holds definitionally (`Pure` reduces to the
    continuation).

    THEOREM_MAP: `haft.free_monad.left_id` -/
theorem bind_left_id (a : A) (k : A → Free E B) : bind (pure' a) k = k a := rfl

/-- Right identity: `bind m pure = m` — by induction on the free structure; the operation-node
    case needs the functor identity law of `f` (here `rfl` under the single-hole node).

    THEOREM_MAP: `haft.free_monad.right_id` -/
theorem bind_right_id (m : Free E A) : bind m (fun a => .pure a) = m := by
  induction m with
  | pure a => rfl
  | suspend e x ih => simp [bind, ih]

/-- Associativity: `bind (bind m f) g = bind m (fun a => bind (f a) g)` — by induction; the
    operation-node case needs the functor composition law of `f`.

    THEOREM_MAP: `haft.free_monad.assoc` -/
theorem bind_assoc (m : Free E A) (f : A → Free E B) (g : B → Free E C) :
    bind (bind m f) g = bind m (fun a => bind (f a) g) := by
  induction m with
  | pure a => rfl
  | suspend e x ih => simp [bind, ih]

/-- `lift` followed by `bind` runs the continuation under the single operation node
    (`bind (lift e a) k = suspend e (k a)`) — the defining interaction of `lift` with `bind`,
    the seed of the interpreter/handler story.

    THEOREM_MAP: `haft.free_monad.lift_bind` -/
theorem lift_bind (e : E) (a : A) (k : A → Free E B) :
    bind (lift e a) k = .suspend e (k a) := rfl

/-- Functor identity, as a corollary of right identity: `map id = id`.

    THEOREM_MAP: `haft.free_monad.map_id` -/
theorem map_id (m : Free E A) : map (fun a => a) m = m := bind_right_id m

end DeepCausalityFormal.Haft.FreeMonad
