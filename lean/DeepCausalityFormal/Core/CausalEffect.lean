/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Core — CausalEffect: the success channel of a causal computation.

Rust source: `deep_causality_core/src/types/causal_effect/mod.rs` (`CausalEffect<V>`,
`CausalEffect::{value,none,from_option,into_value,map}`). Replaces the deleted `EffectValue`.

`CausalEffect<V> = Free<CausalCommandWitness, Option<V>>` — the free monad
(`Core/CausalCommand.lean`, itself `haft.free_monad.*`) with **`Option<V>` value leaves**:
  * `Pure(Some v)`        — a value,
  * `Pure(None)`          — the absence-of-evidence effect,
  * `Suspend(RelayTo t k)` — an adaptive-reasoning jump (a command).

Layering: the value content is `Option<V>`, whose functor identity/composition are **already
proved** in `Haft/Functor.lean` (`haft.functor.laws`, the `OptionWitness` instance). This file does
NOT re-prove a bespoke value-type functor (the former `EffectValue` re-proof is gone); it cites the
`Option` laws and shows the total `CausalEffect::map` lifts them through the command tree. It then
proves `into_value` is the **honest `Maybe` projection** — `Pure(Some v) ↦ Some v`,
`Pure(None) ↦ None`, command `↦ None` — with no lossy/negative caveat (there is no `Map` variant or
`RelayTo` payload to drop; those no longer inhabit the type).

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witnesses: `deep_causality_core/tests/formalization_lean/causal_effect_tests.rs` (and the 18
behavioural unit tests in `tests/types/causal_effect/`).
-/

namespace DeepCausalityFormal.Core.CausalEffect

variable {V W X : Type}

/-- `CausalEffect<V> = Free<CausalCommand, Option<V>>`: `Option<V>` value leaves and `RelayTo`
    control nodes (the `relay` node is `Suspend(RelayTo(target, sub))`). -/
inductive CEffect (V : Type) where
  | pure  : Option V → CEffect V
  | relay : Nat → CEffect V → CEffect V

/-- `CausalEffect::value` — `Pure(Some v)`. -/
def value (v : V) : CEffect V := .pure (some v)

/-- `CausalEffect::none` — `Pure(None)`. -/
def noneEffect : CEffect V := .pure Option.none

/-- `CausalEffect::from_option` — `Pure(o)`. -/
def fromOption (o : Option V) : CEffect V := .pure o

-- ------------------------------------------------------------------
-- Value functor = the `Option` functor (`haft.functor.laws`). Transcribed, NOT re-proved: these are
-- the `OptionWitness` identity/composition already closed in `Haft/Functor.lean`.
-- ------------------------------------------------------------------

/-- Value-functor identity — the `Option` instance of `haft.functor.laws`. -/
theorem value_fmap_id (o : Option V) : o.map (fun v => v) = o := by
  cases o <;> rfl

/-- Value-functor composition — the `Option` instance of `haft.functor.laws`. -/
theorem value_fmap_comp (f : V → W) (g : W → X) (o : Option V) :
    o.map (fun v => g (f v)) = (o.map f).map g := by
  cases o <;> rfl

/-- The **total** functor map (`CausalEffect::map`): apply `f` to every `Option` value leaf,
    threading through the command tree — no error, no panic (the former arity-5 `.expect` is gone),
    commands map their sub-programs' leaves. -/
def map (f : V → W) : CEffect V → CEffect W
  | .pure o    => .pure (o.map f)
  | .relay t k => .relay t (map f k)

/-- `map` is total and uniform: `map id = id`. Lifts the `Option` identity law
    (`value_fmap_id` = `haft.functor.laws`) through the whole `RelayTo` tree — the map never
    branches on a case it cannot handle, so there is no reachable panic. -/
theorem map_id (m : CEffect V) : map (fun v => v) m = m := by
  induction m with
  | pure o => simp [map, value_fmap_id]
  | relay t k ih => simp [map, ih]

/-- `map` composition: `map (g ∘ f) = map g ∘ map f`. Lifts the `Option` composition law through the
    tree. -/
theorem map_comp (f : V → W) (g : W → X) (m : CEffect V) :
    map (fun v => g (f v)) m = map g (map f m) := by
  induction m with
  | pure o => simp [map, value_fmap_comp]
  | relay t k ih => simp [map, ih]

-- ------------------------------------------------------------------
-- `into_value` is the honest `Maybe` projection.
-- ------------------------------------------------------------------

/-- `CausalEffect::into_value`: `Pure(o) ↦ o`; a command `↦ None`. -/
def intoValue : CEffect V → Option V
  | .pure o    => o
  | .relay _ _ => Option.none

/-- A value effect projects to its scalar: `into_value (value v) = Some v`.

    THEOREM_MAP: `core.causal_effect.into_value` -/
theorem into_value_value (v : V) : intoValue (value v) = some v := rfl

/-- The `None` effect projects to `None`: `into_value none = None`.

    THEOREM_MAP: `core.causal_effect.into_value` -/
theorem into_value_none : intoValue (noneEffect : CEffect V) = Option.none := rfl

/-- A command projects to `None` (honest — a command carries no value):
    `into_value (RelayTo t k) = None`.

    THEOREM_MAP: `core.causal_effect.into_value` -/
theorem into_value_command (t : Nat) (k : CEffect V) :
    intoValue (.relay t k) = Option.none := rfl

/-- `from_option` round-trips through `into_value` on the `Pure` fragment — the projection loses
    nothing a value leaf carries. -/
theorem into_value_from_option (o : Option V) : intoValue (fromOption o) = o := rfl

end DeepCausalityFormal.Core.CausalEffect
