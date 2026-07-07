/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Core — CausalFlow: the fluent facade over the causal monad, and its documented extensions.

Rust source: `deep_causality_core/src/types/causal_flow/` (`mod.rs` — the `CausalFlow(inner)`
newtype; `steps.rs` — `map`/`and_then`; `terminals.rs` — `finish`; `iterate.rs` —
`iterate_n`/`iterate_until`/`iterate_to_fixpoint`; `steps.rs` — `recover`).

Layering: `CausalFlow<V,S,C>` is a **newtype** over `PropagatingProcess<V,S,C>` (the causal monad,
`Core/CausalMonad.lean`). The facade *lowers* to the monad — it adds no new algebra — so:
  * the `≅ Process` iso is the newtype wrap/unwrap (`rfl`);
  * `map`/`and_then` are the monad's `fmap`/`bind` with the boilerplate hidden, and the corrected
    law `map f = and_then (pure ∘ f)` now holds on the `None` effect as well as a value (D14: `map`
    is the total value functor, not a partial one).
Operations that EXCEED the base monad are formalized as **documented extensions** with their own
stated contracts, not as monad sugar: `recover` (`MonadError.catch`), the bounded `iterate*`
combinators (`MaxStepsExceeded` on budget exhaustion — D11/D12), and `finish` (a value-observation
terminal that deliberately drops state/context/log).

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witnesses: `deep_causality_core/tests/formalization_lean/causal_flow_tests.rs`.
-/

namespace DeepCausalityFormal.Core.CausalFlow

/-- The causal carrier's value fragment (as in `CausalMonad.lean`). -/
structure Process (V S C E Λ : Type) where
  outcome : Except E (Option V)
  state : S
  ctx : Option C
  logs : List Λ

/-- The `CausalFlow` newtype — a single-field wrapper over the process. -/
structure Flow (V S C E Λ : Type) where
  inner : Process V S C E Λ

variable {V W X S C E Λ : Type}

-- ------------------------------------------------------------------
-- flow_iso: CausalFlow ≅ Process (the newtype wrap/unwrap).
-- ------------------------------------------------------------------

/-- `CausalFlow::from` (wrap) and `into_process` (unwrap). -/
def fromProcess (p : Process V S C E Λ) : Flow V S C E Λ := ⟨p⟩
def intoProcess (fl : Flow V S C E Λ) : Process V S C E Λ := fl.inner

/-- The facade lowers faithfully: wrap-then-unwrap and unwrap-then-wrap are both the identity.

    THEOREM_MAP: `core.causal_flow.flow_iso` -/
theorem flow_iso (p : Process V S C E Λ) (fl : Flow V S C E Λ) :
    intoProcess (fromProcess p) = p ∧ fromProcess (intoProcess fl) = fl :=
  ⟨rfl, rfl⟩

-- ------------------------------------------------------------------
-- map (the total value functor) and its laws.
-- ------------------------------------------------------------------

/-- `CausalFlow::map`, lowered to the value fragment: map the `Option` leaf through `Except`
    (`Ok(o) ↦ Ok(o.map f)`, `Err ↦ Err`); state/context/logs preserved (`bind` threads them and
    the fresh step-log is empty). Total on `Some`/`None`/`Err` — no partial functor. -/
def mapProc (f : V → W) (p : Process V S C E Λ) : Process W S C E Λ :=
  { outcome := p.outcome.map (fun o => o.map f), state := p.state, ctx := p.ctx, logs := p.logs }

/-- Functor identity: `map id = id`. THEOREM_MAP: `core.causal_flow.map_id` -/
theorem map_id (p : Process V S C E Λ) : mapProc (fun v => v) p = p := by
  obtain ⟨outcome, state, ctx, logs⟩ := p
  cases outcome with
  | error e => rfl
  | ok o => cases o <;> rfl

/-- Functor composition: `map (g ∘ f) = map g ∘ map f`.
    THEOREM_MAP: `core.causal_flow.map_comp` -/
theorem map_comp (f : V → W) (g : W → X) (p : Process V S C E Λ) :
    mapProc (fun v => g (f v)) p = mapProc g (mapProc f p) := by
  obtain ⟨outcome, state, ctx, logs⟩ := p
  cases outcome with
  | error e => rfl
  | ok o => cases o <;> rfl

/-- `and_then (pure ∘ f)`, lowered: run the value continuation on a present value, pass a `None`
    effect through unchanged, short-circuit on error. -/
def andThenPure (f : V → W) (p : Process V S C E Λ) : Process W S C E Λ :=
  match p.outcome with
  | .error e     => { outcome := .error e, state := p.state, ctx := p.ctx, logs := p.logs }
  | .ok (some v) => { outcome := .ok (some (f v)), state := p.state, ctx := p.ctx, logs := p.logs }
  | .ok none     => { outcome := .ok none, state := p.state, ctx := p.ctx, logs := p.logs }

/-- The corrected law `map f = and_then (pure ∘ f)` — holding on the `None` effect as well as a
    value (D14: the two agree on every carrier of the fragment, not just `Ok(Value _)`).

    THEOREM_MAP: `core.causal_flow.map_eq_andThen` -/
theorem map_eq_andThen (f : V → W) (p : Process V S C E Λ) :
    mapProc f p = andThenPure f p := by
  obtain ⟨outcome, state, ctx, logs⟩ := p
  cases outcome with
  | error e => rfl
  | ok o => cases o <;> rfl

-- ------------------------------------------------------------------
-- Documented extension: recover (MonadError.catch).
-- ------------------------------------------------------------------

/-- `CausalFlow::recover`: turn the error channel back into a value (`Err e ↦ Ok(value (h e))`); a
    no-op on a successful flow. -/
def recover (h : E → V) (p : Process V S C E Λ) : Process V S C E Λ :=
  match p.outcome with
  | .error e => { outcome := .ok (some (h e)), state := p.state, ctx := p.ctx, logs := p.logs }
  | .ok _    => p

/-- Catch law: `recover` is a no-op on a successful flow, and maps a raise to the handler's value.

    THEOREM_MAP: `core.causal_flow.recover` -/
theorem recover_catch (h : E → V) (p : Process V S C E Λ) (e : E)
    (hp : p.outcome = .error e) :
    recover h p = { outcome := .ok (some (h e)), state := p.state, ctx := p.ctx, logs := p.logs }
      ∧ (∀ (o : Option V), p.outcome = .ok o → recover h p = p) := by
  refine ⟨by simp [recover, hp], ?_⟩
  intro o ho
  simp [recover, ho]

-- ------------------------------------------------------------------
-- Documented extension: bounded iterate (MaxStepsExceeded contract).
-- ------------------------------------------------------------------

/-- Apply a flow endomorphism `step` exactly `n` times (`iterate_n`) — structurally terminating. -/
def iterateN (step : Process V S C E Λ → Process V S C E Λ) :
    Nat → Process V S C E Λ → Process V S C E Λ
  | 0, p => p
  | (n + 1), p => iterateN step n (step p)

/-- `fail_not_converged`: replace the flow with `MaxStepsExceeded` (`maxErr`) on budget exhaustion,
    preserving state, context, and logs. -/
def failNotConverged (maxErr : E) (p : Process V S C E Λ) : Process V S C E Λ :=
  { outcome := .error maxErr, state := p.state, ctx := p.ctx, logs := p.logs }

/-- The iterate contract: bounded search terminates (`iterateN` is total; `iterateN 0 = id`), and on
    budget exhaustion `iterate_until`/`iterate_to_fixpoint` inject `MaxStepsExceeded` while
    preserving state/context/logs.

    THEOREM_MAP: `core.causal_flow.iterate` -/
theorem iterate_contract (maxErr : E) (step : Process V S C E Λ → Process V S C E Λ)
    (p : Process V S C E Λ) :
    iterateN step 0 p = p
      ∧ (failNotConverged maxErr p).outcome = .error maxErr
      ∧ (failNotConverged maxErr p).state = p.state
      ∧ (failNotConverged maxErr p).ctx = p.ctx
      ∧ (failNotConverged maxErr p).logs = p.logs := by
  refine ⟨rfl, ?_, ?_, ?_, ?_⟩ <;> simp [failNotConverged]

-- ------------------------------------------------------------------
-- Documented extension: finish (value-observation terminal that drops state/context/log).
-- ------------------------------------------------------------------

/-- `CausalFlow::finish`: observe the value or the error, dropping state/context/log
    (`outcome.and_then(into_value)`; a value-less `Ok(None)` becomes `ValueNotAvailable`). -/
def finish (valErr : E) (p : Process V S C E Λ) : Except E V :=
  match p.outcome with
  | .error e     => .error e
  | .ok (some v) => .ok v
  | .ok none     => .error valErr

/-- `finish` depends ONLY on the outcome channel — it drops state, context, and log (two carriers
    with the same outcome finish identically, whatever their state/context/log).

    THEOREM_MAP: `core.causal_flow.finish` -/
theorem finish_drops_state_ctx_log (valErr : E) (p q : Process V S C E Λ)
    (h : p.outcome = q.outcome) : finish valErr p = finish valErr q := by
  simp [finish, h]

end DeepCausalityFormal.Core.CausalFlow
