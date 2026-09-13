<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Unified math: container types without an HKT witness

**Scope.** The seventeen crates under `deep_causality_unified_math/`, read on `main` at `a1f8e78e9`
on 2026-09-13. Which container types are generic in their element and carry no witness, what a
witness would buy each one, what blocks it, and what is correctly absent.

**Not in scope.** New mathematics. Performance. The physics, CFD, quantum and discovery crates,
which consume this stack.

**Method.** Every `pub struct X<...>` in each crate's `src/` was listed and matched against every
`impl HKT for` and `type Type<T> =` binding. Struct-level bounds were read from the definitions.
Nothing here was inferred from documentation; where the documentation and the code disagree, §7
says so.

**Predecessor.** `openspec/notes/archive/unified_math/unified_math_gaps.md` did this analysis on
an earlier `main`. §1 records what has closed since, and this note supersedes its §3.1 and §4
for the items it covers.

---

## 1. What closed since the archived note

| Archived item | State on `main` |
|---|---|
| E1 `Functor`, `Foldable` and a monoidal applicative for `Complex`, `Quaternion`, `Octonion` | closed; `num_complex` ships three witnesses |
| E2 `Functor` for `Dual` | closed; `DualWitness` binds `Dual<T>` after the struct-level `T: Real` bound was dropped. `CoMonad` stays deferred |
| E3 `right_adjunct` panicked | closed; both partial adjunction operations return `Result` |
| E4 law tests for the four original witnesses | closed; `tensor` now has `test_monad_causal_tensor_{right_identity, left_identity, associativity}` |
| H1 shaped-container monad laws | settled per crate rather than once: `linear` withdrew `Monad` from its shaped witnesses, `multivector` withdrew it, `tensor` kept it and documents the singleton corner where right identity wins and associativity parts company (`ext_hkt.rs`, the `Monad` impl docstring) |
| the `Constraint` slot on `HKT` | removed; `HKT` is `type Type<T>;` and nothing else. `lax_monoidal/mod.rs:64-66` records the removal |

The last row changes the recipe for every gap below. A witness can no longer admit a bounded
element type through a constraint; a struct that carries a bound on its element parameter has to
drop that bound to the impl blocks, which is what `Dual` did.

## 2. Coverage today

Seventeen crates, twenty-five `HKT` impls, all in six crates.

| Crate | Witnesses | Bind |
|---|---|---|
| `linear` | `CsrMatrixWitness`, `DenseMatrixWitness`, `DenseVectorWitness`, `ZipDenseVectorWitness` | the three containers, one zipped reading |
| `tensor` | `CausalTensorWitness`, `ZipTensorWitness`, `CausalTensorTrainWitness` | `CausalTensor<T>` twice, `CausalTensorTrain<T>` |
| `multivector` | `CausalMultiVectorWitness`, `CausalMultiFieldWitness<T>` | `CausalMultiVector<T>`, `CausalMultiField<A, S>` |
| `topology` | twelve: `GraphWitness`, `MixedGraphWitness`, `HypergraphWitness`, `ManifoldWitness<C>`, `GenericManifoldWitness<K>`, `TopologyWitness<R>`, `ChainWitness<R>`, `CellComplexWitness<C>`, `LatticeComplexWitness<D, R>`, `PointCloudWitness<C>`, `ExteriorDerivativeWitness`, `LatticeGaugeFieldWitness<G, D, M>` | `Graph`, `MixedGraph`, `Hypergraph`, `Manifold<K, T>`, `Topology<R, G>`, `Chain<R, G>`, `CellField<C, T>`, `LatticeField<D, R, T>`, `PointCloud<C, T>`, `DifferentialForm<T>`, `LatticeGaugeField<G, D, T, R>` |
| `num_complex` | `ComplexWitness`, `QuaternionWitness`, `OctonionWitness` | the three number types |
| `num_dual` | `DualWitness` | `Dual<T>` |

`num`, `algebra` and `metric` sit below `haft` and cannot take it. `calculus` implements `Arrow`
on its operators and owns no container. `fft`, `stats`, `homology`, `num_rational`, `rand` and
`uncertain` carry no witness; §3 and §4 sort those into gaps and correct absences.

## 3. Container types without a witness

Four types are generic in their element, hold a collection of it, and have no witness. They are
ranked by what a witness buys, and the first is worth more than the other three together.

### 3.1 `Uncertain<T>` and `MaybeUncertain<T>`

`deep_causality_uncertain`, tier 5. Still the sharpest finding, and unchanged since the archived
note's §3.2.

The monad is written and interpreted. `types/computation/uncertain_node_content/mod.rs:51-62`
carries `PureOp`, `FmapOp` and `BindOp` arms; the sequential sampler evaluates all three; the
quasi-Monte Carlo sampler evaluates the first two and rejects `BindOp` at `qmc_sampler.rs:170-173`
with "QMC requires a static stochastic structure". No public builder produces a `BindOp`. The only
`map` is a method on `Uncertain<f64>` at `types/uncertain/uncertain_f64.rs:48`, and there is no
`bind` or `and_then` on any instantiation. The crate does not depend on `haft`.

**What a witness buys.** Today an `Uncertain` value can only be a payload: `CausalTensor<Uncertain<T>>`
compiles and the tensor layer maps, and the uncertainty rides inert. With a witness and a
`Traversable` on the tensor, `CausalTensor<Uncertain<T>>` becomes `Uncertain<CausalTensor<T>>`, and
a linear solve or a contraction runs under the distribution instead of beside it. That is the
largest single capability the stack does not have.

**What blocks it.** Two things, one mechanical and one a design fork.

- The struct is `Uncertain<T: ProbabilisticType>` (`types/uncertain/mod.rs:32`). A witness binds
  `type Type<T> = Uncertain<T>` for every `T`, so the bound has to move from the struct to the
  impl blocks. `Dual` made the same move. `ProbabilisticType` is implemented for `bool`, `f64`,
  `Float106` and `SampledValue`, so the impls stay narrow.
- The sampler fork the archived note recorded still stands. A public `bind` splits the crate into a
  sequential path that supports it and a QMC path that correctly refuses it, and `BindOp` holds an
  `Arc<dyn SampledBindFn>` where the workspace prefers static dispatch.

**Cost.** Larger than this section first assumed. `hkt_uncertain.md` reads the `haft` signatures
against the lazy graph and finds that `fmap`, `pure`, `apply` and `bind` withhold the `'static`,
`Send`, `Sync` and `Clone` bounds a stored closure needs, so the lazy graph cannot take the
witness at all. The resolution there is a strict `Particles<T>` carrier that takes the witness,
with the graph materialising into it, and precision as a parameter on the graph itself. That
note carries the staging.

### 3.2 `CausalTensorTrainOperator<T>`

`deep_causality_tensor`, tier 5. `types/causal_tensor_network/causal_tensor_train_operator/mod.rs:30`.

The operator is `cores: Vec<CausalTensor<T>>` plus two dimension vectors. Its sibling
`CausalTensorTrain<T>` has the same shape and carries `CausalTensorTrainWitness` with `Functor`,
`Foldable` and `Pure`. The operator has no witness and no `map`, `cast` or `convert` method.

**What a witness buys.** Precision migration of a matrix-product operator by one `fmap`, so an
`f64` operator becomes a `Float106` reference operator without a second constructor, and the
train and operator read uniformly at a call site that already maps the train.

**What blocks it.** The struct bound `T: ConjugateScalar`. Drop it to the impls, as `Dual` did.

**Cost.** An afternoon. `fmap` delegates core by core to `CausalTensorWitness::fmap`; `fold`
folds across cores. Stop at `Functor` and `Foldable`. `Pure` has no defensible rank structure to
pick for an operator, and the train's `Pure` is already the weakest claim in that file.

### 3.3 `Cochain<R>`

`deep_causality_topology`, tier 7. `types/cochain/mod.rs:31`.

`values: Vec<R>` and a degree, no bound on the struct. Its two siblings are witnessed:
`ChainWitness<R>` binds `Chain<R, G>` and `ExteriorDerivativeWitness` binds `DifferentialForm<T>`.
`Cochain` is live, not superseded: the cup product (`types/cup_product/mod.rs`,
`types/topology/ops/cup_product.rs`), the cut-cell registry (`types/cut_cell/mod.rs`) and the
CFD graded-MMS verification all construct it.

**What a witness buys.** Uniformity among three siblings, and a coefficient lift on the cochain
side of the Stokes pairing to match the one the chain side already has.

**What blocks it.** Nothing structural.

**Cost.** Under an afternoon. `Functor` maps `values`, `Foldable` folds them, the degree is
carried through. Whether `Pure` is claimed follows whatever `ChainWitness` claims.

### 3.4 `rand`: `Map<D, F, T, S>` and `Distribution`

`deep_causality_rand`, tier 2. `types/map/mod.rs:9`, `traits/distribution.rs:24`.

A hand-rolled functor, `Distribution::map` returning `Map<D, F, T, S>`, with no `Functor` impl
behind it and no `and_then`. Unchanged since the archived note's §3.3 and its M3.

**What a witness buys.** The sampling monad. Its value is tied to §3.1, because `uncertain`
samples through this crate; on its own it changes no call site in the workspace.

**What blocks it.** `Distribution<T>` is a trait, not a type constructor, so a witness needs a
carrier type; `Map` would have to become one. `hkt_uncertain.md` resolves this row as no witness
in `rand`: the carrier worth having is the particle type in `uncertain`, and `rand`'s work is the
generic cleanup of its six per-type files, which breaks no call site.

### 3.5 Marginal: `HilbertState<R>` and `HopfState<R>`

`deep_causality_multivector`. Each wraps a `CausalMultiVector<Complex<R>>` under a fixed metric.
A lawful `fmap` over `R` is precision migration and nothing more, the same caveat E2 recorded
for `Dual`, and both carry a struct bound `R: RealField`. A `lift` method that delegates to the
inner witness serves the one use without claiming a functor for a type whose invariant is the
metric. Not a gap.

## 4. Correctly absent

Recorded so the list above does not read as incomplete.

| Type | Crate | Why no witness |
|---|---|---|
| `PackedGf2<W>`, `PackedGf2Vector<W>` | `linear` | generic in the storage word, not the element; `fmap` would repack widths and map no content |
| `Lu<T>` | `linear` | a factorisation; mapping its entries breaks the invariant, the way mapping a `Ratio` breaks coprimality |
| `FftPlan<R>`, `RfftPlan<R>`, `FftPlanNd<R>`, `RfftPlanNd<R>`, `DctPlan<R>` | `fft` | operators over scratch buffers; the DFT as a natural transformation is archived item H4 and remains the lowest-value item |
| `Diff<A, R>`, `Euler<S, R, F>`, `Rk4<S, R, F>` | `calculus` | arrows, already `Arrow` |
| `EntropyConfig<T>`, `RidgeFit<T>`, `LogisticFit<T>`, `MeanAccumulator<T>` and the other configs | `stats` | records and fold states, not containers |
| `Rational<T>` | `num_rational` | `Ratio<A> -> Ratio<B>` under an arbitrary `f` breaks coprimality |
| `ChainComplex` and `Gf2Chain<W>` | `homology` | no coefficient parameter; the archived note's §3.7 argues it and nothing has changed |
| `CutCell<D, R>`, `CutCellRegistry<D, R>`, `HodgeDecomposition<R>`, `LerayProjection<R>`, `DecStencilTables<R>` | `topology` | geometry and operator records parameterised by the scalar, not collections of it |

## 5. Trait-level gaps

These are not container gaps, and they set how much any container witness is worth.

| Trait | Implementers on `main` | Note |
|---|---|---|
| `Traversable` | `OptionWitness`, `ResultWitness<E>` in `haft` only | archived M1. It was gated on H1, and H1 is now settled per crate, so `Traversable` for `DenseVectorWitness` and `CausalTensorWitness` is unblocked. It is the multiplier: without `sequence`, nesting composes layers and nothing reorders them, and §3.1 pays off only with it |
| `NaturalTransformation` | the `OptionToVec` fixture in `haft` only | archived M2. `Chain`'s functor still delegates to `CsrMatrixWitness::fmap` as a call rather than a typed transformation |
| `Kleisli` | none | archived H3, and it depended on H1; `linear`'s `DenseVector` and `tensor` both hold a `Monad` now |

## 6. Ranking

| # | Item | Class | Depends on |
|---|---|---|---|
| 1 | `Traversable` for `DenseVectorWitness` and `CausalTensorWitness` | moderate | nothing; H1 is settled |
| 2 | `Uncertain` witness and public `bind`, split by sampler | hard | its own change proposal; item 1 for the payoff |
| 3 | `CausalTensorTrainOperator` witness | easy | dropping one struct bound |
| 4 | `Cochain` witness | easy | nothing |
| 5 | `rand` carrier type and `Functor` | moderate | item 2 |
| 6 | `NaturalTransformation` for the existing conversions | moderate | nothing |

Items 3 and 4 fit one change together. Item 1 should precede item 2, because a witness on
`Uncertain` without a `Traversable` on the containers around it leaves the uncertainty a payload,
which is where it is today.

## 7. Side finding: the documentation describes a `Constraint` slot that no longer exists

`website/docs/src/content/docs/concepts/uniform-math.md:194-205` and
`website/docs/src/content/docs/concepts/hkt.md:16-19` show `HKT` with
`type Constraint: ?Sized` and `T: Satisfies<Self::Constraint>`, and say most math containers use
`NoConstraint`. On `main`, `haft/src/hkt/mod.rs:119-123` defines `HKT` as `type Type<T>;` alone,
and `haft/src/lax_monoidal/mod.rs:64-66` records that `Satisfies` and the associated `Constraint`
are gone. Two comments inside `haft` (`monad/comonad.rs:20`, `monad/mod.rs:27`) still mention the
slot as well. The project website pages and those two comments need the same one-line correction:
a witness admits every element type, and a container that needs a bound puts it on its impls.

## 8. Reproducing

From the workspace root:

```bash
# every generic public struct per crate
for c in deep_causality_unified_math/deep_causality_*; do
  echo "== $c"; grep -rhoE "^pub struct [A-Za-z0-9_]+<[^>]*>" "$c/src" --include=*.rs | sort -u
done

# every witness binding
grep -rn "type Type<" deep_causality_unified_math --include=*.rs | grep -v /haft/

# trait-level implementers outside haft
grep -rn "impl.*Traversable<.*for\|impl.*NaturalTransformation<.*for\|impl.*Kleisli<.*for" \
  --include=*.rs deep_causality_unified_math | grep -v /haft/

# the dead monad arms and the missing builder
grep -rn "BindOp" deep_causality_unified_math/deep_causality_uncertain/src --include=*.rs
```
