# Note: Property-Testing the Causal Monad

Status: **design / test-spec.** Specifies how to property-test the monad laws of the
`CausalMonad` (`deep_causality_core`). Companion to `arrow-assumption.md` (§2/A4: the laws
*cannot* be type-encoded in Rust, so property tests are the correct and sufficient bar; A1: the
existing suite tests *behavior*, not laws). Grounded in source as of this writing — verify
file:line before relying on a citation.

> **Reconciliation (2026-07-10) — the `CausalEffect` model.** This note predates the single-channel
> refactor. The value channel is no longer `EffectValue<Value>` (the 5-arm sum) but
> `CausalEffect<V> = Free<CausalCommandWitness, Option<V>>` (`Pure(None)`, `Pure(Some v)`,
> `Suspend(RelayTo)`; `ContextualLink`/`Map` removed), and value **and** error now share one channel:
> `CausalEffectPropagationProcess { outcome: Result<CausalEffect<V>, E>, .. }` with **private** fields
> and a **total** `new`. Three claims below change:
>
> - **§2 Law 2 (right identity) — now unconditional.** "Holds *iff* the invariant
>   `error.is_some() ⇒ value == None`" is superseded: value-XOR-error is one `Result`, so an errored
>   process *cannot* also carry a value. Right identity `m.bind(eta) ≡ m` holds on **all** carriers,
>   machine-checked as `core.causal_monad.right_id` (tracker #7).
> - **The "hand-built invariant-violating `m`" test — now impossible, hence dropped.** With private
>   fields and a single `Result` channel there is no representable value-AND-error state to construct;
>   the "generate one explicit invariant-violating `m` and document that it fails" step (§2, §6
>   `prop_right_identity`) is obsolete.
> - **§7 "the `error ⇒ value None` invariant, still open" — CLOSED.** The design recommendation
>   ("private fields + smart constructors, or normalize in one place") is exactly what landed: the
>   single-channel `Result` makes the invariant structural.
>
> Unchanged: §3 (the timestamp-agnostic `EffectLog` equality — already landed, still the correct law
> equality) and §1/§2 Laws 1 and 3 (left identity, associativity hold as stated). The generator
> restriction to the *data* variants (§4) now reads "`Pure(Some v)`/`Pure(None)` leaves; `RelayTo`
> commands out of the first suite's scope."

## 0. Why this note, and the two things that make it non-trivial

The monad/arrow laws are universally-quantified equations over programs; Rust (like Haskell)
cannot encode them in types (`arrow-assumption.md` §2). Property testing is therefore not a
fallback — it is the standard of evidence, the same standard the EPP monograph meets for
inference by *proof* (§4.1) and the same one `arrow-assumption.md` A1 flags as missing here:
`core/tests/types/causal_monad/causal_monad_tests.rs` checks `pure`/`bind`/`fmap` *behavior*
(value, log count, state threading, error short-circuit) but has **no left-identity,
right-identity, or associativity test**. This note closes that gap.

Two facts about the actual implementation reshape the task and must drive the test design:

1. **The bind is non-standard**, so the laws must be *restated* before they can be tested (§1–§2).
2. **`EffectLog` timestamps every entry with `SystemTime::now()`** and is compared structurally,
   so the derived `PartialEq` is the **wrong equality** for law checks — a custom,
   timestamp-agnostic equality is mandatory (§3).

## 1. The monad as implemented (the signature that reshapes the laws)

`PropagatingEffect<T> = CausalEffectPropagationProcess<T, (), (), CausalityError, EffectLog>`;
the stateful carrier is `PropagatingProcess<T, S, C>`
(`core/src/types/causal_effect_propagation_process/mod.rs:41`,
`core/src/traits/causal_monad/mod.rs:34`).

The `bind` continuation does **not** take a bare value — it takes the whole triple
`(EffectValue<Value>, State, Option<Context>)` (the inherent `bind`,
`…/causal_effect_propagation_process/mod.rs:66`):

- **error short-circuit:** if `self.error` is `Some`, return a process with
  `value = EffectValue::None` (`EffectValue::default()`, `#[default]` on `None`,
  `effect_value/mod.rs:30`), the *same* `state`/`context`/`error`/`logs` — `f` is **not** called.
- **otherwise:** `next = f(self.value, self.state, self.context)`, then
  `logs = self.logs ++ next.logs` (via `Vec::append`, `effect_log/log_effect.rs:89`), and the
  result's `state`/`context`/`value`/`error` are **`next`'s**.

`pure(v)` = `{ value: Value(v), state: S::default(), context: None, error: None, logs: empty }`
(`causal_monad/mod.rs:80`).

Because the continuation receives the triple, the **unit for these laws is not `pure`** but the
triple-level *return*:

```rust
// η — the return appropriate to THIS bind: re-wrap the triple, no error, empty log.
fn eta<V, S, C>(v: EffectValue<V>, s: S, c: Option<C>)
    -> CausalEffectPropagationProcess<V, S, C, CausalityError, EffectLog> {
    CausalEffectPropagationProcess { value: v, state: s, context: c, error: None, logs: EffectLog::new() }
}
// pure is η precomposed with the default-state / no-context seed:
//   pure(a) ≡ eta(EffectValue::Value(a), S::default(), None)
```

This restatement is not pedantry — `m.bind(pure)` does not even type-check (`pure` takes one
arg, the continuation takes three), so right identity *cannot* be written in textbook form.

## 2. The three laws, restated — with the verdict and the failure mode for each

### Law 1 — Left identity
**Statement (for this bind):** `pure(a).bind(f) ≡ f(EffectValue::Value(a), S::default(), None)`.
**Verdict: holds by construction.** `pure(a)` has no error, so `bind` calls
`f(Value(a), default, None)` and prepends an *empty* log (no-op); state/context/value/error are
`f`'s. The result equals the RHS exactly.
**What the test guards (regression, not discovery):** (i) `pure` seeds `(Value(a), default, None,
no error, empty log)`; (ii) `bind` prepends *empty* logs — a future "improvement" that makes
`pure` stamp a log entry, or makes `bind` log on entry, silently breaks left identity. (iii) the
default-state seed is what `f` receives — meaningful for stateful carriers.

### Law 2 — Right identity  *(the one with a real, findable failure)*
**Statement:** `m.bind(eta) ≡ m`.
**Verdict: holds iff the invariant `error.is_some() ⇒ value == None` holds.**
- Non-error `m`: `bind` calls `eta(m.value, m.state, m.context)` = `{m.value, m.state, m.context,
  None, empty}`, then logs `= m.logs ++ empty = m.logs`; result `= {m.value, m.state, m.context,
  None, m.logs} = m`. ✔
- Errored `m`: `bind` short-circuits to `{None, m.state, m.context, m.error, m.logs}`, which
  equals `m` **only if `m.value` was already `None`**.

The constructors maintain that invariant — `from_error` and `none` set `value: None`
(`…/mod.rs:140,151`), every short-circuit sets `value: None`. **But the struct fields are
`pub`**, so a hand-built `CausalEffectPropagationProcess { value: Value(x), error: Some(e), .. }`
violates it, and right identity *fails* for it (bind clears the value). **Test both**: generate
`m` through constructors/bind (right identity must hold) *and* one hand-built invariant-violating
`m` (document that it fails). **Design recommendation:** if you want *unconditional* right
identity (and a cleaner monad), enforce `error ⇒ value None` — make the fields private behind
smart constructors, or normalize in one place — then right identity becomes total.

### Law 3 — Associativity
**Statement:** `m.bind(f).bind(g) ≡ m.bind(|v, s, c| f(v, s, c).bind(g))`.
**Verdict: holds.** Threading is identical on both sides: state/context flow through `f` then `g`
the same way; the error short-circuit carries the *same* `(state, context, logs)` regardless of
grouping; and log concatenation is `Vec::append`, which is associative and order-preserving. The
identical messages on both sides get *different* `SystemTime::now()` timestamps, but value
equality compares messages, not timestamps (§3 — now fixed at the source), so `==` is the right
test. (Historically this discrepancy was the reason §3 prescribed a custom equality; the
`EffectLog` fix removed the need.)

## 3. The equality problem — surfaced here, now **fixed at the source**

Writing this note surfaced a latent bug: `EffectLog` derived `PartialEq`/`Eq` *structurally*
while every entry carries a `timestamp_micros` from `SystemTime::now()`. Because the process
derives `PartialEq`, the derived `==` compared logs **including wall-clock timestamps** — so any
two independently-built processes with non-empty logs compared *unequal*, making `==`
non-deterministic and breaking value comparison generally (not just law tests).

**Resolved** (`effect_log/log_effect.rs`): `EffectLog`'s `PartialEq` is now hand-written to
compare the **message sequence only** (timestamp-agnostic, an order-preserving equivalence;
`Eq` retained), and a `pub fn eq_with_timestamps(&self, &Self) -> bool` provides the strict
compare when the temporal record is the subject. Blast radius was nil (nothing keyed on or
`==`-compared `EffectLog`); 125 core + 24 main-crate tests stay green; 5 tests added.

**Consequence for law-testing — the workaround this note originally prescribed is no longer
needed.** The process's derived `==` is now itself the correct, timestamp-agnostic law equality:

```rust
// Before the fix this note defined a `law_eq` that compared fields and only `logs.len()`,
// because LogEntry is pub(in crate::types) and external tests cannot read messages.
// After the fix: just use `==`. It compares value/state/context/error structurally and logs
// by message sequence — exactly the law equality.
assert_eq!(lhs, rhs);                       // monad-law equality
assert!(lhs.logs.eq_with_timestamps(&rhs.logs)); // ONLY if a test asserts timing specifically
```

This also makes log *content* (not just `len()`) checkable from an external `tests/` crate
without any new accessor: `EffectLog`'s `==` already compares messages. The earlier two-tier
discussion and the `messages()` accessor suggestion are superseded by the fix.

Aside: under `--no-default-features` (no `std`) the timestamp is hard-coded `0`
(`log_effect.rs:62`), making the derived `==` deterministic — handy for a quick example-based
check, but the property suite should still use `law_eq` so it is correct under `std`.

## 4. Generators without a framework (dependency-free, per house style)

`proptest`/`quickcheck` appear **only** in vendored `thirdparty/` crates; no DeepCausality crate
uses them and `deep_causality_core` has **no `[dev-dependencies]`**. Honor that: hand-roll
generation. (If policy ever admits a dev-only property crate, `proptest` would be the mainstream
choice — but it is macro-heavy and `std`-only, against the repo's zero-macro/zero-dep ethos.)

**Deterministic PRNG** (reproducible, seedable, no deps):

```rust
struct Rng(u64); // xorshift64; seed per case from a fixed base so failures reproduce
impl Rng {
    fn next(&mut self) -> u64 { let mut x = self.0; x ^= x << 13; x ^= x >> 7; x ^= x << 17; self.0 = x; x }
    fn int(&mut self, lo: i32, hi: i32) -> i32 { lo + (self.next() % (hi - lo) as u64) as i32 }
    fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T { &xs[(self.next() as usize) % xs.len()] }
}
```

**Continuations as generated *data*, not closures.** You cannot `Arbitrary`-generate arbitrary
functions, but the laws only need a *representative* function space. Generate a small step
"syntax" and interpret it into a continuation of the bind shape — the standard trick for
higher-order laws:

```rust
#[derive(Clone)]
enum Step { Incr(i32), Log(&'static str), Set(i32), Fail(&'static str) }

// Build a FRESH continuation from a (cloned) Step each time — bind consumes FnOnce, and
// associativity calls each continuation on both sides, so never reuse a moved closure.
fn cont(step: Step) -> impl FnOnce(EffectValue<i32>, i32, Option<String>) -> PropagatingProcess<i32, i32, String> {
    move |v, s, c| {
        let cur = v.into_value().unwrap_or_default();
        match step {
            Step::Incr(k) => mk(EffectValue::Value(cur), s + k, c, None, "incr"),
            Step::Log(m)  => mk(EffectValue::Value(cur), s, c, None, m),
            Step::Set(x)  => mk(EffectValue::Value(x), s, c, None, "set"),
            Step::Fail(m) => mk(EffectValue::None, s, c, Some(CausalityError::new(CausalityErrorEnum::Custom(m.into()))), m),
        }
    }
}
```

**Coverage to generate:** restrict `EffectValue` to `Value(_)` and `None` (the *data* variants);
`ContextualLink`/`RelayTo`/`Map` are routing *commands* embedding sub-`PropagatingEffect`s
(`fmap` even errors on `RelayTo`/`Map`, `…/mod.rs:199`) — their monad-law contract is a separate
question, out of scope for the first suite. Test **both carriers**: the stateless
`PropagatingEffect<T>` (`S = C = ()`) and a stateful `PropagatingProcess<i32, i32, String>`,
since state threading is where associativity earns its keep. Include error-producing steps so the
short-circuit branch of every law is exercised.

## 5. Test layout

Mirror the source tree (AGENTS.md): add
`core/tests/types/causal_monad/causal_monad_law_tests.rs` beside the existing
`causal_monad_tests.rs`, registered in that directory's `mod.rs`. With the §3 fix in place these
can be plain external `tests/` (no in-crate access needed — `==` already compares log messages).
Each law is one `#[test]` that loops N seeded cases (e.g. 1_000) and asserts with `==`, printing
the seed on failure so any counterexample reproduces.

## 6. Skeleton (dependency-free; fill in `mk`, imports)

```rust
// Rng, Step, cont as above. mk(value, state, ctx, error, log_msg) builds a process.
// Equality is the process's derived `==` — timestamp-agnostic since the §3 fix.

#[test]
fn prop_left_identity() {
    let steps = [Step::Incr(3), Step::Log("a"), Step::Set(9), Step::Fail("boom")];
    let mut rng = Rng(0x1234_5678);
    for _ in 0..1_000 {
        let a = rng.int(-50, 50);
        let step = rng.pick(&steps).clone();
        let lhs = PropagatingProcess::<i32, i32, String>::pure(a).bind(cont(step.clone()));
        let rhs = cont(step)(EffectValue::Value(a), i32::default(), None);
        assert_eq!(lhs, rhs, "left identity: a={a}");
    }
}

#[test]
fn prop_right_identity() {
    // generate m THROUGH constructors/bind so the `error ⇒ value None` invariant holds,
    // then assert m.bind(eta) == m. Add one explicit invariant-violating m and assert it is NOT
    // equal, documenting the precondition (see §2).
}

#[test]
fn prop_associativity() {
    // for random m, f=cont(sf), g=cont(sg):
    //   lhs = m.clone().bind(cont(sf.clone())).bind(cont(sg.clone()));
    //   rhs = m.bind(move |v,s,c| cont(sf)(v,s,c).bind(cont(sg)));
    //   assert_eq!(lhs, rhs)  // `==` is now timestamp-agnostic (§3 fix); use
    //                         // logs.eq_with_timestamps only when asserting timing
}
```

## 7. What this buys — and what it does not

Passing these establishes the monad laws **empirically**, for the generated function space and
carriers — the fidelity standard `arrow-assumption.md` A1 asks for, and a precondition for the
Kleisli⇒Arrow lift (A0a: Hughes' construction needs the *monad* laws to hold for the derived
*arrow* laws to follow). It does **not** *prove* the laws (no dependent types — §2/A4); a
crafted carrier outside the generated space could still violate them. The honest packaging:
once the suite is green, a marker `trait LawfulCausalMonad: CausalMonad {}` may *assert-and-
require* the tested claim downstream (the ceiling Rust allows, `arrow-assumption.md` §2) — its
trustworthiness is exactly this suite's, no more. Two findings here were independently actionable
regardless of the laws: the **timestamped-log equality bug** — *now fixed* (`EffectLog` `==`
compares messages; `eq_with_timestamps` for the strict compare; §3) — and the **`error ⇒ value
None` invariant**, still open: enforce it (private fields + smart constructors, or normalize in
one place) to make right identity total rather than conditional.

Cross-references: `arrow-assumption.md` (§1 A1 monad-laws-untested, §2/A4 why not type-level);
`causal-arrow-generalization.md` (the Kleisli⇒Arrow lift these laws underwrite).
