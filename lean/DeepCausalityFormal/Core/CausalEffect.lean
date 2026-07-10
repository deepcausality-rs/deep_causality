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

-- ------------------------------------------------------------------
-- The transformer stack: `Outcome V = Except E (Free CausalCommand (Maybe V))` is a lawful monad.
--
-- Textbook definition: a monad transformer stack composes monads so the composite is again a
-- monad; `ExceptT` adds a left-zero failure layer, a free monad adds an operation layer whose
-- `bind` threads under the operation nodes functorially, and `Maybe` adds a local absence zero
-- (Moggi, "Notions of Computation and Monads", Inf. and Comput. 93(1), 1991; Liang, Hudak & Jones,
-- "Monad Transformers and Modular Interpreters", POPL 1995). The three layers are each proved
-- (`haft.effect3.monad_laws` for Except, `haft.free_monad.*` for Free, `haft.functor.laws`/Maybe);
-- this section proves the COMPOSITE is lawful — the value-Kleisli `obind` that dispatches through
-- all three layers satisfies the monad laws plus the two zeros.
--
-- DEVIATION NOTE: the composite is transcribed at the concrete operation (`RelayTo`, single-hole)
-- rather than via a generic distributive law; the error layer is OUTERMOST (an error inside a
-- relayed sub-program aborts the whole outcome — matching the engine's node-error short-circuit),
-- and the `Maybe` zero is LOCAL (a `None` leaf stays where it is; the surrounding command tree is
-- preserved). Rust realization: `CausalEffect::try_and_then` (this `obind`'s success layer) and
-- `Result` as the `Except` layer.
-- ------------------------------------------------------------------

/-- The outcome channel: `Result<CausalEffect<V>, Error>` = `Except E (Free Cmd (Maybe V))`. -/
inductive Outcome (E V : Type) where
  | err : E → Outcome E V
  | ok  : CEffect V → Outcome E V

variable {E : Type}

/-- The success-layer bind (`CausalEffect::try_and_then`): dispatch on the effect — a `None` leaf is
    a local zero, a value leaf runs the continuation, a relay node threads the bind under the
    operation and hoists an inner error outward. -/
def ebind : CEffect V → (V → Outcome E W) → Outcome E W
  | .pure Option.none,    _ => .ok (.pure Option.none)
  | .pure (Option.some v), k => k v
  | .relay t sub,          k =>
      match ebind sub k with
      | .err e => .err e
      | .ok w  => .ok (.relay t w)

/-- The composite bind (`obind`): the `Except` layer is outermost — `Err` is a global left zero. -/
def obind : Outcome E V → (V → Outcome E W) → Outcome E W
  | .err e, _ => .err e
  | .ok m,  k => ebind m k

/-- The composite unit: `pure v = Ok(Pure(Some v))`. -/
def opure (v : V) : Outcome E V := .ok (.pure (Option.some v))

/-- Left identity: `obind (opure v) k = k v` (definitional).

    THEOREM_MAP: `core.causal_effect.transformer_stack` -/
theorem obind_left_id (v : V) (k : V → Outcome E W) : obind (opure v) k = k v := rfl

/-- Error is a global left zero: `obind (err e) k = err e` (definitional).

    THEOREM_MAP: `core.causal_effect.transformer_stack` -/
theorem obind_err_zero (e : E) (k : V → Outcome E W) : obind (.err e) k = .err e := rfl

/-- `None` is a local zero: binding the `None` effect preserves it (definitional).

    THEOREM_MAP: `core.causal_effect.transformer_stack` -/
theorem obind_none_zero (k : V → Outcome E W) :
    obind (.ok (.pure Option.none)) k = .ok (.pure Option.none) := rfl

/-- Right identity: `obind m opure = m` — by induction over the relay chain; the `None` and value
    leaves are the `Maybe` right identity, the relay case is the free layer's.

    THEOREM_MAP: `core.causal_effect.transformer_stack` -/
theorem obind_right_id (m : Outcome E V) : obind m opure = m := by
  cases m with
  | err e => rfl
  | ok eff =>
      show ebind eff opure = Outcome.ok eff
      induction eff with
      | pure o => cases o <;> rfl
      | relay t sub ih => simp [ebind, ih]

/-- Associativity: `obind (obind m f) g = obind m (fun v => obind (f v) g)` — by induction over the
    relay chain, with cases on the leaf and on whether the inner bind erred.

    THEOREM_MAP: `core.causal_effect.transformer_stack` -/
theorem obind_assoc (m : Outcome E V) (f : V → Outcome E W) (g : W → Outcome E X) :
    obind (obind m f) g = obind m (fun v => obind (f v) g) := by
  cases m with
  | err e => rfl
  | ok eff =>
      show obind (ebind eff f) g = ebind eff (fun v => obind (f v) g)
      induction eff with
      | pure o =>
          cases o with
          | none => rfl
          | some v => rfl
      | relay t sub ih =>
          simp only [ebind]
          rw [← ih]
          cases h : ebind sub f with
          | err e => rfl
          | ok w => rfl

-- ------------------------------------------------------------------
-- The handler is the unique interpreter: `CausalEffect::fold` is the free-monad catamorphism.
--
-- Textbook definition: the free monad on a signature is the initial algebra of programs over that
-- signature; a handler assigns meaning to the value leaf and to each operation, and initiality
-- makes the induced interpreter the UNIQUE map satisfying the handler equations (Plotkin & Power,
-- "Algebraic Operations and Generic Effects", 2003; Plotkin & Pretnar, "Handling Algebraic
-- Effects", LMCS 9(4), 2013). DEVIATION NOTE: transcribed at the single operation `RelayTo`
-- (one target tag, one sub-program hole), matching `CausalEffect::fold`'s
-- `(pure_case, algebra)` signature verbatim.
-- ------------------------------------------------------------------

/-- `CausalEffect::fold` — the catamorphism: `pure_case` on the value leaf, `algebra` on a
    `RelayTo(target, folded_sub)` node. -/
def fold (pureCase : Option V → X) (algebra : Nat → X → X) : CEffect V → X
  | .pure o     => pureCase o
  | .relay t sub => algebra t (fold pureCase algebra sub)

/-- The two handler equations hold definitionally.

    THEOREM_MAP: `core.causal_effect.fold_universal` -/
theorem fold_pure (pureCase : Option V → X) (algebra : Nat → X → X) (o : Option V) :
    fold pureCase algebra (.pure o) = pureCase o := rfl

/-- THEOREM_MAP: `core.causal_effect.fold_universal` -/
theorem fold_relay (pureCase : Option V → X) (algebra : Nat → X → X) (t : Nat) (sub : CEffect V) :
    fold pureCase algebra (.relay t sub) = algebra t (fold pureCase algebra sub) := rfl

/-- Uniqueness (initiality): ANY function satisfying the two handler equations IS the fold — the
    reasoning engine's interpreter is determined by its value case and its command algebra.

    THEOREM_MAP: `core.causal_effect.fold_universal` -/
theorem fold_unique (pureCase : Option V → X) (algebra : Nat → X → X)
    (h : CEffect V → X)
    (hp : ∀ o, h (.pure o) = pureCase o)
    (hr : ∀ t sub, h (.relay t sub) = algebra t (h sub)) :
    ∀ m, h m = fold pureCase algebra m := by
  intro m
  induction m with
  | pure o => exact hp o
  | relay t sub ih => rw [hr t sub, ih]; rfl

-- ------------------------------------------------------------------
-- Relay termination: the fuel-bounded relay handler is total.
--
-- The engine's adaptive-reasoning loop re-enters the graph on every `RelayTo` with a NEW program
-- produced at runtime, so no structural measure on the program bounds it — two causaloids relaying
-- to each other loop forever. The fuel bound makes the handler total BY CONSTRUCTION (structural
-- recursion on fuel): exhaustion is reported (`Option.none`), never looped. This is the formal
-- content of assumption #2 Q3's "relay-termination bound" (fuel-bounded step-indexed interpreters:
-- standard in operational semantics; cf. Amin & Rompf, "Type Soundness Proofs with Definitional
-- Interpreters", POPL 2017). DEVIATION NOTE: the graph is abstracted to a step function
-- `g : Nat → CEffect V → CEffect V` (target index + relayed sub-program ↦ the target causaloid's
-- next program), which is exactly what the engine's round loop consumes.
-- ------------------------------------------------------------------

/-- The fuel-bounded relay loop: `run fuel m g`. A value leaf answers; a relay consumes one unit of
    fuel and re-enters with the target's next program; fuel exhaustion reports `none`. Total by
    structural recursion on the fuel. -/
def run : Nat → CEffect V → (Nat → CEffect V → CEffect V) → Option (Option V)
  | 0,     _,           _ => Option.none
  | _ + 1, .pure o,     _ => Option.some o
  | n + 1, .relay t sub, g => run n (g t sub) g

/-- A value leaf answers immediately (any nonzero fuel).

    THEOREM_MAP: `core.causal_effect.relay_termination` -/
theorem run_pure (n : Nat) (o : Option V) (g : Nat → CEffect V → CEffect V) :
    run (n + 1) (.pure o) g = Option.some o := rfl

/-- Fuel monotonicity: an answer reached within `n` units is stable under more fuel — the bound
    only cuts divergence, never changes a result.

    THEOREM_MAP: `core.causal_effect.relay_termination` -/
theorem run_fuel_monotone (n : Nat) (g : Nat → CEffect V → CEffect V) :
    ∀ (m : CEffect V) (x : Option V), run n m g = Option.some x → run (n + 1) m g = Option.some x := by
  induction n with
  | zero => intro m x h; simp [run] at h
  | succ n ih =>
      intro m x h
      cases m with
      | pure o => exact h
      | relay t sub => exact ih (g t sub) x h

/-- The divergence cut: under a self-relaying step function (the two-causaloid relay loop), the
    handler reports fuel exhaustion for EVERY fuel — the unbounded loop is provably cut, the
    handler total.

    THEOREM_MAP: `core.causal_effect.relay_termination` -/
theorem run_self_relay_none (g : Nat → CEffect V → CEffect V)
    (hg : ∀ t sub, g t sub = .relay t sub) (t : Nat) (sub : CEffect V) :
    ∀ n, run n (.relay t sub) g = Option.none := by
  intro n
  induction n with
  | zero => rfl
  | succ n ih =>
      show run n (g t sub) g = Option.none
      rw [hg t sub]
      exact ih

-- ------------------------------------------------------------------
-- Relay-round composition: multi-round adaptive evaluation is the sequential (Kleisli) composition
-- of its rounds (graph-reasoning-formalization, task 3.6).
--
-- The graph engine's adaptive loop (`deep_causality/.../graph_reasoning/mod.rs`, the `'rounds` loop)
-- evaluates one reachable acyclic sub-DAG per round; a `RelayTo(target, sub)` ends the round and
-- seeds the next round at `target` with `sub` — "sequential composition of rounds" (its own comment,
-- l.174–176). Abstracting a round to the step `g : target ↦ sub ↦ next-program`, one round is
-- `nextRound` and the whole run is the fuel-bounded `run` (above). This section proves the rounds
-- COMPOSE: iterating the round step is additive over round counts (`rounds_add` — round `m` seeds
-- round `m+1`), a reached answer is stable under further rounds (`run_monotone_add`), and the
-- fuel-bounded run splits at any round boundary (`run_rounds_compose` — `m` rounds then `n` rounds
-- IS the `m + n`-round run; `run_relay_peel` is the single two-round step). The fuel bound composes
-- with NO new termination argument: it is inherited from `core.causal_effect.relay_termination`
-- (`run` is fuel-total by construction; `run_self_relay_none` cuts a relay cycle).
-- ------------------------------------------------------------------

/-- One adaptive round as a program step: a value is a fixpoint (an answered run stays answered); a
    command advances to the target's next program `g t sub` (the `RelayTo` seeding round `m+1`). -/
def nextRound (g : Nat → CEffect V → CEffect V) : CEffect V → CEffect V
  | .pure o      => .pure o
  | .relay t sub => g t sub

/-- `m` adaptive rounds — the round step iterated. -/
def rounds (g : Nat → CEffect V → CEffect V) : Nat → CEffect V → CEffect V
  | 0,     prog => prog
  | m + 1, prog => rounds g m (nextRound g prog)

/-- One more round peels from the right: `rounds (m+1) = nextRound ∘ rounds m`. -/
theorem rounds_succ_right (g : Nat → CEffect V → CEffect V) :
    ∀ (m : Nat) (prog : CEffect V), rounds g (m + 1) prog = nextRound g (rounds g m prog) := by
  intro m
  induction m with
  | zero => intro prog; rfl
  | succ m ih => intro prog; exact ih (nextRound g prog)

/-- **Rounds compose** — the sequential (Kleisli) composition of adaptive rounds: `m + n` rounds is
    `n` rounds after `m` rounds. Round `m`'s program seeds round `m+1`, so an adaptive run is the
    composite of its per-round steps.

    THEOREM_MAP: `core.causal_effect.relay_round_composition` -/
theorem rounds_add (g : Nat → CEffect V → CEffect V) (m : Nat) :
    ∀ (n : Nat) (prog : CEffect V), rounds g (m + n) prog = rounds g n (rounds g m prog) := by
  intro n
  induction n with
  | zero => intro prog; rfl
  | succ n ih =>
      intro prog
      show rounds g ((m + n) + 1) prog = rounds g (n + 1) (rounds g m prog)
      rw [rounds_succ_right g (m + n) prog, ih prog, ← rounds_succ_right g n (rounds g m prog)]

/-- A reached answer is stable under further rounds: an `m`-round answer persists to `m + n` rounds,
    so the composition is well-defined (generalizes `run_fuel_monotone` from `+1` to `+n`).

    THEOREM_MAP: `core.causal_effect.relay_round_composition` -/
theorem run_monotone_add (g : Nat → CEffect V → CEffect V) (m : Nat) (prog : CEffect V)
    (o : Option V) (h : run m prog g = Option.some o) :
    ∀ n, run (m + n) prog g = Option.some o := by
  intro n
  induction n with
  | zero => exact h
  | succ n ih => exact run_fuel_monotone (m + n) g prog o ih

/-- **The fuel-bounded run splits at a round boundary**: if the first `m` rounds do not answer, the
    `m + n`-round run is the `n`-round run continued from the `m`-round program — sequential
    composition on the real fuel-bounded loop. With `run_monotone_add` (answered runs persist) and
    `rounds_add` (rounds compose), the whole adaptive run is the Kleisli composition of its rounds;
    totality is inherited from `core.causal_effect.relay_termination` (no new termination argument).

    THEOREM_MAP: `core.causal_effect.relay_round_composition` -/
theorem run_rounds_compose (g : Nat → CEffect V → CEffect V) (n : Nat) :
    ∀ (m : Nat) (prog : CEffect V), run m prog g = Option.none →
      run (m + n) prog g = run n (rounds g m prog) g := by
  intro m
  induction m with
  | zero => intro prog _; rw [Nat.zero_add]; rfl
  | succ m ih =>
      intro prog h
      cases prog with
      | pure o => simp [run] at h
      | relay t sub =>
          simp only [run] at h
          rw [Nat.succ_add]
          exact ih (g t sub) h

/-- **Two rounds compose as one arrow** (the spec scenario): round 1 ends in `RelayTo(t, sub)`; the
    single peeling step feeds the target `t` its next program `g t sub`, and rounds 2… run from
    there — exactly the engine's `round_start := target; round_input := sub; continue 'rounds`.

    THEOREM_MAP: `core.causal_effect.relay_round_composition` -/
theorem run_relay_peel (g : Nat → CEffect V → CEffect V) (n t : Nat) (sub : CEffect V) :
    run (n + 1) (.relay t sub) g = run n (g t sub) g := rfl

end DeepCausalityFormal.Core.CausalEffect
