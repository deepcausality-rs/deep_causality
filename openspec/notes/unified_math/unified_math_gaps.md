<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Unified math: where the HAFT layer stops

**Scope.** The sixteen mathematics crates and their use of `deep_causality_haft`. Which crates carry
categorical witnesses, which do not, what each absent witness costs, and how hard each one is to add.

**Not in scope.** Physics, CFD, discovery, the causality engine. New mathematics. Performance.

**Method.** Trait impls read from `src/` on `main`, dev-dependencies excluded. Two law results in
§3.6 were measured by running code rather than inferred from reading it; the probe is in §6.

---

## 1. What composes today

Cross-crate composition works. `examples/mathematics_examples/composable_multi_math/` carries seven
worked cases, and the mechanism is worth naming precisely, because the gaps are defined against it.

Composition happens two ways:

**Nesting.** A witness whose `Constraint` is `NoConstraint` accepts any element type, including a
type owned by another crate. `tensor_x_algebra_rotation_field/main.rs:40` builds a
`CausalTensor<CausalMultiVector<FloatType>>` and rotates every cell with a single
`CausalTensorWitness::fmap`. The outer container is tensor's; the inner value is multivector's.

**Closure reach.** `triple_hkt_stress_field/main.rs:124` runs a six-step elasticity pipeline inside
one `ManifoldWitness::extend` call. Topology supplies the cursor walk; tensor and multivector are
called from inside the closure.

Both mechanisms share a direction. The container crate holds the witness, and the other crate's type
rides along as an inert payload. `CausalTensor<Dual<T>>` compiles, and the tensor layer maps; the
`Dual` layer cannot, because `Dual` is not a functor.

## 2. Coverage

Four of sixteen crates carry witnesses.

| Crate | Tier | Role | Witnesses |
|---|---|---|---|
| `linear` | 3 | container | `CsrMatrixWitness`, `DenseMatrixWitness`, `DenseVectorWitness` |
| `tensor` | 4 | container | `CausalTensorWitness`, `StrictCausalTensorWitness`, `CausalTensorTrainWitness` |
| `multivector` | 5 | container | `CausalMultiVectorWitness`, `CausalMultiFieldWitness<T>` |
| `topology` | 6 | container | twelve, including `ManifoldWitness<C>`, `ChainWitness`, `GraphWitness` |
| `calculus` | 3 | operator | `Arrow` on `Diff`, `Euler`, `Rk4`; no HKT |
| `num` | 0 | element | none, and none possible: `haft` depends on it |
| `metric` | 0 | element | none; a signature enum has nothing to map |
| `algebra` | 1 | element | none, and none possible: `haft` depends on it |
| `num_complex` | 2 | element | none |
| `num_dual` | 2 | element | none |
| `num_rational` | 2 | element | none, correctly (§3.1) |
| `rand` | 2 | effect | none; hand-rolled `map` (§3.3) |
| `fft` | 3 | operator | none |
| `uncertain` | 3 | effect | none; a private monad (§3.2) |
| `homology` | 4 | fixed-coefficient | none, correctly (§3.7) |

Every witnessed crate is a container crate: it owns a type generic in its element, which is what
`type Type<T>` binds to. Every crate without a witness either has no such type or cannot take `haft`
at all. That is the shape of the whole problem. Composition runs from containers down onto elements
and nowhere else, and four of the crates without a witness do own a mappable element type (§3.1,
§3.2, §3.3).

Trait-by-trait, outside `haft` itself:

| Trait | Implementers |
|---|---|
| `HKT`, `Functor`, `Applicative`, `Monad`, `CoMonad`, `Pure`, `Foldable` | `linear`, `tensor`, `multivector`, `topology` |
| `Arrow` | `calculus`, `tensor` |
| `Adjunction`, `MonoidalMerge`, `ParametricMonad`, `RiemannMap` | `topology` only, one impl each |
| `Traversable` | none |
| `NaturalTransformation` | none |
| `Category`, `Kleisli` | none |
| `Bifunctor`, `Profunctor` | none |

## 3. The gaps

### 3.1 Element crates carry no witness

`num_complex`, `num_dual` and `rand` are the three element crates that could take `haft` and have
not. None of them cycles: `haft` depends on `algebra`, so any crate at tier 2 or above may depend on
`haft` freely.

`Complex<T: RealField>` is `{ re, im }` and `Dual<T: Real>` is `{ re, du }`. Both are two-field
products over a constrained parameter, and `haft`'s `HKT` already supports constrained witnesses:
`StrictCausalTensorWitness` declares `type Constraint = TensorConstraint`
(`deep_causality_tensor/src/extensions/ext_hkt_strict.rs:71`). Neither crate exposes a `map` of any
kind today.

`num_rational` is a correct absence rather than a gap. `Ratio<A> -> Ratio<B>` under an arbitrary `f`
breaks the coprimality invariant, so a componentwise functor would not be lawful.

`num` and `algebra` can never take `haft`, since `haft` depends on both. They are the two permanent
holes in the layer.

**Cost.** A precision migration on a complex-valued tensor is a nested `fmap` on paper and hand-rolled
today. Forward-mode AD under a tensor cannot map its own layer.

### 3.2 `uncertain` has a complete monad that nothing can reach

This is the sharpest finding in the note.

`deep_causality_uncertain/src/types/computation/uncertain_node_content/mod.rs:50` opens with the
comment `// HKT Operations` over four node kinds:

```rust
    // HKT Operations
    PureOp  { value: SampledValue },
    FmapOp  { func: Arc<dyn SampledFmapFn>, operand: ConstTree<UncertainNodeContent> },
    ApplyOp { func: Arc<dyn SampledFmapFn>, arg: ConstTree<UncertainNodeContent> },
    BindOp  { func: Arc<dyn SampledBindFn>, operand: ConstTree<UncertainNodeContent> },
```

`SampledFmapFn` and `SampledBindFn` are public, exported from `lib.rs:34`. The sequential sampler
interprets all four arms (`types/sampler/sequential_sampler.rs:132-145`).

No public builder constructs three of them. `Uncertain::point` builds `Value`, not `PureOp`.
`Uncertain::map` (`types/uncertain/uncertain_f64.rs:41`) builds `FunctionOpF64`, not `FmapOp`. There
is no public `bind` or `and_then` at all. The crate states the consequence itself at
`types/sampler/qmc_sampler.rs:481`:

> No `Uncertain` builder produces a `BindOp` node.

So the probability monad is defined, interpreted, unreachable, and disconnected from `haft`, which
`uncertain` does not depend on. A private re-implementation of the structure `haft` already owns sits
in the tree as dead arms.

**Cost.** `Uncertain` can only ever be a payload. Uncertainty cannot propagate through a linear solve
or a tensor contraction, because there is no witness to compose and no `Traversable` to swap layers.

### 3.3 `rand` has a hand-rolled functor

`Distribution<T>::map` (`traits/distribution.rs:24`) returns `Map<D, F, T, S>`
(`types/map/mod.rs:9`), a functor written by hand with no `Functor` impl behind it and no `and_then`.
The sampling monad is absent, and `uncertain` samples through this crate, so §3.2 and §3.3 are one
problem in two crates.

### 3.4 No `Traversable` in the workspace

`haft` exports `Traversable`; nothing implements it, in any crate.

`sequence : F<G<A>> -> G<F<A>>` is the only operation that swaps two container layers. Without it,
`CausalTensor<Uncertain<f64>>` cannot become `Uncertain<CausalTensor<f64>>`, and
`DenseVector<Result<T>>` cannot become `Result<DenseVector<T>>`. Nesting composes the layers; only
traversal reorders them, and reordering is what an aggregate result needs.

This is the single largest multiplier in the note. It turns the existing four witnesses from a set of
independent containers into a lattice.

### 3.5 No `NaturalTransformation`, `Category` or `Kleisli`

Three exported traits with no implementers anywhere.

The conversions between witnesses exist as concrete functions. `Chain`'s functor already delegates to
`CsrMatrixWitness::fmap` (`topology/src/extensions/hkt_simplicial_complex/mod.rs:35`), which is a
natural transformation written as a call. Typing it as one would put the naturality square under
test.

`Kleisli` exists so that `A -> M<B>` arrows compose. `calculus` implements `Arrow` for `Diff`,
`Euler` and `Rk4`, but those are value-level arrows over plain functions. No Kleisli arrow is built
over any of the four monads, so the monadic and the arrow layers do not meet.

### 3.6 Two shipped monads violate right identity

Measured, not read. Probe source and command in §6.

```
[tensor] CausalTensorWitness
  m            shape=[2, 3] data=[1, 2, 3, 4, 5, 6]
  bind(m,pure) shape=[6]    data=[1, 2, 3, 4, 5, 6]
  right identity holds: false
  rank-3: [2, 2, 2] -> [8]  holds: false

[multivector] CausalMultiVectorWitness
  m            metric=Euclidean(2) data=[1, 2, 3, 4]
  bind(m,pure) metric=Euclidean(0) data=[1, 2, 3, 4]
  right identity holds: false

[linear] DenseVectorWitness
  m=[1, 2, 3]  bind(m,pure)=[1, 2, 3]  holds: true
```

`CausalTensorWitness::pure` builds a rank-0 tensor and `bind` rebuilds as `shape: [len]`
(`tensor/src/extensions/ext_hkt.rs:56-79`), so every rank above one collapses.
`CausalMultiVectorWitness::pure` hardcodes `Metric::Euclidean(0)` and `bind` overwrites the output
metric on each step (`multivector/src/extensions/hkt_multivector/mod.rs:46,110-119`), so the
algebra's signature is discarded. The data survives in both cases; the structure does not.

`linear` already met this problem and answered it. `HKT-LAW-FINDINGS.md`
records the same violation in `CsrMatrixWitness::bind` and argues it is structural: `pure` must pick
a shape, `1 x 1` is the only defensible choice, and no `bind` general enough to accept other shapes
can rebuild an `m x n`. The crate's answer was to stop at `Applicative` and `CoMonad` for the shaped
witnesses and implement `Monad` only for the unshaped `DenseVector`. That answer did not travel:
`tensor`, `multivector` and `topology` all ship `Monad` on shaped containers.

`ManifoldWitness<C>` also implements `Monad` and was not probed here.

### 3.7 `homology` correctly has neither dependency

Recorded because the absence reads as a gap off the dependency graph and is not one. The crate argues
the case in its own module docs at `homology/src/lib.rs:35-63`; the argument is restated here so this
note does not leave it open.

**No `algebra`, because there is no coefficient type to bound.** Everywhere else the tower appears as
a bound on a container's element parameter: `RealField` on 77 sites, `Field` on 24, `Ring` on 7, as in
topology's `Chain<T>` where `T: AbelianGroup` or `SimplicialComplex<T>` where `T: RealField`. No
container in the workspace implements an algebra trait; the only `impl AbelianGroup for` in the
workspace is the tower's own blanket over scalars (`algebra/src/algebra/field_real.rs:40`). In
`homology` the coefficients are fixed. `ChainComplex` carries no type parameter and its
`boundary_matrix` returns `CsrMatrix<i8>`, because incidence numbers lie in `{−1, 0, 1}` by
construction. `HomologyField` names a field at the call site rather than ranging over one. A
dependency on the tower would be an unused one, and `cargo machete`
(`.github/workflows/rust_deps.yml:16`) fails the build on those.

**No `haft`, because `Gf2Chain<W>` is not a functor in `W`.** Every witness in the workspace binds
`type Type<T>` to a container generic in its *element*: `CsrMatrix<T>`, `DenseVector<T>`, `Chain<T>`.
`W` is the storage word, `u8` through `u64`, over `words: Vec<W>`
(`linear/src/types/packed_gf2_vector/mod.rs:46`); the elements are single bits.
`fmap: Gf2Chain<A> -> Gf2Chain<B>` would re-pack into a different word width. It maps no content, so
no functor law about content constrains it. `ChainComplex` has no type parameter to witness either.

`ChainWitness` therefore sits in the right crate already. The coefficient-parametric type is
`Chain<T>`, it holds an `Arc<SimplicialComplex<T>>`, and both are topology's.

Both crates stay reachable through `linear` if that ever changes. A coefficient parameter would bring
the algebra bound with it, and `openspec/changes/extract-homology-crate/design.md` rejects
`ChainComplex<R>` on the merits.

### 3.8 `right_adjunct` panics

`topology/src/extensions/hkt_simplicial_complex/mod.rs:147` ends `Adjunction::right_adjunct` with
`panic!("Adjunction::right_adjunct resulted in empty chain.")`. The trait method returns `B`, so
there is no error channel. An empty chain is reachable input.

## 4. Ranking

### 4.1 Easy

Mechanical work. No design decision, one crate touched, laws follow from the shape of the type.

| # | Item | Gap | Why easy |
|---|---|---|---|
| E1 | `Functor`, `Applicative`, `Foldable` for `Complex`, `Quaternion`, `Octonion` | §3.1 | Fixed-arity products. No shape to lose, so no law to break. Constrained-witness pattern already exists |
| E2 | `Functor` and `CoMonad` for `Dual` | §3.1 | Two fields; `extract = re`. See the caveat below |
| E3 | `right_adjunct` returns `Result` instead of panicking | §3.8 | Signature change plus one call site |
| E4 | Law tests for the four existing witnesses | §3.6 | The probe in §6 is the template. Cheap, and it pins the two known violations before anything is built on them |

Caveat on E2: the lawful `fmap` over `Dual` maps `re` and `du` independently, which is the pair
functor and carries no chain rule. It is worth having for precision migration and for structural
traversal. It is not forward-mode AD, and the docstring should say so.

### 4.2 Moderate

One design decision each, or more than one crate touched.

| # | Item | Gap | The decision |
|---|---|---|---|
| M1 | `Traversable` for `DenseVectorWitness` and the unshaped witnesses | §3.4 | Which applicative to traverse into, and whether shaped containers get `traverse` at all. Shape preservation is the same question as §3.6 |
| M2 | `NaturalTransformation` for the conversions that already exist as functions | §3.5 | Which pairs to declare, and how the naturality square gets tested |
| M3 | `Functor` for `rand`'s `Distribution` | §3.3 | `Distribution<T>` is a trait, not a type constructor. A witness needs a carrier type, so `Map<D, F, T, S>` has to become one |

### 4.3 Hard

Design forks. Each one can be answered several ways and the answers are not equivalent.

| # | Item | Gap | Why hard |
|---|---|---|---|
| H1 | Shaped-container monad laws | §3.6 | `linear` says the violation is structural. Options: withdraw `Monad` from `tensor`, `multivector` and `topology` and keep `Applicative`/`CoMonad`, which is breaking; or define a shape-preserving parametric monad. `topology` already has `ParametricMonad` on `GaugeFieldWitness`, so the second option has a precedent to read |
| H2 | Expose `Uncertain`'s monad and wire it to `haft` | §3.2 | The interpreter is written and the arms are dead, so the coding is small. The fork is sampler capability: `qmc_sampler.rs:328` rejects `BindOp` because quasi-Monte Carlo needs a static stochastic structure, and that rejection is correct. A public `bind` splits the crate into a sequential path that supports it and a QMC path that cannot. Also `BindOp` holds `Arc<dyn SampledBindFn>`, and the repo prefers static dispatch |
| H3 | `Kleisli` composition across witnesses | §3.5 | Depends on H1. A Kleisli category over a monad whose right identity fails is a category whose identity law fails |
| H4 | `fft` as a `NaturalTransformation` | §3.1 | The DFT is the textbook natural transformation between the time and frequency functors. The crate's API is `execute`/`execute_inverse` over scratch buffers, so this is an API redesign rather than an added impl. Lowest value of the four |

## 5. Suggested order

E4 first. Law tests are the cheapest item and they fix the meaning of everything after: H1 cannot be
argued without them, and M1 depends on the answer to H1.

Then E3, which is local and cheap. Then E1 and E2, which widen the element layer and make the first
genuinely new composition possible: a nested `fmap` over `CausalTensor<Complex<T>>` at both layers.

M1 is the multiplier and should wait for H1. H2 is the largest single capability in the note and the
one with a real fork in it; it wants its own change proposal, not a task in this one.

## 6. Reproducing the law probe

A throwaway binary outside the workspace, depending on the three crates by path:

```rust
use deep_causality_haft::{Monad, Pure};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};

let m = CausalTensor::from_vec(vec![1i64, 2, 3, 4, 5, 6], &[2, 3]);
let shape_before = m.shape().to_vec();
let out = <CausalTensorWitness as Monad<CausalTensorWitness>>::bind(m, |a| {
    <CausalTensorWitness as Pure<CausalTensorWitness>>::pure(a)
});
assert_eq!(out.shape(), shape_before.as_slice()); // fails: [6] != [2, 3]
```

The multivector arm is the same shape with `CausalMultiVector::new(vec![1i64, 2, 3, 4],
Metric::Euclidean(2))` and a comparison on `metric()` rather than shape.

These belong in the crates as real law tests, which is item E4.
