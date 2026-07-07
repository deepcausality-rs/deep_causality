<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Core Formalization — Resolved Deviations Audit

Companion to the Lean formalization of `deep_causality_core`
(`lean/DeepCausalityFormal/Core/`, bound to Rust witnesses via `lean/THEOREM_MAP.md`).
Mirrors the haft effort (`haft-formalization-deviations.md`, `deep_causality_haft/LEAN_HAFT.md`).

This document is the **resolved audit** (graduated from the review plan once the Lean files landed):
it records (1) the layered approach, (2) the per-mechanism plan as executed, and (3) every deviation
found during the survey, each with a **terminal disposition**. All 9 Core Lean files typecheck
standalone and are witnessed; the crate-local status view is
[`deep_causality_core/LEAN_CORE.md`](../../../deep_causality_core/LEAN_CORE.md). Every D1–D17 below is
settled — no item remains **Fix-planned**.

## 0. Layered approach — base (Haft, proven) → causal extension (Core, the delta)

Per the crate architecture (`deep_causality_core` builds on `deep_causality_haft`) and the review
guidance: **do not re-prove the base categorical mechanisms.** They are already machine-checked in
the Haft layer. Core cites the base theorem and proves only the *causal extension* on top.

| Base mechanism (proven in Haft) | Haft THEOREM_MAP id | Causal extension proven in Core |
|---|---|---|
| Monad (Kleisli triple: left/right id, assoc) | `haft.monad.laws` | state threading + Writer log + `Except` error channel + W-invariant → laws preserved, **+ error left-zero** |
| Arrow / Category (`arr`, `>>>`, id, assoc) | `haft.arrow.category_laws` | Kleisli category of the causal monad (`arr`=lift, `>>>`=`bind_or_error`) → category laws on the `Value | Err` fragment |
| IO monad (`Io E A = Unit → Except E A`, triple laws) | `haft.io.laws` | core `IoAction` leaves add only concrete `run` bodies → **no new monad file**; only a conditional CSV round-trip lemma |
| Maybe / Option functor | (implicit in `haft.functor.laws`) | the success channel's value content **is** `Option<V>` (post `separate-control-channel`; `EffectValue` deleted); control is lifted out into `CausalEffect<V> = Free<CausalCommandWitness, Option<V>>` (`haft.free_monad.*`), so there is no P1 seam left to prove around |
| Monoid | `num.add_monoid.*` (shape) | `EffectLog` = free monoid on messages (`List Λ`, `++`, `[]`), + append-only/monotone |

The base laws are **reused, not restated**; the Core files reference the Haft id in their docstrings
and prove the delta.

## 1. Per-mechanism plan (proposed Core Lean files)

Each file is self-contained (no imports, bare-`lean`-checkable), SPDX header, `namespace
DeepCausalityFormal.Core.<X>`, `THEOREM_MAP: <id>` tags, Rust witnesses cited. Rust witnesses go in a
new `deep_causality_core/tests/formalization_lean/` mirror (matching the haft convention).

1. **`Core/CausalMonad.lean`** — *exists.* Reframe docstring to cite `haft.monad.laws` as the base;
   it already proves `left_id`, `right_id` (unconditional), `assoc`, `left_zero`. No new theorems.
2. **`Core/EffectLog.lean`** — Writer monoid: `left_id`, `right_id`, `assoc`, `monotone_prefix`.
   ids `core.effect_log.{left_id,right_id,assoc,monotone}`.
3. **`Core/CausalEffect.lean`** — the success channel `CausalEffect<V> = Free CausalCommand (Maybe V)`
   (post `separate-control-channel`; `EffectValue` is deleted). The value functor is `Option`, so
   `fmap_id`/`fmap_comp` cite haft's `Option` functor rather than a bespoke type; the free-monad laws
   over `CausalCommand` cite `haft.free_monad.*`. Prove the single-hole **`CausalCommand` functor
   laws** (identity + composition on the one hole) and `into_value = Maybe` projection. ids
   `core.causal_command.functor_laws`, `core.causal_effect.into_value`. (Replaces the old
   `EffectValue.lean`; no fragment/`relay_eq_target_only` refutation — the deviations are fixed, not
   documented.)
4. **`Core/CausalArrow.lean`** — *exists (landed in `causal-arrow-state-threading`).* Kleisli category
   of the state-threading causal monad: `category_laws` (left/right id + assoc, threading state) and
   `left_zero`, **unconditional** (no D1 `None`-collapse refutation — D1/D2 are fixed). ids
   `core.causal_arrow.{category_laws,left_zero}`.
5. **`Core/Alternatable.lean`** — the lens-setter family (value/state/context + `clear_context`):
   `set_get`, `set_set_proj` (idempotence up-to-log), `channel_independence`, `error_noop`, and the
   honest `not_setset_with_logs`. (No `intervene` alias — `Intervenable` was removed from core.) ids
   `core.alternatable.*`.
6. **`Core/CausalFlow.lean`** — facade lowering: `flow_iso` (≅ Process, `rfl`), `map_id`/`map_comp`,
   `map_eq_andThen` on the value fragment, `iterate_n_zero`/`iterate_n_succ`, `branch_noop_on_error`/
   `branch_selects`, `recover_escapes_kleisli` (deviation D11, refutation), `finish_*` (projection,
   note log-drop). ids `core.causal_flow.*`.
7. **`Core/Csv.lean`** — conditional codec round-trip `parse (render h rows) = h :: rows` under the
   comma/newline-free precondition (deviation D16). id `core.io.csv_roundtrip`.
8. **`Core/Consistency.lean`** *(small)* — witness `pure`/`fmap` **agreement** (`rfl`): post
   `separate-control-channel` the witness `fmap`s are total and uniform (no panic, no seam), so this
   proves agreement rather than the former disagreement (D15 fixed). id `core.witness.agree`.

`core.causal_monad.lawful` is now **provable** — P1 is resolved (control separated into
`CausalCommand`/`CausalEffect`), so the carrier is the transformer stack `Except ∘ Free ∘ Maybe` of
already-proven monads and the value-level `LawfulMonad` instance no longer waits on a fused type.

## 2. Deviations found (for review)

Severity: **[H]** shapes a law statement / correctness claim · **[M]** modeling judgment call ·
**[L]** benign/cosmetic. Each cites the survey evidence.

### Causal Arrow
- **D1 [H] — Right-identity is conditional.** `>>>` lowers to `bind_or_error`, which collapses any
  non-`Value` carrier (`None`, `ContextualLink`, `RelayTo`, `Map`) to `Err("…received no value")`
  (`causal_effect_propagation_process/mod.rs:343-350`, `effect_value/predicates.rs:37-42`). So
  `f >=> value = f` **fails** whenever `f` can emit a non-`Value` carrier. The causal arrow is a
  lawful Kleisli category only on the `Value | Err` sub-fragment. *Recommend:* model that fragment;
  machine-check the failure as a negative lemma.
- **D2 [H] — Composition does not thread State/Context.** `and_then` binds `|v, _state, _context|`
  (`causal_flow/steps.rs:26-29`), dropping upstream state/ctx; the composite's state/ctx come from
  the downstream stage. Contradicts the `KleisliCompose` doc (`compose.rs:14-15`). Invisible while
  `S=C=()` (all tests). *Recommend:* Lean model erases `S,C` and says so explicitly.
- **D3 [H] — The arrow's bind ≠ the monad's bind.** The raw monad `bind` preserves `None`
  (`mod.rs:129`, modeled as `bind'` over `Option V`); the arrow's `bind_or_error` path forces
  `None → Err`. Two different composition operators. (Root cause of D1.)
- **D4 [L] — Vestigial `Clone` bounds.** `KleisliCompose` requires `State: Clone`, `Context: Clone`
  (`compose.rs:33-34`) that the lowering never uses. Harmless; note only.

### EffectValue *(type deleted by `separate-control-channel`; D5/D6 resolved — see §2B ledger)*
- **D5 [H] — `PartialEq` is a partial equivalence relation, not an equivalence.** `Map(_)==Map(_)`
  is always `false` (non-reflexive; `partial_eq.rs:16`) and `RelayTo` equality ignores its payload
  (target-only, not a congruence; `partial_eq.rs:14`). This is *why* `Map`/`RelayTo` are excluded
  from the algebra (the P1 seam). **Resolved:** `EffectValue` is deleted; the value functor is
  `Option<V>` (a lawful congruence) and control (`RelayTo`) moved to `CausalEffect<V> =
  Free<CausalCommandWitness, Option<V>>` with a structural congruent `PartialEq`. `Map` is gone.
- **D6 [M] — `into_value` is lossy/partial.** Collapses `None`/`ContextualLink`/`RelayTo`/`Map` all
  to `Option::None` (`predicates.rs:37-42`), conflating "absent" with "dispatch/link". Faithful to
  Maybe only on `{None, Value}`. **Resolved:** `CausalEffect::into_value` is the honest `Maybe`
  projection — a command genuinely has no scalar value, so mapping it to `None` is faithful, not lossy.

### Intervenable / Alternatable
- **D7 [M] — do-operator ≡ counterfactual substitution.** `Intervenable::intervene` is literally
  `AlternatableValue::alternate_value` (`intervenable/mod.rs:35`), same `!!ValueAlternation!!` log
  marker. Pearl separates Rung-2 (intervention) from Rung-3 (counterfactual); here they are one
  function, indistinguishable at code and audit-log level.
- **D8 [H] — `intervene` is the value-substitution fragment; the true do-operator is DEFERRED to
  the `deep_causality` hypergraph layer.** Pearl's `do(X:=x)` is graph surgery — delete `Pa(X)→X`,
  pin `X:=x`, evaluate the mutilated model (Pearl, *Causality* 2nd ed. §3.2). That edge-deletion is
  what isolates the variable and separates causation from correlation (it blocks the back-door
  paths). Core's `intervene` (`alternatable_value.rs:19-33`) is a local point-wise value overwrite
  on one carrier in a Kleisli chain — it has **no graph in scope**, so it structurally cannot
  de-confound. This is not a defect to fix at the core layer: the do-operator **cannot** be
  expressed as a monad operation (there is no topology to mutilate). It **can** be expressed
  properly in the `deep_causality` crate over the **Causaloid + hypergraph** structure, where
  `do(X:=x)` becomes genuine graph surgery on the causaloid graph (sever the intervened node's
  inbound hyperedges, pin its output, re-propagate). **Decision (author):** core formalizes only
  the lens/substitution laws for `intervene`; the semantic do-operator is a **deferred future item
  at the hypergraph layer** — architecturally supported, not yet built or formalized. The Lean
  `Alternatable`/`intervene` docstring states this scope explicitly and points forward to the
  hypergraph layer. (Companion: with the `RelayTo` P1 seam now resolved — control lifted into
  `CausalEffect`/`CausalCommand` by `separate-control-channel` — this do-operator deferral is the one
  remaining place where the algebra intentionally stops at the core boundary and continues in
  `deep_causality`.)
- **D9 [H] — Idempotence / set-set / absorption / commutation hold ONLY up-to-log.** Every setter
  unconditionally appends a marker on the success path (`alternatable_value.rs:27`,
  `alternatable_state.rs:21`, `alternatable_context.rs:21`), so `do(x);do(x) ≠ do(x)` and
  `do(x);do(y) ≠ do(y)` as *whole carriers* — they hold only after projecting away `logs`. (The
  error path is a clean total no-op with no log.) This is the load-bearing caveat; the Lean
  statements must use a `proj` eraser, and a negative lemma machine-checks the full-carrier failure.
- **D10 [L] — `alternate_context` cannot produce `None`.** Always wraps `Some`
  (`alternatable_context.rs:24`); the setter's codomain excludes clearing the context.

### CausalFlow facade
- **D11 [H] — `recover` escapes the Kleisli fragment.** `recover(f)` turns `Err(e) → Ok(Value(f e))`
  (`causal_flow/steps.rs:100`). Since `bind` is a left-zero on error (`CausalMonad.lean ::
  bind_raise_left_zero`), no Kleisli composite can do this — `recover` is a genuine new operation
  (`MonadError.catch`), not sugar. It also emits no log entry. *Recommend:* document as an
  extension; machine-check the impossibility.
- **D12 [H] — `iterate_until` / `iterate_to_fixpoint` inject `MaxStepsExceeded`.**
  (`iterate.rs:41,67`) A failure mode absent from the base monad; not reducible to bind/pure. New
  bounded-search semantics.
- **D13 [M] — `finish` discards state, context, AND the audit log.** (`terminals.rs:16`) The
  observation map drops the log — the log's purpose is lost at the boundary unless `into_process`
  is used first. Expected for a terminal, but worth an explicit note.
- **D14 [H] — `map` (= inherent `fmap`) ≠ `bind(pure ∘ f)` off the `Value` carrier.** `fmap`
  threads `None` through (`mod.rs:243-252`); `and_then`/`bind_or_error` errors on `None`. The
  Functor and the Kleisli-derived map disagree on non-`Value` carriers. Same seam as D1/D3.

### Witnesses
- **D15 [H] — Four disagreeing `fmap`s on a value-less `Ok` carrier.** arity-5 witness **panics**
  (`.expect`, `causal_effect_propagation_process/hkt.rs:48-53`); Effect/Process witnesses return
  `Err(InternalLogicError)`; the inherent `fmap` (used by `CausalFlow::map`) passes `None` through.
  The witness-Functor is not the Functor `CausalFlow` uses; the consistency test misses it (only
  exercises `Value`). *Recommend:* model both and machine-check they agree only on `Ok(Value _)`.

### IO / EffectLog
- **D16 [M] — CSV round-trip is conditional.** `parse ∘ render = id` only when no field contains
  `','` or `'\n'` (no quoting/escaping; `write_csv.rs:51`). State the precondition as a hypothesis.
- **D17 [L] — EffectLog timestamp handling.** `PartialEq` deliberately ignores timestamps
  (`log_effect.rs:45-54`) — which is exactly what makes the `List Λ` abstraction faithful; `Eq` is
  derived while `PartialEq` is hand-written; `add_entry` is nondeterministic (`SystemTime::now`).
  Sound and deliberate; note only.

## 2A. Decision — separate the control channel from the value functor (LANDED)

**Decided (author) and landed as change `separate-control-channel`.** `RelayTo` and `Map` were
*control operations* (a computed jump; a dispatch table), not values, and their presence inside
`EffectValue<T>` was the root conflation behind D5/D6/D14/D15 and the P1 block. Rather than making the
conflation lawful in place (Option A: recursive `fmap` + structural eq on the fused type), we **ended
the conflation** — and the simplification went one step further than the original Option-B sketch:
`EffectValue` was **deleted entirely** and its value content collapsed to `Option<V>`, while the two
control operations were folded into a single-hole command functor lifted into a free monad.

```rust
// The success channel: value / none / command, unified in one free-monad newtype.
pub struct CausalEffect<V>(Free<CausalCommandWitness, Option<V>>);
//   Pure(Some v)                         = a value
//   Pure(None)                           = the `None` effect (absence)
//   Suspend(CausalCommand::RelayTo(t,k)) = a command (computed jump)

pub enum CausalCommand<K> { RelayTo(usize, K) }   // single-hole operation functor; `Map`/`Dispatch` dropped (unused)
```

This is the textbook **algebraic-effects / free-monad** shape (`Free f a = Pure a | Op (f (Free f a))`,
Plotkin & Power 2003; Swierstra 2008): `Option<V>` = the value/`Pure` part, `CausalCommand` = the
operation functor `f`, and the graph-reasoning traversal is the **handler** (`Free::fold`) that
interprets the jump. `Option<V>` is a lawful functor already (cite `haft.functor.laws`), and
`CausalEffect` gets a total `map`, a derivable congruent `PartialEq` (structural over the `RelayTo`
tree), and an honest `into_value` — **unblocking `core.causal_monad.lawful`**.

**Structuring (as landed):** the process outcome is `Result<CausalEffect<V>, Error>` =
`Except E (Free CausalCommand (Maybe V))` — the value-XOR-error W-invariant is preserved (the `Except`
layer) and control lives inside the success channel as the `Free`'s `Suspend` layer, a **second left
zero** of bind that the engine folds *before* any value-level bind. `EPP = CausalMonad ⊕ CausalEffect`.
**Scope (as delivered):** a breaking change to the adaptive-reasoning surface — user `causal_fn`
closures emit `CausalEffect::relay_to(..)` instead of `EffectValue::RelayTo(..)`, and the ~6
reasoning-engine sites interpret the command via `command_target()` / `into_command()`. Landed as its
own OpenSpec change `separate-control-channel` (the resolution of P1), archived
`2026-07-06-separate-control-channel`; workspace + `bazel test //...` green.

## 2B. Resolution ledger (every deviation gets a disposition — the spec derives from this)

Disposition: **Fixed** (code corrected already) · **Fix-planned** (own change) · **Documented
extension** (beyond a base def; keep, with its own laws) · **Accepted property** (intended
behavior; state precisely) · **Deferred** (correct home is another layer).

| # | Disposition | Resolution |
|---|---|---|
| D1 right-id conditional | **Fixed** | `and_then` now propagates `None` lawfully (right identity holds); with `separate-control-channel` landed, `RelayTo` left the value type into `CausalCommand`/`CausalEffect` and `Map` was deleted, so the collapse cases vanish. |
| D2 state/ctx not threaded | **Fixed** | Landed as change `causal-arrow-state-threading` (Option B). There is now **one** Kleisli bind `CausalFlow::and_then : Fn(Value, State, Option<Context>) -> CausalFlow<U,S,C>` threading state exactly as the monad's `bind`, and **one** reified arrow stage `(A,S,Option<C>) -> CausalFlow<B,S,C>`; the stateless case is the specialization (`next` = value-only sugar `and_then(|v,_,_| p(v))`; `|a,_,_|` stages; `run_value`). The former state-discarding value-only `and_then` (the bug) is removed. Proved in `Core/CausalArrow.lean` (`core.causal_arrow.{category_laws,left_zero}`, reducing to the monad theorems) and witnessed by `arrow_threads_accumulated_state` / `arrow_error_short_circuit_preserves_state`. 960 bazel tests + lake build green. |
| D3 arrow bind ≠ monad bind | **Fixed** | `and_then` preserves `None → None` (chosen policy), matching the lawful Maybe-Kleisli bind. |
| D4 vestigial `Clone` bounds | **Fixed** | Removed from `KleisliCompose`; workspace builds + tests green. |
| D5 `PartialEq` is a PER | **Fixed** (landed in `separate-control-channel`) | `EffectValue` is deleted; the value functor is `Option<V>` (already a lawful congruence). Control (`RelayTo`) lives in `CausalEffect<V> = Free<CausalCommandWitness, Option<V>>` with a **structural** congruent `PartialEq` (walks the `RelayTo` tree recursively). The PER (`Map(_)==Map(_)=false`, target-only `RelayTo`) is eliminated. |
| D6 `into_value` lossy | **Fixed** (landed) | `EffectValue` deleted; `CausalEffect::into_value` is the faithful `Maybe` projection (`Pure(Some v) → Some`, `Pure(None)`/command → `None`). Total and honest — a command genuinely has no scalar. |
| D7 do ≡ counterfactual | **Fixed** | `Intervenable::intervene` **removed from core** (trait + blanket impl + files); the value-substitution operation is now spelled `alternate_value` everywhere (`CausalFlow::intervene`→`alternate_value`, `intervene_if`→`alternate_value_if`; all callers in core/cfd/examples/benches migrated). The name "intervene"/`do()` is reserved for the graph layer where the real operator lives — no value-level over-claim remains. |
| D8 no graph mutilation | **Deferred** | True Pearl do-operator (graph surgery / variable isolation) belongs at the `deep_causality` Causaloid + hypergraph layer; core proves only the lens laws. |
| D9 idempotence up-to-log | **Accepted property** | The success-path audit entry is deliberate (Writer). Lens laws hold on the value projection `proj`; state the projected laws + machine-check the full-carrier log growth. Not a bug. |
| D10 `alternate_context` can't clear to `None` | **Fixed** | Added `CausalEffectPropagationProcess::clear_context` (inherent) — sets the context to `None` with a `!!ContextCleared!!` audit entry, no-op on an errored carrier (symmetric with `alternate_context`). Inherent rather than a trait method to avoid forcing the 3 cfd `AlternatableContext` impls; the core carrier — where the gap was — can now clear. Tested. |
| D11 `recover` = catch | **Documented extension** | `MonadError.catch`; a lawful extension beyond the Kleisli fragment (bind is left-zero on error). State its catch laws separately; keep. |
| D12 `iterate_until`/`iterate_to_fixpoint` inject `MaxStepsExceeded` | **Documented extension** | Bounded-search / fixpoint combinators with an intended failure mode; document their own contracts; keep. |
| D13 `finish` drops state/ctx/log | **Accepted property** | `finish` is the value-observation terminal (`Result<Value,Error>`); full extraction is via `into_process`/`into_parts`. Document the boundary. |
| D14 `map` ≠ `bind(pure∘f)` off `Value` | **Fixed** (landed) | The value functor is `Option`; `CausalEffect::map` is a **total** functor (maps `Option` leaves through the `Free`, including command sub-programs). `map f = bind(pure∘f)` holds on the value fragment (the monad-law surface). |
| D15 four disagreeing `fmap`s (one panics) | **Fixed** (landed) | `CausalEffect::map` and the witness `fmap`s are total and uniform; the arity-5 `.expect` **panic is removed** (a value-less carrier maps to `None`, no manufactured error). The non-reflexive `Map` eq is gone with `Map`. |
| D16 CSV round-trip conditional | **Accepted property** | The round-trip theorem is conditional on comma/newline-free fields (no escaping); `Core/Csv.lean :: csv_roundtrip` states the precondition as an explicit hypothesis (`core.io.csv_roundtrip`, witnessed via a real `write_csv`/`read_csv` temp-file round-trip). RFC-4180 quoting is a possible future hardening. |
| D17 EffectLog timestamp/`PartialEq` | **Accepted property** | The timestamp-quotient in `PartialEq` is what makes the `List Λ` abstraction faithful; deliberate and sound. Document; not a bug. |

**Two soft items flagged for the author** (all others are decided): **D10** (keep context setter as-is
vs add `clear_context`) and **D16** (document the CSV precondition vs implement RFC-4180 quoting).
Once these two are settled, every deviation is resolved and the specification can be derived.

**Order of changes:** both code prerequisites are now **landed**. The causal-arrow corrections
(D2/D3/D4/D1) shipped in `causal-arrow-state-threading`; `separate-control-channel` (B) then deleted
`EffectValue`, made the success channel `CausalEffect<V> = Free<CausalCommandWitness, Option<V>>`, and
resolved D5/D6/D14/D15 — including the two must-fix bugs (the `fmap` panic and the `Map` reflexivity),
both of which are gone with `EffectValue`. This formalization change therefore targets the corrected
code directly: the Lean/witness proofs describe the faithful implementation, and no deviation remains
to be documented as an accepted gap in the value/control channels.

## 3. Execution order

Foundational → dependent, each verified with bare `lean` before moving on:
`EffectLog` → `EffectValue` → (`CausalMonad` reframe) → `CausalArrow` → `Alternatable` →
`CausalFlow` → `Consistency` → `Csv`. Then: Rust witnesses in
`deep_causality_core/tests/formalization_lean/`, THEOREM_MAP rows, and `deep_causality_core/LEAN_CORE.md`
(status table mirroring `LEAN_HAFT.md`).
