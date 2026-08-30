<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Unified constraints are available on stable Rust

**Scope.** Every HKT witness in `deep_causality_unified_math/`, measured. What a one-line change to
the `HKT` trait would simplify, whether it is semantically equivalent for the code that exists, and
whether the constraint stays enforced.

**Not in scope.** No implementation. No HKT impl was edited to produce this note. Every result below
comes from probes compiled outside the repository.

**Method.** Inventory from `git ls-files` over the seventeen crates. Every compiler claim was run on
`rustc 1.98.0` stable and `rustc 1.100.0-nightly (bff8e12ff 2026-08-26)`. §9 reproduces them.

---

## 1. The finding

The deferred HKT-GAT spec and its review, retired from `openspec/changes/deferred/` and superseded
by this note, waited for the next-generation trait solver before unifying the dual-witness pattern. That wait is unnecessary. **The unification compiles
on stable today**, and the new solver does not help.

The point of the change is that the compiler starts checking algebra. Today it checks none:
thirty-five of the thirty-six witnesses declare `NoConstraint`, and the one that declares a real
constraint cannot carry `Monad` or `CoMonad` because of it.

The change is one clause:

```rust
pub trait HKT {
    type Constraint: ?Sized;

    type Type<T>
    where
        T: Satisfies<Self::Constraint>;   // <-- remove this
}
```

The `Satisfies` bounds stay on the methods. That is the whole edit.

## 2. Why it was blocked

`Monad::bind` and `CoMonad::extend` are the only two operations whose closure signature mentions the
GAT: `FnMut(A) -> F::Type<B>` and `FnMut(&F::Type<A>) -> B`. To check such an impl, rustc must
normalise `F::Type<B>` for a *generic* `B`, which means discharging `B: Satisfies<C>`. The bound sits
right there in the method's where-clause. rustc will not use it.

It succeeds only when the constraint's impl has no premise at all:

| Constraint impl | Discharged for a generic `B`? | `bind` / `extend` |
|---|---|---|
| `impl<T> Satisfies<NoConstraint> for T` | yes, unconditionally | compile |
| `impl<T: Field> Satisfies<FieldConstraint> for T` | no | fail |
| `impl Satisfies<C> for f64` (whitelist) | no | fail |

Delete the GAT's where-clause and there is no obligation to discharge. `Functor`, `Pure`, `Foldable`
and `Applicative` were never affected; none of them mentions the GAT inside a closure.

## 3. What the codebase actually looks like

Thirty-six witnesses across five crates.

| Fact | Count |
|---|---|
| Witnesses in total | 36 |
| Carrying a real constraint | **1** (`StrictCausalTensorWitness` → `TensorConstraint`) |
| Carrying `NoConstraint` | 35 |
| Impls that already omit the GAT where-clause | **30** |
| Impls that write it | 16 |

The second pair matters. **The where-clause is already optional in practice.** Thirty impls in
`haft`, `linear` and `multivector` omit it and compile; sixteen in `tensor` and `topology` write it.
Nothing in the codebase depends on the distinction, because an impl may always be weaker than its
trait.

Trait coverage across all witnesses:

| Trait | Impls | Trait | Impls |
|---|---|---|---|
| `Functor` | 29 | `Adjunction` | 3 |
| `Pure` | 18 | `Traversable` | 2 (haft only) |
| `Foldable` | 17 | `Bifunctor` | 2 (haft only) |
| `Applicative` | 15 | `MonoidalMerge` | 2 |
| `CoMonad` | 15 | `NaturalTransformation` | 1 (haft only) |
| `Monad` | 14 | `ParametricMonad`, `RiemannMap` | 1 each |
| | | `Profunctor`, `CyberneticLoop` | 0 |

`Traversable`, `Bifunctor` and `NaturalTransformation` exist only inside `haft`'s own extensions. No
mathematics crate implements any of them, which is what `unified_math_gaps.md` §3.4 and §3.5 record.

## 4. Semantic equivalence

Two files, identical but for the GAT where-clause, each carrying a `NoConstraint` witness written
exactly as the thirty in the repository are written, exercised through `fmap` and `bind` with both a
numeric and a `String` element.

```
with_where.rs      COMPILES
without_where.rs   COMPILES
```

For a `NoConstraint` witness the clause is vacuous, since `impl<T> Satisfies<NoConstraint> for T`
admits every type. **All thirty-five existing witnesses are unaffected.** The sixteen impls that
spell the clause out would drop three lines each; the thirty that omit it change not at all.

The two are not equivalent for a *constrained* witness, and that is the point: with the clause, a
constrained `Monad` cannot be written at all; without it, it can.

## 5. Is the constraint still enforced?

Yes, at every operation. Probed with a real `Field` bound, a second `Ring` bound, and one generic
function serving both.

| Probe | Result |
|---|---|
| `Field` constraint with `bind` **and** `extend` | compiles |
| Two constraints (`Field`, `Ring`) coexisting | compiles |
| One generic `fn twice<F: Monad<F>, T: Satisfies<F::Constraint>>` over both | compiles |
| `String` through the `Field` witness | **rejected** — `String: Satisfies<FieldConstraint> is not satisfied` |
| `i64` (is `Ring`, is not `Field`) through the `Field` witness | **rejected** |
| Naming the type `<FieldW as HKT>::Type<String>` | **accepted** |
| Operating on that named value through `bind` | **rejected** |

**What is traded is nameability. What is bought is enforcement.** Without the clause,
`<FieldW as HKT>::Type<String>` is a well-formed type name for `Bag<String>`. Nothing can be done
with it: every method demands `Satisfies`, so no call is admitted that was admitted before.

The trade is lopsided, and worth stating plainly. Today the codebase enforces **no** algebraic
constraint anywhere: thirty-five of thirty-six witnesses are `NoConstraint`, and the one that is not
cannot carry `Monad` or `CoMonad`. The where-clause forbids writing a type nobody writes, at the
price of the compiler checking no algebra at all. Dropping it lets `Field`, `Ring` and
`AssociativeRing` become compiler-checked preconditions on every operation. An unusable type name is
not a defect worth that.

## 6. What it would improve, per crate

| Crate | Today | After |
|---|---|---|
| `haft` | `Satisfies` and the dual-witness pattern documented as transitional, pending a compiler fix that does not deliver | The constraint system works as specified. The transitional framing can be retired |
| `tensor` | Settled 2026-08-30: the strict witness was deleted, not enabled. `CausalTensorWitness` keeps `NoConstraint`, which is correct for a structural functor over a container with no element bound | Unchanged. Tensor's algebra is enforced per-impl and needs nothing from the constraint system |
| `topology` | Ten witnesses write the clause; all use `NoConstraint`, so none enforces anything | Free to adopt the constraints the retired spec already assigned them |
| `linear`, `multivector` | `NoConstraint` throughout, no clause written | Unchanged. Free to adopt constraints when wanted |

The spec's migration table can be executed rather than deferred. The constraints it assigned — `FieldConstraint` for `Complex`, `AssociativeRingConstraint` for `Quaternion`,
`AbelianGroupConstraint` for `Octonion`, `RealFieldConstraint` for `f32`/`f64` — become expressible.

Two corrections to the deferred documents, both measured:

- The spec claimed the strict impls "compile successfully without modification" under
  `-Znext-solver`. They do not. The error changes from E0276/E0277 to **E0271** and still fails.
  Nightly 1.100.0 has the new solver on by default; its result is identical to `-Znext-solver`.
- The review stated a strict witness "CANNOT implement `Applicative` or `Monad`". It can
  implement `Applicative`, on stable, against the real `CausalTensor`. What fails is *calling* it
  with a closure, and that fails identically under the new solver. A function pointer works, and
  `impl<A, B> Satisfies<C> for fn(A) -> B` compiles on stable because both parameters appear in the
  self type.

## 7. The one real limit

A container carrying a **struct-level bound** cannot be a witness, with or without the clause:

```rust
pub struct Cx<T: Field> { re: T, im: T }
// error[E0277]: the trait bound `T: Field` is not satisfied
//               required by a bound in `Cx`
```

Moving the bound from the struct to its impls fixes it and compiles. Inside `num_complex` this is
already inconsistent: `Quaternion<F>` carries no struct-level bound, while `Complex<T: RealField>`
and `Octonion<F: RealField>` do. Struct-level bounds are an anti-pattern independently of this note;
here they are the only thing standing between those two types and a witness.

## 8. Recommendation

1. ~~**Drop the GAT where-clause from `HKT`.**~~ **Done 2026-08-30.** Removed from `HKT` and from
   the whole `HKT2Unbound`..`HKT6Unbound` family, plus 24 now-redundant sites across 17 files. The
   thesis held: a constrained `Monad` compiles and runs, `bind(m, pure) == m`, and a `String`
   element is still rejected with `the trait bound String: Satisfies<FieldConstraint> is not
   satisfied`. `bazel test //...` 1231 pass.

   One trap worth recording: `cargo build` does not compile doctests, so a `rust` doctest in
   `hkt/mod.rs` that spelled the clause out survived a green workspace build and failed only under
   `bazel test`. Anything asserting the old shape has to be found by a doctest run, not a build.
2. ~~**Then enable the strict witness.**~~ Superseded 2026-08-30. `StrictCausalTensorWitness` and
   `TensorConstraint` were deleted rather than enabled: unexported, uncalled outside their own unit
   tests, and carrying a whitelist that contradicted the tier it claimed. The measurement is in
   `openspec/notes/archive/hkt_gat/hkt_CausalTensor.md`. Tensor now has no constrained witness, so
   the workspace has none; step 3 is where the first real one would come from.
3. **Then adopt real constraints** where they are wanted, following the table the retired spec
   carried (recoverable with `git show HEAD:openspec/changes/deferred/hkt_gat.md`). This is the part with genuine design content and it should be taken crate by crate.
4. **This note replaces the retired documents.** Their premise, that the wait is on a compiler
   release, is false, and §6 records why.

Ordering matters only between 1 and 2. Step 1 is mechanical and reversible; nothing else should ride
along with it.

## 9. Reproducing

Probes live outside the repository. The shape of the decisive one:

```rust
// The change under test: no where-clause on the GAT.
pub trait HKT { type Constraint: ?Sized; type Type<T>; }

pub trait Field {}                 impl Field for f64 {}
pub struct FieldConstraint;        impl<T: Field> Satisfies<FieldConstraint> for T {}
pub trait Ring {}                  impl Ring for i64 {}
pub struct RingConstraint;         impl<T: Ring> Satisfies<RingConstraint> for T {}

// One generic function, any witness, any constraint.
pub fn twice<F, T>(m: F::Type<T>, f: fn(T) -> F::Type<T>) -> F::Type<T>
where F: HKT + Monad<F>, T: Satisfies<F::Constraint> { /* bind twice */ }

pub fn use_field() -> Bag<f64> { twice::<FieldW, f64>(..) }   // compiles
pub fn use_ring()  -> Bag<i64> { twice::<RingW,  i64>(..) }   // compiles
pub fn negative()  -> Bag<String> { twice::<FieldW, String>(..) }  // E0277, as intended
```

```
rustc +stable  --crate-type lib --out-dir /tmp/out probe.rs
rustc +nightly --crate-type lib --out-dir /tmp/out probe.rs
```

Both toolchains agree on every result in this note.
