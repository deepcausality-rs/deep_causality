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

## Why it is hard rather than careless

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
