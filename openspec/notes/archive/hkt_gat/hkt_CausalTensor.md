<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# How `deep_causality_tensor` should enforce HKT and algebra bounds

**Scope.** `deep_causality_tensor` only. What the crate actually requires of its element type, where
that requirement is enforced today, and what a correct design looks like with no forgeable holes, no
vacuous stand-ins and no gaps.

**Not in scope.** Any other crate. No code was changed to produce this note.

**Method.** Every claim below was compiled. Probes ran on `rustc 1.98.0` stable and
`1.100.0-nightly (bff8e12ff 2026-08-26)`, against the real `deep_causality_algebra` and
`deep_causality_num_complex`. §8 reproduces them. Companion: `hkt_gat.md`, which establishes that
constrained `Monad`/`CoMonad` compile on stable once the GAT where-clause is dropped.

---

## 1. Summary

The premise that tensor needs an algebraic element constraint on its HKT witness does not survive
contact with the crate. `CausalTensor<T>` has **no inherent element bound**, its structural
operations do no arithmetic, and its algebraic operations already carry correct per-impl bounds that
the compiler already enforces.

What is actually wrong in tensor is narrower and more fixable than the deferred design assumed:

| | Status |
|---|---|
| `TensorConstraint` | Dead. Referenced nowhere outside its own file |
| `StrictCausalTensorWitness` | Dead. Unexported and called by no non-test code; its only callers were three `#[cfg(test)]` unit tests in the same directory, which `bazel test` never ran because the tensor `rust_library` declares no unit-test target |
| Its whitelist | Internally inconsistent with the tier it claims |
| `NoConstraint` on `CausalTensorWitness` | **Correct.** Not a gap |
| Algebra enforcement on tensor arithmetic | **Already correct**, per-impl, already enforced |

## 2. The forgery hole is real

An unsealed marker trait can be implemented by any downstream crate for any local type. Compiled,
stable:

```rust
// downstream crate, no unsafe, no nightly, no macros
#[derive(Clone)] pub struct Forged;          // zero algebraic structure
impl Satisfies<FieldConstraint> for Forged {}
pub fn exploit() -> Bag<Forged> { field_constrained_op(Bag(vec![Forged])) }
```

**This compiles.** `Satisfies` is `pub trait Satisfies<C: ?Sized> {}` with no seal, and
`impl ForeignTrait<ForeignType> for LocalType` is exactly what the orphan rules permit. Any
constraint that is publicly nameable is therefore advisory, not enforced.

**It is not exploitable in tensor today**, for a reason that is luck rather than design:
`extensions` is declared `mod extensions;` rather than `pub mod`, and `lib.rs` re-exports
`CausalTensorWitness` and `CausalTensorTrainWitness` but neither `TensorConstraint` nor
`StrictCausalTensorWitness`. The constraint type cannot be named downstream, so it cannot be forged.

Privacy is a real seal, and it is the one tensor currently relies on. It is also brittle: one
`pub use` reopens the hole silently, and it forbids downstream code from writing anything generic
over the constraint.

## 3. Sealing works, and has one hard limit

A supertrait seal closes the hole. Measured, stable:

| Probe | Result |
|---|---|
| Downstream forges the **unsealed** marker | **compiles** — hole confirmed |
| Downstream forges the **sealed** marker | **refused** — `Forged: sealed::Sealed is not satisfied` |
| `f64` through the sealed `Field` gate | accepted |
| `f64` through the sealed `RealField` gate | accepted |
| `Complex<f64>` through the sealed `Field` gate | accepted |
| `Complex<f64>` through the sealed `RealField` gate | **refused** — a field, but unordered |

The seal grants membership only through the algebra tower:

```rust
mod sealed { pub trait Sealed {} }
pub trait SatisfiesSealed<C: ?Sized>: sealed::Sealed {}
impl<T: Field> sealed::Sealed for T {}
impl<T: Field>     SatisfiesSealed<FieldConstraint>     for T {}
impl<T: RealField> SatisfiesSealed<RealFieldConstraint> for T {}
```

Downstream cannot forge, and a downstream type that genuinely implements `Field` is admitted
automatically. Extension stays open; forgery closes.

**The limit.** Sealing and universality cannot share one marker:

```rust
impl<T> SatisfiesSealed<NoConstraint> for T {}
// error[E0277]: the trait bound `T: Sealed` is not satisfied
```

So `haft`'s `Satisfies` cannot be sealed without breaking `NoConstraint`, and with it `VecWitness`,
`OptionWitness`, `BoxWitness` and `ResultWitness`. A sealed algebra marker has to be a **second**
mechanism alongside the universal one, not a replacement for it.

## 4. What tensor actually requires of `T`

`CausalTensor<T>` declares no bound on `T`. Bounds live on the impls that need them, which is the
correct Rust design and makes the crate's real requirements legible. Counted across `src/`:

| Bound | Sites | Serves |
|---|---|---|
| `T: ConjugateScalar` | 18 | Complex-aware operations, tensor-train |
| `T: Scalar + ConjugateScalar<Real = …>` | 11 | Decompositions |
| `T: Clone` | 13 | Structure: reshape, view, slice |
| `T: Clone + Default + PartialOrd` | 5 | Ordering, comparison |
| `T: Default + Clone + RealField + Zero + One + PartialEq + FromPrimitive` | 5 | Statistics |
| `T: Clone + RealField + Zero + One + Sum + PartialEq + FromPrimitive` | 4 | Reductions |
| `T: Clone + PartialOrd + {Add,Sub,Mul,Div}<Output = T>` | 4 each | Elementwise arithmetic |

There is no single tier that covers this. Statistics need `RealField`; conjugation needs
`ConjugateScalar`, which `Complex` satisfies and `f64` also satisfies; reshaping needs `Clone` and
nothing more. A one-constraint-per-container model cannot express that, and it should not try.

## 5. The conflation at the root of the problem

Two different populations of operation have been treated as one.

**Structural operations** — `fmap`, `fold`, `pure`, `bind`, `extend`. These move elements; they never
compute with them. `CausalTensorWitness::fmap` maps `CausalTensor<A>` to `CausalTensor<B>` with `A`
and `B` unrelated. Constraining them to `Field` would forbid mapping a tensor of strings, or mapping
`f64` to a label, both of which are legitimate and both of which work today.

**`NoConstraint` on `CausalTensorWitness` is therefore correct.** It is not a stand-in, not a gap and
not a compromise. It is the accurate statement that a functor over a container does not care what the
container holds.

**Algebraic operations** — arithmetic, statistics, decomposition, conjugation. These do compute, they
do need bounds, and they already carry them per-impl. `T: RealField + Zero + One + Sum +
FromPrimitive` on a mean is precise, enforced, and cannot be forged, because it names real traits
rather than a marker.

The deferred design proposed routing algebra through the HKT constraint. That points the enforcement
at the operations that do not need it and leaves untouched the ones that do — which already have it.

## 6. The dead code, and what is wrong with it

`TensorConstraint` claims to be "Tier 4: TensorDataConstraint" and to limit usage "to types that are
mathematically valid for tensor physics (Fields, Rings)". Its whitelist admits `f32`, `f64`,
`Complex<f32>`, `Complex<f64>`, every signed and unsigned integer, `usize`, `isize`, and any nested
`CausalTensor<T>`.

Integers are not fields; they have no division. `usize` is not a ring under the tower's own
definitions. The whitelist and its docstring disagree, and nothing catches it because the type is
used nowhere. The docstring also states the witness "does NOT implement `Applicative`", which
`hkt_gat.md` §6 shows to be false.

A constraint used nowhere, admitting members its own documentation excludes, is worse than no
constraint: it reads as a guarantee and is not one.

## 7. Recommendation for tensor

**Keep `NoConstraint` on `CausalTensorWitness`.** Structural HKT over a polymorphic container is
correct as it stands. Do not attach algebra to it.

**Retire `StrictCausalTensorWitness` and `TensorConstraint`.** Both are dead. The 66 commented-out
lines beneath them, and the note blaming the trait solver, go with them. `hkt_gat.md` §6 records why
the solver was never the obstacle. If they are kept for later, they should be `#[cfg(test)]` or moved
out of `src/`, not left as commented prose that documents a diagnosis now known to be wrong.

**Leave algebra where it is.** Per-impl bounds naming real traits are already sound, already
enforced, and already unforgeable. `T: ConjugateScalar` is a stronger guarantee than any marker,
because there is no marker to forge.

**If a constrained witness is ever wanted**, it should be for operations that are algebraic rather
than structural, and it should use the sealed pattern from §3 with the constraint defined where
`Satisfies` lives. Until such an operation exists, there is nothing for it to constrain.

**Drop the GAT where-clause** from `HKT` regardless. It buys nothing for tensor, whose three impls
spell it out, and it is what blocks every other crate from having a real constraint. That is
`hkt_gat.md`'s recommendation and this note does not change it.

## 8. Against the status quo

| Dimension | Status quo | This recommendation |
|---|---|---|
| Structural HKT | `NoConstraint`, correct but described as a compromise | `NoConstraint`, correct and documented as correct |
| Algebraic operations | Per-impl real bounds, enforced, unforgeable | Unchanged |
| `TensorConstraint` | Dead, self-inconsistent, reads as a guarantee | Removed |
| `StrictCausalTensorWitness` | Dead, plus 66 commented lines and a wrong diagnosis | Removed |
| Its three unit tests | Sat in `src/`, so `bazel test` never ran them | Removed with their subject |
| Forgeability | Not exploitable, by privacy rather than design | Not exploitable, and nothing to exploit |
| Lines in `ext_hkt_strict.rs` | ~170, none reachable | 0 |
| Compiler-checked algebra in tensor | Real, in the per-impl bounds | Unchanged |

The status quo's defect is not missing enforcement. It is a dead abstraction that claims enforcement
it does not perform, next to real enforcement that works and gets no credit.

**What this does not deliver.** No new algebraic checking in tensor, because tensor does not lack
any. Crates whose containers *do* have a uniform element requirement — `topology`'s manifolds and
lattice gauge fields, whose `hkt_lattice_gauge.rs` carries the same blocked-on-the-solver comment —
are where the sealed constraint pattern earns its cost. That case should be assessed on its own.

## 9. Reproducing

Two crates, `up` and `down`, `down` depending on `up`:

```rust
// up: the unsealed marker, as haft has it today
pub trait Satisfies<C: ?Sized> {}
pub struct FieldConstraint;
impl<T: Field> Satisfies<FieldConstraint> for T {}

// up: the sealed alternative
mod sealed { pub trait Sealed {} }
pub trait SatisfiesSealed<C: ?Sized>: sealed::Sealed {}
impl<T: Field> sealed::Sealed for T {}
impl<T: Field> SatisfiesSealed<SealedFieldConstraint> for T {}

// down: the same forgery against each
#[derive(Clone)] pub struct Forged;
impl Satisfies<FieldConstraint> for Forged {}              // compiles
impl SatisfiesSealed<SealedFieldConstraint> for Forged {}  // E0277: Forged: Sealed not satisfied
```

```
cargo +stable  check --manifest-path down/Cargo.toml
cargo +nightly check --manifest-path down/Cargo.toml
```

Both toolchains agree on every result in this note.

---

## 10. Executed

Applied 2026-08-30. `src/extensions/ext_hkt_strict.rs` and `src/extensions/ext_hkt_strict_tests.rs`
deleted, along with their two `mod` declarations. `CausalTensorWitness` gained a docstring stating
why `NoConstraint` is correct rather than provisional. Six docstrings in `deep_causality_haft`
named `TensorData`, `TensorDataConstraint` or `StrictCausalTensorWitness`; none of those three
types existed, and all six were corrected. The `rust,ignore` constrained-witness example on `HKT`
became a compiling doctest.

§7's remaining item, dropping the GAT where-clause from `HKT`, is untouched and still open. It is
`hkt_gat.md`'s recommendation, and it belongs to the crates that want a real constraint; tensor
does not.
