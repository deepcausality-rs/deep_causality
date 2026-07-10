<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Causaloid Formalization

This note formalizes the causaloid **form by form**, from the current implementation, per the
"three local algebras + meta-signature" plan. It does **not** assume a single algebra fits all
three forms. Each section states the structure, gives the exact code↔math correspondence, and flags
anything that is representable-but-not-formalized. If a form has no gap, it says so.

Foundations first: the singleton causaloid denotes a morphism in the Kleisli category of the
**causal monad**, so Part I formalizes that monad before Part II formalizes the singleton.

Sources: `deep_causality_core/src/types/causal_effect_propagation_process/mod.rs`,
`.../causal_monad/mod.rs`, `.../effect_value/mod.rs`;
`deep_causality/src/types/causal_types/causaloid/{causable.rs,causable_utils.rs,mod.rs}`.

---

## Reconciliation (2026-07-10) — the `CausalEffect` model

> This note was written against the pre-refactor **`EffectValue`** model: a 5-arm value sum
> (`None + Value + ContextualLink + RelayTo + Map`) carried alongside a *separate* `error` field on
> the arity-5 carrier. That model is gone. The body below is preserved as the historical derivation;
> the current model, and every claim it changes, is enumerated here. Sources for the current model:
> `deep_causality_core/src/types/{causal_effect/mod.rs, causal_command/mod.rs,
> causal_effect_propagation_process/mod.rs, propagating_effect/mod.rs}`.
>
> **Model now.** The success channel is
> `CausalEffect<V> = Free<CausalCommandWitness, Option<V>>` — the free monad on the single control
> operation `CausalCommand::RelayTo(target, sub)` over `Maybe` value leaves:
> `Pure(None)` (no evidence), `Pure(Some v)` (a value), `Suspend(RelayTo(t, k))` (a command). The
> carrier is `CausalEffectPropagationProcess { outcome: Result<CausalEffect<V>, E>, state, context,
> logs }` — value **and** error share one channel (`outcome`), and the full outcome is the transformer
> stack `Except E (Free CausalCommand (Maybe V))`. `ContextualLink` and `Map` no longer exist.
>
> Each changed claim, keyed to the flags in the body:
>
> | Body claim | Current status |
> |---|---|
> | Carrier `M(T) = EffectValue(T) × S × Option(C) × Option(E) × L` (I.1) | **Changed.** Value-XOR-error is one channel: `outcome: Result<CausalEffect<V>, E>`, fields `(outcome, state, context, logs)`. |
> | `EffectValue` = 5-arm sum with `ContextualLink`, `Map` (I.1) | **Changed.** `CausalEffect<V> = Free<CausalCommand, Option<V>>`; arms are `Pure(None)`, `Pure(Some v)`, `Suspend(RelayTo)`. `ContextualLink`/`Map` **removed**; `RelayTo` is the sole command. |
> | **F-0** — `EffectValue` is a fixpoint over `PE(T)` via `RelayTo`/`Map`, not a plain functor | **Superseded.** `CausalEffect` *is* the free monad `Free<CausalCommand, Option<V>>` — an honest functor with `Option<V>` leaves; the recursion is the `Free` structure with `RelayTo` its one operation (`Map`, the arm that made it a `PE`-fixpoint, is gone). Value functor is lawful; `fold` is the unique handler (`core.causal_effect.fold_universal`, `.transformer_stack`). |
> | **F-1** — well-formedness `error ⇒ value=None` not type-enforced; right identity conditional | **CLOSED.** One `Result` channel makes "value AND error" unrepresentable; `new` is total; right identity holds **unconditionally**, machine-checked (`core.causal_monad.right_id`, tracker #7). This dissolves the F-1 caveat inherited by F-6 and the "conditional right identity" note in I.5. |
> | **F-2** — log monoid is free (non-commutative) | **Unchanged.** Still a `Vec`-append monoid; order-sensitivity is on the log channel only. |
> | **F-3** — input/output asymmetry (command inputs, structured outputs) | **Now a stated theorem** (`causaloid-formalization` spec; task 2.5). The singleton detects a command on the input channel (`incoming_effect.command_target().is_some()`) and returns the *command-specific* error — never a silent `None`; command **outputs** still pass through for the engine to fold. Behaviour unchanged; the flag becomes a named, witnessed law. |
> | **F-4** — representable invalid singletons (missing/both closures) | **Unchanged.** The `causal_fn`/`context_causal_fn` `Option` fields still admit "missing both"/"both set"; still well-formed only by construction. A causaloid-construction concern, not a monad-carrier one. |
> | **F-5** — dual stateless/stateful semantics; context as constant | **Unchanged.** Two interpreters; context read-only (#11b). |

---

## Part I — The Causal Monad

### I.1 Carrier

For fixed parameters `(S, C, E, L)` the carrier is the record

```
M(T)  =  EffectValue(T) × S × Option(C) × Option(E) × L
```

(`CausalEffectPropagationProcess<T, S, C, E, L>`; fields `value, state, context, error, logs`).

The propagating effect fixes `S = 1` (unit), `C = 1`, `E = CausalityError`, `L = EffectLog`:

```
PE(T)  =  M_{1, 1, CausalityError, EffectLog}(T).
```

`EffectValue(T)` is itself a sum, and two of its arms embed `PE(T)` recursively:

```
EffectValue(T) = 1                                  (None)
               + T                                  (Value)
               + ContextoidId × ContextoidId        (ContextualLink)
               + ℕ × PE(T)                           (RelayTo)        ← recursive
               + (Id ⇀ PE(T)).                       (Map)            ← recursive
```

> **F‑0 (flag).** `EffectValue` is not a plain functor `T ↦ T + …`; via `RelayTo`/`Map` it is a
> fixpoint over `PE(T)`. The singleton's *interpretation* (Part II) only consumes the `Value` arm
> and only re‑emits the others by pass‑through, so this recursion does not enter the singleton
> algebra — but it is the reason the carrier is not a simple `Maybe`/`Either` composite.

### I.2 Unit

```
η(t)  =  ( Value(t),  s₀,  None,  None,  ε )
```

`s₀ = S::default()`, `ε = ` empty log (`pure`, `causal_monad/mod.rs`).

### I.3 Bind (the actual operation)

The bind is **not** the textbook Kleisli bind `A → M B`. Its continuation receives the whole
`EffectValue`, the threaded state, and the read context, and the log accumulates:

```
f : EffectValue(A) × S × Option(C) → M(B)

bind(m, f) =
  if m.error = Some(e):                                   -- error short‑circuit
      ( None, m.state, m.context, Some(e), m.logs )
  else:
      let n = f(m.value, m.state, m.context) in
      ( n.value, n.state, n.context, n.error, m.logs ⧺ n.logs )   -- logs accumulate
```

(`CausalEffectPropagationProcess::bind`, `causal_effect_propagation_process/mod.rs`.)

So the monad **composes three effects at once**:

| Effect | Mechanism | Algebra |
|---|---|---|
| Writer (audit log) | `m.logs ⧺ n.logs` | `L` is a **monoid** `(L, ⧺, ε)` (append, empty) |
| Exception (error) | `error = Some ⇒` skip `f`, propagate | short‑circuit / `Either`-like |
| State + Reader | `f` reads `(state, context)`, returns next `state`/`context` | state‑threading + context‑reading |

### I.4 Derived operations

- **`bind_or_error(m, f, msg)`** = bind that *unwraps* the value: error ⇒ short‑circuit;
  else `m.value.into_value()` gives `Some(v) ⇒ f(v, s, c)` (with log merge) or `None ⇒ error(msg)`
  (state/context/logs preserved). Crucially `into_value` returns `Some` **only** for the `Value`
  arm; `None`/`ContextualLink`/`RelayTo`/`Map` all become the `msg` error.
- **`fmap(m, f)`** = error short‑circuit; else `Value(v) ↦ Value(f v)`, `None ↦ None`,
  `ContextualLink ↦` pass‑through, `RelayTo`/`Map ↦ ValueNotAvailable` error.

### I.5 Laws and well‑formedness

Define a process **well‑formed** iff `error = Some ⇒ value = None`.

- **Left identity.** `bind(η(a), f) ≡ f(Value(a), s₀, None)`. Holds by construction (`η(a).logs = ε`,
  `η(a).error = None`). Note the equality is *up to* `η` injecting `(s₀, None)` — `η` overwrites
  state to the default and context to `None`.
- **Associativity.** Holds (log append is associative; error short‑circuit and state threading
  compose).
- **Right identity.** `bind(m, η')` ≡ `m` **iff `m` is well‑formed.** On error, bind sets
  `value = None`, so a process with `error = Some` *and* `value = Value(_)` is not recovered.
  (Matches `CausalMonadProptest`; assumptions tracker #7.)

> **F‑1 (flag).** Well‑formedness is **not type‑enforced**: the struct fields are `pub` (within the
> crate) and nothing makes `error = Some ⇒ value = None` a type invariant. It is *maintained* by the
> public constructors (`pure`, `from_error`, `from_value`, …) and by `bind` (which sets `value =
> None` on the error path). On the constructor‑reachable subset the monad laws hold unconditionally;
> the raw type admits ill‑formed values for which right identity fails. **Not fully formalized:** the
> invariant is a convention, not a guarantee.

> **F‑2 (flag).** The log monoid `L` is a *free* monoid (ordered list of entries). Therefore the
> monad is order‑sensitive in the log channel by construction — the same observation recorded for
> Collection (#1). For the singleton this is benign (a single linear chain), but it means `M` is a
> Writer over a non‑commutative monoid; any later claim of order‑independence is about the `value`
> channel only.

**Verdict (Part I).** The carrier is an arity‑5 effect monad = Writer(log monoid) ∘ Exception ∘
State/Reader. Unit, bind, and associativity match the code exactly. Right identity is **conditional**
on a well‑formedness invariant that the code maintains but the type does not enforce (F‑1).

---

## Part II — The Singleton Causaloid

### II.1 Data

`Causaloid<I,O,STATE,CTX>` with `causal_type = Singleton` carries (other fields `None`):

```
id        : IdentificationValue
causal_fn         : Option( CausalFn<I,O> )            -- CausalFn<I,O> = fn(I) -> PE(O)   (a Kleisli arrow)
context_causal_fn : Option( ContextualCausalFn<…> )    -- fn(EffectValue(I), STATE, Option(CTX)) -> PProcess(O)
context           : Option( CTX )
```

(`causaloid/mod.rs`; constructors `new`, `new_with_context`.)

### II.2 Denotation: a context‑parameterized Kleisli arrow

A well‑formed singleton denotes a single morphism in `Kleisli(PE)`:

```
⟦singleton⟧  :  I ⤳ O          i.e.   I → PE(O),
```

parameterized by the causaloid's own (read‑only) `context`. The public `evaluate` is the **Kleisli
extension** of that arrow, restricted to `Value` inputs:

```
evaluate : PE(I) → PE(O)
evaluate(m) = ( m  ⟫ₑ  logIn_id  ⟫ₑ  exec  ⟫  finalize )
```

where `⟫ₑ` = `bind_or_error`, `⟫` = `bind`, and the three stages are Kleisli arrows:

```
logIn_id : I → PE(I)          i ↦ η(i) with log entry "Causaloid id: Incoming effect: Value(i)"
exec     : I → PE(O)          i ↦ causal_fn(i)            if causal_fn = Some
                              i ↦ π(context_fn(Value(i), s₀, ctx))   if context_causal_fn = Some
                              i ↦ error("missing both")   otherwise
finalize : EffectValue(O) → PE(O)
              Value(v)        ↦ logOut_id(v) = η(v) with "Outgoing effect: Value(v)"
              None            ↦ error("causal_fn returned None output")
              ContextualLink / RelayTo / Map ↦ from_effect_value(·)   -- pass‑through, no out‑log
```

`π` projects `PProcess(O) → PE(O)`: keep value/logs, map `error`/`None‑value` to an error effect,
drop the threaded state (`causable_utils::execute_causal_logic`).

### II.3 Correspondence (code ↔ math)

| Code | Formal object |
|---|---|
| `causal_fn : fn(I) -> PE(O)` | Kleisli arrow `I → PE(O)` (the generator) |
| `evaluate(&self, m)` | Kleisli extension `evaluate = (− ⟫ₑ logIn ⟫ₑ exec ⟫ finalize)` |
| `bind_or_error(f, msg)` | `bind` restricted to the `Value` arm; non‑`Value` ⇒ `error(msg)` |
| `log_input` / `log_output` | Writer arrows `η(·)` ⧺ one log entry |
| `execute_causal_logic` | `exec`, with the `π` projection for the context path |
| incoming error | short‑circuit (bind law) |

The chain is exactly `η`/`bind`/`bind_or_error` over the Part‑I monad, so the singleton **is** a
morphism in `Kleisli(PE)` and `evaluate` **is** its (Value‑restricted) extension. This identification
holds with the following gaps.

### II.4 Gaps — not fully formalized

> **F‑3 — input/output asymmetry (the morphism is not an endomorphism on `PE`).**
> Input is restricted to the `Value` arm: `bind_or_error` sends an incoming `None`/`ContextualLink`/
> `RelayTo`/`Map` to the error `"input value is None"`. But `finalize` *passes through*
> `ContextualLink`/`RelayTo`/`Map` outputs. So the singleton accepts only `Value`‑carrying inputs yet
> may emit structured outputs. Formally `⟦singleton⟧` is a morphism on the sub‑object `Value(I) ↪ PE(I)`,
> **not** a total endomorphism on `PE`. A clean Kleisli/category statement must name this domain
> restriction; the implementation enforces it dynamically (by erroring), not by type.

> **F‑4 — representable invalid singletons (well‑formedness only by construction).**
> The type permits `causal_fn = None ∧ context_causal_fn = None` (→ runtime `"missing both"` error)
> and `causal_fn = Some ∧ context_causal_fn = Some` (→ `causal_fn` is silently shadowed; the context
> arm wins). Only the constructors (`new`, `new_with_context`) build well‑formed singletons. So
> "`singleton = Kleisli arrow`" is a theorem about **constructor‑built** singletons, not about every
> inhabitant of the struct. **Not fully formalized:** the well‑formedness (exactly one closure set)
> is not type‑enforced (no sum type for `causal_type` payloads; `Option` fields instead).

> **F‑5 — two evaluation semantics; context is a constant; stateless path drops state.**
> There are two interpreters: the stateless `MonadicCausable::evaluate` formalized above, and a
> stateful `StatefulMonadicCausable::evaluate_stateful` (`execute_causal_logic_stateful`) that threads
> `STATE`/`CTX`. The stateless `evaluate` calls the context closure with `s₀ = STATE::default()` and the
> causaloid's **own stored** `context` (read‑only), discarding any incoming state — faithful only when
> `S = 1` (which `PE` fixes). So the singleton is a **context‑parameterized** arrow `ctx ⊢ I → PE(O)`
> with `ctx` an immutable constant (assumptions tracker #11b). The two semantics must be unified or one
> declared canonical before the meta‑layer; this note formalizes the stateless one.

> **F‑6 — inherited monad‑law caveat.** The singleton's laws are those of Part I; right identity
> carries the F‑1 invariant. Benign for a single arrow, relevant when singletons are composed.

> Out of scope here: the `Collection`/`Graph` arms of `evaluate` return `"not available"` errors
> (the split‑brain of tracker #10); they are formalized in their own sections.

### II.5 Verdict

**The singleton is faithfully a (context‑parameterized) Kleisli arrow `I → PE(O)`, and `evaluate`
is its `Value`‑restricted extension `PE(I) → PE(O)`.** Unit/bind/log/error mechanics match the code
exactly. It is **not yet fully formalized** in four respects, all flagged: the input/output
asymmetry F‑3 (domain is the `Value` sub‑object, not all of `PE`), the representable invalid
singletons F‑4 (well‑formedness only by construction, not by type), the dual stateless/stateful
semantics + context‑as‑constant F‑5, and the inherited conditional right‑identity F‑6 (F‑1).

None of these break the core identification; each is a precise place where the *type* permits more
than the *algebra* allows. The cleanest closure is to make well‑formedness structural (a single
closure field; an `error ⇒ value=None` smart constructor) and to fix the input domain explicitly —
but that is a (justifiable) change, recorded here, not made.

---

## Status

Reconciled to the `CausalEffect` model 2026-07-10 (see the Reconciliation banner above); the
per-form status below reflects the current model.

- **Part I — Causal Monad:** formalized; matches code. **F‑1 CLOSED** (single value-XOR-error
  channel; right identity unconditional, `core.causal_monad.right_id`). Only F‑2 remains (log
  non‑commutative, by design).
- **Part II — Singleton:** formalized; matches code on the constructor‑reachable subset. **F‑3 is
  now a stated theorem** (`causaloid-formalization` spec; task 2.5 — command input on the value
  channel yields the command‑specific error). F‑4/F‑5 unchanged (by construction; dual semantics).
  F‑6 dissolved with F‑1.
- **Collection:** closure landed (`core.verdict.closure`, `causaloid-verdict-closure`); the #1
  permutation‑invariance theorem is the residual task 2.3 (`causaloid-formalization` spec).
- **Graph:** join semantics decided and landed — the defined merge `∇ ∘ (Λ₁ ⊗ Λ₂)` with
  `∇ = Verdict::join` (`core.causaloid.graph_fold_order_invariant`, tracker #2); relay-round
  composition is the residual task 3.6 (`graph-reasoning-formalization` spec).
- **Meta‑signature:** landed as the catamorphism keystone (`core.causaloid.catamorphism_unique`).
