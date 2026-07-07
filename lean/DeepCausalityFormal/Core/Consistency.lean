/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Core — Consistency: every law-bearing HKT instance over the success channel is the SAME total
function — `fmap` and `apply` agree across all three witnesses and the inherent map, TOTALLY, and
never fabricate an error on the success channel.

Rust source — the three witnesses plus the inherent functor, ALL of which now compute the same
success-channel functor/applicative:
  * `causal_effect_propagation_process/hkt.rs`  — `CausalEffectPropagationProcessWitness` (generic),
  * `propagating_effect/hkt.rs`                 — `PropagatingEffectWitness`   (concrete),
  * `propagating_process/hkt.rs`                — `PropagatingProcessWitness`  (concrete),
  * `causal_effect_propagation_process/mod.rs`  — `CausalEffectPropagationProcess::fmap` (inherent),
  * `causal_flow/steps.rs`                      — `CausalFlow::map` (the same `effect.map(f)`).
`fmap` delegates to the **total** `CausalEffect::map` (`causal_effect/mod.rs`); `apply` yields a value
iff both operands carry one, else absence (`none()`).

The value functor is total over the whole success channel `CausalEffect<V> = Free CausalCommand
(Option V)` — a value, an absence, or a **command** (`RelayTo`). `CausalEffect::map` maps the single
`Option` value leaf through the `RelayTo` tree, so:
  * `Pure(Some v) ↦ Pure(Some (f v))`,
  * `Pure(None)   ↦ Pure(None)`,
  * `RelayTo(t, k) ↦ RelayTo(t, map f k)` — the **command is preserved**, its leaf mapped.

Every witness `fmap` and the inherent `fmap` are `Err e ↦ Err e` on the error channel and
`Ok eff ↦ Ok (map f eff)` on success — i.e. **the same function** (`Except.map (CausalEffect.map f)`).
`apply` is likewise `Err`-short-circuit on the error channel and `apMaybe` on success (value iff both
operands are values, else `Pure None`). Deviation D15 is FULLY retired: the `fmap`/`apply` surfaces
that once diverged — one **panicking** via `.expect`, the two concrete witnesses collapsing a
value-less carrier to an `InternalLogicError`, the fluent map to a `ValueNotAvailable` error — now all
compute this one total function. This file proves the witness and inherent maps agree on EVERY
carrier, including absence and commands (no `Ok(Value _)`-only restriction, no panic, no
`InternalLogicError`), and that the functor **preserves commands** rather than erasing them.

(The deliberately *strict* fluent Kleisli steps `CausalFlow::and_then` and `bind_or_error` — which
block on a missing value and surface it as an error — are a separate convenience layer, NOT these
law-bearing functor/applicative instances, and are out of scope of this agreement.)

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witnesses: `deep_causality_core/tests/formalization_lean/consistency_tests.rs` and
`deep_causality_core/tests/iso/effect_process_consistency_tests.rs`.
-/

namespace DeepCausalityFormal.Core.Consistency

variable {V W E : Type}

/-- `CausalEffect<V> = Free CausalCommand (Option V)`: `Option` value leaves and `RelayTo` control
    nodes (the `relay` node is `Suspend(RelayTo(target, sub))`), as in `Core/CausalEffect.lean`. -/
inductive CEffect (V : Type) where
  | pure  : Option V → CEffect V
  | relay : Nat → CEffect V → CEffect V

/-- The **total** value functor (`CausalEffect::map`): map the single `Option` value leaf through the
    `RelayTo` tree; a command is preserved with its target intact. -/
def mapEffect (f : V → W) : CEffect V → CEffect W
  | .pure o    => .pure (o.map f)
  | .relay t k => .relay t (mapEffect f k)

/-- The success-or-error carrier (`outcome : Result<CausalEffect<V>, E>`). -/
abbrev Carrier (E V : Type) := Except E (CEffect V)

/-- The **process witness** `fmap` (`hkt.rs`): error short-circuits; otherwise map the effect with the
    total `CausalEffect::map`. -/
def witnessFmap (f : V → W) : Carrier E V → Carrier E W
  | .error e => .error e
  | .ok eff  => .ok (mapEffect f eff)

/-- The **inherent** `fmap` (`mod.rs` / `CausalFlow::map`): error short-circuits; otherwise map the
    effect with the total `CausalEffect::map`. Post-refactor this is the *same* function as the
    witness. -/
def inherentFmap (f : V → W) : Carrier E V → Carrier E W
  | .error e => .error e
  | .ok eff  => .ok (mapEffect f eff)

/-- The witness and inherent functors coincide on EVERY carrier — value (`Some`), absence (`None`),
    error (`Err`), AND command (`RelayTo`). No `Ok(Value _)`-only restriction, no reachable panic,
    no `None`-collapse: the retirement of D15.

    THEOREM_MAP: `core.witness.agree` -/
theorem witness_agree (f : V → W) (m : Carrier E V) :
    witnessFmap f m = inherentFmap f m := by
  cases m with
  | error e => rfl
  | ok eff => rfl

/-- Totality over commands: mapping a command carrier **preserves the command** (same target,
    sub-program leaf mapped) — it is neither collapsed to `None` nor turned into an error. This is the
    concrete content of the D15 fix. -/
theorem fmap_preserves_command (f : V → W) (t : Nat) (k : CEffect V) :
    (witnessFmap f (.ok (.relay t k)) : Carrier E W) = .ok (.relay t (mapEffect f k)) := rfl

/-- On the `Pure` value fragment the maps behave exactly as the pre-command functor did — a value
    maps its leaf and a `None` passes through — so the earlier value-fragment agreement is subsumed. -/
theorem fmap_value_fragment (f : V → W) (o : Option V) :
    (witnessFmap f (.ok (.pure o)) : Carrier E W) = .ok (.pure (o.map f)) := rfl

-- ------------------------------------------------------------------
-- Applicative `apply`: total over the success channel, value-less collapses to absence (NOT error).
-- ------------------------------------------------------------------

/-- The success-channel applicative combine (`apply` on two `Ok` carriers): a value iff both operands
    carry one, else absence. Value-less operands (a `None`, or a command the engine folds first) yield
    `Pure None` — never a fabricated error. -/
def apMaybe : CEffect (V → W) → CEffect V → CEffect W
  | .pure (some f), .pure (some a) => .pure (some (f a))
  | _,              _              => .pure Option.none

/-- The `apply` all three witnesses (`propagating_effect/hkt.rs`, `propagating_process/hkt.rs`,
    `causal_effect_propagation_process/hkt.rs`) compute — they share ONE definition (`apply` lives
    only on the witnesses; there is no inherent `apply` method): first error wins on the error
    channel; otherwise the total `apMaybe`. -/
def applyCarrier : Carrier E (V → W) → Carrier E V → Carrier E W
  | .error e, _         => .error e
  | _,        .error e  => .error e
  | .ok ef,   .ok ev    => .ok (apMaybe ef ev)

/-- Totality of `apply` over absence: a value-less operand yields absence (`Pure None`), NOT an error
    — the concrete content of retiring the `InternalLogicError` collapse the concrete witnesses used
    to do. -/
theorem apply_none_yields_none (f : CEffect (V → W)) :
    (applyCarrier (.ok f) (.ok (.pure Option.none)) : Carrier E W) = .ok (.pure Option.none) := by
  cases f with
  | pure o => cases o <;> rfl
  | relay _ _ => rfl

/-- Totality of `apply` over a command operand: a command in either position yields absence, never an
    error (commands are folded by `CausalEffect::fold`, not applied). -/
theorem apply_command_yields_none (t : Nat) (k : CEffect V) (f : CEffect (V → W)) :
    (applyCarrier (.ok f) (.ok (.relay t k)) : Carrier E W) = .ok (.pure Option.none) := by
  cases f with
  | pure o => cases o <;> rfl
  | relay _ _ => rfl

/-- `apply` short-circuits the error channel (left zero): a function-side error propagates regardless
    of the argument — the applicative counterpart of the functor's `Err ↦ Err`. -/
theorem apply_error_short_circuits (e : E) (ma : Carrier E V) :
    (applyCarrier (.error e) ma : Carrier E W) = .error e := by
  cases ma <;> rfl

end DeepCausalityFormal.Core.Consistency
