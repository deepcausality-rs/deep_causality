<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Causal Arrow — full state-threading refactoring (Option B, no shortcuts)

Resolves **D2** (composition does not thread State/Context) in
`core-formalization-plan.md`. Sibling to the D3/D4 fixes (already landed) and to
`separate-control-channel` (the RelayTo/Map Option B).

## 1. Goal & lawfulness target

Make the causal arrow the **faithful Kleisli category of the full state-passing causal monad** —
the same monad `Core/CausalMonad.lean` already proves lawful, whose `bind'` hands the continuation
`(value, state, context)` and whose unit `eta v s c` re-emits all three. Today the arrow's stage is
value-only (`A -> CausalFlow<B,S,C>`), which *discards* the state the monad threads, so it is a
Kleisli arrow only of the stateless sub-monad (`S=C=()`). Option B widens the stage to **receive**
state/context, so composition threads `s0 → stage1 → s1 → stage2 → s2` exactly as the monad does.

**Target theorems (over arbitrary `S`, `C`):** left identity `η >=> f = f`, right identity
`f >=> η = f`, associativity `(f >=> g) >=> h = f >=> (g >=> h)`, and error left-zero. These are the
arrow-from-monad laws (Mac Lane, *CWM* §VI.5: the Kleisli category of a lawful monad is a lawful
category) and reduce to `bind_left_id`/`bind_right_id`/`bind_assoc`/`bind_raise_left_zero`. Right
identity is unconditional on the value fragment now, and fully unconditional once
`separate-control-channel` removes the `RelayTo`/`Map` residue.

## 2. Design

### 2.1 The Kleisli arrow
- Stage type: `(A, S, Option<C>) -> CausalFlow<B, S, C>` — receives the incoming state/context,
  produces the next. In `deep_causality_haft::Arrow` terms: **`In = (A, S, Option<C>)`**,
  `Out = CausalFlow<B, S, C>`.
- Unit `η(a, s, c) = CausalFlow::from_parts(Ok(EffectValue::Value(a)), s, c, EffectLog::new())` —
  re-emit value/state/context, empty log (the `eta` of `CausalMonad.lean`).
- Composition `f >=> g = λ(a,s,c). bind'(f(a,s,c), g)` — run `f`, thread its output `(b, s1, c1)`
  into `g`, short-circuiting on error and propagating `None`/`ContextualLink` lawfully.

### 2.2 Components (all in `deep_causality_core/src/types/causal_arrow/`)
1. **`CausalLift` → state-receiving.** `F: Fn(A, S, Option<C>) -> CausalFlow<B,S,C>`;
   `Arrow<In=(A,S,Option<C>), Out=CausalFlow<B,S,C>>`; `run((a,s,c)) = f(a,s,c)`. Variance marker
   updated to `PhantomData<fn(A,S,Option<C>) -> (B,S,C)>`.
2. **`KleisliCompose::run` → thread `(value, state, context)`.** Run `p` on the tuple, extract
   `(b, s1, c1)` from `p`'s output flow via the new stateful bind, feed `g`. Drops the vestigial
   `Clone` bounds (already removed in the D4 fix — keep them out).
3. **Builder.** `causal_arrow_stateful(f)` (state-receiving) + `.next_stateful(g)`; **keep**
   `causal_arrow(f)` value-only + `.next(g)` as the **state-preserving** lawful special case
   (`|a,s,c| f(a).with_state(s,c)` — take `f`'s value/logs, preserve incoming state/context).
4. **`CausalArrow<A,B,S,C>` marker** → `Arrow<In=(A,S,Option<C>), Out=CausalFlow<B,S,C>>`.
5. **Run entry.** `arrow.run((a, s0, c0))`; add `run_value(a)` convenience for `S=C=()` that calls
   `run((a, (), None))`.

### 2.3 CausalFlow support (in `causal_flow/`)
- **Keep** the value-only `and_then`/`next` (already the lawful state-preserving Kleisli step after
  the D3 fix — its `Value` arm takes the continuation's outcome+logs and preserves incoming
  state/context). ~125 callers unaffected.
- **Add** `and_then_stateful<U, F>(self, f)` where
  `F: FnOnce(Value, State, Option<Context>) -> CausalFlow<U, State, Context>`: lowers to the monad
  `bind` (which passes `(value, state, context)`), running `f` on `Value`, propagating
  `None`/`ContextualLink`, erroring on `RelayTo`/`Map` (residue until `separate-control-channel`).
  This is the state-threading Kleisli bind `KleisliCompose` uses.
- **Add** `with_state(self, s: State, c: Option<Context>) -> Self` and/or confirm `from_parts` —
  build/rebind a flow with explicit state/context (needed by the value-only state-preserving lift
  and by `η`).

### 2.4 Why the value-only path stays lawful
A value-only stage lifted as `|a,s,c| f(a).with_state(s,c)` is the Kleisli arrow that **does not
touch state** — it preserves `(s,c)` and only transforms the value. That is a lawful morphism in the
same category; Option B *subsumes* the old value-only arrow rather than replacing it. So the common
stateless ergonomics (`causal_arrow(|x| …)`) are retained with zero regression.

## 3. Lean formalization

`lean/DeepCausalityFormal/Core/CausalArrow.lean` (self-contained, bare-`lean`):
- Stage = `A → S → Option C → Process B S C E Λ` (identical to `CausalMonad.lean`'s continuation).
- `karrow_id = eta`; `kcomp f g = fun a s c => bind' (f a s c) g`.
- Prove `kcomp_left_id`, `kcomp_right_id`, `kcomp_assoc`, `kcomp_left_zero` by reducing to the
  monad theorems already proved (arrow-from-monad). THEOREM_MAP ids `core.causal_arrow.category_laws`,
  `core.causal_arrow.left_zero`. Rust witnesses in
  `deep_causality_core/tests/formalization_lean/causal_arrow_tests.rs`.

## 4. Blast radius (verified)

- **Reified arrow engine** (`causal_arrow`/`CausalArrow`/`CausalLift`/`KleisliCompose`/
  `CausalArrowBuilder`): **no production or example user** — only the module, the `lib.rs`
  re-exports, and `causal_arrow_tests.rs`. Public impact = rewrite the arrow tests, extend the
  re-exports (add `causal_arrow_stateful`).
- **Value-only DSL** (`and_then`/`next`): preserved; ~125 callers unaffected. State-threading is
  purely additive.
- Net: **low, mostly additive.**

## 5. Ordering & interaction

- Independent of `separate-control-channel`; can land in either order. Full *unconditional* right
  identity needs the control-channel change (removes the `RelayTo`/`Map` error residue); note the
  residue in the interim docstring.
- Completes the causal-arrow correction trio: **D3 (done)**, **D4 (done)**, **D2 (this plan)**.

## 6. Task breakdown

1. `CausalFlow::with_state` / confirm `from_parts`; `η` helper.
2. `CausalFlow::and_then_stateful` (state-threading bind; preserve None/ContextualLink).
3. `CausalLift` → state-receiving (`In = (A,S,Option<C>)`).
4. `KleisliCompose::run` → thread `(value, state, context)`.
5. Builder: `causal_arrow_stateful` + `.next_stateful`; keep value-only `causal_arrow`/`.next`.
6. `CausalArrow` marker trait signature; `run_value` convenience.
7. `lib.rs` re-exports.
8. `causal_arrow_tests.rs`: keep value-only tests (adjust for `run_value`), add state-threading
   tests — a reusable stateful pipeline threads accumulated state; left/right id; assoc; error
   short-circuit preserves state.
9. `Core/CausalArrow.lean` + Rust witnesses + THEOREM_MAP rows; bare-`lean` typecheck.
10. Update `mod.rs` / `compose.rs` docs — the state/context-threading claim is now true and precise.
11. Verify: `bazel test //...`; bare-`lean` on the new file.

## 7. Deviation-ledger update

D2 resolution upgrades from "Fixed (value-only, S=C=() scope)" to **"Fixed — full state-threading
Kleisli arrow of the causal monad (Option B); value-only stages retained as the lawful
state-preserving sub-case."** The arrow now matches the carrier `CausalMonad.lean` proves.
