<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# 6: the diagonal traversal, and a defect the compiler refuses to let you write

## Phases 1, 2 and 4

`DiagonalTraversable` declared in `haft` beside `Traversable`, implemented for
`CausalTensorWitness` and `DenseVectorWitness` with `unimplemented!()` bodies. Six tests observed
failing **6 of 6 at the phase-1 body**, four more written for the vector witness. Implemented:
**6 and 4 passing**.

## What the traversal is

`Traversable::sequence` flips `F<M<A>>` into `M<F<A>>` and is bounded on `M: Applicative`. An
applicative that also carries `Monad` owes `apply(ff, fa) == bind(ff, |f| fmap(fa, f))`, and `bind`
runs its continuation once per function — which forces the **cartesian** reading. A 2×2 structure
whose four cells each hold 50 values comes back with `50^4 = 6 250 000` entries.

For parallel runs that is the wrong operation. Value *i* of one run belongs with value *i* of the
others. That pairing is `Semigroupal::zip_with`, and `sequence_zip` is `sequence` driven by it.

The seed parameter is where the absence of a unit surfaces. A positional zip's unit is the infinite
repeat, which no finite carrier can hold, so the zip witnesses implement `Semigroupal` and stop —
and with no `Pure` there is nothing to build a starting accumulator from. The caller supplies it,
and for an ensemble that is correct: how many draws you want is a decision, not an inference.

## Phase 3 — the audit

| # | Defect | Result |
|---|---|---|
| (a) | the cartesian applicative in place of the diagonal | **cannot be written** |
| (b) | pairing position *i* with position *i+1*, by `skip` | **caught** — 5 tests |
| (b′) | the same, by rotation, so the count is unchanged | **caught** — 4 tests, all by value |
| (c) | truncating the structure's cells instead of the ensemble | **caught** — 4 tests |

### (a) is refused by the type system, not by a test

Substituting the cartesian traversal for the diagonal does not compile, and the error is the one
the design note predicted:

```text
error[E0277]: the trait bound `ZipTensorWitness: Applicative<ZipTensorWitness>` is not satisfied
```

It fails in the other direction too, for the mirror reason: `CausalTensorWitness` has the cartesian
`apply` but no `Semigroupal`, which is `sequence_zip`'s bound. Each traversal accepts exactly the
witness whose pairing it means and refuses the other.

So the audit for (a) is not a failing test. What is kept instead is
`the_cartesian_traversal_is_a_different_operation`, which measures what the cartesian one does:
`3^4 = 81` against the diagonal's 3, the same ratio that gives 6 250 000 against 50 at the size
sampling uses.

### (b′) is why the assertions are on values

The task list called for the exact-value assertion and said why: a count cannot separate *paired by
index* from *paired by index, off by one*. That is easy to assert and worth measuring, so both
forms were run.

`skip(1)` shortens the result, so it fails the count test as well — it does not demonstrate the
point. **Rotating** does: every field is present, the ensemble is exactly as long as it should be,
and only the values are wrong.

| | count test | value tests |
|---|---|---|
| (b) `skip(1)` | fails | fail |
| (b′) rotate | **passes** | fail |

A suite written on counts would have shipped (b′).

## Cost

One `zip_with` per cell, each linear in the ensemble length: `O(cells × draws)`, with no clone of
the accumulator.

Worth contrasting with the traversal beside it. `CausalTensorWitness::sequence` clones its
accumulator once per step — `n(n-1)/2` element clones, quadratic — and its own documentation
explains why: `Applicative::apply` takes `Func: FnMut`, which may be invoked repeatedly, so the
closure cannot move a captured accumulator out. `Semigroupal::zip_with` takes `FnMut(A, B) -> C`
and consumes both sides once, so nothing needs cloning. The diagonal traversal is the cheaper
operation as well as the correct one here.

## Phase 5 — mutation testing reaches nothing here, and the reason is the signature

```
63 mutants tested: 7 caught, 56 unviable, 0 missed
```

The headline is misleading and worth stating plainly: **`sequence_zip` was not tested at all.**
Thirteen mutants were generated for it and all thirteen were unviable. The seven caught are in the
`bind`, `apply` and `zip_with` around it.

Every one of the thirteen is the same move — replace the body with a constructed value:

```text
replace ...::sequence_zip -> M::Type<CausalTensor<A>> with Type::new()
replace ...::sequence_zip -> M::Type<CausalTensor<A>> with Type::from(CausalTensor::new())
replace ...::sequence_zip -> M::Type<CausalTensor<A>> with Type::new(Default::default())
...
```

`M::Type<CausalTensor<A>>` is an opaque generic associated type. There is no expression that
constructs one — that is what makes the traversal generic over the inner witness — so each
substitution fails to compile and is discarded. Nor are there operator mutants to fall back on: the
body is a fold of `zip_with` calls and a `push`, with no comparison or arithmetic to flip.

So for this function the mutation score carries no information, and a run reporting **0 missed** is
not evidence of anything. **The scripted audit in phase 3 is the phase-5 evidence**, and it was
built to be: defect (b′) exists precisely because it is the mutation a careless implementation would
actually contain, and it survives every assertion except the ones on values.

This generalises past this function. A traversal's signature is what makes it generic, and the same
property that lets it accept any witness denies the mutation engine anything to put in its place.
Where that holds, the deliberate-defect audit is not a supplement to mutation testing; it is the
whole of the verification, and writing it carelessly leaves the function unverified.
