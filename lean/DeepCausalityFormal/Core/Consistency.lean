/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Core — Consistency: the HKT witness `fmap` agrees with the inherent `fmap`, TOTALLY over commands.

Rust source: `deep_causality_core/src/types/causal_effect_propagation_process/hkt.rs`
(`CausalEffectPropagationProcessWitness::fmap` — the process witness) and
`src/types/causal_effect_propagation_process/mod.rs` (`CausalEffectPropagationProcess::fmap` — the
inherent functor), both of which now delegate to the **total** `CausalEffect::map`
(`src/types/causal_effect/mod.rs`), and `src/types/causal_flow/steps.rs` (`CausalFlow::map`, the same
`effect.map(f)` under `bind`).

The value functor is total over the whole success channel `CausalEffect<V> = Free CausalCommand
(Option V)` — a value, an absence, or a **command** (`RelayTo`). `CausalEffect::map` maps the single
`Option` value leaf through the `RelayTo` tree, so:
  * `Pure(Some v) ↦ Pure(Some (f v))`,
  * `Pure(None)   ↦ Pure(None)`,
  * `RelayTo(t, k) ↦ RelayTo(t, map f k)` — the **command is preserved**, its leaf mapped.

Both the process witness `fmap` and the inherent `fmap` are `Err e ↦ Err e` on the error channel and
`Ok eff ↦ Ok (map f eff)` on success — i.e. **the same function** (`Except.map (CausalEffect.map f)`).
Deviation D15 is fully retired: the four `fmap`s that once diverged — one **panicking** via `.expect`,
others collapsing a command to `None` or a `ValueNotAvailable` error — are gone. This file proves the
witness and inherent maps agree on EVERY carrier, including commands (no `Ok(Value _)`-only
restriction, no panic, no `None`-collapse), and that the functor **preserves commands** rather than
erasing them.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witnesses: `deep_causality_core/tests/formalization_lean/consistency_tests.rs`.
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

end DeepCausalityFormal.Core.Consistency
