<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Core Formalization — Plan & Deviations (for review)

Companion to the Lean formalization of `deep_causality_core`
(`lean/DeepCausalityFormal/Core/`, bound to Rust witnesses via `lean/THEOREM_MAP.md`).
Mirrors the haft effort (`haft-formalization-deviations.md`, `deep_causality_haft/LEAN_HAFT.md`).

This document is the **review artifact**: it records (1) the layered approach, (2) the per-mechanism
plan, and (3) every deviation found during the survey, for the author to approve before the Lean
files are generated.

## 0. Layered approach — base (Haft, proven) → causal extension (Core, the delta)

Per the crate architecture (`deep_causality_core` builds on `deep_causality_haft`) and the review
guidance: **do not re-prove the base categorical mechanisms.** They are already machine-checked in
the Haft layer. Core cites the base theorem and proves only the *causal extension* on top.

| Base mechanism (proven in Haft) | Haft THEOREM_MAP id | Causal extension proven in Core |
|---|---|---|
| Monad (Kleisli triple: left/right id, assoc) | `haft.monad.laws` | state threading + Writer log + `Except` error channel + W-invariant → laws preserved, **+ error left-zero** |
| Arrow / Category (`arr`, `>>>`, id, assoc) | `haft.arrow.category_laws` | Kleisli category of the causal monad (`arr`=lift, `>>>`=`bind_or_error`) → category laws on the `Value | Err` fragment |
| IO monad (`Io E A = Unit → Except E A`, triple laws) | `haft.io.laws` | core `IoAction` leaves add only concrete `run` bodies → **no new monad file**; only a conditional CSV round-trip lemma |
| Maybe / Option functor | (implicit in `haft.functor.laws`) | `EffectValue`'s `{None, Value}` fragment ≅ `Option`; `ContextualLink` a trivial leaf; `RelayTo`/`Map` are the **P1 seam** |
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
3. **`Core/EffectValue.lean`** — `into_from_roundtrip`, `maybe_section` (fragment ≅ Option),
   `fmap_id`/`fmap_comp` (fragment functor), `relay_eq_target_only` (deviation, machine-checked).
   ids `core.effect_value.*`.
4. **`Core/CausalArrow.lean`** — Kleisli category of the causal monad: `category_laws`
   (left/right id + assoc on the `Value|Err` fragment), `left_zero`; **plus** a machine-checked
   refutation that right-identity fails once a stage emits `None` (deviation D1). ids
   `core.causal_arrow.{category_laws,left_zero,right_id_conditional}`.
5. **`Core/Alternatable.lean`** — the lens-setter family (value/state/context) + the `intervene`
   alias: `set_get`, `set_set_proj` (idempotence up-to-log), `channel_independence`,
   `error_noop`, `override`, and the honest refutation `not_setset_with_logs`. ids
   `core.alternatable.*`, `core.intervene.*`.
6. **`Core/CausalFlow.lean`** — facade lowering: `flow_iso` (≅ Process, `rfl`), `map_id`/`map_comp`,
   `map_eq_andThen_on_value` (the seam), `iterate_n_zero`/`iterate_n_succ`, `branch_noop_on_error`/
   `branch_selects`, `recover_escapes_kleisli` (deviation D11, refutation), `finish_*` (projection,
   note log-drop). ids `core.causal_flow.*`.
7. **`Core/Csv.lean`** — conditional codec round-trip `parse (render h rows) = h :: rows` under the
   comma/newline-free precondition (deviation D16). id `core.io.csv_roundtrip`.
8. **`Core/Consistency.lean`** *(small)* — witness `pure`/`fmap` agreement (`rfl`), and the
   machine-checked *disagreement* of the four `fmap`s off the `Value` carrier (deviation D15). ids
   `core.witness.{agree,fmap_seam}`.

`LawfulMonad`/full-functor over the *complete* `EffectValue` (with `RelayTo`/`Map`) stays **blocked
on P1** — not stated as a total theorem, consistent with `core.causal_monad.lawful`.

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

### EffectValue
- **D5 [H] — `PartialEq` is a partial equivalence relation, not an equivalence.** `Map(_)==Map(_)`
  is always `false` (non-reflexive; `partial_eq.rs:16`) and `RelayTo` equality ignores its payload
  (target-only, not a congruence; `partial_eq.rs:14`). This is *why* `Map`/`RelayTo` are excluded
  from the algebra (the P1 seam). *Recommend:* model the `{None, Value, ContextualLink}` fragment;
  machine-check `relay_eq_target_only` to document the payload-drop.
- **D6 [M] — `into_value` is lossy/partial.** Collapses `None`/`ContextualLink`/`RelayTo`/`Map` all
  to `Option::None` (`predicates.rs:37-42`), conflating "absent" with "dispatch/link". Faithful to
  Maybe only on `{None, Value}`.

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
  hypergraph layer. (Companion: the `RelayTo` P1 seam and this do-operator deferral are the two
  places where the algebra intentionally stops at the core boundary and continues in
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

## 2A. Decision — separate the control channel from the value functor (Option B)

**Decided (author): Option B.** `RelayTo` and `Map` are *control operations* (a computed jump; a
dispatch table), not values, and their presence inside `EffectValue<T>` is the root conflation
behind D5/D6/D14/D15 and the P1 block. Rather than making the conflation lawful in place (Option A:
recursive `fmap` + structural eq on the fused type), we **end the conflation**: the value variants
stay in `EffectValue`, the control operations move to a separate operation type consumed by a
handler.

```rust
enum EffectValue<T> { None, Value(T), ContextualLink(ContextoidId, ContextoidId) }  // pure value functor
enum CausalCommand<T> {                                                              // the operation functor `f`
    RelayTo(usize, Box<PropagatingEffect<T>>),
    Dispatch(HashMap<IdentificationValue, Box<PropagatingEffect<T>>>),               // was `Map`
}
```

This is the textbook **algebraic-effects / free-monad** shape (`Free f a = Pure a | Op (f (Free f a))`,
Plotkin & Power 2003; Swierstra 2008): `EffectValue` = the value/`Pure` part, `CausalCommand` = the
operation functor `f`, and the graph-reasoning BFS is the **handler** that interprets the jump. It
makes `EffectValue` a lawful pointed functor *unconditionally* (total `fmap`, derivable congruent
`PartialEq`, honest `into_value`, no fragment caveat) and **unblocks `core.causal_monad.lawful`**.

**Structuring:** the process outcome becomes a 3-way sum — `Value(EffectValue<V>) | Error(E) |
Control(CausalCommand<V>)` — keeping the W-invariant on value/error and giving control its own arm
(no re-widening of the carrier). **Scope:** this is a **breaking public-API change** to the adaptive-
reasoning surface (causaloids emit a control-carrier instead of `EffectValue::RelayTo(..)`; ~4
graph/CSM src consumers + ~7 test files move to the control arm). It is its **own OpenSpec change**
(working name `separate-control-channel`, the proper resolution of P1), a sibling to
`enforce-w-invariant` — not folded into the causal-arrow pass.

## 2B. Resolution ledger (every deviation gets a disposition — the spec derives from this)

Disposition: **Fixed** (code corrected already) · **Fix-planned** (own change) · **Documented
extension** (beyond a base def; keep, with its own laws) · **Accepted property** (intended
behavior; state precisely) · **Deferred** (correct home is another layer).

| # | Disposition | Resolution |
|---|---|---|
| D1 right-id conditional | **Fixed** | `and_then` now propagates `None` lawfully (right identity holds); `RelayTo`/`Map` leave the value type under B, so the collapse cases vanish. |
| D2 state/ctx not threaded | **Fix-planned (Option B, full)** | Value-only `and_then` already corrected to preserve state/context (steps.rs; 960 bazel tests pass). **Decided: no shortcut** — widen the arrow's stage to the state-receiving Kleisli arrow `(A,S,Option<C>) -> CausalFlow<B,S,C>` so composition threads state exactly as the monad's `bind` (and `CausalMonad.lean`) do; value-only stages stay as the lawful state-preserving sub-case. Full plan: `causal-arrow-state-threading-plan.md`. |
| D3 arrow bind ≠ monad bind | **Fixed** | `and_then` preserves `None → None` (chosen policy), matching the lawful Maybe-Kleisli bind. |
| D4 vestigial `Clone` bounds | **Fixed** | Removed from `KleisliCompose`; workspace builds + tests green. |
| D5 `PartialEq` is a PER | **Fix-planned (B) — fully resolved** | Both offenders leave `EffectValue`: `Map`/`Dispatch` (non-reflexive) and `RelayTo` (target-only) move to `CausalCommand`. The residual `EffectValue = {None, Value, ContextualLink}` has a genuine equivalence/congruence — now plain `#[derive(PartialEq)]`. **Requirement:** `CausalCommand` gets a *structural* eq (`RelayTo` payload + `Dispatch` map compared recursively) — a lawful congruence replacing the broken one; for the free monad over `CausalCommand`, program equality is by `fold`-canonicalization (as in the haft free-monad witnesses), so a derived eq isn't even required. The PER is eliminated, not relocated. |
| D6 `into_value` lossy | **Fix-planned (B) — resolved** | The defect was conflating "absent" with "dispatch" (`RelayTo`/`Map` → `None`); those leave. On the clean functor `into_value` is the faithful `Maybe` projection — `Value → Some`, `None`/`ContextualLink → None`. Total and honest: a `ContextualLink` genuinely has no scalar `T`. Distinguishing the two `None` reasons is `effect()`'s job, not `into_value`'s. |
| D7 do ≡ counterfactual | **Fixed** | `Intervenable::intervene` **removed from core** (trait + blanket impl + files); the value-substitution operation is now spelled `alternate_value` everywhere (`CausalFlow::intervene`→`alternate_value`, `intervene_if`→`alternate_value_if`; all callers in core/cfd/examples/benches migrated). The name "intervene"/`do()` is reserved for the graph layer where the real operator lives — no value-level over-claim remains. |
| D8 no graph mutilation | **Deferred** | True Pearl do-operator (graph surgery / variable isolation) belongs at the `deep_causality` Causaloid + hypergraph layer; core proves only the lens laws. |
| D9 idempotence up-to-log | **Accepted property** | The success-path audit entry is deliberate (Writer). Lens laws hold on the value projection `proj`; state the projected laws + machine-check the full-carrier log growth. Not a bug. |
| D10 `alternate_context` can't clear to `None` | **Fixed** | Added `CausalEffectPropagationProcess::clear_context` (inherent) — sets the context to `None` with a `!!ContextCleared!!` audit entry, no-op on an errored carrier (symmetric with `alternate_context`). Inherent rather than a trait method to avoid forcing the 3 cfd `AlternatableContext` impls; the core carrier — where the gap was — can now clear. Tested. |
| D11 `recover` = catch | **Documented extension** | `MonadError.catch`; a lawful extension beyond the Kleisli fragment (bind is left-zero on error). State its catch laws separately; keep. |
| D12 `iterate_until`/`iterate_to_fixpoint` inject `MaxStepsExceeded` | **Documented extension** | Bounded-search / fixpoint combinators with an intended failure mode; document their own contracts; keep. |
| D13 `finish` drops state/ctx/log | **Accepted property** | `finish` is the value-observation terminal (`Result<Value,Error>`); full extraction is via `into_process`/`into_parts`. Document the boundary. |
| D14 `map` ≠ `bind(pure∘f)` off `Value` | **Fixed (via D3)** | Now that `and_then` preserves `None`, `map f = and_then(pure∘f)` holds on the full `EffectValue`. *(To verify with a witness test.)* |
| D15 four disagreeing `fmap`s (one panics) | **Fix-planned (B) + must-fix bug** | Under B, `fmap` over the clean value functor is total/uniform and the `.expect` **panic** disappears; align the witness `fmap`s to the inherent one. The panic + non-reflexive `Map` eq are unconditional safety/soundness bugs to fix. |
| D16 CSV round-trip conditional | **Accepted property** *(confirm)* | The round-trip theorem is conditional on comma/newline-free fields (no escaping). Document the precondition as a hypothesis; RFC-4180 quoting is a possible future hardening. (Flag for author.) |
| D17 EffectLog timestamp/`PartialEq` | **Accepted property** | The timestamp-quotient in `PartialEq` is what makes the `List Λ` abstraction faithful; deliberate and sound. Document; not a bug. |

**Two soft items flagged for the author** (all others are decided): **D10** (keep context setter as-is
vs add `clear_context`) and **D16** (document the CSV precondition vs implement RFC-4180 quoting).
Once these two are settled, every deviation is resolved and the specification can be derived.

**Order of changes:** the causal-arrow corrections (D2/D3/D4) are **done**. `separate-control-channel`
(B — resolves D5/D6/D15 and completes D1) is the next change and should land the two must-fix bugs
(the `fmap` panic, the `Map` reflexivity) as part of it. The Lean/witness formalization then targets
the corrected code, so the proofs describe the faithful implementation rather than the deviations.

## 3. Execution order

Foundational → dependent, each verified with bare `lean` before moving on:
`EffectLog` → `EffectValue` → (`CausalMonad` reframe) → `CausalArrow` → `Alternatable` →
`CausalFlow` → `Consistency` → `Csv`. Then: Rust witnesses in
`deep_causality_core/tests/formalization_lean/`, THEOREM_MAP rows, and `deep_causality_core/LEAN_CORE.md`
(status table mirroring `LEAN_HAFT.md`).
