/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Core — Alternatable: the value/state/context setter family as lenses up-to-log.

Rust source: `deep_causality_core/src/types/causal_effect_propagation_process/`
(`alternatable_value.rs`, `alternatable_state.rs`, `alternatable_context.rs` — the latter also
carries `clear_context`) and the traits under `src/traits/alternatable*`.

Layering: the lens concept is Foster et al., *Combinators for Bidirectional Tree Transformations*,
POPL 2005. The causal setters are lenses on the (value | state | context) channels **up to the
audit log**: every successful `alternate_*` appends one `!!…Alternation!!` / `!!ContextCleared!!`
entry (deviation D9, an accepted Writer property — the log is a monotone side-output, never
overwritten). This file proves the laws that actually hold for these setters: set-get, set-set
(up-to-log, on the log-erasing projection `proj = (outcome, state, ctx)`), channel independence, and
error no-op; a companion lemma shows the full carrier grows the log (so set-set is up-to-log, not
on-the-nose). No get-set (put-get / GetPut) law is claimed — the value setter always produces
`.ok (some v)` and so cannot restore an `.ok none` carrier, so these are NOT full very-well-behaved
lenses. Every setter is a **no-op on an errored carrier** (an alternation cannot repair a broken
chain — the Rust early-returns `self`).

`clear_context` is the `None`-setting counterpart `alternate_context` (codomain `Some _`) lacked.
The Pearl do-operator (D8) is NOT formalized here: intervention on the causal *hypergraph* belongs
to the `deep_causality` layer; these lenses only substitute a channel of one carrier.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witnesses: `deep_causality_core/tests/formalization_lean/alternatable_tests.rs`.
-/

namespace DeepCausalityFormal.Core.Alternatable

/-- The causal carrier's value fragment (as in `CausalMonad.lean`); `Λ` is an audit-log entry. -/
structure Carrier (V S C E Λ : Type) where
  outcome : Except E (Option V)
  state : S
  ctx : Option C
  logs : List Λ

variable {V S C E Λ : Type}

/-- `alternate_value`: on a non-errored carrier, replace the value with `Pure(Some new)` and append
    the `!!ValueAlternation!!` entry `a`; a no-op on an errored carrier. -/
def alternateValue (a : Λ) (new : V) (m : Carrier V S C E Λ) : Carrier V S C E Λ :=
  match m.outcome with
  | .error _ => m
  | .ok _    => { m with outcome := .ok (some new), logs := m.logs ++ [a] }

/-- `alternate_state`: on a non-errored carrier, replace the state and append `!!StateAlternation!!`;
    no-op on error. -/
def alternateState (a : Λ) (new : S) (m : Carrier V S C E Λ) : Carrier V S C E Λ :=
  match m.outcome with
  | .error _ => m
  | .ok _    => { m with state := new, logs := m.logs ++ [a] }

/-- `alternate_context`: on a non-errored carrier, set the context to `Some new` and append
    `!!ContextAlternation!!`; no-op on error. -/
def alternateContext (a : Λ) (new : C) (m : Carrier V S C E Λ) : Carrier V S C E Λ :=
  match m.outcome with
  | .error _ => m
  | .ok _    => { m with ctx := some new, logs := m.logs ++ [a] }

/-- `clear_context`: on a non-errored carrier, set the context to `None` and append
    `!!ContextCleared!!`; no-op on error. -/
def clearContext (a : Λ) (m : Carrier V S C E Λ) : Carrier V S C E Λ :=
  match m.outcome with
  | .error _ => m
  | .ok _    => { m with ctx := none, logs := m.logs ++ [a] }

/-- The audit-log-erasing projection the lens laws hold on. -/
def proj (m : Carrier V S C E Λ) : Except E (Option V) × S × Option C :=
  (m.outcome, m.state, m.ctx)

-- ------------------------------------------------------------------
-- Set-get: after a successful set, the projected channel is the set value.
-- ------------------------------------------------------------------

/-- Set-get (value): on a non-errored carrier, reading back the value yields exactly what was set.

    THEOREM_MAP: `core.alternatable.set_get` -/
theorem set_get_value (a : Λ) (new : V) (m : Carrier V S C E Λ) (o : Option V)
    (h : m.outcome = .ok o) : (alternateValue a new m).outcome = .ok (some new) := by
  simp [alternateValue, h]

/-- Set-get (state). THEOREM_MAP: `core.alternatable.set_get` -/
theorem set_get_state (a : Λ) (new : S) (m : Carrier V S C E Λ) (o : Option V)
    (h : m.outcome = .ok o) : (alternateState a new m).state = new := by
  simp [alternateState, h]

/-- Set-get (context). THEOREM_MAP: `core.alternatable.set_get` -/
theorem set_get_context (a : Λ) (new : C) (m : Carrier V S C E Λ) (o : Option V)
    (h : m.outcome = .ok o) : (alternateContext a new m).ctx = some new := by
  simp [alternateContext, h]

-- ------------------------------------------------------------------
-- Set-set idempotence, on the projection (the second write wins; the log grows — see below).
-- ------------------------------------------------------------------

/-- Set-set (value), projected: the second write wins — `proj` erases the two accumulated entries.

    THEOREM_MAP: `core.alternatable.set_set_proj` -/
theorem set_set_value_proj (a b : Λ) (v1 v2 : V) (m : Carrier V S C E Λ) :
    proj (alternateValue b v2 (alternateValue a v1 m)) = proj (alternateValue b v2 m) := by
  cases h : m.outcome <;> simp [alternateValue, proj, h]

/-- Set-set (state), projected. THEOREM_MAP: `core.alternatable.set_set_proj` -/
theorem set_set_state_proj (a b : Λ) (s1 s2 : S) (m : Carrier V S C E Λ) :
    proj (alternateState b s2 (alternateState a s1 m)) = proj (alternateState b s2 m) := by
  cases h : m.outcome <;> simp [alternateState, proj, h]

/-- Set-set (context), projected: the second context write wins — `proj` erases the two accumulated
    entries.

    THEOREM_MAP: `core.alternatable.set_set_proj` -/
theorem set_set_context_proj (a b : Λ) (c1 c2 : C) (m : Carrier V S C E Λ) :
    proj (alternateContext b c2 (alternateContext a c1 m)) = proj (alternateContext b c2 m) := by
  cases h : m.outcome <;> simp [alternateContext, proj, h]

-- ------------------------------------------------------------------
-- Channel independence: each setter touches only its own channel.
-- ------------------------------------------------------------------

/-- `alternate_value` leaves state and context untouched.

    THEOREM_MAP: `core.alternatable.channel_independence` -/
theorem value_preserves_state_ctx (a : Λ) (new : V) (m : Carrier V S C E Λ) :
    (alternateValue a new m).state = m.state ∧ (alternateValue a new m).ctx = m.ctx := by
  cases h : m.outcome <;> simp [alternateValue, h]

/-- `alternate_state` leaves the value outcome and context untouched.

    THEOREM_MAP: `core.alternatable.channel_independence` -/
theorem state_preserves_value_ctx (a : Λ) (new : S) (m : Carrier V S C E Λ) :
    (alternateState a new m).outcome = m.outcome ∧ (alternateState a new m).ctx = m.ctx := by
  cases h : m.outcome <;> simp [alternateState, h]

/-- `alternate_context` leaves the value outcome and state untouched.

    THEOREM_MAP: `core.alternatable.channel_independence` -/
theorem context_preserves_value_state (a : Λ) (new : C) (m : Carrier V S C E Λ) :
    (alternateContext a new m).outcome = m.outcome ∧ (alternateContext a new m).state = m.state := by
  cases h : m.outcome <;> simp [alternateContext, h]

-- ------------------------------------------------------------------
-- Error no-op: every setter is the identity on an errored carrier.
-- ------------------------------------------------------------------

/-- Error no-op (value). THEOREM_MAP: `core.alternatable.error_noop` -/
theorem value_error_noop (a : Λ) (new : V) (m : Carrier V S C E Λ) (e : E)
    (h : m.outcome = .error e) : alternateValue a new m = m := by
  simp [alternateValue, h]

/-- Error no-op (state). THEOREM_MAP: `core.alternatable.error_noop` -/
theorem state_error_noop (a : Λ) (new : S) (m : Carrier V S C E Λ) (e : E)
    (h : m.outcome = .error e) : alternateState a new m = m := by
  simp [alternateState, h]

/-- Error no-op (context). THEOREM_MAP: `core.alternatable.error_noop` -/
theorem context_error_noop (a : Λ) (new : C) (m : Carrier V S C E Λ) (e : E)
    (h : m.outcome = .error e) : alternateContext a new m = m := by
  simp [alternateContext, h]

-- ------------------------------------------------------------------
-- The up-to-log caveat (D9): the FULL carrier's set-set grows the audit log, so the lens laws are
-- up-to-log, not on-the-nose — the honest reason `proj` is needed above.
-- ------------------------------------------------------------------

/-- Set-set on the full carrier is NOT idempotent: the log grows by exactly one entry per write, so
    two writes leave a strictly longer log than one. This is the deliberate Writer side-output (D9)
    that makes the on-the-nose lens law false and the `proj` law necessary. -/
theorem set_set_grows_log (a b : Λ) (v1 v2 : V) (m : Carrier V S C E Λ) (o : Option V)
    (h : m.outcome = .ok o) :
    (alternateValue b v2 (alternateValue a v1 m)).logs.length
      = (alternateValue b v2 m).logs.length + 1 := by
  simp [alternateValue, h]

-- ------------------------------------------------------------------
-- clear_context: the None-setting counterpart, no-op on error.
-- ------------------------------------------------------------------

/-- `clear_context` sets the context to `None` on a non-errored carrier and records the substitution
    (`!!ContextCleared!!`) — the counterpart `alternate_context` (codomain `Some _`) lacked. -/
theorem clear_context_sets_none (a : Λ) (m : Carrier V S C E Λ) (o : Option V)
    (h : m.outcome = .ok o) : (clearContext a m).ctx = none := by
  simp [clearContext, h]

/-- `clear_context` is a no-op on an errored carrier. -/
theorem clear_context_error_noop (a : Λ) (m : Carrier V S C E Λ) (e : E)
    (h : m.outcome = .error e) : clearContext a m = m := by
  simp [clearContext, h]

end DeepCausalityFormal.Core.Alternatable
