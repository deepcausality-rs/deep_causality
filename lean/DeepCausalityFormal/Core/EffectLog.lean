/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Core — EffectLog: the append-only audit log as a free monoid (the Writer output).

Rust source: `deep_causality_core/src/types/effect_log/log_effect.rs` (`EffectLog`,
`LogAppend::append`) and `log_entry.rs` (`LogEntry`).

Layering: the *base* mechanism is the free monoid on a message alphabet (Mac Lane, CWM §I.1) and
the Writer monad's output object (Moggi 1991; Wadler, "Monads for Functional Programming" §3). The
monad that threads this log is the base monad proven in `Haft/Monad.lean` (`haft.monad.laws`); the
causal monad's `bind` combines logs by `++` (`Core/CausalMonad.lean :: bind_assoc`, which consumes
`List.append_assoc` proved here). This file pins the *log channel* on its own.

Modeling (faithful to the Rust): `EffectLog` is a `Vec<LogEntry>`; `append` is exactly
`Vec::append` (order-preserving concatenation, no dedup / cap / reorder). Its `PartialEq`
compares the MESSAGE sequence only, quotienting away timestamps (`log_effect.rs:45-54`) — this is
precisely what makes the value-level abstraction `List Λ` (Λ = opaque message label) faithful. See
deviation D17 in `openspec/notes/causal-algebra/core-formalization-plan.md`.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witnesses: `deep_causality_core/tests/formalization_lean/effect_log_tests.rs`.
-/

namespace DeepCausalityFormal.Core.EffectLog

variable {Λ : Type}

/-- The value-level log: a sequence of message labels (timestamps quotiented away by the Rust
    `PartialEq`, `log_effect.rs:45-54`). -/
abbrev Log (Λ : Type) := List Λ

/-- The identity log — `EffectLog::new()` / `Default` (`log_effect.rs:56-60`). -/
def empty : Log Λ := []

/-- The monoid operation — `LogAppend::append` = `Vec::append` (`log_effect.rs:123-129`). -/
def append (x y : Log Λ) : Log Λ := x ++ y

/-- Left identity: `append empty x = x`.

    THEOREM_MAP (staged — bridged in the core-formalization phase): `core.effect_log.left_id` -/
theorem append_left_id (x : Log Λ) : append empty x = x := by
  simp [append, empty]

/-- Right identity: `append x empty = x`.

    THEOREM_MAP (staged — bridged in the core-formalization phase): `core.effect_log.right_id` -/
theorem append_right_id (x : Log Λ) : append x empty = x := by
  simp [append, empty]

/-- Associativity: `append (append x y) z = append x (append y z)` — the Writer monoid law that
    `Core/CausalMonad.lean :: bind_assoc` relies on for the log channel.

    THEOREM_MAP (staged — bridged in the core-formalization phase): `core.effect_log.assoc` -/
theorem append_assoc (x y z : Log Λ) :
    append (append x y) z = append x (append y z) := by
  simp [append, List.append_assoc]

/-- Append-only / monotone: the incoming log is always a prefix of the combined log — this is how
    `bind` threads logs (`self.logs ++ next.logs`, `causal_effect_propagation_process/mod.rs:150`),
    so no audit history is ever lost. Witnessed by the existence of the remaining suffix.

    THEOREM_MAP (staged — bridged in the core-formalization phase): `core.effect_log.monotone` -/
theorem append_monotone (x y : Log Λ) : ∃ z, x ++ z = append x y :=
  ⟨y, rfl⟩

end DeepCausalityFormal.Core.EffectLog
