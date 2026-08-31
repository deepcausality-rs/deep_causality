<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# How `deep_causality_topology` should enforce HKT and algebra bounds

**Status: all ten tracker items closed; §§6, 7 and 12 describe the code BEFORE the fix.** The
findings below were acted on, and §10 records the outcome of each. Sections 6 and 7 are kept in
their original present tense as the record of what was found; the unsound `curvature` cast, the
`GaugeFieldHKT` stubs and the `unsafe_code = "allow"` override they describe are all gone. §12's
reproduction steps cannot be followed against this tree for the same reason: P1 uses the
five-parameter `CurvatureTensor` API, which is now `CurvatureTensor<T>`, and the `Cargo.toml` block
it quotes no longer exists. Read those sections as history.

**Scope.** `deep_causality_topology`, cross-checked against `deep_causality_linear` and
`deep_causality_multivector`, because topology's witnesses delegate into both and inherit their
semantics. Fifteen topology witnesses, what each requires of its element type, where that requirement
is enforced today, and what a correct design looks like with no holes, no stand-ins and no gaps.

**Method.** Inventory from `src/extensions/`: 2,880 lines in topology, 686 in linear, 819 in
multivector. Every compiler and behaviour claim below was compiled or run on `rustc 1.98.0` stable,
against the real crates linked as downstream dependencies. §12 reproduces them.

**Not in scope.** No code was changed to produce this note.

**Companions.** `hkt_gat.md` establishes that constrained `Monad`/`CoMonad` compile on stable once
the GAT where-clause is dropped. `archive/hkt_gat/hkt_CausalTensor.md` settles tensor, where
`NoConstraint` proved correct.

---

## 1. Summary

Two separate problems live in this stack, and only one of them is about constraints.

**Problem one is topology's alone, and it is a soundness bug.** Three of fifteen witnesses need to
compute on their elements. None can, because `Satisfies<C>` admits a type without granting any
capability over it. Each worked around that differently:

| Witness | Workaround | Consequence |
|---|---|---|
| `CurvatureTensorWitness` | `unsafe` pointer cast from an unconstrained `A` to `TensorVector<T>` | **Unsound public API.** Safe downstream code, no warning |
| `GaugeFieldWitness` | A shadow container plus three impls that ignore their arguments | `pure` discards its value; `ibind` and `merge` never call their function |
| `LatticeGaugeFieldWitness` | No HKT impl; the real work sits in inherent methods | Correct, and the only honest one |

**Problem two is shared across linear, multivector and topology, and it is not about constraints at
all.** These containers pair a payload with a *geometric context*: a sparsity pattern, a metric, a
complex, a cursor. `haft`'s `Functor`, `Pure` and `Monad` have no channel for that context, so `pure`
must invent one and `bind` must choose one. Measured, `bind(m, pure) == m`:

| Witness | Crate | Context | `Monad` | Right identity |
|---|---|---|---|---|
| `DenseVectorWitness` | linear | none | yes | **holds** |
| `CsrMatrixWitness` | linear | sparsity pattern | **not implemented** | n/a |
| `DenseMatrixWitness` | linear | shape | **not implemented** | n/a |
| `CausalTensorWitness` | tensor | shape | yes | **fails**: `[2,2]` becomes `[4]` |
| `CausalMultiVectorWitness` | multivector | metric | yes | **fails**: `Minkowski(4)` becomes `Euclidean(0)` |
| `ManifoldWitness` | topology | complex, metric, cursor | yes | **holds at cursor 0**, fails at cursor 2 |

`linear` is the crate that got this right, and it got it right by omission: it implements `Monad`
only for `DenseVector`, the one container with no context to fabricate. That is the precedent the
others should follow.

## 2. The topology inventory

Fifteen witnesses in thirteen files. Fourteen declare an HKT arity, and every one of the fourteen
declares `Constraint = NoConstraint`. `LatticeGaugeFieldWitness` declares no HKT impl.

| Population | Witnesses | Elements are | `NoConstraint` |
|---|---|---|---|
| **A. Structural** | `Graph`, `Hypergraph`, `MixedGraph`, `Topology`, `PointCloud`, `Chain`, `Manifold`, `GenericManifold` | moved, never computed on | **correct** |
| **B. Declared only** | `CellComplexWitness`, `LatticeComplexWitness` | never touched; no operation exists | vacuous |
| **C. Adjunction pair** | `ExteriorDerivativeWitness`, `BoundaryWitness` | moved into forms and chains | **correct** |
| **D. Computing** | `CurvatureTensorWitness`, `GaugeFieldWitness`, `LatticeGaugeFieldWitness` | contracted, averaged, multiplied | **wrong** |

Trait coverage: `Functor` 8, `CoMonad` 6, `Adjunction` 2, `Foldable` 2, `Pure` 1, `Monad` 1,
`Applicative` 1, `MonoidalMerge` 1, `ParametricMonad` 1, `RiemannMap` 1.

The last three deserve naming. `MonoidalMerge`, `ParametricMonad` and `RiemannMap` have **exactly one
implementer each in the entire workspace**, all in `src/extensions/hkt_gauge/`, and all three are
population D. An abstraction with one implementer that cannot implement it honestly is not carrying
its weight.

## 3. What `Satisfies` can and cannot do

Everything in §6 follows from one measurement. Compiled, stable:

```rust
pub struct FieldConstraint;
impl<T: Field> Satisfies<FieldConstraint> for T {}

pub fn admits<A: Satisfies<FieldConstraint>>(_a: A) {}
pub fn ok() { admits(1.0f64); }                       // compiles: the gate works

pub fn use_it<A: Satisfies<FieldConstraint>>(a: A, b: A) -> A { a.add(b) }
// error[E0599]: no method named `add` found for type parameter `A`
//   help: items from traits can only be used if the type parameter is bounded by the trait
```

**The gate holds; the capability does not exist.** A constrained witness can refuse `String`. It
still cannot add two admitted elements, because `Satisfies<FieldConstraint>` is an empty marker and
`Field` is nowhere in scope for `A`.

For populations A and C that costs nothing, since moving an element needs no capability. For
population D it is fatal, and the three workarounds are what happens when an impl must return a
result it has no legal way to compute.

## 4. Population A: the structural witnesses are correct

`GraphWitness`, `HypergraphWitness`, `MixedGraphWitness`, `TopologyWitness` and `PointCloudWitness`
share one shape: the payload lives in a `CausalTensor<T>`, `fmap` delegates to
`CausalTensorWitness::fmap`, and `extend` walks a cursor. No arithmetic touches `T` anywhere.
`ChainWitness` does the same through `CsrMatrixWitness`; the two manifold witnesses go back through
`CausalTensor`.

`NoConstraint` here is accurate, exactly as in tensor. Document it as correct rather than leaving it
looking like a compromise, and change nothing else.

**One caveat, which belongs to §8 rather than to constraints.** `TopologyWitness::fmap`,
`ChainWitness::fmap` and the Stokes `unit` all rebuild `SimplicialComplex::<B> { skeletons,
boundary_operators, coboundary_operators, ..Default::default() }`. The Hodge star and geometric data
are dropped, so `fmap(id, x)` is not `x`: reading a Hodge star from the result yields an error where
the source yielded a value. The behaviour is deliberate and commented; the law consequence is
recorded nowhere.

## 5. Population B: two witnesses that do nothing

`CellComplexWitness<C>` and `LatticeComplexWitness<D, R>` implement `HKT` and stop. No `Functor`, no
`Pure`, no operation of any kind. Both are re-exported from `lib.rs`; the types their `Type<T>` names,
`CellField<C, T>` and `LatticeField<D, R, T>`, are **not** exported.

A downstream caller can name `<CellComplexWitness<C> as HKT>::Type<T>` and receive a type it cannot
name, construct or use. That is the tensor defect just retired, except exported. Either give them the
functor they imply or remove them.

## 6. Population D: the three that need algebra

> **Historical.** Describes the state before the fix. Every defect in this section was resolved; see §10.

### 6.1 `CurvatureTensorWitness` is unsound

`RiemannMap::curvature` is generic in `A, B, C, D`, bounded only by `Satisfies<P::Constraint>`. With
`Constraint = NoConstraint` that admits every type. The impl then does this:

```rust
unsafe {
    let u_ptr = &u as *const A as *const TensorVector<T>;
    ...
    let ret = std::ptr::read(result_ptr);
```

`TensorVector<T>` is `struct TensorVector<T> { pub data: Vec<T> }` with no `repr` attribute, so its
layout is unspecified. The module doc states the contract plainly: "**SAFETY:** The caller MUST
ensure that `A`, `B`, `C`, and `D` are `TensorVector`. Passing any other type will result in
**Undefined Behavior**." Nothing enforces it.

Three probes, compiled from a downstream crate against the published API, with no `unsafe` at the
call site and no warning emitted:

| Probe | Element type passed | Result |
|---|---|---|
| `p1_wrong_type_vec` | `Vec<f64>` | **compiles** |
| `p1b_wrong_type_zst` | a zero-sized struct | **compiles** |
| `p1c_scalar_mismatch` | `TensorVector<f32>` against an `f64` witness | **compiles** |

A safe function that is undefined behaviour for inputs its own signature admits is unsound, and this
one is published. The probes were compiled and deliberately not run.

**The generality is unused.** Every call site in the workspace passes `TensorVector<T>`. The one
production caller, `deep_causality_physics/src/theories/general_relativity/gr_ops_impl.rs:142`, even
carries the comment "Use TensorVector for HKT safety contract"; the three topology tests pass it too.
The type parameters exist because `RiemannMap` is generic, and the crate pays for that genericity
with `unsafe`.

**A constraint alone would not fix this.** Suppose `Constraint = TensorVectorConstraint<T>`, satisfied
only by `TensorVector<T>`. Wrong types would then be rejected, which is real progress. The cast still
could not go away, because §3 applies: the body knows `A` was admitted and still cannot convert it.
The marker is also forgeable, since `impl Satisfies<TensorVectorConstraint<f64>> for MyLocalType {}`
is what the orphan rules permit, and sealing `Satisfies` is impossible because
`impl<T> Satisfies<NoConstraint> for T` requires universality (measured in `hkt_CausalTensor.md` §3).
A constraint would narrow the hole without closing it.

### 6.2 `GaugeFieldWitness` implements three laws-free stubs

`GaugeField<G, A, F>` requires `G: GaugeGroup`, which `HKT3Unbound::Type<A, B, C>` cannot express, so
the witness targets a parallel struct:

```rust
pub struct GaugeFieldHKT<G, A, F, T> {
    inner: Option<Box<GaugeFieldData<T>>>,     // flat Vec<T> copies
    _phantom: PhantomData<(G, A, F, T)>,       // G, A, F are phantom
}
```

The payload is flat `Vec<T>`, described in its own comment as "In production, this would be the
actual tensor data". The three impls against it:

| Method | Body | Consequence |
|---|---|---|
| `ParametricMonad::pure` | `let _ = value; GaugeFieldHKT::empty()` | discards the value it was given |
| `ParametricMonad::ibind` | `_f` unused; "propagate data unchanged" | the bind function is never called |
| `MonoidalMerge::merge` | `_f` unused; hardcoded `(a + b) / 2` | the merge function is never called |

`pure` losing its argument breaks left identity by construction: `ibind(pure(a), f)` is empty for
every `a` and `f`, while `f(a)` need not be. `merge` documents itself as modelling `∂_μF^μν = J^ν`
and computes an element-wise average.

The docstring also warns that the wrapper "uses **unsafe dispatch**" and that "Misuse causes
Undefined Behavior". There is no `unsafe` in the file and no type erasure; the fields are
`PhantomData` and `Vec<T>`. The warning describes a design that was not built.

**What is real here.** `GaugeFieldWitness`'s inherent methods carry proper bounds
(`G: GaugeGroup, A: Field + Copy + Default + PartialOrd, R: RealField`) and do the actual physics.
`deep_causality_physics` uses them heavily: `compute_field_strength_abelian`,
`compute_field_strength_non_abelian`, `gauge_rotation`, `field_strength_from_eb_vectors`. Every call
site of the three stub methods is in topology's own tests, which assert the stub behaviour.

### 6.3 `LatticeGaugeFieldWitness` is honest, and its comment is wrong twice

This witness implements no HKT trait and says so, keeping `map_field`, `zip_with`, `scale_field` and
`identity_field` as inherent methods with real bounds. That is the right call. Its stated reason is
wrong in two ways.

First, the solver. The comment blamed the next-generation trait solver; measured on nightly 1.100.0,
which has it on by default, the strict impls still fail and the error merely changes to E0271. That
half was corrected when tensor's dead witness was deleted.

Second, and still uncorrected, the parameter. The comment blames `R: RealField` at the struct level
on `LatticeGaugeField<G, D, M, R>`. But `R` is not what a functor over this type maps; `M`, the
matrix element, is, and `M` carries no struct-level bound. A witness over `M` with `R` fixed compiles
today, on stable, with the GAT where-clause still in place:

```rust
pub struct LgfOverM<G: GaugeGroup, const D: usize, R: RealField>(PhantomData<(G, R)>);

impl<G: GaugeGroup, const D: usize, R: RealField> HKT for LgfOverM<G, D, R> {
    type Constraint = NoConstraint;
    type Type<T> = LatticeGaugeField<G, D, T, R> where T: Satisfies<NoConstraint>;
}
// compiles
```

What blocks `Functor` is §3 again: `fmap` would rebuild each `LinkVariable<G, B, R>`, which needs
`B: Field + Copy + Default + PartialOrd + Debug`, and `Satisfies` cannot supply it. The design is
right; only the diagnosis needs replacing.

The same file carries roughly twenty lines of working notes inside `map_field` and `zip_with`
("Wait, if we map scalars A->B, do we change Beta?", "But the previous implementation applied f to
beta?"). Those are drafting artifacts, not documentation.

## 7. The unsafe exemption rests on the wrong diagnosis

> **Historical.** The exemption is gone: the crate now carries `[lints] workspace = true` with no `unsafe_code` override, and the quoted `Cargo.toml` block no longer exists.

`deep_causality_topology/Cargo.toml`:

```toml
[lints.rust]
# Exempt from the workspace `unsafe_code = "forbid"` policy.
# HKT fmap pointer-cast (rustc type-equality limitation) in hkt_gauge. Remove when the compiler
# limitation is resolved.
unsafe_code = "allow"
```

Three problems. The exemption is crate-wide, so it also covers the two `unsafe impl Send/Sync for
LatticeCell<D>` in `types/lattice_complex/lattice_cell.rs`, which have nothing to do with HKT and are
not mentioned. Its removal trigger is a compiler limitation that measurement does not support, so the
trigger will never fire. And what it permits is not a local convenience; it is the unsound public API
in §6.1. The workspace forbids `unsafe`, and topology spends the only exemption working around its
own HKT layer.

## 8. The cross-crate picture: context, not constraints

Topology delegates into linear and multivector, so their semantics are topology's semantics.
Comparing the three crates makes the shared defect visible, and it has nothing to do with element
constraints.

**Every one of these containers pairs a payload with a geometric context.** `CsrMatrix` has a
sparsity pattern, `DenseMatrix` a shape, `CausalTensor` a shape, `CausalMultiVector` a `Metric`,
`Manifold` a complex plus a metric plus a cursor, `Chain` and `Topology` a `SimplicialComplex`.
`haft`'s `Functor`, `Pure` and `Monad` carry no channel for any of it. `Pure::pure` receives one
value and must produce a container, so it has to invent the context from nothing. `Monad::bind`
receives contexts from both the input and every `f(a)`, and has to pick.

What each crate did with that:

| Crate | Choice | Result |
|---|---|---|
| `linear` | `Monad` implemented **only** for `DenseVector`, the context-free container | right identity holds; `CsrMatrix` and `DenseMatrix` get `Functor`/`Foldable`/`Pure`/`Applicative`/`CoMonad` and no `Monad` |
| `multivector` | `pure` fabricates `Metric::Euclidean(0)`; `bind` overwrites the accumulator with the **last** `f(a)`'s metric | `Minkowski(4)` becomes `Euclidean(0)`. Measured |
| `tensor` | `pure` builds an empty shape; `bind` rebuilds shape as `&[len]` | `[2,2]` becomes `[4]`. Measured |
| `topology` (`Manifold`) | `bind` takes complex and metric **from the input**, ignoring `f`'s | right identity holds for those two fields |
| `topology` (`Chain`, `Topology`) | `fmap` rebuilds the complex with `..Default::default()` | Hodge star and geometry silently dropped |

**`linear`'s omission is the correct precedent, and `ManifoldWitness::bind` is the correct
discipline.** Taking the context from the input rather than from the results is what makes the law
hold:

```rust
Manifold {
    complex: m_a.complex.clone(),   // from the input, not from f
    metric:  m_a.metric.clone(),
    data: new_tensor,
    cursor: 0,                      // <-- and here it breaks
}
```

The cursor is the exception, and it is a one-line bug. Measured:

```
cursor 0, shape [3]    bind(m,pure)==m : true    cursor 0->0
cursor 2, shape [3]    bind(m,pure)==m : false   cursor 2->0
```

The irony is worth recording. Six `extend` implementations in topology carry the comment "Preserve
the focus so `extend` satisfies the comonad laws (right identity and associativity); resetting to `0`
breaks them for a non-zero focus." The same crate then resets the cursor in `bind`, breaking the
monad law for exactly that reason. The reasoning is already written down six times; `bind` did not
get it.

**One more inconsistency this comparison exposes**, and it is total. Topology writes the GAT
where-clause in **14 of its 14** HKT declarations. Linear writes it in **0 of 3**
(`type Type<T> = CsrMatrix<T>;`), and multivector's `CausalMultiVectorWitness` writes it in **0 of
1**. Same trait, opposite conventions in one workspace, and nothing depends on the difference. This
is `hkt_gat.md`'s step 1 seen from the other side.

**The channel that already works is the context parameter.** `Adjunction<L, R, Ctx>` takes the
context explicitly, and all three uses in the workspace are correct: `StokesContext<T>`,
`(Arc<SimplicialComplex<T>>, usize)`, and multivector's `Metric`. Where the context has somewhere to
live, the operations behave. `Pure` and `Monad` have nowhere, which is why they are where the
failures cluster.

## 9. How it should be done, and why

**Structural witnesses keep `NoConstraint`, and it gets documented as correct.** Populations A and C
are ten of the fifteen. They move elements and need no capability. This is tensor's conclusion, for
tensor's reason.

**Computing witnesses stop being generic.** Population D is generic in parameters that are concrete
at every call site. `curvature` always receives `TensorVector<T>`; `map_field` always receives a
`Field`. Genericity that no caller uses, bought with `unsafe` or with stubs, is a straight loss. The
concrete signature is safe, is enforced by ordinary trait bounds, and is unforgeable, because
`M: Field + Copy + Default` names real traits rather than a marker. `DeRhamSharpIso` in
`extensions/iso_de_rham/` already shows the pattern inside this crate: the algebra bound sits on the
impl, the witness is concrete, and no marker appears anywhere.

**The constraint mechanism is not the fix for population D, and should not be sold as one.** §3 and
§6.1 measured why: a marker gates admission, grants nothing, and can be forged. Reaching for
`Satisfies<SomeConstraint>` here would swap a documented hole for a quieter one.

**Context belongs in the witness type or in an explicit context argument.** Five witnesses already do
this and work: `ManifoldWitness<C>`, `CausalMultiFieldWitness<T>`, `LatticeComplexWitness<D, R>`,
`DeRhamSharpIso<D, R>`, and the `Adjunction` context parameter. Where a context cannot be threaded,
the honest move is linear's: do not implement the operation. A missing `Monad` is a smaller defect
than a `Monad` that silently changes a Minkowski metric to Euclidean.

**If haft is ever to host a computing witness**, the missing piece is a capability rather than a
marker: the constraint would have to imply a usable bound on the element, which Rust cannot express
through an associated marker type and which no amount of GAT work changes. That is a haft design
question, larger than this crate, and topology should not wait on it. Nothing here needs it once
population D is concrete.

## 10. How to get there

**Status as of 2026-08-30: all ten items are done.** Items 1 through 4
landed with the rewrite (`hkt_gat_topology_rewrite.md` §12); 6, 7 and 8 landed after it.

1. ~~**Close the soundness hole.**~~ **Done.** The `RiemannMap` impl and both `unsafe` blocks are
   gone from `hkt_curvature.rs`. The two private `*_impl` functions were folded into the trait
   methods rather than promoted, since with the cast gone they had no other caller. Three
   `compile_fail` doctests hold the closed paths closed.
2. ~~**Narrow the unsafe exemption.**~~ **Done, and better than this asked.** The `Send`/`Sync`
   impls on `LatticeCell<D>` turned out to be unnecessary, because the struct is
   `{ [usize; D], u32 }` and both are derived. No scoped `#[allow]` was needed: the crate carries
   zero `unsafe` and its `[lints]` section is back to `workspace = true`.
3. ~~**Fix the cursor in `ManifoldWitness::bind`.**~~ **Done.** `cursor: m_a.cursor`, with generated
   law tests over every legal cursor in `tests/extensions/hkt_manifold_law_tests.rs`.
4. ~~**Retire the three stubs.**~~ **Done.** `GaugeFieldHKT`, `GaugeFieldData` and the
   `HKT3Unbound`, `MonoidalMerge` and `ParametricMonad` impls are removed, with the eight tests that
   exercised them, five of which asserted the defect as the specification. Every inherent method on
   `GaugeFieldWitness` is untouched; those are what `deep_causality_physics` calls. `MonoidalMerge`
   and `ParametricMonad` keep lawful witnesses in haft's own tests.
5. ~~**Decide population B.**~~ **Done**, by completing rather than removing them. Both field types
   already had the right shape, with the structure parameters (`C`, and `D`/`R`) separate from the
   value parameter, so `Functor` and `Foldable` were available without any change to the types.
   `CellField` and `LatticeField` are exported, both gained a constructor and accessors, and
   `CellField` needed `Debug` and `PartialEq` written by hand because `CellComplex` implements
   neither.

   `Pure`, `Monad` and `CoMonad` are deliberately absent, and the witness docs say why. `pure`
   receives one value and would have to invent a complex around it; any complex it invented would be
   the wrong one, so `bind(m, pure)` would not return `m` and right identity would fail by
   construction. That is the defect measured on `CausalMultiVectorWitness`, and `linear`'s precedent
   is to omit rather than ship it. `CoMonad` is absent because `extract` needs a distinguished cell
   and neither field carries a cursor. Fourteen tests in
   `tests/extensions/hkt_field_witness_tests.rs`, covering both functor laws on each witness.
6. ~~**Correct the `LatticeGaugeFieldWitness` comment.**~~ **Done.** It now states that `M` is the
   mappable parameter and that the blocker is the missing capability rather than the struct bound on
   `R`, and it carries the witness over `M` that compiles on stable today. The roughly twenty lines
   of drafting notes inside `map_field` and `zip_with` are gone.
7. ~~**Record the context findings in `HKT-LAW-FINDINGS.md`.**~~ **Done.** That file now carries the
   multivector metric reset, the tensor shape flattening, the manifold cursor reset and its fix, the
   `fmap` geometry drop, and the generalization that the shared defect is context rather than shape.
   It also records that the decision it left open was taken: `CsrMatrixWitness` moved into `linear`
   without its `Monad` impl.
8. ~~**Document `NoConstraint` on populations A and C.**~~ **Done.** All ten witnesses across eight
   files carry a `# Why NoConstraint` section; the four that drop geometry also carry the caveat
   from item 10.
9. ~~**Drop the GAT where-clause** from `HKT`.~~ **Done 2026-08-30**, workspace-wide: the clause is
   gone from `HKT` and the `*Unbound` family, and the 24 redundant impl sites with it, ending the
   split convention in §8. A constrained `Monad` is now writable, which was the point.
   `bazel test //...` 1231 pass.
10. ~~**Decide what to do about the `fmap` geometry drop.**~~ **Done**, by the second option:
   the geometry was made independent of the payload type. `Chain<T>` became `Chain<R, G>` and
   `Topology<T>` became `Topology<R, G>`, separating the complex's metric precision `R` from the
   coefficient group `G`, which had been sharing one parameter. `fmap` now carries the complex
   across instead of rebuilding it, so the Hodge ⋆ operators survive and the functor identity law
   holds. Measured before and after; three regression tests in
   `tests/extensions/hkt_adjunction_law_tests.rs` hold it. Scope and rationale in
   `chain-topology-parameter-split.md`.

## 11. Against the status quo

| Dimension | Status quo | This recommendation |
|---|---|---|
| `curvature` with a non-`TensorVector` element | compiles, undefined behaviour | rejected by the signature |
| `unsafe` in the crate | 2 HKT casts plus 2 `Send`/`Sync` impls, one crate-wide exemption | `Send`/`Sync` only, scoped `#[allow]`, `forbid` restored |
| `ParametricMonad` laws | left identity broken by construction | impl removed |
| `MonoidalMerge` | ignores the caller's function and averages | impl removed |
| `ManifoldWitness` right identity | holds at cursor 0, fails otherwise | holds |
| Structural `NoConstraint` | correct, described as a limitation | correct, documented as correct |
| Compiler-checked algebra | real, in the inherent methods | unchanged, and no longer bypassed |

The status quo's defect is not missing constraints. It is that a vacuous constraint was accepted as
the price of an abstraction, and three workarounds were built to pay it: one unsound, one
law-violating, one honest. Removing the abstraction where it does not fit costs nothing any caller
uses.

**What this does not deliver.** No new algebraic checking, because the checking that matters already
lives on the inherent methods. Anyone hoping the constraint system would start enforcing algebra here
should read §3 first: it cannot, in this crate or any other, until a constraint can carry a
capability.

## 12. Reproducing

> **Historical, and no longer runnable.** These steps reproduce the defects as they stood before the fix. P1 uses the five-parameter `CurvatureTensor<T, A, B, C, D>` API, which is now `CurvatureTensor<T>`, so it will not compile against this tree.

A probe crate depending on the real crates by path.

```rust
// P1: the unsound path, from safe downstream code
let ct: CurvatureTensor<f64, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>> = CurvatureTensor::flat(4);
<CurvatureTensorWitness<f64> as RiemannMap<CurvatureTensorWitness<f64>>>::curvature(
    ct, vec![1.0, 0.0, 0.0, 0.0], vec![0.0, 1.0, 0.0, 0.0], vec![0.0, 0.0, 1.0, 0.0])
// compiles. Also compiles with a ZST, and with TensorVector<f32> against an f64 witness.

// P4: a witness over M for LatticeGaugeField
impl<G: GaugeGroup, const D: usize, R: RealField> HKT for LgfOverM<G, D, R> {
    type Constraint = NoConstraint;
    type Type<T> = LatticeGaugeField<G, D, T, R> where T: Satisfies<NoConstraint>;
}
// compiles

// P6: monad right identity across the stack
let rt = <W as Monad<W>>::bind(m.clone(), <W as Pure<W>>::pure);
```

```
DenseVector       data [1.0, 2.0, 3.0] -> [1.0, 2.0, 3.0]     HOLDS: true
CausalTensor      shape [2, 2] -> [4]                         HOLDS: false
CausalMultiVector metric Minkowski(4) -> Euclidean(0)         HOLDS: false
Manifold          cursor 0 -> 0                               HOLDS: true
Manifold          cursor 2 -> 0                               HOLDS: false
```

Standalone, no dependencies:

```rust
// P5: Satisfies gates but does not enable
pub fn use_it<A: Satisfies<FieldConstraint>>(a: A, b: A) -> A { a.add(b) }
// error[E0599]: no method named `add` found for type parameter `A`
```

```
cargo build                                    # P1, P4 probe crate, stable
cargo run                                      # P6
rustc --edition 2024 --crate-type lib gate.rs  # P5
```

The P1 probes were compiled and deliberately not executed.
