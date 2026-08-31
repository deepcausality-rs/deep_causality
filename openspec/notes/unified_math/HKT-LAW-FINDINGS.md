<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `CsrMatrixWitness` violates monad right identity

Found by writing the law tests `linear-hkt-composition` requires, before implementing the new
witnesses. Recorded rather than reproduced.

## The defect

`deep_causality_sparse::CsrMatrixWitness` implements `Monad`. Its `bind` flattens the result to a
`1 x count` matrix (`src/extensions/ext_hkt.rs`), so `bind(m, pure)` does not return `m`:

```
  original shape      = (2, 2)
  bind(m, pure) shape = (1, 4)
  monad right identity holds: false
```

Probed against the crate as published, not inferred from reading it.

## Where it holds and where it does not

Re-measured during phase 5, when the question of porting the impl into `deep_causality_linear` came
up and the claim above had to be checked rather than repeated. `bind` collects every stored value and
rebuilds as `shape: (1, count)` with `col_indices: (0..count)`, so both the shape and the column
positions are discarded:

| input | `m` | `bind(m, pure)` | law |
|---|---|---|---|
| 2×2 diagonal | shape (2,2), 2 stored | shape (1,2) | **violated** |
| 3×3 sparse | shape (3,3), 2 stored | shape (1,2) | **violated** |
| 1×3 full row | shape (1,3), cols [0,1,2] | identical | holds |
| 1×1 | identical | identical | holds |
| 1×3 with a gap | cols **[0,2]** | cols **[0,1]** | **violated** |

Right identity holds exactly when `m` is already a fully-dense `1 × n` row — the shape `bind`
happens to produce. The 2×2 in the original probe was dense, which is why its count was 4.

The last row is the sharpest case and was not in the original probe. The value count is unchanged and
the shape stays `1 × 3`, but the entry at column 2 **moves to column 1**. This is not a shape
annotation being lost: a caller who binds a sparse row gets a matrix whose non-zeros are in different
places, and nothing reports it.

## Why it is not a matter of taste

Monad right identity is `bind(m, pure) == m`. It is one of the three laws the trait exists to
promise, and `linear-hkt-composition` states the consequence plainly: an HKT impl that does not
satisfy its laws is worse than no impl, because it composes and produces wrong answers only when a
caller relies on the law.

## Why it is hard

`pure` must build a container holding one value, and a **shaped** container has no canonical shape to
choose. Take `pure(a)` to be the `1 x 1` — the only defensible choice — and right identity then
requires `bind` to reassemble an `m x n` matrix out of `m*n` one-by-ones. A `bind` general enough to
accept an `f` returning other shapes cannot also do that.

This is a property of shaped containers, not of sparsity. `DenseMatrix` has it too.

A **vector** does not. Its only shape is its length, so `bind` is list concatenation and all three
laws hold. `DenseVectorWitness` therefore claims `Monad` and satisfies it; the test suite exercises
left identity, right identity and associativity for it.

## What this change does

`DenseMatrixWitness` implements `HKT`, `Functor`, `Foldable`, `Pure`, `Applicative` and `CoMonad`,
and **not** `Monad`, with the reason at the impl site. `linear-hkt-composition` allows exactly this:
a witness "implements the same trait set, or documents at that impl site which trait it cannot
support and why."

`CoMonad` is implemented with the shifted-view focus that `CsrMatrixWitness` already uses correctly —
for each position, rotate it to the front and apply `f` there — which is what makes
`extend(extract) == id` hold. A first attempt applied `f` to the whole container at every position;
the law test caught it.

## Decision owed at task 4.11

`CsrMatrixWitness` moves into `deep_causality_linear` with the rest of the sparse crate. Three
options, none of them "carry it across unexamined":

1. **Drop the `Monad` impl**, matching `DenseMatrixWitness`. Honest, and breaking for any caller
   that binds a `CsrMatrix` — a search for such callers is owed before choosing this.
2. **Keep it and document the violated law** at the impl site. Preserves the surface and leaves a
   false promise in the type system.
3. **Reshape `bind`** so the law holds for the `1 x 1` case and document what it does otherwise.

The move is when this gets decided, and the decision belongs with whoever owns the sparse surface.

---

# Further findings, 2026-08-30

The decision owed above was taken: `CsrMatrixWitness` moved into `deep_causality_linear` **without**
its `Monad` impl, option 1. `deep_causality_linear` now implements `Monad` for `DenseVector` only,
the one container it owns with no context to fabricate, and that is the precedent the findings below
should be read against.

Measured while assessing the topology HKT layer, against the crates as they stand. Method and probes
in `openspec/notes/archive/hkt_gat/hkt_gat_topology.md` §8 and §12.

## The generalization: it is context, not shape

The `CsrMatrixWitness` finding above is one instance of a wider pattern, and naming the pattern
predicts the rest. **These containers pair a payload with a context**: a sparsity pattern, a shape, a
metric, a complex, a cursor. `Pure::pure` receives one value and must produce a container, so it has
to invent the context from nothing. `Monad::bind` receives a context from the input and another from
every `f(a)`, and has to pick. `haft`'s `Functor`, `Pure` and `Monad` carry no channel for any of it.

Where the context has somewhere to live, the operations behave. `Adjunction<L, R, Ctx>` takes the
context explicitly, and all three uses in the workspace are correct. `Pure` and `Monad` have nowhere,
which is where the failures cluster.

Monad right identity, `bind(m, pure) == m`, measured across the stack:

| Witness | Crate | Context | `Monad` | Right identity |
|---|---|---|---|---|
| `DenseVectorWitness` | linear | none | yes | **holds** |
| `CsrMatrixWitness` | linear | sparsity pattern | not implemented | n/a |
| `DenseMatrixWitness` | linear | shape | not implemented | n/a |
| `CausalTensorWitness` | tensor | shape | yes | **fails** |
| `CausalMultiVectorWitness` | multivector | metric | yes | **fails** |
| `ManifoldWitness` | topology | complex, metric, cursor | yes | **fixed 2026-08-30** |

## `CausalMultiVectorWitness` lost the metric (fixed 2026-08-30 by removing `Monad`)

`Pure::pure` has no metric to work from and fabricates `Metric::Euclidean(0)`. `bind` then overwrites
its accumulator with the metric of the **last** `f(a)` it evaluated, so the input's metric does not
survive:

```
m.metric            = Minkowski(4)
bind(m, pure).metric = Euclidean(0)

data preserved      : true
metric preserved    : false
```

The payload is intact; the geometry is not. A caller who binds a multivector in `Cl(3,1)` gets one
whose metric says `Cl(0,0)`, and nothing reports it. This is the same class as the `CsrMatrix`
column shift above: the values look right and the context silently changed.

`Euclidean(0)` is not an identity element for `Metric`, because `Metric` is not a monoid. That is the
difference between this case and a lawful one: `deep_causality_cfd`'s `StudyEffectWitness` carries a
`StudyWarningLog`, `pure` produces the **empty** log, `bind` **merges** rather than overwrites, and
right identity holds. A context that forms a monoid can be threaded; one that does not has to be
taken from the input, or the operation should not be implemented.

**It was worse than a lost metric.** A multivector holds exactly `2^dim` coefficients, so the metric
fixes the length, and `bind` bypassed `CausalMultiVector::new` by building the struct directly. The
result held 16 coefficients under `Euclidean(0)`, whose algebra admits one:

```
is bind(m, pure) a value `CausalMultiVector::new` would accept? false
  constructor says: Data length mismatch: expected 1, found 16
```

**No metric choice fixes it, which is why the impl is gone rather than corrected.** Measured: left
identity holds only when `bind` takes the metric from `f`'s result, and right identity only when it
takes it from the input. `pure` carries `Euclidean(0)` and has nothing to reconcile them with, so
the two laws are in direct conflict.

`Pure` survives, and that is deliberate. `pure(x)` names `Cl(0)`, the one algebra reachable without
inventing geometry, and its single coefficient is exactly `2^0`, so the value is well formed. The
applicative identity law `apply(pure(id), v) == v` holds, because `apply` broadcasts a lone function
and takes the metric from its argument. Eight law tests in
`tests/extensions/hkt_multivector/hkt_law_tests.rs` sweep every metric from `Cl(0)` to `Minkowski(4)`.
The dimension-changing operation the old `bind` was standing in for is a tensor product, which the
example now writes directly and names as such.

## `CausalTensorWitness` flattens the shape

```
m.shape            = [2, 2]
bind(m, pure).shape = [4]
```

The same defect the `CsrMatrixWitness` analysis above predicts for any shaped container, and for the
reason given there: `pure` must choose a shape, and no choice lets `bind` reassemble the original.

## `ManifoldWitness` reset the focus (fixed)

`bind` took the complex and the metric from the input, which is the correct discipline, and then
hardcoded `cursor: 0`:

```
cursor 0, shape [3]    bind(m,pure)==m : true
cursor 2, shape [3]    bind(m,pure)==m : false
```

Six `extend` implementations in the same crate carry a comment explaining that resetting the focus to
`0` breaks the comonad laws for a non-zero focus. `bind` broke the monad law for exactly that reason
and did not have the comment. Fixed 2026-08-30 (`cursor: m_a.cursor`), with generated law tests over
every legal cursor in `tests/extensions/hkt_manifold_law_tests.rs`.

## `fmap` dropped complex geometry (fixed 2026-08-30)

`TopologyWitness::fmap`, `ChainWitness::fmap` and the Stokes `unit` and `left_adjunct` rebuild
`SimplicialComplex::<B> { skeletons, boundary_operators, coboundary_operators, ..Default::default() }`.
The Hodge ⋆ operators are typed by the payload, so they cannot carry from `A` to `B` and are dropped.
Measured on a complex built with real coordinates:

```
source complex hodge stars: true
Topology  before fmap: hodge ok = true
Topology  after  fmap: hodge ok = false
Chain     before fmap: hodge ok = true
Chain     after  fmap: hodge ok = false
```

So `fmap(id, x)` is not `x` whenever `x` carries geometry, and the **functor identity law fails**.
The behaviour is deliberate and commented at each site; the law consequence was recorded nowhere
until now.

**Fixed by making the geometry independent of the payload type.** The dropped data was typed by the
parameter being mapped only because one parameter was doing two jobs: in `Chain<T>` and
`Topology<T>`, `T` was simultaneously the complex's metric precision and the coefficient group.
Mathematically those are independent — `C_k(K; G)` is a functor in `G` with `K` fixed, and the Hodge
⋆ is determined by the metric on `K` — and `SimplicialComplex<T>` confirmed it, using `T` in exactly
two fields, both geometric, with the combinatorics at `CsrMatrix<i8>`.

Splitting them into `Chain<R, G>` and `Topology<R, G>` lets `fmap` carry the complex across instead
of rebuilding it:

```
Topology  before fmap: hodge ok = true → after: true
Chain     before fmap: hodge ok = true → after: true
Chain     fmap(id, c) == c : true
```

The workspace already had the correct design in `Manifold<SimplicialComplex<R>, F>`, which passed
this test throughout. Three regression tests in `tests/extensions/hkt_adjunction_law_tests.rs` guard
it, including the type-level one: mapping `f64` coefficients to `i32` now yields `Chain<f64, i32>`,
so the complex keeps its precision. Scope in
`openspec/notes/archive/hkt_gat/chain-topology-parameter-split.md`.
