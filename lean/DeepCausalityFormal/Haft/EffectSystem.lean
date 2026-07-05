/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — Effect system (arity-3 fixed-channel monad).

Rust source: `deep_causality_haft/src/effect_system/` — `Effect3/4/5` (`effect.rs`) fix all but
one parameter of an arity-k constructor (partial application; see `Hkt.lean`), and
`MonadEffect3/4/5` (`monad_effect.rs`) impose `pure`/`bind` on the result. The unbound variants
(`monad_effect_unbound.rs`) additionally thread a state-type transition `S1 → S2 → S3`; their
laws are the parameterised-monad laws already proved in `ParametricMonad.lean` (Atkey 2009) and
are not repeated here.

Model: the error+log channels — `Eff3 E Λ T = Except E T × List Λ` — i.e. `ExceptT E (Writer Λ)`
with bind semantics matching the docstring: "If effect contains an error, the error is
propagated. Otherwise f is applied and warnings are combined." The monad laws hold
(Moggi 1991; the Writer monoid is `List` under append, Wadler 1995 §2.7).

DEVIATION NOTES against this model:
1. RESOLVED — the trait previously bounded `bind` with `U: Default`, which the mathematical
   bind does not have; the bound existed only so product-encoded carriers could manufacture a
   `U` in the error branch (the same value/error-product defect as the core carrier's
   W-invariant, Formalization.md precondition P2). The bound has been removed from
   `MonadEffect3/4/5` (deviations note, D6/P-2); the sum encoding below never needed it.
2. RESOLVED — the crate's reference implementation (`src/utils_tests.rs`) previously RAN THE
   CONTINUATION when an error was present, violating raise-as-left-zero
   (`bind (raise e) f = raise e`, `f` not invoked). The carriers now hold `value: Option<T>`
   and short-circuit lawfully; `bind3_raise_left_zero` below is the law they implement.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/algebra/formalization_law_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.EffectSystem

variable {E Λ T U V : Type}

/-- Arity-3 effect carrier: value-or-error (sum!) plus an append-only log. -/
structure Eff3 (E Λ T : Type) where
  result : Except E T
  logs : List Λ

/-- `MonadEffect3::pure`: value, no error, empty log. -/
def pure3 (t : T) : Eff3 E Λ T := ⟨.ok t, []⟩

/-- `MonadEffect3::bind` with the documented semantics: error short-circuits (log preserved,
    `f` NOT run); otherwise run `f` and append its log. -/
def bind3 (m : Eff3 E Λ T) (f : T → Eff3 E Λ U) : Eff3 E Λ U :=
  match m.result with
  | .error e => ⟨.error e, m.logs⟩
  | .ok t => ⟨(f t).result, m.logs ++ (f t).logs⟩

/-- Left identity: `bind (pure t) f = f t` (Moggi 1991). Holds because `pure`'s log is empty
    (`[] ++ l = l` definitionally) — no `Default` needed anywhere.

    THEOREM_MAP: `haft.effect3.monad_laws` -/
theorem bind3_left_id (t : T) (f : T → Eff3 E Λ U) :
    bind3 (pure3 t) f = f t := rfl

/-- Right identity: `bind m pure = m` (Moggi 1991). Holds because the carrier is a SUM —
    the error case returns `m` verbatim; the ok case needs only `l ++ [] = l`.

    THEOREM_MAP: `haft.effect3.monad_laws` -/
theorem bind3_right_id (m : Eff3 E Λ T) :
    bind3 m pure3 = m := by
  cases m with
  | mk result logs =>
    cases result with
    | error e => rfl
    | ok t => simp [bind3, pure3]

/-- Associativity: `bind (bind m f) g = bind m (fun t => bind (f t) g)` (Moggi 1991); the log
    channel needs associativity of append — the Writer monoid law (Wadler 1995 §2.7).

    THEOREM_MAP: `haft.effect3.monad_laws` -/
theorem bind3_assoc (m : Eff3 E Λ T) (f : T → Eff3 E Λ U) (g : U → Eff3 E Λ V) :
    bind3 (bind3 m f) g = bind3 m (fun t => bind3 (f t) g) := by
  cases m with
  | mk result logs =>
    cases result with
    | error e => rfl
    | ok t =>
      cases h : (f t).result with
      | error e => simp [bind3, h]
      | ok u => simp [bind3, h, List.append_assoc]

/-- Raise is a left zero: an errored carrier short-circuits — `f` is never consulted, its
    effects cannot leak. The reference implementation in `utils_tests.rs` now implements
    exactly this semantics (deviation note 2 above, resolved). -/
theorem bind3_raise_left_zero (e : E) (l : List Λ) (f : T → Eff3 E Λ U) :
    bind3 ⟨.error e, l⟩ f = ⟨.error e, l⟩ := rfl

end DeepCausalityFormal.Haft.EffectSystem
