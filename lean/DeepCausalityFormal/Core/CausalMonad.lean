/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Core — the Causal Monad: the full five-channel carrier, post-P2.

Rust source: `deep_causality_core/src/types/causal_effect_propagation_process/mod.rs`
(`CausalEffectPropagationProcess`) and `src/traits/causal_monad/mod.rs` (`CausalMonad`).

With precondition P2 of the Causal Algebra program enforced
(`openspec/notes/causal-algebra/Formalization.md` §2; change `enforce-w-invariant`), the Rust
carrier encodes value-XOR-error as ONE channel — so the W-invariant (`error ⇒ no value`) holds
by construction and the invalid state the original walking skeleton had to exclude is
unrepresentable.

Post the `separate-control-channel` change the Rust carrier is
`outcome: Result<CausalEffect<Value>, Error>` where
`CausalEffect<V> = Free<CausalCommandWitness, Option<V>>` — i.e. the full transformer stack
`Except E (Free CausalCommand (Maybe V))`. This model is **congruent** with that carrier: it is
its restriction to the `Pure` fragment. `Free CausalCommand (Maybe V)` restricted to `Pure` **is**
`Option V`, so `Except E (Option V)` = the carrier whenever there is no command. Control (a
`RelayTo` jump) is the `Free`'s `Suspend` layer — a SECOND left zero of `bind`, interpreted by the
reasoning engine's `Free::fold` handler (laws from `haft.free_monad.*`) and NOT part of the value
monad-law surface. So the three monad laws below hold over the value fragment exactly as before;
`EPP = CausalMonad ⊕ CausalEffect`.

The model below transcribes the Rust carrier's value fragment channel-for-channel:
  * `outcome : Except E (Option V)` — the value-XOR-error channel; `Option` is the `Maybe` value
    content (`Pure(Some v)` = a value, `Pure(None)` = the `None` effect). The `Free`'s `Suspend`
    control layer is the free-monad extension, orthogonal to these laws.
  * `state : S` — the threaded Markovian state.
  * `ctx : Option C` — the read context threaded by `bind`.
  * `logs : List Λ` — the append-only audit log (Writer over the `List` monoid).

`bind'` transcribes the Rust `bind`: the `Err` arm returns the carrier reassembled verbatim
(the continuation is NOT invoked — raise is a left zero), the `Ok` arm runs the continuation
on (value, state, ctx) and prepends the incoming log. `eta` is the Kleisli unit of the
state-threading monad: it re-emits the received value, state, and context with an empty log
(the arity-5 lift `pure'`, which resets state to a caller-supplied initial value and clears
context, is the unit of the LIFT, not of the Kleisli triple — left identity is stated for it,
right identity for `eta`).

Theorems: left identity, right identity, associativity — all three now hold, right identity
UNCONDITIONALLY (the theorem `bind_right_id` closes the id `core.causal_monad.right_id`,
formerly blocked on P2, and `bind_assoc` closes `core.causal_monad.assoc`). Precondition P1 is now
resolved — control (`RelayTo`) is separated into `CausalCommand` / `CausalEffect` and is a lawful
free monad (`haft.free_monad.*`), no longer a non-lawful variant fused into the value type — so the
value monad here composes cleanly with it (`Except ∘ Free ∘ Maybe`, each layer already proved).

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witnesses: `deep_causality_core/tests/kani_proofs.rs` (Kani, bounded) and
`deep_causality_core/tests/types/causal_monad/causal_monad_tests.rs`.
-/

namespace DeepCausalityFormal.Core

/-- The five-channel causal carrier, transcribing
    `CausalEffectPropagationProcess<Value, State, Context, Error, Log>` post-P2. -/
structure Process (V S C E Λ : Type) where
  /-- Value-XOR-error: `Result<EffectValue<V>, E>` with `Option` as the value-or-absent
      content of `EffectValue`. -/
  outcome : Except E (Option V)
  state : S
  ctx : Option C
  logs : List Λ

variable {V W X S C E Λ : Type}

/-- The arity-5 lift (`CausalMonad::pure`): value in, caller-supplied initial state, no
    context, empty log. -/
def pure' (v : V) (s : S) : Process V S C E Λ :=
  { outcome := .ok (some v), state := s, ctx := none, logs := [] }

/-- The Kleisli unit of the state-threading monad: re-emit the received value, state, and
    context with an empty log. -/
def eta (v : Option V) (s : S) (c : Option C) : Process V S C E Λ :=
  { outcome := .ok v, state := s, ctx := c, logs := [] }

/-- State-threading `bind`, transcribing the Rust `bind`: the `Err` arm returns the carrier
    verbatim (continuation NOT invoked — left zero); the `Ok` arm runs the continuation on
    (value, state, ctx) and prepends the incoming log. -/
def bind' (m : Process V S C E Λ) (f : Option V → S → Option C → Process W S C E Λ) :
    Process W S C E Λ :=
  match m.outcome with
  | .error e => { outcome := .error e, state := m.state, ctx := m.ctx, logs := m.logs }
  | .ok v =>
    let n := f v m.state m.ctx
    { outcome := n.outcome, state := n.state, ctx := n.ctx, logs := m.logs ++ n.logs }

/-- Left identity: `bind (pure v s) f = f (some v) s none` — binding the lift is exactly the
    continuation applied at the injected point (Moggi 1991).

    THEOREM_MAP: `core.causal_monad.left_id` -/
theorem bind_left_id (v : V) (s : S) (f : Option V → S → Option C → Process W S C E Λ) :
    bind' (pure' v s) f = f (some v) s none := by
  simp [bind', pure']

/-- Right identity, UNCONDITIONAL: `bind m eta = m` for EVERY carrier — including errored
    ones, where `bind` returns the carrier verbatim. This is the law P2 unblocked: with
    value and error in one channel, no carrier state exists on which the law could fail
    (Formalization.md §2: "under W the right-identity law holds unconditionally").

    THEOREM_MAP: `core.causal_monad.right_id` -/
theorem bind_right_id (m : Process V S C E Λ) :
    bind' m eta = m := by
  cases m with
  | mk outcome state ctx logs =>
    cases outcome with
    | error e => rfl
    | ok v => simp [bind', eta]

/-- Associativity: `bind (bind m f) g = bind m (fun v s c => bind (f v s c) g)` — the log
    channel needs associativity of append (the Writer monoid law); every other channel is
    threaded identically on both sides (Moggi 1991).

    THEOREM_MAP: `core.causal_monad.assoc` -/
theorem bind_assoc (m : Process V S C E Λ)
    (f : Option V → S → Option C → Process W S C E Λ)
    (g : Option W → S → Option C → Process X S C E Λ) :
    bind' (bind' m f) g = bind' m (fun v s c => bind' (f v s c) g) := by
  cases hm : m.outcome with
  | error e => simp [bind', hm]
  | ok v =>
    cases hn : (f v m.state m.ctx).outcome with
    | error e => simp [bind', hm, hn]
    | ok w => simp [bind', hm, hn, List.append_assoc]

/-- Raise is a left zero: an errored carrier short-circuits `bind` — the continuation is
    never consulted and error, state, context, and logs survive verbatim. In Rust this is
    the `Err` arm returning `self` reassembled; pinned by the Kani harness
    `causal_monad_short_circuit` and the witness tests.

    THEOREM_MAP: `core.causal_monad.left_zero` -/
theorem bind_raise_left_zero (e : E) (s : S) (c : Option C) (l : List Λ)
    (f : Option V → S → Option C → Process W S C E Λ) :
    bind' { outcome := .error e, state := s, ctx := c, logs := l } f
      = { outcome := .error e, state := s, ctx := c, logs := l } := rfl

end DeepCausalityFormal.Core
