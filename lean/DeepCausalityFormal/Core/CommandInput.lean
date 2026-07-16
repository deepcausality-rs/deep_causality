/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Core — the F-3 command-input theorem: a command (`RelayTo`) on a singleton's INPUT channel yields a
specific, named error — never a silent `None`, never a dropped signal (the F-3 resolution of
`openspec/notes/causal-algebra/Causaloid-Formalization.md`).

Statement of the gap (F-3, input/output asymmetry). A singleton denotes a Kleisli arrow on the
VALUE sub-object of the effect: it consumes a value, not a control command. The old model left this
as an open flag — an incoming command was reported by the generic `bind_or_error` step as
"input value is None", conflating a dropped command with absence of evidence. The current engine
detects the command FIRST (`incoming_effect.command_target().is_some()`) and returns a
command-specific error, matching `evaluate` and `evaluate_stateful` exactly. This file makes that a
theorem: the singleton's input dispatch is TOTAL (always an outcome), the command path is SPECIFIC
(the command error, not the absence error) and never a success (`ok`), and — when the two error
tokens differ — the command error is DISTINCT from the absence error.

The input effect mirrors `CausalEffect<V> = Free<CausalCommand, Option<V>>`
(`Core/CausalEffect.lean`): `value` = `Pure(Some v)`, `absent` = `Pure(None)`,
`command t sub` = `Suspend(RelayTo(t, sub))`. The outcome mirrors the carrier's single value-XOR-
error channel `Result<CausalEffect<W>, E>`: `err` (the `Err` arm) or `ok` (a value/absence success).

Rust source: `deep_causality/src/types/causal_types/causaloid/causable.rs`
(`Causaloid::evaluate`, the `command_target().is_some()` guard) and its stateful sibling
`StatefulMonadicCausable::evaluate_stateful`.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witnesses: `deep_causality/tests/formalization_lean/command_input_tests.rs`.
-/

namespace DeepCausalityFormal.Core.CommandInput

variable {V W E : Type}

/-- The singleton's input effect (`CausalEffect<V>`, `Core/CausalEffect.lean`): a value leaf, the
    absence-of-evidence leaf, or a command node (`RelayTo(target, sub)`). -/
inductive InEffect (V : Type) where
  | value   : V → InEffect V
  | absent  : InEffect V
  | command : Nat → InEffect V → InEffect V

/-- The outcome channel `Result<CausalEffect<W>, E>` reduced to what F-3 observes: an error, or a
    success carrying an `Option W` (value / absence). -/
inductive Outcome (E W : Type) where
  | err : E → Outcome E W
  | ok  : Option W → Outcome E W

/-- The singleton's input dispatch (`Causaloid::evaluate`): a command on the input channel yields
    the **command-specific** error `cmdErr`; the absence effect yields the distinct `noneErr`
    ("input value is None"); a value runs the element map. The command guard fires FIRST — exactly
    the Rust `command_target().is_some()` early return, shared by `evaluate` / `evaluate_stateful`. -/
def evalIn (cmdErr noneErr : E) (elem : V → Outcome E W) : InEffect V → Outcome E W
  | .command _ _ => .err cmdErr
  | .absent      => .err noneErr
  | .value v     => elem v

/-- A value input runs the element (definitional). -/
theorem value_runs_elem (cmdErr noneErr : E) (elem : V → Outcome E W) (v : V) :
    evalIn cmdErr noneErr elem (.value v) = elem v := rfl

/-- An absence input yields the absence error (definitional). -/
theorem absent_yields_none_err (cmdErr noneErr : E) (elem : V → Outcome E W) :
    evalIn cmdErr noneErr elem (.absent) = .err noneErr := rfl

/-- **F-3**: a command (`RelayTo`) on the input channel yields the command-specific error `cmdErr`
    — a total, named result, matching the Rust singleton's early return.

    THEOREM_MAP: `core.causaloid.command_input` -/
theorem command_yields_cmd_err (cmdErr noneErr : E) (elem : V → Outcome E W)
    (t : Nat) (sub : InEffect V) :
    evalIn cmdErr noneErr elem (.command t sub) = .err cmdErr := rfl

/-- A command input is **never a success** — never a silent `None`, never a value, never a dropped
    signal. The command path is an error for every possible success payload.

    THEOREM_MAP: `core.causaloid.command_input` -/
theorem command_never_ok (cmdErr noneErr : E) (elem : V → Outcome E W)
    (t : Nat) (sub : InEffect V) :
    ∀ o : Option W, evalIn cmdErr noneErr elem (.command t sub) ≠ .ok o := by
  intro o h; nomatch h

/-- The command error is **distinct from** the absence error whenever the two error tokens differ:
    a dropped command is not conflated with absence of evidence (the precise F-3 point — the old
    model reported both as "input value is None").

    THEOREM_MAP: `core.causaloid.command_input` -/
theorem command_err_distinct_from_absent (cmdErr noneErr : E) (elem : V → Outcome E W)
    (t : Nat) (sub : InEffect V) (hne : cmdErr ≠ noneErr) :
    evalIn cmdErr noneErr elem (.command t sub) ≠ evalIn cmdErr noneErr elem (.absent) := by
  show Outcome.err cmdErr ≠ (Outcome.err noneErr : Outcome E W)
  intro h; injection h with h'; exact hne h'

end DeepCausalityFormal.Core.CommandInput
