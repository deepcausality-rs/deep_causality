## Context

`deep_causality_effects` is a thin heterogeneous-container layer. Its load-bearing surface is two enums plus an adapter ring of `From` impls:

- **`EffectData`** (12 variants) — the polymorphic payload for an effect propagated through a causal graph. Variants cover scalars (`Bool`, `Float(f64)`, `Int(i64)`), structured numerics (`Numerical(NumericValue)`, `Vector`), heterogeneous storage (`MultiVector(CausalMultiVector<f64>)`, `Tensor(CausalTensor<f64>)`), topology (`PointCloud`, `SimplicialComplex`, `Manifold`), and the `Custom(Arc<dyn Any + Send + Sync>)` escape hatch.
- **`NumericValue`** (13 variants) — an explicit polymorphic numeric value: `None`, the four unsigned integer widths (`U8` through `U128`), the four signed integer widths (`I8` through `I128`), plus `F32(f32)` and `F64(f64)`. Used wherever a numeric value's runtime type needs to be carried explicitly (e.g. cross-language interop, serialization round-trips, late-binding effect interpretation).
- **Adapter impls** — eight `From<...>` impls forming the producer→`EffectData` boundary. Each impl is shape-specific.

R0 (`generalize-topology-over-realfield`) and `generalize-physics-over-realfield` upstream this change set's preconditions. Both install temporary `::<f64>` pins at effects-side consumption sites, tagged `// TEMP: removed by generalize-effects-over-realfield`. Removing those pins is one of this change set's deliverables.

The enum redesign is the load-bearing design question. The mechanical retypes (`EffectData::Float(f64)` → `Float(R)`; the eight `From` impls; the temp pin removals) are downstream of that decision. Stakeholders: anyone propagating effects through causal graphs at non-`f64` precision; anyone consuming the post-R0 / post-physics generic surfaces; anyone building serialization round-trips on `NumericValue`.

This proposal scopes the design only. Specs and tasks are deferred until the enum-redesign decision is committed.

## Goals / Non-Goals

**Goals:**

- After this change set ships, the public API of `deep_causality_effects` contains **zero** hardcoded `f64` or `f32` in any struct field, function/method signature, trait method, error variant, trait bound, or `pub const` declaration.
- `EffectData` is parameterized over `R: RealField`; its four `f64`-baked variants propagate `R` through their storage.
- `NumericValue` is redesigned per the chosen strategy (Decision 1) so it composes with `R: RealField` without breaking serialization round-trips.
- Every R0 and physics temporary pin is removed.
- The eight cross-crate `From<...>` adapter impls retype to consume `R`-parameterized inputs.
- Hard rip-and-replace; no bridge code; propagation through libraries; concrete `R` only at end-consumer sites.

**Non-Goals:**

- Redesigning `EffectData::Custom(Arc<dyn Any>)`. The `dyn Any` boundary is intentional — it's the type-erased escape hatch for effect payloads the type system can't enumerate. Untouched.
- Adding new variants to `EffectData` (e.g. for ratios, quaternions, or specialized physics quantities). Out of scope.
- Generalizing over `ComplexField<R>`. No complex-valued variant exists in `EffectData` today; not part of this change set.
- Redesigning `Effect`, `EffectMap`, or the effect-propagation algorithms. Only the data type changes.

## Decisions

### Decision 1: `NumericValue` enum redesign — three candidates

This is the load-bearing decision. The `F32(f32)` and `F64(f64)` variants are explicitly precision-tagged; a single generic `R: RealField` parameter has to either replace them or coexist with them. Three candidates:

**Candidate A: Collapse to `Real(R)`.**

```rust
pub enum NumericValue<R: RealField> {
    None,
    U8(u8), U16(u16), U32(u32), U64(u64), U128(u128),
    I8(i8), I16(i16), I32(i32), I64(i64), I128(i128),
    Real(R),
}
```

**Trade-offs:** clean. The whole enum is precision-parametric. Serialization round-trips are unambiguous (the round-trip preserves `R`). The cost: callers that need run-time precision dispatch (e.g. a deserializer that reads a precision tag from a byte stream and constructs either an `F32` or `F64` accordingly) lose the explicit variants — they must commit to a single `R` at construction time. For the audit-identified call sites in this crate (numeric-value display, simple From impls), this is fine; for external serialization consumers, it's a breaking-change surface.

**Candidate B: Preserve `F32(f32)` and `F64(f64)`; add `Real(R)`.**

```rust
pub enum NumericValue<R: RealField> {
    None,
    /* integer variants */
    F32(f32),
    F64(f64),
    Real(R),
}
```

**Trade-offs:** preserves the run-time dispatch capability. Cost: violates the "zero hardcoded `f64` / `f32`" invariant — `F32` and `F64` variants are by definition hardcoded. This candidate is **rejected**.

**Candidate C: Split the type.**

```rust
pub enum NumericValueInt {
    None,
    U8(u8), U16(u16), U32(u32), U64(u64), U128(u128),
    I8(i8), I16(i16), I32(i32), I64(i64), I128(i128),
}
pub enum NumericValueReal<R: RealField> {
    None,
    Real(R),
}
pub enum NumericValue<R: RealField> {
    Int(NumericValueInt),
    Real(NumericValueReal<R>),
}
```

**Trade-offs:** maximum structural clarity. Cost: triples the type surface; every pattern-match site at the call boundary becomes two levels deep; serialization round-trips need to encode the outer discriminator before the inner. Effort: ~10 h vs. ~4 h for Candidate A.

**Recommendation: Candidate A.** Clean, satisfies the invariant, minimal effort. The lost run-time precision-dispatch capability has no concrete consumer in the audited call sites; if a future consumer needs it, they can reconstruct via `NumericValue::<f64>` or `NumericValue::<f32>` explicit type parameters at construction time, or build a side-table that maps a discriminator byte to a precision-tagged constructor function.

This decision is deferred to implementation time; the proposal documents the three candidates so the choice is reviewable.

### Decision 2: `EffectData<R: RealField>` is the load-bearing parameterization

```rust
pub enum EffectData<R: RealField> {
    Bool(bool),
    Float(R),
    Int(i64),
    Numerical(NumericValue<R>),
    String(String),
    Vector(Vec<R>),
    MultiVector(CausalMultiVector<R>),
    Tensor(CausalTensor<R>),
    PointCloud(PointCloud<R, R>),
    SimplicialComplex(SimplicialComplex<R>),
    Manifold(SimplicialManifold<R, R>),
    Custom(Arc<dyn Any + Send + Sync>),
}
```

Every pattern-match site at the call boundary updates from `EffectData::Float(x: f64)` to `EffectData::<f64>::Float(x: f64)` (or whichever `R`). The `Custom` variant is unchanged — `Arc<dyn Any>` is type-erased and orthogonal to the parameterization.

### Decision 3: Cross-crate adapter `From` impls retype to consume `R`-parameterized inputs

The eight `From<...>` impls in `src/types/effect_data/effect_data_from.rs` retype:

- `From<f64> for EffectData<R>` becomes `From<R> for EffectData<R>`.
- `From<f32> for EffectData<R>` is removed (no longer meaningful; `f32` is one valid `R`).
- `From<NumericValue> for EffectData<R>` becomes `From<NumericValue<R>> for EffectData<R>`.
- `From<CausalMultiVector<f64>> for EffectData<R>` becomes `From<CausalMultiVector<R>> for EffectData<R>`.
- Same shape for `CausalTensor`, `PointCloud`, `SimplicialComplex`, `SimplicialManifold`.

Each retype is mechanical once the enum redesign is in place.

### Decision 4: `Custom(Arc<dyn Any + Send + Sync>)` is untouched

The `dyn Any` boundary is intentional. Even after this change set ships, consumers downcasting `Arc<dyn Any>` to a concrete type (e.g. `Mass<f64>`) name a concrete `R`. The escape hatch's run-time type erasure doesn't propagate `R` through `dyn Any`. This is by design — `dyn Any` is the documented "I'm carrying something the type system can't enumerate" boundary.

### Decision 5: Hard rip-and-replace, propagation, no bridge code

Same policy as R0 and physics:

- No type aliases (`EffectData64`, `NumericValue64`) — rejected.
- No default type parameters (`<R = f64>`) — rejected.
- No parallel `f64`-flavored variants — rejected (Candidate B above is rejected for this reason).
- No `From<f64>` impls preserved alongside the generalized `From<R>` — rejected.

Library code propagates `R`; only end-consumer call sites bind a concrete `R`.

## Risks / Trade-offs

- **[Risk] The `NumericValue` redesign breaks serialization round-trips for external consumers.** If a downstream consumer reads a `NumericValue::F64` from a byte stream and dispatches differently than for `NumericValue::F32`, the loss of those discriminators is a behavior change.
  → **Mitigation:** the audit found no such consumer in the workspace. External consumers (if any) would adapt by encoding the precision tag separately in their serialization format. Document the breaking-change shape in the change set's release notes.

- **[Risk] `EffectData::Custom(Arc<dyn Any>)` downcast sites pin to concrete `R`.** A producer that constructs `EffectData::Custom(Arc::new(Mass::<f64>::new(5.0)))` and a consumer that downcasts to `Mass<f64>` work, but a producer at `Mass::<f32>` and a consumer expecting `Mass<f64>` silently fail at run time.
  → **Mitigation:** this is the inherent semantics of `dyn Any` and not introduced by this change set. Document the constraint in the doc comment of `EffectData::Custom`.

- **[Risk] Pattern-match sites at every consumer.** Every `match` on `EffectData` needs to acknowledge the new variant payload types. If the workspace has many consumers, the migration ripple is wide.
  → **Mitigation:** workspace-wide grep for `match.*EffectData` and `match.*NumericValue` at implementation time enumerates every site. The fix is mechanical (add `::<R>` annotation, update payload type).

- **[Trade-off] Lowest-priority of the three change sets.** Effects work waits for R0 and physics. The temporary pins R0 and physics install live until this change set ships.
  → **Justification:** the user has flagged this as lower-priority. Effects' enum redesign is genuine design work; rushing it produces a worse outcome than waiting. The temporary pins are tagged and greppable, so the technical debt is bounded and visible.

## Migration Plan

1. **Preconditions:** R0 (`generalize-topology-over-realfield`) and `generalize-physics-over-realfield` have shipped. Confirm by re-grep for `// TEMP: removed by generalize-effects-over-realfield` across the workspace — every hit is a pin to remove in this change set.
2. **Commit the `NumericValue` redesign decision** (Candidate A recommended). The proposal documents the candidates; the implementation pass commits to one.
3. **Retype `EffectData`** to `EffectData<R: RealField>` and propagate to its 12 variants.
4. **Retype `NumericValue`** per Decision 1.
5. **Rewrite the eight `From<...>` adapter impls.**
6. **Remove every `// TEMP: removed by generalize-effects-over-realfield` pin** in this crate and its upstream consumers.
7. **Update consumers** of `EffectData` and `NumericValue` across the workspace. Pattern-match sites add `::<R>` or update payload type names.
8. **Add `f32` duplicates** for the algorithmically-meaningful tests (effect propagation invariants, serialization round-trips, `From` impl correctness).
9. **Verification:** `cargo build --workspace`, `cargo test --workspace`, the four invariant greps from R0 task 11 inside `deep_causality_effects/src/`. Zero hits required.

## Open Questions

1. **Candidate A, B, or C for `NumericValue`?** Recommendation: A. The implementation pass commits to one and updates this document.
2. **Does `EffectData::Vector` mean `Vec<f64>` today or `Vec<NumericValue>`?** Audit-time check. If `Vec<f64>`, retypes to `Vec<R>`. If `Vec<NumericValue>`, retypes to `Vec<NumericValue<R>>`.
3. **Are there workspace consumers downcasting `EffectData::Custom(Arc<dyn Any>)` to physics or topology types?** If yes, those sites pin to concrete `R` at the downcast (inherent `dyn Any` semantics).
4. **`NumericValue::Display` impl** — currently writes `f32` and `f64` values with default formatting. With Candidate A's `Real(R)`, what's the right `Display` format? Recommendation: delegate to `R`'s `Display` impl (which `RealField` does not currently require — verify or add as a supertrait).
5. **Should `Effect` propagation algorithms get any new functionality from being precision-parametric?** Recommendation: no in this change set; algorithmic content is preserved. Any new propagation behavior is a separate change.
