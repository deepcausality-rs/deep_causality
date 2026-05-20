# Design notes — `add-iso-traits`

The full design rationale lives in [`openspec/notes/NumIso.md`](NumIso.md). This file records the binding design decisions for the proposal's implementation. Decisions referenced by `tasks.md` use the labels below.

## D1. Three-tier design, not one unified trait

The note ([`NumIso.md`](NumIso.md) §1) argues for three constructs rather than one. Tier 1 leverages `From`/`Into` for in-crate isomorphisms. Tier 2 introduces a witness-typed `Iso<S, T>` for cross-crate isomorphisms blocked by the orphan rule. Tier 3 introduces `NaturalIso<F, G>` for HKT-witness isomorphisms (where `From`/`Into` cannot apply because witnesses are zero-sized types with no instances). The three tiers cover the design space without duplication.

**Decision:** ship all three tiers in this change. They are mutually independent in implementation (Part A through Part C in `tasks.md`) but together form the complete iso machinery the codebase needs.

## D2. No dedicated `Iso<T>` parent trait at the simple instance level

The first sketch of the design proposed a custom `Iso<T>` trait at Tier 1 with `forward`/`backward` methods. On review, std's `From`/`Into` already provides the bidirectional-conversion mechanics, and the structure-preserving marker subtraits can bound on it directly via `where Self: Group + From<T>, T: Group + From<Self>`. The dedicated `Iso<T>` parent trait would have duplicated std machinery for marginal documentation value.

**Decision:** Tier 1 has no `Iso<T>` parent trait. Marker subtraits (`GroupIso<T>` through `DivisionAlgebraIso<T, R>`) bound on `From` directly. Tier 2's `Iso<S, T>` exists because witness types have no `From` analog.

## D3. Method naming: `to_target` / `to_source`

Three candidates considered and rejected:

- `forward` / `backward` — collides with the EPP framework's temporal vocabulary ("forward in time"); reading `eff.forward()` in the context of `PropagatingEffect` will be misread as "advance one step in time" rather than "convert the type representation."
- `from` / `into` — conflicts with std `From::from(x)` constructs `Self` from `x` and `Into::into(self)` semantics; on a witness type neither matches.
- `apply` / `inverse` — categorical, no collisions, but "inverse" is overloaded in linear-algebra contexts in the codebase.

**Decision:** Tier 2 and Tier 3 traits use `to_target` (source → target) and `to_source` (target → source). The direction is encoded in the method name itself; the reader does not need to remember which generic parameter is the source. For HKT-level `NaturalIso<F, G>`, the same pattern applies with `F` as source witness and `G` as target witness.

Named iso impls and named iso witnesses may additionally provide inherent methods with domain-specific names (e.g. `as_effect`, `as_process`, `from_quaternion`, `to_quaternion`) for call-site readability. These delegate to the trait methods.

## D4. Tier 2 implementer placement: source type, target type, or dedicated witness

The original design proposed dedicated zero-sized witness types as the standard Tier 2 implementer (e.g. `QuaternionRotorIsoEastCoast`). On review, this is overkill for the current codebase:

- Rust coherence forbids two impls of the same trait with identical type parameters on the same type. Dedicated witness types are required if and only if multiple iso conventions between the same source/target pair must coexist.
- **No such case currently exists in the codebase.** The "East-Coast vs West-Coast" framing originally cited conflated two unrelated physics conventions (Minkowski signature vs quaternion-rotor basis); only the first is real and lives elsewhere in the codebase.

**Decision:** Tier 2 isos are implemented on whichever side (source or target) is local to the crate writing the impl. Dedicated witness types are structurally available (the trait surface accepts any implementer) but the pattern is not exercised in this change set. Reserved for a future change if a real multi-convention need emerges.

## D5. Trait inheritance gives marker subtraits for free

Tier 1 chain: `GroupIso<T>` → `RingIso<T>` (extends `GroupIso<T>` + `Ring`) → `FieldIso<T>` (extends `RingIso<T>` + `Field`).
Tier 2 chain: `GroupIso<S, T>: Iso<S, T> where S: Group, T: Group` → `RingIso<S, T>: GroupIso<S, T> where S: Ring, T: Ring` → `FieldIso<S, T>: RingIso<S, T> where S: Field, T: Field`.

For types that satisfy only a subset of the hierarchy (e.g. `Quaternion<F>` ↔ Cl(3,0) rotor satisfies `DivisionAlgebraIso` but not `FieldIso` because quaternions are non-commutative), the corresponding subtraits remain unimplemented. The compiler refuses to substitute them where the unimplemented levels are required.

`AlgebraIso<T, R>` and `DivisionAlgebraIso<T, R>` introduce a second type parameter `R: Ring` for the scalar ring. `DivisionAlgebraIso<T, R>` requires `R: Field`. The vector-structure hierarchy is parallel to the additive/multiplicative chain rather than extending it; impls write both where appropriate.

**Decision:** ship the inheritance chain as described. Empty marker bodies. The compiler does the wiring; consumers writing the most specific marker get the parents for free.

## D6. `StandardIso<S, T>` and the blanket-impl pattern

The generic `StandardIso<S, T>(PhantomData<(S, T)>)` witness with blanket impls is the boilerplate-eliminating mechanism for the single-convention Tier 2 case. The blanket impl pattern:

```rust
impl<S, T> Iso<S, T> for StandardIso<S, T>
where
    S: From<T>,
    T: From<S>,
{
    fn to_target(s: S) -> T { T::from(s) }
    fn to_source(t: T) -> S { S::from(t) }
}

impl<S, T> GroupIso<S, T> for StandardIso<S, T>
where
    S: Group + From<T>,
    T: Group + From<S>,
{}

// ... and so on for RingIso, FieldIso, AlgebraIso, DivisionAlgebraIso.
```

The blanket fires when `S` and `T` satisfy bidirectional `From` plus the relevant algebraic-structure bounds. Consumers writing a new iso provide only the two `From` impls plus a `proptest!` block; `StandardIso<S, T>` automatically picks up every applicable Tier 2 marker.

**Coherence consideration:** `StandardIso<S, T>` is one specific generic type. Manual witnesses with different names (if ever introduced for a multi-convention case) are distinct types with non-overlapping impls. The compiler accepts both side by side without ambiguity. There is no risk that the blanket impl collides with a future named witness.

**Discipline consideration:** the blanket impls fire wherever the bounds are satisfied. If a downstream crate provides bidirectional `From<S> for T` / `From<T> for S` for a type pair that is *not* actually a structure-preserving isomorphism (e.g. lossy primitive conversions), `StandardIso<S, T>` will silently claim `GroupIso<S, T>` and friends. The blanket trusts the consumer to ship `From` impls that satisfy the marker laws. Property-test discipline is the only enforcement; reviewers must reject bidirectional `From` impls that lack the corresponding `proptest!` blocks when the type pair is one the codebase will rely on as an iso.

**Decision:** ship `StandardIso<S, T>` with the full marker hierarchy. Document the blanket-impl semantics and the property-test discipline explicitly in the trait module's docs.

## D7. Module path split: Tier 1 vs Tier 2 in `deep_causality_num`

Tier 1 markers (`GroupIso<T>`, etc.) and Tier 2 markers (`GroupIso<S, T>`, etc.) share short names. Rust does not allow two traits with the same name in the same module.

**Decision:** Tier 1 lives at `deep_causality_num::iso::*`; Tier 2 lives at `deep_causality_num::iso::witness::*`. Public exports re-export both module paths; consumers disambiguate by module path. Documentation in `lib.rs` calls out the distinction explicitly and prefers fully-qualified paths in examples.

## D8. `NaturalIso<F, G>` arity

The `PropagatingEffect<T>` ↔ `PropagatingProcess<T, (), ()>` case operates at the 5-arity `CausalEffectPropagationProcess<V, S, C, E, L>` level. The 1-arity `NaturalIso<F, G>` covers the common Functor/Monad case; the 5-arity `NaturalIso5<F, G>` covers HKT5 witnesses.

**Decision:** ship `NaturalIso` (arity 1) and `NaturalIso5` (arity 5). `NaturalIso2`, `NaturalIso3`, `NaturalIso4` are deferred — add them if and when a concrete consumer needs the corresponding arity.

## D9. Property-test infrastructure

Round-trip and homomorphism laws cannot be type-system-enforced. Discipline is per-impl property tests using `proptest`.

**Decision:** ship helper functions in `test_support.rs` modules (one per tier, two for Tier 2 because of the Tier 1 / witness namespace split). The helpers are generic over the impl and exercise the relevant law with `proptest!`-style randomized inputs. CI enforces presence of tests by code review; the trait surface does not enforce it mechanically.

Naturality testing for Tier 3 uses a fixed bank of test functions (negation, doubling, identity, constant, string-conversion). A more rigorous QuickCheck-style coarbitrary approach is deferred.

## D10. No introduced runtime dependencies

`proptest` is `dev-dependencies` only. Both crates already ship `no_std` support; the iso trait declarations are `no_std`-compatible; only the test support requires `std`.

**Decision:** verify in CI that `deep_causality_num` and `deep_causality_haft` continue to build under `--no-default-features` (no_std mode) after this change.

## D11. Stage-gate protocol

Per AGENTS.md golden rule §1 ("Never commit") and the convention used in `2026-05-19-add-cubical-complexes`, no agent commits. Per-stage sign-off and per-stage commit by the user.

**Decision:** three stages (Part A / Part B / Part C). Each is independently testable and shippable. Each gate is binding; advancing without sign-off is prohibited.
