## Context

`CausalEffectPropagationProcess<Value, State, Context, Error, Log>`
([mod.rs:41-52](../../../deep_causality_core/src/types/causal_effect_propagation_process/mod.rs))
holds `pub value: EffectValue<Value>` and `pub error: Option<Error>` as independent public
fields. The invalid state (value AND error) is representable, and `bind` destroys the value
on it, so right identity fails — precondition P2 of
[`Formalization.md`](../../notes/causal-algebra/Formalization.md) §2. The haft audit hit the
same defect three times (deviations D6/D7) and each fix was a sum encoding; this change
applies the same fix to the production carrier.

Survey results (2026-07-05):
- Struct-literal construction: 78 sites in `deep_causality_core` (src + tests), 4 in
  `examples/*`, **zero** in `deep_causality`, `deep_causality_physics`, `deep_causality_cfd`,
  `deep_causality_ethos` src.
- Field-read hotspots (src): core carrier modules + 3 HKT witness files + `causal_flow`
  (branch/iterate/steps/terminals/construction); `deep_causality`
  (`causable_utils`, `causable_stateful`, `graph_reasoning`, `csm/eval` — ~2–3 each);
  `deep_causality_cfd/src/types/flow/blackout.rs` (verify whether its `.value` reads hit the
  carrier or its own types).
- Test surface: ~20 core test files, ~38 `deep_causality` test files reference the carrier.
- Getters already exist (`value()`, `state()`, `context()`, `error()`, `logs()`), so
  read-sites can be migrated mechanically.
- Constructors already exist (`pure`, `none`, `from_error`, `from_effect_value*`,
  `from_value*`, `with_state`); only fully-general construction (used by tests and the HKT
  witnesses) lacks one.

## Goals / Non-Goals

**Goals:**
- W by construction: value-XOR-error as one channel; invalid states unrepresentable.
- All three monad laws hold unconditionally on the real carrier; error short-circuit is a
  left zero (state/context/logs survive, continuation never runs).
- Fields private; construction/access through a total constructor + getter surface.
- Lean `right_id` + `assoc` theorems proved; Kani harnesses updated; THEOREM_MAP unblocked.
- Full repo mop-up with all tests green.

**Non-Goals:**
- **P1** (removing `RelayTo`/`Map` from `EffectValue`) — separate follow-up; this change
  does not touch the `EffectValue` enum and must not preclude P1.
- Relaxing or tightening existing trait bounds (`Error: Clone`, `Log: LogAppend + Default`)
  beyond what the encoding forces.
- API sugar beyond the minimal accessor surface; no serde, no new deps.
- The context Reader-vs-State decision (Formalization.md work-plan item 3).

## Decisions

**D1 — Encoding: `outcome: Result<EffectValue<Value>, Error>` (one field).**
This is literally `Either E (Maybe T)` with `EffectValue` playing the (extended) `Maybe`.
Alternatives rejected:
- *`EffectValue` gains an `Error(E)` variant*: turns `EffectValue<T>` into
  `EffectValue<T, E>`, rippling a type parameter through every signature that names
  `PropagatingEffect`/`EffectValue` across all downstream crates — far larger blast radius
  for the same guarantee, and it entangles P2 with P1 (the enum is P1's target).
- *Keep two private fields + smart constructors*: W by discipline, not by construction; the
  Lean claim weakens to "holds under maintained invariant". Explicitly rejected by the user
  ("full rectification").
Field name `outcome`: `value` would lie (it can be an error), `result` collides with the
Rust vocabulary, `signal` overloads the doc's `none` generator terminology.

**D2 — All fields private; total public `new(outcome, state, context, logs)`.**
With one channel, every representable state is valid, so a fully-general constructor is
harmless and needs no validation — this is what lets the HKT witnesses, `causal_flow`, and
tests construct arbitrary carriers without friend access. Existing named constructors keep
their exact semantics (`pure` → `Ok(Value(v))`, `from_error` → `Err(e)`, `none` →
`Ok(EffectValue::None)`).

**D3 — Accessor shapes.** _(final surface; revised from the shape below during the downstream
mop-up — see the deviation note.)_
`outcome() -> &Result<EffectValue<Value>, Error>` (raw borrow, the pattern-matching path);
`value() -> Option<&Value>` (the carried **scalar**, `Some` only for `EffectValue::Value`);
`value_cloned() -> Option<Value>` and `into_value(self) -> Option<Value>` (owned borrowing/consuming);
`effect() -> Option<&EffectValue<Value>>` (the full wrapper, for `None`/`ContextualLink`/`RelayTo`/`Map`
discrimination); `error() -> Option<&Error>` (changed from `&Option<Error>`;
implemented as `outcome.as_ref().err()`); `state()`, `context()`, `logs()` unchanged;
`is_ok()`/`is_err()` widened from the `((), ())` specialization to all `State`/`Context`.

> **Deviation (D3-revised).** This task was originally specified with `value() -> Option<&EffectValue<Value>>`
> (the wrapper) and "no further sugar." When the mop-up reached the examples, pulling the carried
> scalar out for display/comparison took an unusable five-term chain
> (`value().unwrap().clone().into_value().unwrap()…`). The accessor was redesigned as above so the
> everyday `value()` yields the scalar and the rare variant-discrimination path uses `effect()`. This
> re-swept ~244 read sites but simplified most of them (`Some(EffectValue::Value(v))` → `Some(v)`).

**D4 — `bind` continuation signature unchanged.**
`F: FnOnce(EffectValue<Value>, State, Option<Context>) -> …` stays: `bind` short-circuits on
`Err` before invoking `f`, and passes the inner `EffectValue` in the `Ok` arm. This keeps
the `CausalMonad` trait, every causal function, and all reasoning paths signature-compatible;
only construction and field access change. Right identity becomes structural: the `Err` arm
returns `self` reassembled verbatim.

**D5 — `fmap` semantics preserved, including the P1 seam.**
`Err` propagates untouched (short-circuit); `Ok(Value(v))` maps; `Ok(None)`/
`Ok(ContextualLink)` pass through; `Ok(RelayTo | Map)` still surfaces
`Err(ValueNotAvailable)` — same behavior as today, now expressed in one channel. When P1
lands, only that match arm disappears.

**D6 — Verification deliverables ride in the same change.**
Lean: extend the walking-skeleton model in `Core/CausalMonad.lean` with the error channel
(`Except`-shaped, transcribing the new `bind`), prove `right_id` and `assoc` alongside the
existing `left_id`; stays self-contained (no Mathlib). Kani: add right-identity-on-errored-
input, continuation-does-not-run, and log-monotonicity harnesses; the planned
W-well-formedness harness is discharged by construction and documented as such. THEOREM_MAP:
move `core.causal_monad.right_id` / `assoc` from "blocked" to proved, each with a Rust
witness (`core.causal_monad.lawful` stays blocked on P1 for the full `LawfulMonad` claim).

**D7 — Mop-up order is compiler-driven, crate by crate.**
`deep_causality_core` src → core tests → `deep_causality` src → its tests →
physics/cfd read-sites → examples. Migration patterns are mechanical:
struct literal → `new(...)` (or a named constructor); `.value` read → `value()`/`outcome()`
match; `.error` read → `error()`/`is_err()`; two-field match → one `Result` match. Tests that
asserted the old lax behavior (value surviving alongside error) are corrected to the lawful
expectation — per AGENTS.md, tests verify the API, and the API was wrong.

## Risks / Trade-offs

- [Hidden reliance on value-alongside-error somewhere in graph reasoning / CSM]
  → The compiler finds every access site (fields are private); the full test suites of
  `deep_causality_core` + `deep_causality` + physics + cfd run at each task-group boundary.
  Any test that encoded the lax behavior is reviewed for intent before correction, not
  blindly rewritten.
- [Getter shape change (`value()`, `error()`) silently alters match logic at read sites]
  → Both shapes change type (`&EffectValue` → `Option<&EffectValue>`, `&Option<E>` →
  `Option<&E>`), so every affected site is a compile error, not a silent drift.
- [`Display`/`explain` output changes ordering or wording]
  → Adapt `display.rs`/`explain.rs` deterministically; their tests pin the new format.
- [Volume: ~120 mechanical edit sites across ~60 test files]
  → Patterns table above; task groups sized so each ends green (fmt + clippy + tests) with a
  commit message per group, per the established workflow.
- [Semver: field access removed from published crates]
  → Breaking release of `deep_causality_core` + `deep_causality` in the workspace's lockstep
  release; CHANGELOGs record the migration patterns.
- [P1 entanglement temptation: `RelayTo`/`Map` arms look removable while editing `fmap`]
  → Explicit non-goal; the P1 seam is marked with a comment, nothing else.

## Migration Plan

Single feature branch (no worktrees). Implement in the task order of `tasks.md`; each task
group ends with the workspace green (`cargo test -p` for touched crates, `make format &&
make fix` once 3+ crates changed) and a prepared commit message. No commits by the agent —
messages are handed to the user at each group boundary. Rollback is `git checkout` of the
branch; no data or deployment surface exists.

## Open Questions

None blocking. Two decisions intentionally deferred to their own changes: P1
(`RelayTo`/`Map` excision) and the context Reader-vs-State question (Formalization.md items
2–3).
