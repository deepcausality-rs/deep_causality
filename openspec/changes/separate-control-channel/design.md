## Context

`EffectValue<T>` (`deep_causality_core/src/types/effect_value/mod.rs`) currently holds five variants:
three values (`None`, `Value`, `ContextualLink`) and two control operations (`RelayTo`, `Map`). The
control variants force hand-written unlawful `PartialEq` (`partial_eq.rs:14,16`), a lossy `into_value`
(`predicates.rs`), and disagreeing/panicking `fmap`s (`hkt.rs`, arity-5 `.expect`). The carrier outcome
is `Result<EffectValue<Value>, Error>` (post `enforce-w-invariant`).

Two facts from the survey drive this design:
1. **`Map` is dead.** It is constructed only in its own unit tests (`effect_value_tests.rs`, 4 sites)
   plus the `is_map`/`display`/`partial_eq` arms. No production, example, or reasoning code builds or
   reads it — the graph handler matches `RelayTo` and lets everything else fall to `_`. It is removed,
   not renamed.
2. **`RelayTo` is already a free-monad shape.** The handler (`graph_reasoning/mod.rs:143-172`) reads
   `RelayTo(target, inner_effect)` and continues reasoning at `target` fed by `inner_effect` — and
   `inner_effect` is itself a `PropagatingEffect<T>` that can again be a value or another `RelayTo`.
   That recursion *is* `Free f a = Pure a | Op (f (Free f a))`: an effect is a value-leaf or a control
   operation carrying sub-programs, and the BFS engine is the fold/handler.

Design source: `core-formalization-plan.md §2A`; the reused machinery is `deep_causality_haft::Free`
(`deep_causality_haft/src/monad/free_monad.rs`, laws proved as `haft.free_monad.*`).

## Goals / Non-Goals

**Goals:**
- Remove `Map` (dead) and its tests.
- Move `RelayTo` out of `EffectValue` into a **control operation functor** `CausalCommand<K>` and make
  the adaptive-reasoning program a **`haft::Free<CausalCommandWitness, EffectValue<V>>`** — the value/
  `Pure` part is `EffectValue`, the operation functor is `CausalCommand`, and the graph BFS is
  `Free::fold` (the F-algebra handler). Reuse `haft::Free`; do not reinvent it.
- Make `EffectValue = { None, Value, ContextualLink }` a lawful pointed functor (total `fmap`, derived
  congruent `PartialEq`, honest `into_value`); retire D5/D6/D14/D15 and the two safety bugs (the `fmap`
  panic, the non-reflexive `Map` eq).
- Carrier outcome 3-way `Value | Error | Control`, W-invariant preserved on value/error.
- Preserve observable reasoning behavior (jump targets, missing-target error, log combination).

**Non-Goals:**
- `core.causal_monad.lawful` and the Lean witness *mirror* — the downstream `formalize-deep-causality-core`
  change. This change proves `CausalCommand` is a lawful functor (so `haft::Free`'s already-proved
  monad laws apply) but does not re-prove the free-monad laws (they are `haft.free_monad.*`).
- Any new reasoning capability; behavior-preserving refactor.

## Decisions

### D1. Option B (separate) over Option A (make the fusion lawful)
Option A — recursive `fmap` + structural eq on the fused `EffectValue` — keeps control inside the value
functor, so `into_value` stays muddled ("a jump is not a value"). Option B matches the algebraic-effects
decomposition and makes `EffectValue` lawful *unconditionally*. Accepted (breaking, across the
core→deep_causality boundary).

### D2. Remove `Map` entirely; `CausalCommand` is a single-operation functor
`Map` is dead (verified). It is deleted along with `is_map`, its `display`/`partial_eq` arms, and its 4
unit tests — not renamed to `Dispatch`. `CausalCommand` therefore has exactly one operation:

```rust
pub enum CausalCommand<K> {                 // the operation functor f; K is the sub-program hole
    RelayTo(usize, K),
}
```

A single-hole functor — the simplest non-trivial operation functor, and all the reasoning engine ever
used. (If a dispatch/table operation is ever genuinely needed, it is added then, as a second variant
with real callers.)

### D3. `CausalCommand` is the operation functor of `haft::Free` (full reuse)
Reuse the haft free monad literally:
- `CausalCommandWitness` implements `HKT<Constraint = NoConstraint>` (`Type<K> = CausalCommand<K>`) and
  `Functor<CausalCommandWitness>` (`fmap` maps the single `RelayTo` hole).
- The adaptive-reasoning **program** is `Free<CausalCommandWitness, EffectValue<V>>`:
  `Pure(EffectValue<V>)` leaves (value / `None` / `ContextualLink`) and `Suspend(RelayTo(target, sub))`
  branches. `PropagatingEffect<V>`'s outcome is isomorphic to this Free.
- The graph BFS becomes the **handler** `Free::fold(pure_case, algebra)`: `pure_case` emits a leaf
  effect; the `algebra` interprets `RelayTo(target, sub)` — reset traversal, jump to `target`, feed it
  `sub`, thread state/context/logs (the current `combined_logs` behavior). The monad laws over the
  program come free from `haft.free_monad.*`; this change only proves `CausalCommandWitness` is a lawful
  functor (identity + composition on the single hole), which is the precondition Free needs.

State/context/logs stay **carrier-level** and are threaded by the fold algebra as it walks (matching
today's log combination) — the Free program carries value/control structure, not per-node state.

### D4. Congruence via structure / fold-canonicalization
`CausalCommand<K>` derives `PartialEq` when `K: PartialEq` (single-hole, structural — a genuine
congruence, unlike the old `Map(_) == Map(_) = false`). Program equality for `Free<CausalCommandWitness, _>`
is by `fold`-to-canonical-value, exactly as the haft free-monad witnesses do (sidestepping the
recursive-GAT `PartialEq` trait-solver overflow documented in `free_monad.rs`). The old
partial-equivalence relation is eliminated, not relocated.

### D5. 3-way outcome; `control()` returns the Free program
The carrier outcome is `Value(EffectValue<V>) | Error(E) | Control(Free<CausalCommandWitness, EffectValue<V>>)`.
A plain value stays in the fast `Value` arm (no `Box`/`Free` allocation); anything with a jump is a
`Control` program. `control() -> Option<&Free<CausalCommandWitness, EffectValue<V>>>`; `effect()`
discriminates `{ None, Value, ContextualLink }` only. W-invariant holds on value/error; control is a
sibling arm (no re-widening of the value carrier).

### D6. Handler rewritten as `Free::fold`
Per the "full reuse" decision, the BFS is re-expressed as a fold over the Free program rather than an
ad-hoc `match` on an `EffectValue` variant. The observable behavior (jump-to-target, missing-target
error, log combination, stop-on-control) is preserved and pinned by the existing reasoning tests moved
to the control arm.

## Risks / Trade-offs

- **[Reasoning-engine refactor to a fold]** → Larger than a match-arm swap. Mitigated by: the operation
  functor is single-hole (simplest case), the fold algebra transcribes the current BFS body, and the
  full graph-reasoning test suite (moved to the control arm) is the regression guard. `bazel test //...`
  gates it.
- **[`Free` `Fn + Clone` bind / `Box` recursion]** → The reasoning engine *folds* programs, it does not
  build them via `bind`, so the `Fn + Clone` bind constraint is mostly irrelevant here; `fold` needs
  only the functor. `Box` recursion is already how `RelayTo` nests today.
- **[`EffectValue<T>: PartialEq` derive]** → With control gone, `EffectValue` is `#[derive(PartialEq)]`
  directly. `CausalCommand<K>` derives when `K: PartialEq`; else programs compare by fold-canon.
- **[Hidden `Map`/`RelayTo` sites]** → The compiler's exhaustiveness check on the removed `EffectValue`
  variants is the backstop; a full `grep` gates completion.
- **[Alloc gating]** → `Free`/`Box` require `alloc`; `CausalCommand`/the control arm are gated exactly as
  the haft `Free` is (list `alloc` explicitly for Bazel, as the free-monad tests already do).

## Migration Plan

1. Remove `Map` from `EffectValue` + `is_map` + `display`/`partial_eq` arms + its 4 unit tests.
2. Add `CausalCommand<K> { RelayTo(usize, K) }`, `CausalCommandWitness: HKT + Functor`, structural/fold
   equality; export from `lib.rs` (gated on `alloc`).
3. Remove `RelayTo` from `EffectValue`; `#[derive(PartialEq)]`; make `fmap` total; fix `into_value`;
   drop the arity-5 `.expect` panic.
4. Widen the carrier outcome to `Value | Error | Control(Free<CausalCommandWitness, EffectValue<V>>)`;
   add `control()`; update constructors/getters/hkt.
5. Rewrite the graph-reasoning handler(s) as `Free::fold` over the control program; migrate `csm/eval.rs`
   and `causable_stateful.rs` to the `Control` arm; move the affected tests.
6. Prove `CausalCommandWitness` functor laws (Lean + witness) citing `haft.free_monad.*` as the base.
7. `grep` clean for `EffectValue::RelayTo`/`EffectValue::Map`; `bazel test //...`; `make format && fix`.

Rollback: revert; compile-time types only.

## Open Questions

- None. (`Map`/`HashMap` removed per verification; `Free` reuse depth settled as "carrier holds
  `haft::Free`.")
