# Isomorphism traits for the algebraic and HKT hierarchies — design note

**Status:** Forward-looking. Operationalizes the practical core of GATlab-style theory morphisms — the special case of *isomorphism* between equivalent presentations of the same structure — using existing Rust trait machinery. No dependent types required. No macro layer required. Composes directly with the algebraic hierarchy in [`deep_causality_num`](../../deep_causality_num/) and the HKT machinery in [`deep_causality_haft`](../../deep_causality_haft/).

Scope: two tiers of instance-level isomorphism (`From`/`Into`-based for single-convention cases, witness-typed for multi-convention cases) plus a separate HKT-level natural-isomorphism trait. Approximate isomorphism (numerical equivalence with error tracking) is explicitly out of scope. A dedicated `Iso<T>` parent trait at the simple instance level is also explicitly out of scope — the std `From`/`Into` machinery covers it.

---

## 0. Where we are now

The codebase already uses isomorphism implicitly in several places without typing it as such:

- **`PropagatingEffect<T>` ↔ `PropagatingProcess<T, (), ()>`** in [`deep_causality_core`](../../deep_causality_core/) — the lift via `PropagatingProcess::with_state(eff, (), None)` is a literal isomorphism. Round-trip is the identity. Already in the framework; not typed as such.
- **`Quaternion<T>` ↔ Cl(3,0) rotor ↔ 2×2 complex matrix** in [`deep_causality_num`](../../deep_causality_num/) and [`deep_causality_multivector`](../../deep_causality_multivector/) — three representations of the same rotation under at least two sign conventions (East-Coast vs West-Coast). The `Rotation<T>` trait unifies *operations* across them; there is no typed way to move *data* between conventions.
- **HKT witnesses** in [`deep_causality_haft`](../../deep_causality_haft/) — `OptionWitness`, `ResultWitness<E>`, `VecWitness`, `VecDequeWitness` collectively encode that different concrete type constructors satisfy the same functor laws. Several pairs are naturally isomorphic; the relationship is not currently typed.
- **`CausalTensor<F>` dense ↔ `CsrMatrix<F>` sparse** across [`deep_causality_tensor`](../../deep_causality_tensor/) and [`deep_causality_sparse`](../../deep_causality_sparse/) — same field, different storage, isomorphic when density is below a threshold. Single canonical conversion.
- **Coordinate systems** on `Manifold<LatticeComplex<3>, F>` in [`deep_causality_topology`](../../deep_causality_topology/) — Cartesian / cylindrical / spherical / curvilinear are isomorphic representations of the same underlying physical state once the Jacobian comes along. Multiple parameterizations possible.

Each would benefit from explicit typing. None requires anything beyond standard Rust traits.

---

## 1. The contribution

A three-construct design that uses the simplest tool that fits each use case:

1. **Tier 1: `From`/`Into` for single-convention isomorphisms.** No dedicated `Iso<T>` parent trait. Implementers provide `From<T> for Self` and `From<Self> for T`; structure-preserving marker subtraits (`GroupIso<T>`, `RingIso<T>`, `FieldIso<T>`, `AlgebraIso<T, R>`, `DivisionAlgebraIso<T, R>`) bound on those `From` impls and promise the iso preserves the relevant algebraic structure.
2. **Tier 2: Witness-typed `Iso<S, T>` for cross-crate isomorphisms blocked by the orphan rule.** When bidirectional `From` cannot be implemented because the source and target live in different crates with an asymmetric dependency (the common case for cross-crate isos), `Iso<S, T>` is implemented on one of the involved types (typically the type local to the crate writing the impl). Structure-preserving marker subtraits live on the implementer with where-clauses constraining `S` and `T`. A generic `StandardIso<S, T>` witness with blanket impls auto-derives every applicable marker subtrait from bidirectional `From` for cases where the orphan rule doesn't block Tier 1, so the in-crate case has zero boilerplate beyond the `From` impls themselves. The dedicated-witness-type pattern is available for forward-looking cases where multiple iso conventions need to coexist; it is not currently exercised by anything in the codebase.
3. **Tier 3: `NaturalIso<F, G>` for HKT witnesses.** HKT witnesses are zero-sized and have no instance to convert via `From`. The trait provides `forward<T>` and `backward<T>` over the type constructor and tests naturality against `fmap`.

What this is not: full categorical theory morphisms (GATlab's GAT machinery). What this is: the practical 80% of GATlab's leverage, expressible with Rust's existing trait system at zero runtime cost. The blanket-impl mechanic on `StandardIso<S, T>` carries the marker traits automatically wherever the underlying types satisfy the algebraic-structure prerequisites — consumers import the marker traits, the compiler does the rest.

---

## 2. The two tiers and the HKT level

### 2.1 Tier 1 — Convention-based via `From`/`Into`

When there is a single canonical iso convention between two types, no dedicated trait is needed. Implementers provide both directions of `From`:

```rust
impl From<CausalTensor<F>> for CsrMatrix<F> {
    fn from(t: CausalTensor<F>) -> Self { /* dense → sparse */ }
}

impl From<CsrMatrix<F>> for CausalTensor<F> {
    fn from(m: CsrMatrix<F>) -> Self { /* sparse → dense */ }
}
```

Call sites use `.into()` directly. `Into` is derived automatically via the blanket impl `impl<T, U: From<T>> Into<U> for T`.

**Round-trip law (property-tested, not type-enforced):**
- `<Self as From<T>>::from(<T as From<Self>>::from(x)) == x` for all `x: Self`.
- The symmetric case for `T`.

**Structure-preserving subtraits live in [`deep_causality_num/src/iso/mod.rs`](../../deep_causality_num/src/iso/mod.rs) (does not exist yet)** and bound on `From`:

```rust
pub trait GroupIso<T>
where
    Self: Group + From<T>,
    T: Group + From<Self>,
{
}

pub trait RingIso<T>: GroupIso<T>
where
    Self: Ring,
    T: Ring,
{
}

pub trait FieldIso<T>: RingIso<T>
where
    Self: Field,
    T: Field,
{
}

pub trait AlgebraIso<T, R>
where
    Self: Algebra<R> + From<T>,
    T: Algebra<R> + From<Self>,
    R: Ring,
{
}

pub trait DivisionAlgebraIso<T, R>: AlgebraIso<T, R>
where
    Self: DivisionAlgebra<R>,
    T: DivisionAlgebra<R>,
    R: Field,
{
}
```

The `where`-clauses require `From` in *both* directions, which is the type-level marker that the conversion is bidirectional. The marker trait body is empty; the homomorphism law is property-tested.

**Trait inheritance gives the subtraits for free.** A type that implements `FieldIso<T>` automatically satisfies `RingIso<T>` and `GroupIso<T>` via the inheritance chain. Only the marker `impl FieldIso<T> for Self {}` is needed when all parent markers also hold.

**Where to use this tier:** when the iso has one canonical form, both types are accessible to a single crate (orphan-rule constraint), and conventions are not a concern. Examples: dense ↔ sparse storage, `Float106` ↔ specific f64-pair representations, identity isos.

### 2.2 Tier 2 — Witness-typed `Iso<S, T>`

When orphan rules forbid the cross-crate bidirectional `From` impls Tier 1 requires, `Iso<S, T>` is implemented on one of the involved types as `Self`. The implementer is typically the type local to the crate writing the impl. The trait surface:

```rust
pub trait Iso<S, T> {
    fn to_target(s: S) -> T;
    fn to_source(t: T) -> S;
}
```

The method names are deliberately `to_target` / `to_source` rather than `forward` / `backward`. See §2.4 for the naming rationale.

For the cross-crate case (the common scenario for Tier 2), the impl lives on one of the involved types — the one local to the crate writing the impl:

```rust
// In deep_causality_multivector (which depends on deep_causality_num):
impl<F: RealField> Iso<Quaternion<F>, CausalMultiVector<F>> for CausalMultiVector<F> {
    fn to_target(q: Quaternion<F>) -> CausalMultiVector<F> { /* canonical convention */ }
    fn to_source(mv: CausalMultiVector<F>) -> Quaternion<F> { /* canonical convention */ }
}
```

`Self = CausalMultiVector<F>` is local to `deep_causality_multivector`; the trait `Iso` lives in `deep_causality_num` and is foreign here, but that's fine because the orphan rule only requires either trait or `Self` to be local. The trait parameter `Quaternion<F>` can be foreign — the impl is valid.

**Why the impl can't live on `Quaternion<F>` instead:** writing `impl Iso<Quaternion<F>, CausalMultiVector<F>> for Quaternion<F>` would have to live in `deep_causality_num` (where `Quaternion<F>` is local), but that crate can't name `CausalMultiVector<F>` because it doesn't depend on `deep_causality_multivector`. The dependency direction forces the impl placement.

**Dedicated witness types** (zero-sized markers separate from either source or target) are available for cases where multiple iso conventions between the same type pair need to coexist — Rust's coherence rules forbid duplicate impls on the same `Self`, so distinct conventions require distinct implementer types. **No such case currently exists in the codebase.** The dedicated-witness pattern is a future-looking provision, not a present need.

**Structure-preserving subtraits live on the witness with where-clauses constraining the type parameters:**

```rust
pub trait GroupIso<S, T>: Iso<S, T>
where
    S: Group,
    T: Group,
{
}

pub trait RingIso<S, T>: GroupIso<S, T>
where
    S: Ring,
    T: Ring,
{
}

pub trait AlgebraIso<S, T, R>: Iso<S, T>
where
    S: Algebra<R>,
    T: Algebra<R>,
    R: Ring,
{
}

pub trait DivisionAlgebraIso<S, T, R>: AlgebraIso<S, T, R>
where
    S: DivisionAlgebra<R>,
    T: DivisionAlgebra<R>,
    R: Field,
{
}
```

The implementer doesn't need to be a `Group` or `Ring` itself — the where-clauses constrain `S` and `T` directly. The implementer is whichever type the iso is hung from; the constraints describe the *type pair*.

**Where it lives:** [`deep_causality_num/src/iso/witness/mod.rs`](../../deep_causality_num/src/iso/witness/mod.rs) (does not exist yet). Separate module to avoid name collisions with the Tier 1 subtraits.

**Where to use this tier:** when the source and target types live in different crates with an asymmetric dependency that blocks Tier 1's bidirectional `From` impls. The forward-looking multi-convention case is also covered (via dedicated witness types) but no such case currently exists in the codebase.

#### 2.2.1 — `StandardIso<S, T>`: default witness with blanket impls

For type pairs that have a single canonical iso and already implement bidirectional `From`, a `StandardIso<S, T>` generic witness with blanket impls eliminates the need to write either Tier 1 marker impls or Tier 2 custom witnesses manually. The witness is generic; the blanket impls auto-derive every marker subtrait based on what algebraic structure `S` and `T` already carry:

```rust
use core::marker::PhantomData;

pub struct StandardIso<S, T>(PhantomData<(S, T)>);

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

impl<S, T> RingIso<S, T> for StandardIso<S, T>
where
    S: Ring + From<T>,
    T: Ring + From<S>,
{}

impl<S, T, R> AlgebraIso<S, T, R> for StandardIso<S, T>
where
    S: Algebra<R> + From<T>,
    T: Algebra<R> + From<S>,
    R: Ring,
{}

impl<S, T, R> DivisionAlgebraIso<S, T, R> for StandardIso<S, T>
where
    S: DivisionAlgebra<R> + From<T>,
    T: DivisionAlgebra<R> + From<S>,
    R: Field,
{}
```

**Consumer ergonomics: implementing bidirectional `From` is enough.** The compiler wires `StandardIso<S, T>` to every applicable marker subtrait automatically based on what algebraic structure `S` and `T` carry. Generic code that bounds on `StandardIso<S, T>: RingIso<S, T>` (or any marker level) accepts the type pair without the consumer writing a single marker-trait impl. The blanket-impl mechanic carries the structure-preservation guarantees through to wherever they are needed.

**What this collapses:** the boilerplate of writing iso impls vanishes for the common case. A new dense ↔ sparse iso requires only the two `From` impls plus a `proptest!` block; every applicable marker trait derives automatically through `StandardIso<S, T>`. The named-witness pattern (Tier 2 proper) is reserved for cases where multiple conventions need explicit names.

**Why this doesn't cause coherence conflicts:** `StandardIso<S, T>` is one specific generic type. The blanket impls fire only on this type. Manual witnesses with different names (`QuaternionRotorIsoEastCoast`, `QuaternionRotorIsoWestCoast`, etc.) are distinct types with their own non-overlapping impls. The compiler accepts the default witness and named witnesses side by side without ambiguity.

**Consumer call-site choice:** the same underlying bidirectional `From` supports two ergonomic paths.

- Direct `From`/`Into`: `let s: CsrMatrix<f64> = t.into();` — most idiomatic when the iso is incidental to the calling code.
- Through `StandardIso`: `let s = StandardIso::<CausalTensor<f64>, CsrMatrix<f64>>::to_target(t);` — most useful when the calling code is generic over `Iso<S, T>` or any of its marker subtraits.

Bounded generic code typically chooses the second form because the witness is what carries the trait bounds the bound code requires.

### 2.3 Tier 3 — `NaturalIso<F, G>` for HKT witnesses

HKT witnesses are zero-sized types with no instances. `From`/`Into` doesn't apply. The trait provides type-constructor-level operations:

```rust
pub trait NaturalIso<F, G>
where
    F: HKT,
    G: HKT,
{
    fn to_target<T>(fa: F::Type<T>) -> G::Type<T>;
    fn to_source<T>(ga: G::Type<T>) -> F::Type<T>;
}
```

**Laws (property-tested):**
- Round-trip: `to_source(to_target(fa)) == fa` and `to_target(to_source(ga)) == ga` for all `T` and all `fa: F::Type<T>`, `ga: G::Type<T>`.
- Naturality: for any function `h: T -> U`, `to_target(F::fmap(fa, h)) == G::fmap(to_target(fa), h)`.

Higher-arity variants (`NaturalIso2<F, G>` through `NaturalIso5<F, G>`) follow the existing HAFT arity pattern when needed. Likely only `NaturalIso` (arity 1) and `NaturalIso5` (for the propagating-effect carrier) are needed in the initial scope.

**Where it lives:** [`deep_causality_haft/src/iso/mod.rs`](../../deep_causality_haft/src/iso/mod.rs) (does not exist yet).

### 2.4 Method naming convention

The Tier 2 and Tier 3 traits use `to_target` (source → target) and `to_source` (target → source) rather than `forward` / `backward` or `from` / `into`. Three reasons:

1. **`forward` / `backward` collide with the framework's temporal vocabulary.** DeepCausality uses "forward in time" for causal propagation throughout the EPP machinery — the [Effect Propagation Process preprint](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/effect_propagation_process/epp.pdf) makes "forward-in-time information propagation" a central axiom. Method names that suggest time direction in the context of `PropagatingEffect` and `PropagatingProcess` will be misread. An iso `forward(eff)` doesn't advance the effect one step in time; it converts the type representation. Different operation, same-sounding name. The collision is at the worst possible site — the exact carrier whose temporal semantics we don't want to muddle.

2. **`from` / `into` would conflict with std semantics.** `From::from(x)` constructs `Self` from `x`; `Into::into(self)` converts `self` to a target type. On a witness, neither matches — the witness is a marker, not the target type. Method names `from` / `into` on a witness would suggest operations the trait doesn't perform. The std vocabulary is reserved for the Tier 1 path where it is semantically correct.

3. **`to_target` / `to_source` is unambiguous in context.** The trait is `Iso<S, T>` with `S` as source and `T` as target. `to_target(s)` reads as "convert this source S into the target T"; `to_source(t)` reads as "convert this target T back to the source S". The direction is encoded in the method name itself; the reader does not need to remember which generic parameter is which.

For HKT-level isos (`NaturalIso<F, G>`), the same pattern applies: `F` is the source witness, `G` is the target witness, `to_target<T>` produces `G::Type<T>`, `to_source<T>` produces `F::Type<T>`.

**Domain-specific aliases on named witnesses.** When call-site readability matters more than trait-level uniformity, named witnesses can additionally provide inherent methods that name the destination directly:

```rust
impl PropEffectProcessIso {
    pub fn as_effect<T>(p: PropagatingProcess<T, (), ()>) -> PropagatingEffect<T> {
        <Self as NaturalIso<PropagatingProcessUnitWitness, PropagatingEffectWitness>>::to_target(p)
    }

    pub fn as_process<T>(e: PropagatingEffect<T>) -> PropagatingProcess<T, (), ()> {
        <Self as NaturalIso<PropagatingProcessUnitWitness, PropagatingEffectWitness>>::to_source(e)
    }
}
```

The underlying trait methods stay generic and uniform; the witness adds prose-clarity aliases. Generic code that bounds on `Iso<S, T>` or `NaturalIso<F, G>` uses the trait methods; concrete call sites that benefit from named operations use the aliases.

---

## 3. Choosing between Tier 1 and Tier 2

Decision tree for any new iso between two types `S` and `T`:

1. **Can a single crate host both `From<S> for T` and `From<T> for S` without orphan-rule violations?** This is true when both types live in the same crate, or when one crate depends on the other and at least one of the two impls can live in the dependent crate with the other type as `Self`. Note that asymmetric cross-crate cases (one crate depends on the other, but you need bidirectional `From`) typically *fail* this check — only one direction's `Self` is reachable in the dependent crate. If yes, use Tier 1. If no, continue.
2. **Use Tier 2.** Impl `Iso<S, T>` on whichever of `S` or `T` is local to the crate writing the impl. No dedicated witness needed.

Forward-looking: if a future scenario requires multiple iso conventions between the same type pair to coexist, use Tier 2 with dedicated witness types — Rust's coherence rules force distinct types for distinct impls of the same trait with the same parameters. **No such case currently exists in the codebase.**

In practice, most in-crate isos (dense ↔ sparse within `deep_causality_sparse` after dependency adjustment, identity isos, primitive-equivalent conversions) fit Tier 1. Most cross-crate isos (the physics types in particular: quaternion ↔ rotor, complex ↔ matrix-form, multivector ↔ tensor) fit Tier 2 because of asymmetric crate dependencies. The decision is local and doesn't need framework-wide enforcement.

---

## 4. Sample implementation: `PropagatingEffect<T>` ↔ `PropagatingProcess<T, (), ()>` (Tier 3)

The trivial natural-iso case. Demonstrates the `NaturalIso` pattern at the HKT level. Data layout is identical; the iso is zero-cost.

### 4.1 The trait impl

There's exactly one canonical iso here — the byte-identical data layout — so no dedicated iso witness is needed. The impl lives directly on the existing source witness:

```rust
pub struct PropagatingEffectWitness;

impl HKT for PropagatingEffectWitness {
    type Type<T> = PropagatingEffect<T>;
}

pub struct PropagatingProcessUnitWitness;

impl HKT for PropagatingProcessUnitWitness {
    type Type<T> = PropagatingProcess<T, (), ()>;
}

impl NaturalIso<PropagatingEffectWitness, PropagatingProcessUnitWitness>
    for PropagatingEffectWitness
{
    fn to_target<T>(eff: PropagatingEffect<T>) -> PropagatingProcess<T, (), ()> {
        PropagatingProcess {
            value: eff.value,
            state: (),
            context: None,
            error: eff.error,
            logs: eff.logs,
        }
    }

    fn to_source<T>(proc: PropagatingProcess<T, (), ()>) -> PropagatingEffect<T> {
        PropagatingEffect {
            value: proc.value,
            state: (),
            context: None,
            error: proc.error,
            logs: proc.logs,
        }
    }
}

impl PropagatingEffectWitness {
    pub fn as_process<T>(eff: PropagatingEffect<T>) -> PropagatingProcess<T, (), ()> {
        <Self as NaturalIso<PropagatingEffectWitness, PropagatingProcessUnitWitness>>::to_target(eff)
    }

    pub fn as_effect<T>(proc: PropagatingProcess<T, (), ()>) -> PropagatingEffect<T> {
        <Self as NaturalIso<PropagatingEffectWitness, PropagatingProcessUnitWitness>>::to_source(proc)
    }
}
```

**Why no dedicated iso witness?** A dedicated witness type (`PropEffectProcessIso`) would be required only if multiple iso conventions between the same two carriers needed to coexist — Rust's coherence rules forbid two impls of the same trait with identical type parameters on the same type. Here there is only one canonical iso (the byte-identical layout-preserving conversion), so the impl lives on the source witness directly. The dedicated-witness pattern is reserved for the multi-convention case in §5.

### 4.2 The usage scenario

The fluid causal-inference pipeline has a library of *stateless* helper transformations — adding audit-log entries, attaching metadata, validating field structure. These are written naturally over `PropagatingEffect<T>` because they don't need state or context. Downstream pipeline code lives in `PropagatingProcess<T, S, C>` because it accumulates rolling history per [`3DCausalFluidDynamics.md`](../../notes/3DCausalFluidDynamics.md) §4.2. Where `S = ()` and `C = ()` — the unit-state segments of the pipeline before the formal lift to `RollingHistory<N>` — the two carriers are literally isomorphic and the iso bridges helper code written for one carrier into pipelines using the other.

### 4.3 Without the iso

Every call site that wants to use a `PropagatingEffect`-typed helper from a `PropagatingProcess`-typed pipeline writes the conversion by hand:

```rust
let process: PropagatingProcess<HodgeSignature, (), ()> = /* ... */;

let as_effect = PropagatingEffect {
    value: process.value,
    state: (),
    context: None,
    error: process.error,
    logs: process.logs,
};

let after = add_audit_entry(as_effect, "stage", "hodge-decompose");

let process = PropagatingProcess {
    value: after.value,
    state: (),
    context: None,
    error: after.error,
    logs: after.logs,
};
```

Five lines of unwrap-and-rewrap per call. The conversion is silent — nothing at the type level says "I'm asserting these are equivalent." Mistakes (forgetting to forward an error, dropping a log entry) are invisible until they cause downstream failures.

### 4.4 With the iso

The named alias makes the conversion explicit and one-line per direction:

```rust
let process: PropagatingProcess<HodgeSignature, (), ()> = /* ... */;
let as_effect = PropEffectProcessIso::as_effect(process);
let after = add_audit_entry(as_effect, "stage", "hodge-decompose");
let process = PropEffectProcessIso::as_process(after);
```

The real value emerges in helper code written once and reused on either carrier through the iso:

```rust
fn annotated<W>(carrier: W::Type<HodgeSignature>, key: &str, value: &str) -> W::Type<HodgeSignature>
where
    W: HKT,
    PropEffectProcessIso: NaturalIso<PropagatingEffectWitness, W>,
{
    let eff = <PropEffectProcessIso as NaturalIso<PropagatingEffectWitness, W>>::to_source(carrier);
    let mut eff = eff;
    eff.logs.add_entry(format!("{}={}", key, value));
    <PropEffectProcessIso as NaturalIso<PropagatingEffectWitness, W>>::to_target(eff)
}
```

The same function operates on both witnesses with the same body. The naturality property guarantees that subsequent `fmap`-style transformations commute across the iso — so generic pipeline code produces the same result regardless of which carrier it operates on.

### 4.5 The value

Helper libraries written once, used on both carriers. The boundary between Markovian and non-Markovian segments of the pipeline becomes one well-defined operation instead of an open-coded unwrap-rewrap pattern repeated throughout the codebase. Without the iso, helpers are either written twice (once per carrier) or pay an error-prone hand-rolled conversion at every boundary. With the iso, a single helper definition serves both regimes and the type system documents which boundary the conversion crosses.

**Where it lives:** [`deep_causality_core/src/iso/mod.rs`](../../deep_causality_core/src/iso/mod.rs) (does not exist yet).

---

## 5. Sample implementation: `Quaternion<F>` ↔ Cl(3,0) rotor (Tier 2)

A cross-crate iso forced into Tier 2 by the orphan rule. Single canonical convention; no multi-convention coexistence required (none currently exists in the codebase). Demonstrates how Tier 2 hangs the iso off one of the involved types when bidirectional `From` is structurally infeasible.

### 5.1 Why Tier 1 doesn't work here

`Quaternion<F>` lives in [`deep_causality_num`](../../deep_causality_num/). `CausalMultiVector<F>` lives in [`deep_causality_multivector`](../../deep_causality_multivector/). The dependency is one-way: `deep_causality_multivector` depends on `deep_causality_num`, never the reverse (no cycle possible).

For Tier 1, both `From` directions would need to live somewhere they compile:

- `impl<F: RealField> From<Quaternion<F>> for CausalMultiVector<F>` — `Self = CausalMultiVector<F>` is local to `deep_causality_multivector`, so this impl can live there. The crate sees `Quaternion<F>` via its dependency on `deep_causality_num`. **Allowed.**
- `impl<F: RealField> From<CausalMultiVector<F>> for Quaternion<F>` — `Self = Quaternion<F>` is local to `deep_causality_num`, so this impl would have to live there. But `deep_causality_num` does not depend on `deep_causality_multivector` and cannot name `CausalMultiVector<F>`. Adding the dependency would create a cycle. **Not allowed.**

The orphan rule forces one direction into nowhere. Tier 1 fails for this type pair.

### 5.2 The Tier 2 trait impl

Hang the iso on `CausalMultiVector<F>` — local to the crate writing the impl, which lets the orphan rule pass:

```rust
// In deep_causality_multivector/src/iso/mod.rs:
impl<F: RealField> Iso<Quaternion<F>, CausalMultiVector<F>> for CausalMultiVector<F> {
    fn to_target(q: Quaternion<F>) -> CausalMultiVector<F> {
        let mut mv = CausalMultiVector::zero(Metric::Euclidean(3));
        mv.set_scalar(q.w);
        mv.set_bivector(2, 3, q.x);
        mv.set_bivector(3, 1, q.y);
        mv.set_bivector(1, 2, q.z);
        mv
    }

    fn to_source(mv: CausalMultiVector<F>) -> Quaternion<F> {
        Quaternion {
            w: mv.scalar(),
            x: mv.bivector(2, 3),
            y: mv.bivector(3, 1),
            z: mv.bivector(1, 2),
        }
    }
}

impl<F: RealField> GroupIso<Quaternion<F>, CausalMultiVector<F>> for CausalMultiVector<F> {}
impl<F: RealField> AlgebraIso<Quaternion<F>, CausalMultiVector<F>, F> for CausalMultiVector<F> {}
impl<F: RealField> DivisionAlgebraIso<Quaternion<F>, CausalMultiVector<F>, F> for CausalMultiVector<F> {}
```

`RingIso` and `FieldIso` are not implemented because quaternions and Cl(3,0) rotors are non-commutative — the iso preserves multiplication but neither side is a `Field`, so the trait bound `T: Field` rules `FieldIso` out at the type level. The trait-inheritance machinery correctly refuses the incorrect claim.

The single canonical convention (right-handed `i, j, k → e₂₃, e₃₁, e₁₂`) lives in the impl bodies. No external convention parameter, no separate witness type.

### 5.3 The usage scenario

An avionics attitude pipeline reads aircraft orientation from an IMU as a quaternion (efficient for chaining rotations), then applies that rotation to a sensor-mounted antenna whose orientation is represented as a `CausalMultiVector` (because the antenna math involves reflections and projections that are natural in Clifford algebra). The bridge between the two algebraic representations is the iso.

### 5.4 Without the iso

Cross-representation code writes the bridge by hand at every call site:

```rust
let q = imu.read_attitude();
let mut rotor = CausalMultiVector::zero(Metric::Euclidean(3));
rotor.set_scalar(q.w);
rotor.set_bivector(2, 3, q.x);
rotor.set_bivector(3, 1, q.y);
rotor.set_bivector(1, 2, q.z);
let antenna_rotated = rotor.clone() * antenna_orientation.clone() * rotor.reverse();
```

Repeated wherever quaternion-to-rotor bridging happens. Refactoring the basis assignment requires touching every call site. No type-level guarantee that the conversion preserves the rotation structure — a typo in the basis indices silently produces a different rotation.

### 5.5 With the iso

```rust
let q = imu.read_attitude();
let rotor = <CausalMultiVector<f64> as Iso<Quaternion<f64>, CausalMultiVector<f64>>>::to_target(q);
let antenna_rotated = rotor.clone() * antenna_orientation * rotor.reverse();
```

Or with a domain-specific alias as an inherent method:

```rust
impl<F: RealField> CausalMultiVector<F> {
    pub fn from_quaternion(q: Quaternion<F>) -> Self {
        <Self as Iso<Quaternion<F>, CausalMultiVector<F>>>::to_target(q)
    }

    pub fn to_quaternion(self) -> Quaternion<F> {
        <Self as Iso<Quaternion<F>, CausalMultiVector<F>>>::to_source(self)
    }
}

// Call sites:
let rotor = CausalMultiVector::<f64>::from_quaternion(imu.read_attitude());
let antenna_rotated = rotor.clone() * antenna_orientation * rotor.reverse();
```

Generic numerical code that operates across the iso bounds on the structure-preserving marker:

```rust
fn apply_attitude_rotation<F: RealField>(
    q: Quaternion<F>,
    target: CausalMultiVector<F>,
) -> CausalMultiVector<F>
where
    CausalMultiVector<F>: DivisionAlgebraIso<Quaternion<F>, CausalMultiVector<F>, F>,
{
    let rotor = <CausalMultiVector<F> as Iso<Quaternion<F>, CausalMultiVector<F>>>::to_target(q);
    rotor.clone() * target * rotor.reverse()
}
```

The `DivisionAlgebraIso` bound asserts at compile time that the iso preserves the conjugation and multiplication structure — which is what makes `rotor.reverse()` and `rotor * mv * rotor.reverse()` semantically meaningful as a rotation. An impl that violated the homomorphism law wouldn't satisfy the bound, and the call would refuse to compile.

### 5.6 The value

Three things:

1. **Cross-crate bidirectional conversion becomes possible at all.** Without Tier 2, the orphan rule blocks bidirectional `From` and the project either has to live with one-way conversion utilities or restructure the crate dependencies. Tier 2 with the impl on the locally-owned type sidesteps the constraint without architectural change.
2. **Structure preservation is type-checked.** `DivisionAlgebraIso` bounds verify that downstream code uses the iso in contexts where the homomorphism law actually matters. Failures surface at compile time, not at the first wrong rotation result.
3. **The convention lives in one place.** The basis assignment is in the impl body in `deep_causality_multivector`, not scattered across every call site that bridges quaternions to rotors. Refactoring the convention (should it ever be needed) is a one-file change.

**Where it lives:** [`deep_causality_multivector/src/iso/mod.rs`](../../deep_causality_multivector/src/iso/mod.rs) (does not exist yet).

---

## 6. Sample implementation: `CausalTensor<F>` ↔ `CsrMatrix<F>` (Tier 1 forward + Tier 2 reverse)

A cross-crate hybrid. The forward direction (`CausalTensor` → `CsrMatrix`) fits Tier 1 cleanly because `CsrMatrix<F>` is local to `deep_causality_sparse`. The reverse direction (`CsrMatrix` → `CausalTensor`) is blocked from Tier 1 by the orphan rule and uses Tier 2 instead. The example demonstrates how the two tiers can coexist on a single conceptual iso when the dependency graph dictates it.

### 6.1 Why bidirectional `From` doesn't work here

`CausalTensor<F>` lives in [`deep_causality_tensor`](../../deep_causality_tensor/); `CsrMatrix<F>` lives in [`deep_causality_sparse`](../../deep_causality_sparse/). The dependency is one-way: `deep_causality_sparse` depends on `deep_causality_tensor`, never the reverse.

For Tier 1, both directions of `From` would need to compile:

- `impl<F: RealField> From<CausalTensor<F>> for CsrMatrix<F>` — `Self = CsrMatrix<F>` is local to `deep_causality_sparse`, the impl can live there, and the crate sees `CausalTensor<F>` via its dependency. **Allowed.**
- `impl<F: RealField> From<CsrMatrix<F>> for CausalTensor<F>` — `Self = CausalTensor<F>` is foreign (lives in `deep_causality_tensor`). The trait is foreign too. The orphan rule requires the first local type in the impl to appear before any foreign type containing uncovered type parameters; here `CausalTensor<F>` (foreign, contains uncovered `F`) precedes the local `CsrMatrix<F>` in trait-parameter position. **Blocked.** Adding the impl in `deep_causality_tensor` would require that crate to depend on `deep_causality_sparse`, creating a cycle.

### 6.2 The trait impl (mixed tier)

Forward direction via Tier 1 `From`:

```rust
// In deep_causality_sparse/src/iso/tensor_csr.rs
impl<F: RealField> From<CausalTensor<F>> for CsrMatrix<F> {
    fn from(t: CausalTensor<F>) -> Self {
        // Pack non-zero entries of t into CSR format.
        // (Convention: zero threshold is exact equality with F::zero().)
    }
}
```

Reverse direction via Tier 2 `Iso<S, T>` on `CsrMatrix<F>` as `Self` (the locally-owned type, which sidesteps the orphan rule the same way Quaternion ↔ rotor did in §5):

```rust
// In deep_causality_sparse/src/iso/tensor_csr.rs
impl<F: RealField> Iso<CsrMatrix<F>, CausalTensor<F>> for CsrMatrix<F> {
    fn to_target(s: CsrMatrix<F>) -> CausalTensor<F> {
        // Expand sparse entries into a dense tensor with zeros elsewhere.
    }
    fn to_source(t: CausalTensor<F>) -> CsrMatrix<F> {
        // Delegates to the forward From impl above.
        CsrMatrix::from(t)
    }
}

impl<F: RealField> GroupIso<CsrMatrix<F>, CausalTensor<F>> for CsrMatrix<F> {}
impl<F: RealField> RingIso<CsrMatrix<F>, CausalTensor<F>> for CsrMatrix<F> {}
impl<F: RealField> AlgebraIso<CsrMatrix<F>, CausalTensor<F>, F> for CsrMatrix<F> {}
```

Domain-specific aliases as inherent methods on `CsrMatrix<F>` give call sites a cleaner idiom for the reverse direction:

```rust
impl<F: RealField> CsrMatrix<F> {
    pub fn to_dense(self) -> CausalTensor<F> {
        <Self as Iso<CsrMatrix<F>, CausalTensor<F>>>::to_target(self)
    }
}
```

**What's lost compared to a pure Tier 1 example:** `StandardIso<CausalTensor<F>, CsrMatrix<F>>` does NOT auto-derive every marker in this case, because the blanket-impl bounds require bidirectional `From` and only the forward direction exists. The example demonstrates that the design tolerates partial Tier 1 + Tier 2 hybrid placement; for a clean Tier 1 demonstration the two types would need to live in the same crate.

### 6.2 The usage scenario

A discrete-Hodge-Laplacian operator on a `Manifold<LatticeComplex<3>, f64>` produces a coboundary matrix. For small `D` and dense fields, the matrix is best stored as a `CausalTensor`. For large fields where most entries are zero, the same matrix is best stored as a `CsrMatrix`. The choice depends on density, which depends on the field — and the same code paths sometimes need both.

### 6.3 Without the iso

Each conversion site is hand-written, with no compile-time guarantee that the round-trip preserves the matrix's algebraic structure:

```rust
let dense_laplacian: CausalTensor<f64> = manifold.laplacian(0);

let sparse = CsrMatrix::from_dense_lossy(&dense_laplacian);
// ^ could quietly drop near-zero entries

let result_sparse = sparse_solve(sparse, &rhs);

let result_dense = result_sparse.densify();
// ^ could allocate inconsistently or zero-fill in a way that breaks ring structure
```

The hazard is silent: if `from_dense_lossy` and `densify` are not exact inverses, the round-trip introduces drift that compounds across iterations. Numerical solvers fail in non-obvious ways. The conversion functions are conventionally treated as "obvious utilities" and rarely property-tested.

### 6.4 With the iso

Forward direction uses standard `.into()` ergonomics (Tier 1). Reverse direction uses the `to_dense()` inherent method that delegates to the Tier 2 iso (or equivalently, the Tier 2 trait method directly):

```rust
let dense_laplacian: CausalTensor<f64> = manifold.laplacian(0);

// Forward: Tier 1 `From`/`Into`.
let sparse: CsrMatrix<f64> = dense_laplacian.into();

let result_sparse = sparse_solve(sparse, &rhs);

// Reverse: inherent method delegating to the Tier 2 iso.
let result_dense: CausalTensor<f64> = result_sparse.to_dense();
```

Generic numerical code that crosses the iso boundary bounds on the actual impls it uses — `From<CausalTensor<f64>>` for the forward leg, `Iso<CsrMatrix<f64>, CausalTensor<f64>>` for the reverse leg:

```rust
fn matrix_exponential_adaptive(
    operator: CausalTensor<f64>,
    duration: f64,
    steps: usize,
) -> CausalTensor<f64> {
    let sparse: CsrMatrix<f64> = operator.into();           // Tier 1
    let exp = sparse_matrix_exp_via_krylov(sparse, duration, steps);
    exp.to_dense()                                          // Tier 2 via inherent alias
}

let dense_op: CausalTensor<f64> = build_evolution_operator();
let evolved = matrix_exponential_adaptive(dense_op, dt, 50);
```

For a fully generic version that abstracts over the source type, the bound surface is split across the two tiers — `S: From<CsrMatrix<f64>>` plus `CsrMatrix<f64>: Iso<CsrMatrix<f64>, S>`. In practice, this kind of hybrid bound is awkward enough that most generic code commits to a single tier and accepts a concrete type pair.

The `RingIso<CsrMatrix<f64>, CausalTensor<f64>>` marker on `CsrMatrix<f64>` (Tier 2) is the compile-time assertion that the reverse-direction iso preserves ring structure. The forward direction is verified separately via property tests on the `From` impl.

### 6.5 The value

Three things, with one honest caveat:

1. **Adaptive storage becomes routine.** Dense for small / hot regions, sparse for large / cold regions, with `.into()` in one direction and `.to_dense()` in the other. The choice of representation is local to the function that needs it; callers don't see the tier split.
2. **Structure preservation is verified per-tier.** The forward `From` carries a `proptest!` block exercising the round-trip and homomorphism laws on the `From` path. The reverse Tier 2 impl carries equivalent property tests through the witness's `to_target` / `to_source`. Both legs of the iso get the same discipline.
3. **The cost is one `From` impl plus one Tier 2 impl plus property tests for both.** Higher than a pure in-crate Tier 1 example, but the orphan rule dictates this is the minimum viable structure for cross-crate cases with asymmetric dependencies.

**Honest caveat: `StandardIso<S, T>` does not auto-derive markers here.** The blanket impls on `StandardIso<S, T>` require bidirectional `From`, which is unavailable for this type pair. The "zero-boilerplate Tier 1" benefit applies only to fully-in-crate cases. For mixed-tier cross-crate cases like this one, the marker impls are written manually on the Tier 2 implementer (`CsrMatrix<F>` here).

**Where it lives:** [`deep_causality_sparse/src/iso/tensor_csr.rs`](../../deep_causality_sparse/src/iso/tensor_csr.rs) (does not exist yet). `deep_causality_sparse` already depends on `deep_causality_tensor`, so both types are reachable from a single crate — but the dependency is one-way, which is exactly what forces the mixed-tier shape.

### 6.6 Summary: where each tier earns its keep

| Tier | Earns its keep when... | Cost saved |
|---|---|---|
| **Tier 1 (`From`/`Into` + markers via `StandardIso`)** | Both source and target types are reachable to a single crate that can host both `From` impls (in-crate or carefully arranged cross-crate); generic numerical code that needs structure-preservation as a precondition. | Hand-rolled unwrap-rewrap utilities; silent loss-of-structure bugs; per-call-site round-trip code. |
| **Tier 2 (`Iso<S, T>` on one of the involved types)** | The orphan rule blocks Tier 1's bidirectional `From` — typically because source and target live in different crates with an asymmetric dependency. Impl `Iso<S, T>` on whichever side is local to the crate writing the impl. | Cross-crate iso conversions that would otherwise require crate-architecture changes or one-way conversion utilities. |
| **Tier 3 (`NaturalIso` on HKT witnesses)** | Functor-equivalent type constructors that differ only in fixed-parameter values (the `PropagatingEffect` ↔ `PropagatingProcess<T, (), ()>` case is the canonical example); helper libraries that should work on either carrier. | Writing the same helper twice for two morally-equivalent carriers; manually maintaining the equivalence at every call site. |

Forward-looking: dedicated witness types (separate from either source or target) are reserved for cases where multiple iso conventions between the same type pair need to coexist. **No such case currently exists in the codebase**, and the dedicated-witness pattern is a future-looking provision rather than a present need.

Each tier's value is concrete and demonstrable on existing or near-term code paths. The three samples in §4, §5, and §6 walk through the actual code each enables.

---

## 7. Trait inheritance and tier composition

Both Tier 1 and Tier 2 use the trait-inheritance-for-free pattern: implementing the most specific marker subtrait automatically satisfies all parent markers.

**Tier 1 example (single-convention, in-crate):**

```rust
// Hypothetical in-crate type pair `MyTypeA` and `MyTypeB` where bidirectional
// From is implementable (both types are local to the same crate, or the
// dependency graph allows both impl directions). Tier 1 marker impls go on
// the source type and stack up empty:
impl GroupIso<MyTypeB> for MyTypeA {}
impl RingIso<MyTypeB> for MyTypeA {}
impl AlgebraIso<MyTypeB, f64> for MyTypeA {}
```

**Tier 2 example (cross-crate, source/target as `Self`):**

```rust
// CausalMultiVector<F> is local to deep_causality_multivector (per §5).
// DivisionAlgebraIso extends AlgebraIso which extends Iso, so implementing the
// most specific marker requires every parent in the chain to also be
// implemented (empty bodies). Per-level impls:
impl<F: RealField> GroupIso<Quaternion<F>, CausalMultiVector<F>> for CausalMultiVector<F> {}
impl<F: RealField> AlgebraIso<Quaternion<F>, CausalMultiVector<F>, F> for CausalMultiVector<F> {}
impl<F: RealField> DivisionAlgebraIso<Quaternion<F>, CausalMultiVector<F>, F> for CausalMultiVector<F> {}
```

For types that satisfy only a subset of the hierarchy (e.g., a quaternion-to-rotor iso satisfies `DivisionAlgebraIso` but not `FieldIso` because quaternions are non-commutative), the corresponding subtraits simply remain unimplemented. The compiler refuses to substitute them where the unimplemented levels are required.

---

## 8. Property-based testing strategy

Every iso impl (Tier 1, Tier 2, or Tier 3) ships with a `proptest`-based round-trip test. The infrastructure lives in [`deep_causality_num/src/iso/test_support.rs`](../../deep_causality_num/src/iso/test_support.rs) (does not exist yet) and is gated behind a `proptest` dev-dependency.

**Tier 1 helpers:**

```rust
#[cfg(test)]
pub fn assert_iso_from_round_trip<S, T>(s: S)
where
    S: From<T> + Clone + PartialEq + std::fmt::Debug,
    T: From<S> + Clone + PartialEq + std::fmt::Debug,
{
    let t: T = T::from(s.clone());
    let s_back: S = S::from(t.clone());
    assert_eq!(s, s_back, "From round-trip S -> T -> S failed");

    let s_again: S = S::from(t.clone());
    let t_again: T = T::from(s_again);
    assert_eq!(t, t_again, "From round-trip T -> S -> T failed");
}

#[cfg(test)]
pub fn assert_group_iso_from_law<S, T>(a: S, b: S)
where
    S: GroupIso<T> + Clone + PartialEq + std::fmt::Debug,
    T: Group + Clone + PartialEq + std::fmt::Debug,
{
    let lhs: T = T::from(a.clone() * b.clone());
    let rhs: T = T::from(a) * T::from(b);
    assert_eq!(lhs, rhs, "GroupIso homomorphism law failed");
}
```

**Tier 2 helpers** (witness-typed):

```rust
#[cfg(test)]
pub fn assert_witness_iso_round_trip<W, S, T>(s: S)
where
    W: Iso<S, T>,
    S: Clone + PartialEq + std::fmt::Debug,
    T: Clone + PartialEq + std::fmt::Debug,
{
    let t: T = W::to_target(s.clone());
    let s_back: S = W::to_source(t.clone());
    assert_eq!(s, s_back, "Witness iso round-trip S -> T -> S failed");

    let s_again: S = W::to_source(t.clone());
    let t_again: T = W::to_target(s_again);
    assert_eq!(t, t_again, "Witness iso round-trip T -> S -> T failed");
}

#[cfg(test)]
pub fn assert_witness_group_iso_law<W, S, T>(a: S, b: S)
where
    W: GroupIso<S, T>,
    S: Group + Clone + PartialEq + std::fmt::Debug,
    T: Group + Clone + PartialEq + std::fmt::Debug,
{
    let lhs: T = W::to_target(a.clone() * b.clone());
    let rhs: T = W::to_target(a) * W::to_target(b);
    assert_eq!(lhs, rhs, "Witness GroupIso homomorphism law failed");
}
```

**Tier 3 helpers** (HKT naturality): exercise round-trip across multiple `T` instances and naturality against a small bank of test functions (negation, doubling, identity, constant, string-conversion).

Convention: every iso impl includes a `proptest!` block exercising the corresponding helper. CI enforces presence of tests by code review; the trait surface does not enforce it mechanically. Reviewers reject iso impls without property-test coverage.

---

## 9. Integration plan

### 9.1 `deep_causality_num` — Tier 1 subtraits

**New module:** `src/iso/`.

**Files:**
- `src/iso/mod.rs` — re-exports for Tier 1.
- `src/iso/group_iso.rs` — `GroupIso<T>`.
- `src/iso/ring_iso.rs` — `RingIso<T>`.
- `src/iso/field_iso.rs` — `FieldIso<T>`.
- `src/iso/algebra_iso.rs` — `AlgebraIso<T, R>`.
- `src/iso/division_algebra_iso.rs` — `DivisionAlgebraIso<T, R>`.
- `src/iso/test_support.rs` — property-test helpers, `#[cfg(test)]`-gated.
- `src/iso/witness/mod.rs` — re-exports for Tier 2.
- `src/iso/witness/iso.rs` — `Iso<S, T>`.
- `src/iso/witness/standard.rs` — `StandardIso<S, T>` generic witness plus its blanket impls for every marker subtrait at the witness level.
- `src/iso/witness/group_iso.rs` — `GroupIso<S, T>` (separate module avoids collision with Tier 1).
- `src/iso/witness/ring_iso.rs`, `field_iso.rs`, `algebra_iso.rs`, `division_algebra_iso.rs` — corresponding Tier 2 subtraits.
- `src/iso/witness/test_support.rs` — Tier 2 property-test helpers.

**Public exports** added to `src/lib.rs`: every trait above, with the Tier 2 module path preserved (`pub use iso::witness;`) so consumers disambiguate `iso::GroupIso` (Tier 1) vs `iso::witness::GroupIso` (Tier 2).

### 9.2 `deep_causality_haft` — Tier 3 traits

**New module:** `src/iso/`.

**Files:**
- `src/iso/mod.rs` — re-exports.
- `src/iso/natural_iso.rs` — `NaturalIso<F, G>` (arity 1).
- `src/iso/natural_iso_5.rs` — `NaturalIso5<F, G>` for the 5-arity propagating-effect carrier.
- `src/iso/test_support.rs` — naturality-property helpers.

**Public exports** added to `src/lib.rs`: `NaturalIso`, `NaturalIso5`.

### 9.3 `deep_causality_core` — `PropagatingEffect` ↔ `PropagatingProcess<T, (), ()>` (Tier 3)

**New module:** `src/iso/mod.rs`.

**Files:**
- `src/iso/prop_effect_process_iso.rs` — the HKT witness types and the `NaturalIso` impl per §4. The iso lives on `PropagatingEffectWitness` directly (no dedicated iso witness type).

**Public exports** added to `src/lib.rs`: `PropagatingEffectWitness`, `PropagatingProcessUnitWitness`, plus the inherent-method aliases `as_effect` / `as_process`.

This is the lowest-risk concrete iso to ship first because the data layout is byte-identical between source and target.

### 9.4 `deep_causality_multivector` — `Quaternion<F>` ↔ Cl(3,0) rotor (Tier 2)

**New module:** `src/iso/`.

**Files:**
- `src/iso/quaternion_rotor.rs` — single canonical iso, implemented on `CausalMultiVector<F>` as `Self` per §5. Plus inherent-method aliases (`from_quaternion`, `to_quaternion`) for call-site readability.

The orphan rule forces the impl onto `CausalMultiVector<F>` rather than `Quaternion<F>` (see §5.1). No dedicated iso witness needed.

The impl plus the relevant `AlgebraIso` / `DivisionAlgebraIso` markers are re-exported from `lib.rs`.

### 9.5 `deep_causality_physics` — Pauli matrix iso (Tier 2)

`Iso<Quaternion<F>, PauliMatrix<F>>` for the quantum-classical bridge. Same orphan-rule motivation as §5: `Quaternion<F>` lives in `deep_causality_num`, `PauliMatrix<F>` lives in `deep_causality_physics`, and the dependency is one-way. The impl lives on `PauliMatrix<F>` as `Self`. Single canonical convention.

### 9.6 `deep_causality_topology` — Coordinate isos on `Manifold`

A specialized `CoordinateIso<C1, C2>` trait specific to manifold coordinate systems. Not strictly an instance of either Tier 1 or Tier 2 because the Jacobian metadata travels with the transformation. It's a domain-specific specialization that follows the same naming convention so consumers know what to expect.

**Sketch:**

```rust
pub trait CoordinateIso<C1, C2, F: RealField> {
    fn forward_coords(coords: [F; 3]) -> [F; 3];
    fn backward_coords(coords: [F; 3]) -> [F; 3];
    fn forward_jacobian(coords: [F; 3]) -> CausalTensor<F>;
    fn backward_jacobian(coords: [F; 3]) -> CausalTensor<F>;
}
```

Implementations as witness types per Tier 2 convention. Files:
- `src/iso/coordinate_iso.rs` — trait.
- `src/iso/cartesian_cylindrical.rs` — Cartesian ↔ cylindrical impl.
- `src/iso/cartesian_spherical.rs` — Cartesian ↔ spherical impl.

### 9.7 `deep_causality_sparse` — Dense ↔ sparse (Tier 1)

`From<CausalTensor<F>> for CsrMatrix<F>` and the reverse, plus the Tier 1 marker traits per §6. Lives in `deep_causality_sparse` since that crate already depends on `deep_causality_tensor`.

---

## 10. Phase plan and effort estimates

Each phase is independently shippable and has property-test coverage as its acceptance gate.

| Phase | Description | Effort |
|---|---|---|
| **I1** | Tier 1 subtraits in `deep_causality_num/src/iso/` (`GroupIso<T>` through `DivisionAlgebraIso<T, R>`). Property-test infrastructure with `From`-based round-trip and homomorphism helpers. | 3 days |
| **I2** | Tier 2 traits in `deep_causality_num/src/iso/witness/` (`Iso<S, T>`, the witness-typed marker subtraits, and the `StandardIso<S, T>` generic witness with full blanket impls for the marker hierarchy). Property-test infrastructure including blanket-impl coverage. | 4 days |
| **I3** | `NaturalIso<F, G>` and `NaturalIso5<F, G>` in `deep_causality_haft`. Naturality-property test infrastructure. | 3 days |
| **I4** | First concrete Tier 3 instance: `PropagatingEffect<T>` ↔ `PropagatingProcess<T, (), ()>` in `deep_causality_core`. | 2 days |
| **I5** | First concrete Tier 2 instance: `Quaternion<F>` ↔ Cl(3,0) rotor in `deep_causality_multivector`, both East-Coast and West-Coast witnesses, with `DivisionAlgebraIso` markers and rotation-action property tests. | 5 days |
| **I6** | First concrete Tier 1 instance: dense ↔ sparse in `deep_causality_sparse`. Validation that round-trip and ring/algebra preservation hold within numerical tolerance. | 3 days |
| **I7** | Documentation: trait-doc comments per crate convention, README updates in `deep_causality_num` and `deep_causality_haft`, integration example in `examples/mathematics_examples/iso_examples/`. | 3 days |

**Total core foundation: ~23 working days (~4.5 weeks).**

Downstream consumer phases, each independent:

| Phase | Description | Effort |
|---|---|---|
| **I8** | Coordinate isos on `Manifold` in `deep_causality_topology`. Cartesian / cylindrical / spherical with Jacobian computation. Specialized trait per §9.6. | 5 days |
| **I9** | Pauli-matrix iso for `Quaternion<F>` in `deep_causality_physics` for quantum-classical bridging. Both sign conventions. | 4 days |

Downstream phases land as the consuming work needs them. Each is independently testable.

---

## 11. Coherence and orphan-rule considerations

Rust's orphan rule restricts trait impls to "either trait or type local to the current crate." This affects iso impls between types from different crates and is the primary reason Tier 2 exists.

**Tier 1 placements (in-crate or carefully arranged cross-crate):**

- `From<f64> for Float106` / `From<Float106> for f64`: both types in `deep_causality_num`. Trivial.
- `From<CausalTensor<F>> for CsrMatrix<F>` and the reverse: `CausalTensor<F>` is in `deep_causality_tensor`, `CsrMatrix<F>` is in `deep_causality_sparse`. `deep_causality_sparse` depends on `deep_causality_tensor`, so the `From<CausalTensor<F>> for CsrMatrix<F>` impl can live there (`CsrMatrix<F>` is local). The reverse direction `From<CsrMatrix<F>> for CausalTensor<F>` requires `CausalTensor<F>` to be local, which means it would need to live in `deep_causality_tensor` — but that crate can't see `CsrMatrix<F>` without a circular dependency. Resolution: only one direction is genuinely a Tier 1 `From`; the other direction is provided either as an inherent method on `CsrMatrix<F>` (e.g., `to_dense(self) -> CausalTensor<F>`) or via Tier 2. **Tier 1 is most natural for fully in-crate type pairs.**

**Tier 2 placements (cross-crate with asymmetric dependency):**

- Quaternion ↔ rotor: `Quaternion<F>` is in `deep_causality_num`, `CausalMultiVector<F>` is in `deep_causality_multivector`. The impl lives in `deep_causality_multivector` with `Self = CausalMultiVector<F>` (local). The `Iso` trait is foreign (from `deep_causality_num`) but the orphan rule allows the impl because `Self` is local. **No dedicated witness type needed for the single-convention case** — the local target type serves as the implementer.
- Pauli-matrix iso: same shape. `Quaternion<F>` is foreign, `PauliMatrix<F>` is local to `deep_causality_physics`. Impl on `PauliMatrix<F>` as `Self`.

**Why Tier 2 sidesteps the orphan rule:** the impl lives on the locally-owned type. The `Iso<S, T>` trait is local to `deep_causality_num`; when used from a downstream crate, `Self` being local satisfies "either trait or `Self` local." The convention is that the iso lives on whichever side is local to the crate doing the work, which is usually the downstream crate in an asymmetric dependency.

**General rule:** for Tier 1, both `From` impls must be implementable in some crate. When the asymmetric cross-crate dependency blocks one direction, Tier 2 with `Self` as the locally-owned type is the standard workaround. No dedicated iso witness type is required unless the codebase actually needs multiple iso conventions to coexist between the same type pair — which it currently doesn't.

---

## 12. `no_std` support

The Tier 1 subtraits, Tier 2 `Iso<S, T>` and its subtraits, and Tier 3 `NaturalIso<F, G>` are all `no_std`-compatible — pure trait declarations with no allocation or platform dependencies.

The property-test helpers in `test_support.rs` modules require `std` and are gated behind `#[cfg(test)]`. Production code never depends on the test infrastructure.

Both `deep_causality_num` and `deep_causality_haft` already ship `no_std` support per their READMEs. The iso modules inherit that support without additional feature flags.

---

## 13. What's out of scope

Listed for the next contributor:

- **A dedicated `Iso<T>` parent trait at the simple instance level.** Considered, then dropped: `From<T>` and `From<Self>` already provide the bidirectional conversion operations, and the structure-preserving subtraits can bound on them directly. The dedicated `Iso<T>` trait would have duplicated std machinery for marginal documentation value. Tier 2's `Iso<S, T>` exists because witness types have no `From` analog (they're zero-sized and there is no instance to convert from).
- **Approximate / numerical-equivalence isomorphism with error tracking.** Useful in principle, not needed for the immediate use cases. `Float106 ↔ f64` round-trips with precision loss, but the loss is not modeled by an iso impl — the backward direction simply truncates and downstream code uses the existing `Float106` traits to track drift explicitly. If a future use case requires typed error tracking on a non-strict iso, a separate `ApproxIso<S, T>` trait can be added without disturbing the strict hierarchy.
- **Theory morphisms.** The general case where a morphism `F: T₁ → T₂` is not bijective. Requires dependent types or extensive macro machinery to express usefully; the GATlab approach. Skipped because the isomorphism special case covers ~80% of the practical use cases without the dependent-type tax.
- **Equivalence of categories.** Weaker than natural isomorphism (allows "fattening" of objects). Less useful for engineering than for category-theory research. Not part of this scope.
- **Auto-derivation of iso composition.** If `S iso T` and `T iso U`, then `S iso U` by composition. Could be auto-derived but the two-step composition is often suboptimal compared to a hand-written direct `S iso U`. Convention: ship both when both matter, document the composition as a fallback.

---

## 14. Honest caveats

**Property tests are the only enforcement of the laws.** Rust cannot structurally prove `T::from(S::from(x)) == x` or that `forward` is a group homomorphism. If an iso impl violates the round-trip or homomorphism law, the consequence is silent data corruption at consumer sites. Every iso impl — Tier 1, Tier 2, or Tier 3 — must ship with a `proptest!` block exercising the relevant `assert_*` helper. CI enforces this by code-review convention; the trait surface does not enforce it mechanically.

**Naturality is harder to property-test than round-trip.** Round-trip needs only a generator for `S` or `T`. Naturality needs a generator for `T -> U` functions, which is not straightforward in randomized testing. Convention: use a small fixed bank of test functions (negation, doubling, identity, constant, string-conversion) and exercise naturality against each. This catches obvious violations; it does not exhaustively prove naturality. A more rigorous approach would generate function values via QuickCheck-style coarbitrary, which is feasible but not in the initial scope.

**The Tier 1 / Tier 2 namespace split requires consumers to know which module a marker lives in.** `deep_causality_num::iso::GroupIso<T>` (Tier 1) and `deep_causality_num::iso::witness::GroupIso<S, T>` (Tier 2) are distinct traits with similar names. The disambiguation by module path is intentional but can confuse new contributors. Documentation in `lib.rs` should call out the distinction explicitly and prefer fully-qualified paths in examples.

**Iso impls in physics code commit the codebase to one canonical sign convention.** When the codebase eventually adds, say, a Pauli-matrix iso, the basis choice in the impl body fixes the convention every consumer relies on. Document the chosen convention in the impl's module docs and align it with related conventions elsewhere in the codebase (e.g., the `Metric` enum in `deep_causality_multivector`). Mismatches between conventions are a real source of silent bugs in physics code; the centralization-to-one-impl is how the iso machinery limits that exposure.

**Iso composition can be deceptively expensive.** A two-step iso `S → T → U` does the forward direction in two function applications — two allocations if the intermediate `T` is heap-allocated. If both intermediate forms are large tensors, this is O(N) per step. A direct `S iso U` is one step. The trait surface doesn't distinguish; users need to know. Convention: provide direct isos for hot paths; rely on composition only for rare-path conversions.

**Tier 1 `From`/`Into` impls outside this hierarchy will exist.** The std and ecosystem ship `From` impls everywhere, many of them lossy or one-directional. Code that uses `.into()` cannot assume the conversion is an isomorphism. The marker trait `GroupIso<T>` and friends do convey the iso guarantee, but only when explicitly named as bounds — `.into()` at a call site does not advertise that the conversion is an iso. Reviewers should recognize that "`From` is implemented in both directions" is *not* automatically equivalent to "this is an `Iso`-bound code path." The marker trait is the documentation handle.

**`StandardIso<S, T>` blanket impls fire wherever the underlying bounds are satisfied.** This is the source of the boilerplate reduction, and also a discipline question. If a downstream crate accidentally provides bidirectional `From<S> for T` / `From<T> for S` for a type pair that *isn't* actually a structure-preserving isomorphism — say, lossy primitive conversions that round-trip on representable values but break the group homomorphism law on operations — `StandardIso<S, T>` will silently claim `GroupIso<S, T>` and friends. The blanket impl trusts that the consumer wrote `From` impls that satisfy the marker laws. There is no compile-time check. Property-test discipline is the only enforcement: every bidirectional `From` pair intended to participate in `StandardIso<S, T>` must ship round-trip and homomorphism tests covering the marker levels it claims. Reviewers should reject bidirectional `From` impls that lack the corresponding `proptest!` blocks when the type pair is one the codebase will rely on as an iso.

---

## 15. Suggested change-set naming

When this work is opened, suggested OpenSpec change name: **`add-iso-traits`**.

Phases I1–I3 (foundation traits across `deep_causality_num` and `deep_causality_haft`) could be one change set. Phases I4–I6 (first concrete instances at each tier) could be a second. Documentation and examples (I7) are a third. Downstream consumers (I8–I9) are individually scoped and land as the consuming work needs them.

Per the project convention, no agent commits — the developer reviews each phase and commits.

---

## 16. Bottom line

The iso machinery decomposes into three constructs that each use the simplest tool fitting their job:

- **Tier 1 leverages `From`/`Into`** for in-crate isomorphisms where no new trait surface is needed. The marker subtraits (`GroupIso<T>` ... `DivisionAlgebraIso<T, R>`) bound on `From` in both directions to require bidirectionality and add structure-preservation guarantees.
- **Tier 2 introduces `Iso<S, T>`** for cross-crate cases where orphan-rule constraints forbid the bidirectional `From` placement Tier 1 requires. The impl is hung on whichever of `S` or `T` is local to the crate writing the impl — no dedicated witness type is needed in the single-convention case (which is every current case in the codebase). The generic `StandardIso<S, T>` witness with blanket impls auto-derives every applicable marker subtrait from bidirectional `From` for in-crate cases. Dedicated witness types are reserved for a forward-looking multi-convention scenario that does not yet exist in the codebase.
- **Tier 3 uses `NaturalIso<F, G>`** for HKT witnesses where there is no instance to convert from. Naturality is property-tested via a small bank of test functions.

The foundation is ~4.5 weeks of focused work across `deep_causality_num`, `deep_causality_haft`, `deep_causality_core`, `deep_causality_multivector`, and `deep_causality_sparse`. Downstream consumers each take a few days. The immediate concrete win is typing `PropagatingEffect<T>` ↔ `PropagatingProcess<T, (), ()>` as a natural isomorphism, which cleans up the lift point in the fluid causal-inference pipeline. The medium-term win is making quaternion ↔ Cl(3,0) rotor ↔ Pauli-matrix representations interchangeable as data under explicit sign conventions, which sets up future quantum-classical hybrid work.

The simpler-than-originally-proposed design follows from three observations: `From`/`Into` already provides the bidirectional-conversion mechanics for the simple case; the witness pattern HAFT already uses extends cleanly to multi-convention isos; and blanket impls on a generic `StandardIso<S, T>` witness collapse the marker-trait wiring for any type pair that already satisfies the underlying `From` and algebraic-structure prerequisites. Three constructs cover the design space; no dedicated `Iso<T>` parent trait at the instance level is needed; and the compiler does the marker-trait wiring whenever the underlying conditions hold. This is the right grain for Rust — uses std where it fits, introduces new traits only where existing machinery is genuinely insufficient, and exploits Rust's blanket-impl mechanic to make the common case cost zero boilerplate beyond the underlying `From` impls.

The Tier 2 and Tier 3 method names `to_target` / `to_source` (rather than `forward` / `backward` or `from` / `into`) avoid collision with DeepCausality's temporal vocabulary and std's `From`/`Into` semantics; the direction is encoded in the method name itself. Iso impls provide domain-specific inherent aliases (`as_effect`, `as_process`, `from_quaternion`, `to_quaternion`, etc.) for call-site readability where prose clarity matters more than trait-level uniformity.
