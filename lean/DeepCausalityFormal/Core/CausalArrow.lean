/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Core — the Causal Arrow: the Kleisli category of the causal monad, with full state/context
threading (Option B; deviation D2 of the core formalization plan).

Rust source: `deep_causality_core/src/types/causal_arrow/` (`CausalLift`, `KleisliCompose`,
`causal_arrow`) and `src/types/causal_flow/steps.rs` (`CausalFlow::and_then` — the one bind).

Layering: the base mechanism is the Kleisli category of a lawful monad — the Kleisli category of a
monad is itself a lawful category (Mac Lane, *CWM* §VI.5). The monad here is the state-threading
causal monad already proved lawful in `Core/CausalMonad.lean` (`core.causal_monad.{left_id,right_id,
assoc,left_zero}`). This file reuses that carrier verbatim and proves the arrow laws by reduction to
the monad laws — nothing new is assumed.

Model (faithful to the Rust `and_then`): a *stage* is `A → S → Option C → Process B S C E Λ`,
receiving the incoming `(value, state, context)` and producing the next carrier. The arrow's bind
`akbind` runs a stage on the concrete carried value when present (`some b`), threading state/context
forward exactly as `CausalFlow::and_then` does; a `none` carrier short-circuits lawfully (re-emit
`none`), and an errored carrier short-circuits via the monad `bind'` (left zero). `karrow_id` is the
unit `η(a, s, c) = eta (some a) s c`. Composition is `kcomp f g = fun a s c => akbind (f a s c) g`.

Theorems: left identity, right identity, associativity, error left-zero — all over ARBITRARY `S`, `C`
(the state/context are threaded on both sides of every equation; no `S,C`-erasure). Each reduces to
the corresponding monad theorem.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witnesses: `deep_causality_core/tests/types/causal_arrow/causal_arrow_tests.rs`
(`arrow_threads_accumulated_state`, `arrow_error_short_circuit_preserves_state`, identity/compose)
and the formalization mirror `tests/formalization_lean/causal_arrow_tests.rs`.
-/

namespace DeepCausalityFormal.Core.CausalArrow

/-- The five-channel causal carrier (transcribed from `Core/CausalMonad.lean`). -/
structure Process (V S C E Λ : Type) where
  outcome : Except E (Option V)
  state : S
  ctx : Option C
  logs : List Λ

variable {A B D S C E Λ : Type}

/-- The Kleisli unit of the state-threading monad (as in `CausalMonad.lean`). -/
def eta (v : Option V) (s : S) (c : Option C) : Process V S C E Λ :=
  { outcome := .ok v, state := s, ctx := c, logs := [] }

/-- State-threading monad `bind` (transcribed from `CausalMonad.lean :: bind'`): the `error` arm
    short-circuits (left zero); the `ok` arm runs the continuation on `(value, state, ctx)` and
    prepends the incoming log. -/
def bind' (m : Process V S C E Λ) (f : Option V → S → Option C → Process W S C E Λ) :
    Process W S C E Λ :=
  match m.outcome with
  | .error e => { outcome := .error e, state := m.state, ctx := m.ctx, logs := m.logs }
  | .ok v =>
    let n := f v m.state m.ctx
    { outcome := n.outcome, state := n.state, ctx := n.ctx, logs := m.logs ++ n.logs }

/-- A causal-arrow *stage*: receives the concrete value, state, and context. -/
abbrev Stage (A B S C E Λ : Type) := A → S → Option C → Process B S C E Λ

/-- The arrow's value-threading bind, transcribing `CausalFlow::and_then`: on a present value
    (`some b`) run the stage threading state/context; a `none` carrier short-circuits lawfully
    (re-emit `none`); an errored carrier short-circuits via `bind'` (left zero). -/
def akbind (m : Process B S C E Λ) (g : Stage B D S C E Λ) : Process D S C E Λ :=
  bind' m (fun ov s c =>
    match ov with
    | some b => g b s c
    | none => eta none s c)

/-- The identity arrow / unit: re-emit the value, thread state/context, empty log. -/
def karrow_id (a : A) (s : S) (c : Option C) : Process A S C E Λ := eta (some a) s c

/-- Kleisli composition of two stages: run `f`, thread its `(value, state, context)` into `g`. -/
def kcomp (f : Stage A B S C E Λ) (g : Stage B D S C E Λ) : Stage A D S C E Λ :=
  fun a s c => akbind (f a s c) g

/-- Left identity: `karrow_id >=> f = f` — the unit on the left threads through untouched.

    THEOREM_MAP: `core.causal_arrow.category_laws` -/
theorem kcomp_left_id (f : Stage A B S C E Λ) : kcomp karrow_id f = f := by
  funext a s c
  simp [kcomp, akbind, karrow_id, eta, bind']

/-- Right identity: `f >=> karrow_id = f` — the unit on the right is exactly the monad unit under
    `akbind`, so this reduces to the monad's right identity (`core.causal_monad.right_id`).

    THEOREM_MAP: `core.causal_arrow.category_laws` -/
theorem kcomp_right_id (f : Stage A B S C E Λ) : kcomp f karrow_id = f := by
  funext a s c
  show akbind (f a s c) karrow_id = f a s c
  -- `akbind m karrow_id = bind' m eta`, and `bind' m eta = m`.
  cases hm : (f a s c) with
  | mk outcome state ctx logs =>
    cases outcome with
    | error e => simp [akbind, bind', hm]
    | ok v =>
      cases v with
      | none => simp [akbind, bind', karrow_id, eta, hm]
      | some b => simp [akbind, bind', karrow_id, eta, hm]

/-- Associativity: `(f >=> g) >=> h = f >=> (g >=> h)` — the log channel needs `List.append_assoc`
    (the Writer law); every other channel threads identically on both sides. Reduces to the monad's
    associativity (`core.causal_monad.assoc`).

    THEOREM_MAP: `core.causal_arrow.category_laws` -/
theorem kcomp_assoc (f : Stage A B S C E Λ) (g : Stage B D S C E Λ) (h : Stage D V S C E Λ) :
    kcomp (kcomp f g) h = kcomp f (kcomp g h) := by
  funext a s c
  show akbind (akbind (f a s c) g) h = akbind (f a s c) (fun b s c => akbind (g b s c) h)
  cases hf : (f a s c) with
  | mk outcome state ctx logs =>
    cases outcome with
    | error e => simp [akbind, bind', hf]
    | ok v =>
      cases v with
      | none => simp [akbind, bind', eta, hf]
      | some b =>
        cases hg : (g b state ctx) with
        | mk o2 s2 c2 l2 =>
          cases o2 with
          | error e => simp [akbind, bind', hf, hg]
          | ok w =>
            cases w with
            | none => simp [akbind, bind', eta, hf, hg]
            | some d => simp [akbind, bind', hf, hg, List.append_assoc]

/-- Error left-zero: an errored stage short-circuits composition — the downstream stage is never
    consulted and error/state/context/logs survive verbatim (the monad's left zero,
    `core.causal_monad.left_zero`).

    THEOREM_MAP: `core.causal_arrow.left_zero` -/
theorem kcomp_left_zero (e : E) (s : S) (c : Option C) (l : List Λ)
    (a : A) (f : Stage A B S C E Λ) (g : Stage B D S C E Λ)
    (hf : f a s c = { outcome := .error e, state := s, ctx := c, logs := l }) :
    kcomp f g a s c = { outcome := .error e, state := s, ctx := c, logs := l } := by
  show akbind (f a s c) g = { outcome := .error e, state := s, ctx := c, logs := l }
  rw [hf]
  simp [akbind, bind']

end DeepCausalityFormal.Core.CausalArrow
