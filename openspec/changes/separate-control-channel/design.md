## Context

`EffectValue<T>` (`deep_causality_core/src/types/effect_value/mod.rs`) currently holds five variants:
three values (`None`, `Value`, `ContextualLink`) and two control operations (`RelayTo`, `Map`). The
control variants force hand-written unlawful `PartialEq` (`partial_eq.rs:14,16`), a lossy `into_value`
(`predicates.rs`), and disagreeing/panicking `fmap`s (`hkt.rs`, arity-5 `.expect`). The carrier outcome
is `Result<EffectValue<Value>, Error>` (post `enforce-w-invariant`). The adaptive-reasoning handlers
that consume `RelayTo`/`Map` are enumerated: `graph_reasoning/mod.rs`, `graph_reasoning/stateful.rs`,
`csm/eval.rs`, `causable_stateful.rs` (all in `deep_causality`), plus ~7 test files. Design source:
`core-formalization-plan.md §2A`.

## Goals / Non-Goals

**Goals:**
- Split values from control: `EffectValue` becomes the pure value functor; `CausalCommand` becomes the
  control operation functor with a lawful congruent equality.
- Make the carrier outcome a 3-way sum `Value | Error | Control`, W-invariant preserved on value/error.
- Retire deviations D5/D6/D14/D15 and the two safety bugs (the `fmap` panic, non-reflexive `Map` eq).
- Preserve all observable adaptive-reasoning behavior (jump targets, error messages).

**Non-Goals:**
- The `Free`-monad Lean proof over `CausalCommand` and `core.causal_monad.lawful` — those are the
  downstream `formalize-deep-causality-core` change (this change makes them possible; it does not add
  the proofs).
- Any new adaptive-reasoning capability; this is a structural refactor, behavior-preserving at the
  handler.
- RFC-4180 / unrelated cleanups.

## Decisions

### D1. Option B (separate) over Option A (make the fusion lawful)
Option A — recursive `fmap` + structural eq on the fused `EffectValue` — was rejected in `§2A`. It
keeps control inside the value functor, so `into_value` stays semantically muddled ("dispatch" is not
"a value") even if made total. Option B matches the algebraic-effects decomposition (`Free f a`), makes
`EffectValue` lawful *unconditionally*, and gives the graph BFS its honest name: a handler. Cost: a
breaking API change across the core→deep_causality boundary. Accepted.

### D2. 3-way outcome, not a re-widened value carrier
The carrier outcome becomes `Value(EffectValue<V>) | Error(E) | Control(CausalCommand<V>)`. The
alternative — folding control back into `EffectValue` as a fourth arm — would re-introduce the exact
conflation being removed. Keeping control as a *sibling* of value/error preserves the W-invariant on the
value/error arms (the `enforce-w-invariant` guarantee) and gives control a clean accessor `control()`.

### D3. `Map` → `Dispatch` rename
`Map` collides with the ubiquitous functor `map`/`fmap` and reads as a container, not an operation.
`Dispatch` names what it is (a dispatch table). Done as part of the move so no stale name survives.

### D4. Structural congruent equality on `CausalCommand`
`RelayTo` compares target + boxed payload recursively; `Dispatch` compares the map structurally. This
replaces the PER with a genuine congruence. Where feasible this is `#[derive(PartialEq)]` (requires
`PropagatingEffect<T>: PartialEq` — verify; the boxed effect already carries a `PartialEq`). For the
downstream free-monad witnesses, program equality is by `fold`-canonicalization anyway, so a derived eq
is a convenience, not a correctness dependency.

### D5. Handler seam is the existing BFS
No new handler is written. The graph-reasoning BFS that already resolves `RelayTo` becomes the
interpreter of `CausalCommand`; the change is purely which arm it matches (`control()` vs
`effect()`), with identical downstream behavior. This keeps the blast radius at "change the match arm,"
not "rewrite reasoning."

## Risks / Trade-offs

- **[Breaking API across a crate boundary]** → The consumer set is small and enumerated (4 src + ~7
  tests). `bazel test //...` covers all of it; migrate consumers in the same change so the workspace is
  never red between commits.
- **[`PropagatingEffect<T>: PartialEq` may not hold for all `T`]** → If a derive is impossible, hand-
  write the congruent eq (target+payload / structural map) — still lawful, just not derived. Verify
  early (task 1).
- **[Hidden `RelayTo`/`Map` matches outside the enumerated set]** → A full `grep` for `RelayTo`/`Map`
  gates completion (task 6); the compiler's exhaustiveness check on the removed variants is the
  backstop — every match site must be updated or it won't build.
- **[Behavior drift in reasoning]** → The spec pins jump-to-target and missing-target-error scenarios;
  existing graph-reasoning tests (moved to the control arm) are the regression guard.

## Migration Plan

1. Verify `PropagatingEffect<T>: PartialEq` (derive feasibility for `CausalCommand`).
2. Add `CausalCommand<T>` (variants, `Functor`, congruent eq, display/predicates).
3. Remove `RelayTo`/`Map` from `EffectValue`; derive `PartialEq`; make `fmap` total; fix `into_value`;
   drop the arity-5 `.expect` panic.
4. Widen the carrier outcome to the 3-way sum; add `control()`; update constructors/getters/hkt.
5. Migrate `deep_causality` consumers (graph reasoning ×2, CSM eval, stateful causaloid) + tests to the
   `Control` arm.
6. `grep` clean for `EffectValue::RelayTo`/`EffectValue::Map`; `bazel test //...`; `make format && fix`.

Rollback: revert the change set; no data/state migration involved (compile-time types only).

## Open Questions

- Should `Dispatch` keep `HashMap` (std-only, as `Map` was `#[cfg(feature = "std")]`) or move to a
  no_std-friendly map? (Recommendation: keep `HashMap` under `#[cfg(feature = "std")]` to preserve
  current feature-gating; revisit only if a no_std consumer needs dispatch.)
