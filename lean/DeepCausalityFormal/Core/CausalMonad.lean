/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Exemplar 2 — left identity of the Causal Monad `bind` (Core layer).

This walking skeleton models a MINIMAL, cleaned carrier: value + Markovian state + an
append-only log. Context and error are deliberately dropped here — neither participates in the
left-identity law, and modelling them requires the (P1) control-free and (P2) W-invariant fixes
that are the first work item AFTER this skeleton (see
`openspec/notes/causal-algebra/Formalization.md`, §2 and items 1-2). The full arity-5
`LawfulMonad` model is scaling work (items 4-5).

`pure'` / `bind'` mirror the Rust semantics in
`deep_causality_core/src/types/causal_effect_propagation_process/mod.rs`:
`bind` runs the continuation on the carried value and threaded state, then prepends the
incoming log to the continuation's log.

This file needs no Mathlib — it is pure core Lean (no `Type*`, no imported lemmas): the law holds
by definitional reduction (`[] ++ l = l` and structure eta), so the proof is `rfl`.

Rust witness: `deep_causality_core/tests/kani_proofs.rs :: causal_monad_left_identity`.
-/

namespace DeepCausalityFormal.Core

/-- Minimal model of the cleaned causal carrier for the left-identity skeleton. -/
structure Process (V S L : Type) where
  value : V
  state : S
  logs  : List L

variable {V W S L : Type}

/-- `pure`: inject the value, take an initial state from the caller (the Rust `pure` uses
    `State::default()`), and start with an empty log. -/
def pure' (v : V) (s : S) : Process V S L :=
  { value := v, state := s, logs := [] }

/-- State-threading `bind`: run `f` on the carried value and threaded state, then prepend the
    incoming log to the continuation's log (log accumulation across the step). -/
def bind' (m : Process V S L) (f : V → S → Process W S L) : Process W S L :=
  let n := f m.value m.state
  { n with logs := m.logs ++ n.logs }

/-- Left identity: `pure a >>= f = f a`.

    Because `pure'` starts with an empty log, the log-prepend in `bind'` is a no-op
    (`[] ++ l` reduces to `l`), so the result is `f` applied at the injected value and state,
    up to structure eta — both definitional, hence `rfl`. Mirrors the Kani harness
    `causal_monad_left_identity`.

    THEOREM_MAP: `core.causal_monad.left_id` -/
theorem bind_left_id (v : V) (s : S) (f : V → S → Process W S L) :
    bind' (pure' v s) f = f v s :=
  rfl

end DeepCausalityFormal.Core
