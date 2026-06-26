<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# The Causal Algebra

**Goal.** Present the Causal Monad *algebraically*, with **Kleisli composition as the core operator**
("the extension of the Kleisli operator") and a small fixed set of causal faculties as generating
operations. The intended theorem is **parity**: the monad induced by this algebra is the Causal
Monad, so "Causal Monad" and "Causal Algebra" are two presentations — computational and equational —
of one structure.

This note is a construction *and* an honest ledger: every claim is marked **[holds]**, **[holds under
precondition]**, or **[open]**. It is written to survive a seasoned PL/category-theory reviewer, which
means it does **not** claim more than the construction earns.

Companion to [`Causaloid-Formalization.md`](../causaloid/Causaloid-Formalization.md) (singleton already
formalized) and the assumptions tracker (#2 graph join, #5 verdict carrier, #7 monad lawfulness).

---

## 0. Two preconditions, stated up front

The algebra is sound only on a cleaned base. Both are *necessary* and currently *not met* by the raw
type; a reviewer will check them first.

- **(P1) Control-free.** Exclude the `RelayTo` constructor (a computed jump = a *control* effect; not
  presentable by operations + equations — it needs handlers). Treat `Map` likewise for the core.
  Remaining value shape: `EffectValue⁻(T) = None + Value(T)` (optionally `+ ContextualLink`, modelled
  as a context read; see §4). **[required]**
- **(P2) Lawful carrier.** Impose the well-formedness invariant **W: `error = Some ⇒ value = None`**,
  and make it structural (a smart constructor or a single sum, not `pub` fields). Under W the carrier
  collapses to a genuine, recognizable monad (§1) and the laws become unconditional. **[required]**

Everything below assumes (P1) and (P2). Without them the "monad" fails right identity (tracker #7)
and the theory is not algebraic.

---

## 1. Carrier (the Causal Monad, cleaned)

For fixed `(S, C, E, L)`:

```
M(T)  ≅  (S × Option(C))  ⟶  ( Either E (Maybe T) ,  S ,  Option(C) ,  L )
```

i.e. a computation reads-and-threads state `S` and context `Option(C)`, may fail with `E`, otherwise
yields `Maybe T` (`None` = "no signal", `Value` = a signal), and always accumulates a log in the
monoid `L = (L, ⧺, ε)`. Under **W** the pair `(EffectValue⁻(T), Option(E))` is exactly
`Either E (Maybe T)` (the forbidden state `value = Value ∧ error = Some` is unrepresentable), which is
why W is load-bearing.

Read as a transformer stack, the bind's behaviour (error keeps the accumulated log *and* the threaded
state/context; success threads both and appends the log) fixes the order:

```
M  =  StateCC ∘ ExceptT E ∘ Writer L            -- StateCC = the (S, Option C) state pair
```

with `ExceptT` *outside* `Writer` (so a partial log survives an error) and the state pair *outside*
`ExceptT` (so state survives an error). **Note for the write-up:** `context` is *threaded*, not
read-only — `bind` takes the continuation's returned context — so context is a second **state**
channel, not a Reader, even though causaloids use it read-only. **[holds: this is a standard stack;
the only obligation is to prove the implementation's `bind` equals this stack's bind, which (P2)
makes true]**

`PE(T) = M_{1,1,CausalityError,EffectLog}(T)` (unit state, unit context).

---

## 2. The Kleisli operator (the propagation operator)

This is the heart: the algebra's binary operation **is** Kleisli composition of `M`.

- **Causal arrow.** `f : A ⤳ B  :=  A → M(B)`. A causal arrow is *one causal step*: from an
  antecedent it produces a consequent effect (a value, updated state, accumulated provenance, possible
  failure).
- **Unit (trivial cause).** `η_A : A ⤳ A`, `η(a) = ( Value(a), s₀, None, None, ε )`.
- **Propagation (composition).** For `f : A ⤳ B`, `g : B ⤳ C`:

```
(f >=> g)  :  A ⤳ C
(f >=> g)(a)  =  f(a)  >>=  g
```

  where `>>=` is the causal bind of §1: error short-circuits (keeping state/context/log); otherwise
  thread state/context and append logs. This `>=>` **is the EPP axiom** `m₂ = m₁ >>= f`, read as the
  composition of causal steps.

**Definition (Causal Algebra, version 0).** The Causal Algebra is the category
`𝓒 = (Ob, ⤳, >=>, η)` — the Kleisli category of `M` — together with the generating operations of §4
and the equations of §3 and §4. `>=>` is its multiplication; `η` is its unit.

---

## 3. Category laws (the algebra is associative and unital)

Under (P1)+(P2):

- **Left identity.** `η >=> f = f`. **[holds]** (`η` has empty log and no error; `ε ⧺ w = w`.)
- **Right identity.** `f >=> η = f`. **[holds under W]** — exactly the law that fails without W
  (on `error = Some ∧ value = Value`); W removes that state, so it holds unconditionally.
- **Associativity.** `(f >=> g) >=> h = f >=> (g >=> h)`. **[holds]** (`⧺` associative; state threading
  and error short-circuit compose; inherited from the stack of §1.)

So `(𝓒, >=>, η)` is a genuine category. **[holds under P1,P2]**

---

## 4. Generating operations (the causal faculties)

The "extension" of bare Kleisli composition is a finite signature of **algebraic effects**
(Plotkin–Power). Each is a generator of the theory `𝒯_Causal`; together with `>=>` they generate every
causal arrow.

| Operation | Signature | Faculty | Equational laws (selected) |
|---|---|---|---|
| `tell` | `L → 1` | provenance / audit log | `tell(ε)=skip`; `tell(a);tell(b)=tell(a⧺b)` (monoid action) |
| `raise` | `E → 0` | hard failure | `raise(e); k = raise(e)` (left zero / short-circuit) |
| `none` | `1 → 0` | absence of signal | at a value demand, `none = raise(NoSignal)` (the `bind_or_error` rule) |
| `get` / `put` | `1→S` / `S→1` | internal (Markovian) state | the four state laws (get-put, put-get, put-put, get-get) |
| `getc`/`putc` | `1→Option C` / `Option C→1` | context (threaded; used read-only) | same state laws on the context channel |

> **Design note (context).** Causaloids *read* context, but `M`'s `bind` *threads* it, so the honest
> signature is `getc`/`putc` (state-shaped). If the write-up wants context to be a Reader (`ask` only,
> no `putc`), that is a **deliberate restriction of `M`** to be made and enforced — currently the type
> permits `putc`. Decide and state it. **[open: Reader vs State for context]**

**Theory.** `𝒯_Causal` = the algebraic theory with these operations modulo these equations (the sum
of Writer, Exception, and two State theories, with their standard non-interaction). **[holds: each is a
textbook algebraic effect; the combination is the standard sum/tensor]**

---

## 5. Parity (the target theorem)

> **Theorem (parity, to be discharged).** The free monad of `𝒯_Causal` is isomorphic to `M`; equivalently,
> `Kl(M) ≅` the classifying category of `𝒯_Causal`. Hence **Causal Monad ≅ Causal Algebra**.

**Proof strategy.** Writer(`L`), Exception(`E`), and State(`S`), State(`Option C`) are each presented by
the operations/equations in §4 (standard results). `M` is their combination in the order fixed by §1.
The free monad of the combined theory is that same combination. The only non-standard step is the
`Maybe`/`None` value layer, handled by P1 (no recursive `RelayTo`/`Map`) folding `None` into the
exception layer at value-demand points (the `none = raise(NoSignal)` law). **[holds under P1,P2 for the
control-free core; the assembly is routine but must be written out — it is not yet written]**

**Explicitly out of scope of parity:** `RelayTo` (control → handlers), `Map` (structured dispatch),
and `ContextualLink` if kept as a first-class structured value rather than a context read. These are
the **extension layer**, formalized separately (handlers, not algebra). **[open]**

---

## 6. Causaloids inside the algebra

- **Singleton** = a *generated morphism* of `𝓒`:
  ```
  ⟦c⟧  =  (tell ∘ in_id)  >=>  k_c  >=>  (tell ∘ out_id)        :  A ⤳ B
  ```
  where `k_c : A → M(B)` is the causaloid's base causal function lifted as a causal arrow, and
  `in_id`/`out_id` emit the incoming/outgoing log entries. This matches `Causaloid::evaluate` exactly
  (the `bind_or_error ∘ bind` pipeline), **modulo F-3** (the pipeline restricts the input to `Value`,
  so `⟦c⟧` is a morphism on the sub-object `Value(A) ↪ M(A)`, not all of `M(A)`). **[holds under
  P1,P2; F-3 must be stated]**
- **Collection** = an aggregation: evaluate each child to `M(verdict)`, run the accumulated effect via
  an **Eilenberg–Moore algebra** `α : M(List V) → List V`, then fold with a **commutative monoid**
  `∇ : List V → V` (the `AggregateLogic`). The monoid `∇` and the verdict carrier `V` are *additional*
  structure beyond `>=>` (the EM-algebra side, not the Kleisli side). **[open: name `V` (#5); state
  `∇` and that order-independence is value-only and up-to-log (#1, DECIDED)]**
- **Graph** = composition in `𝓒` extended with an explicit **copy** `Δ` and **join** `∇_G`. The join is
  the undecided operator: today the engine silently drops all but one parent at a reconvergent node.
  No graph algebra exists until `∇_G` is defined. **[open: tracker #2]**

So the three forms split exactly along monad theory: **Kleisli arrows** (singleton), **EM-algebra +
monoid** (collection), **Kleisli + copy/join dataflow** (graph) — one monad, three uses.

---

## 7. Ledger — what is earned vs owed

**Earned (under P1, P2):**
- `>=>` is a lawful associative, unital composition; `(𝓒, >=>, η)` is a category. [§3]
- The causal faculties are algebraic effects with a standard presentation. [§4]
- The singleton is a generated morphism matching the code. [§6]
- Parity for the control-free core is a routine assembly of standard results. [§5]

**Owed (must be done before the monograph claims a sound Causal Algebra):**
1. **Enforce W (P2)** structurally so `M` is a genuine monad (close tracker #7). *This is the gating fix.*
2. **Remove `RelayTo`** from the core (P1); relocate it to a handler layer or future work.
3. **Write the parity proof** of §5 (assembly + the `none = raise` step).
4. **Decide context = Reader or State** (§4 note) and enforce it.
5. **Verdict carrier `V`** for collection (#5); state `∇` and the scoped order-independence (#1).
6. **Define the graph join `∇_G`** (#2) — the only fully-open item; until then the Causal Algebra
   covers singleton + collection, not graph.

**Honest scope for the book:** with 1–4 done, you can soundly state "the Causal Algebra: an algebraic
theory whose free monad is the Causal Monad, with Kleisli composition as the propagation operator,"
and formalize singleton + collection. The **graph** and the **control (`RelayTo`) layer** should be
presented as precisely-posed open problems, not as solved — that is the framing a reviewer accepts.
