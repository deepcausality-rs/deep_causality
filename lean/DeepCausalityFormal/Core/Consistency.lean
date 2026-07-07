/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Core — Consistency: the HKT witness `fmap` agrees with the inherent `fmap`.

Rust source: `deep_causality_core/src/types/causal_effect_propagation_process/hkt.rs`
(`CausalEffectPropagationProcessWitness::fmap` — the process witness) and
`src/types/causal_flow/steps.rs` (`CausalFlow::map` — the inherent value map) over
`CausalEffect::{map,from_option,into_value}` (`src/types/causal_effect/mod.rs` — the total value
functor, `Core/CausalEffect.lean`).

The value functor acts on the value-XOR-error fragment `Except E (Option V)` — the `Pure` leaves of
the carrier, since the reasoning engine folds every command (`RelayTo`) *before* the value functor
is reached (`CausalFlow::map`/the witness never run on a live command). Over that fragment both maps
are the same function:
  * process witness (`hkt.rs`): `Err e ↦ Err e`; `Ok effect ↦` (`into_value`: `Some v ↦ Ok(value (f v))`,
    `None ↦ Ok none`),
  * inherent (`CausalFlow::map`): `Err e ↦ Err e`; `Ok o ↦ Ok(from_option (o.map f))`.

Deviation D15 is retired: the four `fmap`s that formerly diverged — one of them **panicking** via
`.expect` on the arity-5 witness — are gone. With control lifted out of the value channel
(`separate-control-channel`, landed), the map is total on `Some`/`None`/`Err` alike (no
`Ok(Value _)`-only restriction) and there is **no reachable panic**: this file proves the two
transcribed maps are equal on every carrier of the fragment. The total `CausalEffect::map` that lifts
this action through the command tree is proved total (`map_id`) in `Core/CausalEffect.lean`.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witnesses: `deep_causality_core/tests/formalization_lean/consistency_tests.rs`.
-/

namespace DeepCausalityFormal.Core.Consistency

variable {V W E : Type}

/-- The value-XOR-error fragment the value functor acts on — the `Pure` leaves of the carrier
    (`Except E (Option V)`), commands having been folded first. -/
abbrev Carrier (E V : Type) := Except E (Option V)

/-- The **process witness** `fmap` (`hkt.rs`), transcribed on the `Pure` fragment: error
    short-circuits (left zero, `f` not invoked); `into_value` splits the value case from the `None`
    case, rebuilding `value`/`none`. No panic, no manufactured error. -/
def witnessFmap (f : V → W) : Carrier E V → Carrier E W
  | .error e       => .error e
  | .ok (some v)   => .ok (some (f v))
  | .ok Option.none => .ok Option.none

/-- The **inherent** `fmap` (`CausalFlow::map`), transcribed on the `Pure` fragment:
    `Ok(from_option (into_value ().map f))` = `Ok (o.map f)`; error short-circuits. -/
def inherentFmap (f : V → W) : Carrier E V → Carrier E W
  | .error e => .error e
  | .ok o    => .ok (o.map f)

/-- The witness and inherent functors coincide on EVERY carrier of the fragment — value (`Some`),
    absence (`None`), and error (`Err`) — with no `Ok(Value _)`-only restriction and no reachable
    panic. This is the retirement of D15 (the former four-way `fmap` divergence + `.expect` panic).

    THEOREM_MAP: `core.witness.agree` -/
theorem witness_agree (f : V → W) (m : Carrier E V) :
    witnessFmap f m = inherentFmap f m := by
  cases m with
  | error e => rfl
  | ok o => cases o <;> rfl

end DeepCausalityFormal.Core.Consistency
