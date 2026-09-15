<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Unified math: witnesses that exist, and the traits they could still carry

**Scope.** The seventeen crates under `deep_causality_unified_math/`, read on `main` at
`cbf3af23b` on 2026-09-15. Every witness type in the folder, which HKT trait each carries, which
traits a witness could take today, and which crates could gain a categorical layer at all.

**Not in scope.** New mathematics. Performance. The consumers above this stack.

**Method.** Mechanical. Every `pub struct X*Witness` and every `impl Trait for X*Witness` was
extracted from each crate's `src/`, with comments stripped first and multi-line `impl` headers
joined — a naive line regex both invents witnesses from doc comments and misses every wrapped
header, which is how an earlier hand count reached the wrong number twice. §5 reproduces it.

**Relationship to the sibling note.** `hkt_gaps.md` asks which *container types* lack a witness.
This note asks the complementary question: given the witnesses that exist, which *traits* are
missing from them. The two overlap in one place, §3 row 5, where `hkt_gaps.md` §3.4 already
decided the matter and this note defers to it.

---

## 1. Coverage today

**40 witness types across 7 of the 17 crates. Every one implements at least one HKT trait.**
23 distinct traits appear over them.

| Crate | Tier | Witnesses | Distinct traits |
|---|---|---|---|
| `haft` | 2 | 13 | 17 |
| `topology` | 7 | 14 | 8 |
| `linear` | 3 | 4 | 13 |
| `num_complex` | 3 | 3 | 7 |
| `tensor` | 5 | 3 | 13 |
| `multivector` | 6 | 2 | 6 |
| `num_dual` | 3 | 1 | 7 |

Ten crates declare no witness: `num`, `metric`, `algebra`, `num_rational`, `rand`, `calculus`,
`fft`, `homology`, `stats`, `uncertain`.

Two types carried the `*Witness` suffix while implementing no HKT trait at all —
`GaugeFieldWitness` and `LatticeGaugeFieldWitness`, both zero-sized namespaces holding inherent
methods. Renamed to `GaugeFieldOps` and `LatticeGaugeFieldOps` on 2026-09-15; the suffix now means
something everywhere it appears.

## 2. Actionable gaps

Ranked by value over cost. Rows 1–4 are delegation onto structure that already exists.

| # | Target | Add | Prereqs met? | Verify with | Unlocks | Cost |
|---|---|---|---|---|---|---|
| 1 | ~~`GraphWitness`, `MixedGraphWitness`, `HypergraphWitness`, `PointCloudWitness`, `TopologyWitness` (`topology`) | `Foldable` | yes — all five already hold a `CausalTensor<T>` payload; four sibling witnesses in the same crate implement it | `grep -rn "impl.*Foldable<" topology/src/extensions` returns Cell/Chain/Cochain/LatticeComplex/Manifold only — none of the five | Reductions over graph payloads inside the witness vocabulary instead of reaching past it to `.data().as_slice()`. Prerequisite for row 2~~ | **closed 2026-09-15** |
| 2 | ~~`ManifoldWitness` (`topology`)~~ | ~~`Traversable`~~; `DiagonalTraversable` **not feasible** | yes — `Traversable<F>: Functor<F> + Foldable<F>`, both implemented at `hkt_manifold/mod.rs:75` | inventory row: has HKT/Functor/Pure/Applicative/Monad/CoMonad/Foldable, lacks only Traversable; `tensor` and `linear` both have it | `Manifold<C, Result<T,E>>` collapses to `Result<Manifold<C,T>, E>` — one failing vertex invalidates the field, which a DEC pipeline hand-rolls today~~ | **closed 2026-09-15**; diagonal half blocked, see below |
| 3 | ~~`ZipTensorWitness` (`tensor`), `ZipDenseVectorWitness` (`linear`)~~ | ~~`Foldable`~~ | — | — | — | **closed 2026-09-15**, both halves |
| 4 | ~~`CausalMultiVectorWitness` (`multivector`)~~ | ~~`Traversable`~~ | — | — | — | **closed 2026-09-15**; see below |
| 5 | `stats` (13 distribution types), `rand` (`Map`, `Uniform`, `StandardWord`, `StandardBool`) | `Arrow` | n/a — needs a new `→ haft` edge in each crate | `grep -rn deep_causality_haft stats/Cargo.toml rand/Cargo.toml` returns nothing today | `Map`/`Iter` become `Compose`/`arr`; sampler pipelines compose with `first`/`split`/`fanout`; inherits the `haft.arrow.*` Lean proofs | **consumer-gated, see below** |

**Row 5 is closed as not feasible, 2026-09-15.** The direct impls cannot be written.
`Arrow::run(&self, input: Self::In) -> Self::Out` fixes `In` as one associated type at impl time,
while `Distribution::sample<R: Rng>(&self, rng: &mut R)` needs a fresh mutable borrow of a
*generic* generator on every call. Writing it gives `error[E0207]` twice on the same line — the
type parameter `R` and the lifetime `'a` both appear only inside `type In = &'a mut R`, and an
associated type does not constrain an impl parameter. `uncertain` clears the same wall only because
its draws are pure functions of an address, so `In = SampleIndex` and there is no generator to
borrow; `rand`'s generators are stateful and have no address.

A wrapper carrying the lifetime and the generics does work — `Sampler<'a, D, R, T>(&'a D, …)` with
`R: Rng + 'a`, measured to run twice, advance the generator and compose with `Lift`. It was not
taken: the four named types still would not implement `Arrow`, `In = &'a mut R` is not `Clone` so
`fanout` and the rest of the duplicating combinators stay unavailable, and the tier move buys an
instance no caller wants. Making `rand`'s draws addressed, as `uncertain`'s are, is the change that
would earn the layer; it is a sampling-model change rather than a trait-surface one.

**The rest of row 5 as originally written.** `hkt_gaps.md` §3.4 reached the same conclusion — the container
traits are for data, a lazy sampler is a program, and `haft`'s home for programs is `Arrow` — and
recorded that such a change *"on its own changes no call site in the workspace."* It is the trap
§6 of that note flags for `NaturalTransformation`: the dependency is **a consumer**, not code. It
is listed here because the count is large and someone will rediscover it otherwise, not because it
should be built. A blanket `impl<D: Distribution<T>> Arrow for D` would collapse it to one impl per
crate, but is unavailable: `Arrow` is foreign to both crates and the self type would be an
uncovered parameter, so it is one impl per concrete type.

## 3. Crates ranked by HKT traits gained

Sorted by the count of new trait implementations each crate would receive.

| Rank | Crate | New impls | What | Today | Cost |
|---|---|---|---|---|---|
| 1 | `stats` | 13 | `Arrow` on `Bernoulli`, `Categorical`, `Cauchy`, `Exponential`, `LogNormal`, `Normal`, `StandardNormal`, `Poisson`, `UniformInt`, `StandardUniform`, `Weibull`, `Open01`, `OpenClosed01` | 0 witnesses, 0 HKT traits, no `haft` edge | new `stats → haft` edge; no tier move (already tier 4). **No consumer** |
| 2 | `topology` | 7 | `Foldable` × 5 + `Traversable` + `DiagonalTraversable` | 14 witnesses, 8 traits | none — all prereqs met |
| 3 | `rand` | 4 | `Arrow` on `Map`, `Uniform`, `StandardWord`, `StandardBool` | 0 witnesses, 0 HKT traits, no `haft` edge | new `rand → haft` edge; **tier 2 → 3**. Verified: nothing downstream moves, `stats` is already tier 4 via `linear`. **No consumer** |
| — | ~~`tensor`~~ | ~~1~~ | ~~`Foldable` on `ZipTensorWitness`~~ | **closed 2026-09-15**: 3 witnesses, 14 traits | — |
| — | ~~`linear`~~ | ~~1~~ | ~~`Foldable` on `ZipDenseVectorWitness`~~ | **closed 2026-09-15**: 4 witnesses, 14 traits | — |
| — | ~~`multivector`~~ | ~~1~~ | ~~`Traversable` on `CausalMultiVectorWitness`~~ | **closed 2026-09-15**: 2 witnesses, 7 traits | — |

**27 impls scoped; 9 landed, 1 found not feasible, 17 closed as not worth their cost.** The
mechanical work is complete. What remains is row 5's `Arrow` layer, closed above as unreachable
without a sampling-model change, and `DiagonalTraversable` on `ManifoldWitness`, blocked by that
type's own invariant.

### Closed: `Foldable` × 5 and `Traversable` on `ManifoldWitness`, 2026-09-15

The five carriers each hold their elements in a `CausalTensor`, so `fold` delegates to
`CausalTensorWitness::fold` exactly as `fmap` already delegated to `CausalTensorWitness::fmap`.
`PointCloudWitness<C>` reads `metadata` rather than `points`, and the two carry different values in
the test so a fold reaching the wrong tensor is caught — though in the event the type checker
catches it first, since `points` is `CausalTensor<C>` and the fold's closure is `FnMut(B, A)`.

`Traversable` on `ManifoldWitness` needed only its two supertraits, both already present.
`sequence` is one-in-one-out, so the data length cannot change and the complex, metric and cursor
are carried across rather than rebuilt — which is what `Manifold::new`'s length and cursor checks
require, and what the tests assert by comparing whole manifolds rather than data tensors.

**`DiagonalTraversable` on `ManifoldWitness` is not feasible.** `sequence_zip` grows each structure
in its accumulator one element at a time from a caller-supplied seed, so for a manifold that seed
must hold zero data against a non-empty complex. `Manifold::new` rejects it on both the length check
(`data.len() != expected_size`) and the cursor check (`cursor >= data.len()`), and
`test_diagonal_seed_cannot_be_constructed` pins that. The obstacle is the manifold's invariant
rather than the trait, and it is the same one that keeps `Collectable` off this witness.

Fourteen tests across
`hkt_foldable_carriers_tests.rs` and `hkt_manifold_traversable_tests.rs`. Three mutations: reversing
the graph fold fails 3, resetting the manifold cursor fails 2, and folding a point cloud's points
instead of its metadata does not compile at all.

### Correction: what a `Foldable` on a zip witness is for

The first version of row 3 said the caller "converts back to the plain witness first." **That was
wrong.** `ZipDenseVectorWitness` and `DenseVectorWitness` project to the same `DenseVector<T>`, so
`DenseVectorWitness::fold` already accepts whatever `zip_with` returns, with no conversion of any
kind. The same holds for the tensor pair. Nothing was blocked at a call site, and the row overstated
the gain.

What the instance actually buys is the **bound**. A generic function written as
`W: Semigroupal<W> + Foldable<W>`, zipping and then reducing through one parameter, cannot be
instantiated at a zip witness without it; the workaround needs two witness parameters plus a
`W::Type<T> == F::Type<T>` constraint Rust cannot express. No such function exists in the workspace
yet, so this is capability rather than repair — worth the four lines, not worth overselling. The
test suite pins both halves, including a negative control that compiles without the instance.

### Closed: `Foldable` on both zip witnesses, 2026-09-15

`Foldable` has no supertraits, so `HKT` was the only requirement and the body delegates to the
same iterator fold `DenseVectorWitness` uses. `fold` involves no applicative, which is the only
thing the two witnesses disagree about, so agreeing element for element and in order is a
requirement rather than a coincidence, and a test pins it across five inputs.

Eight tests in
`deep_causality_linear/tests/extensions/hkt/zip_dense_vector_foldable_tests.rs`, written before the
impl: the generic `zip_then_fold::<Zip>` that fails to compile without it, the negative control
above, left-to-right order via digit accumulation (a reversal yields `4321` instead of `1234`,
which plain subtraction would not catch), agreement with the plain witness, the empty-vector seed,
an accumulator of a different type, and the interaction with `zip_with`'s truncation. Two mutations
confirm they bite: reversing the fold fails 5 of 8, skipping the first element fails 6.

`ZipTensorWitness` followed the same day and the same way, with one addition the vector case does
not have. `zip_with` on tensors keeps the shape when both operands agree and reports the flat
`[len]` of the overlap when they do not, so a fold written against the shape rather than the flat
data would pass every rank-1 test. Ten tests in
`deep_causality_tensor/tests/extensions/ext_hkt_zip_foldable_tests.rs` cover rank 1 through rank 3,
pin row-major visit order, and assert that a flat `[6]`, a `[2, 3]` and a `[3, 2]` holding the same
values fold identically. Three mutations: reversing the fold fails 7 of 10, skipping the first
element fails 8, and folding only the first axis — the shape-sensitive slip — fails 5.

### Closed: `Traversable` on `CausalMultiVectorWitness`, 2026-09-15

Both supertraits were already present, and `sequence` turned out to be the one traversal this
witness can take without meeting the obstacle that denies it `Monad`. `bind`'s continuation may
hand back a different `Metric` than the input carries, and since a `CausalMultiVector` holds
exactly `2^dim` coefficients with `dim` read off that metric, the two identity laws want opposite
choices. `sequence` is one-in-one-out by construction, so the input's metric is the only
defensible answer and the coefficient count cannot change — the invariant `CausalMultiVector::new`
enforces holds without being re-checked.

Eight tests in
`deep_causality_multivector/tests/extensions/hkt_multivector/hkt_traversable_tests.rs`, run over
five algebras from `Cl(0)` to `Cl(3,1)`. Two mutations confirm they bite: replacing the carried
metric with `Euclidean(0)` — the exact shape of the `Monad` failure — fails 4 of them, and
reversing the fold direction fails 5.

## 4. Reproducing

```bash
python3 scripts/witness_inventory.py     # from the repository root
```

It prints the counts in §1 and writes the full witness-by-trait matrix to
`/tmp/witness_inv.json`. On `cbf3af23b` it reports `declared: 40  with impls: 40  traits: 23`,
with both mismatch lists empty — every declared witness implements something, and every
implemented witness is declared.

Two parser requirements, both learned by getting them wrong:

- **Strip comments first.** A doc comment showing `pub struct MyCustomTypeWitness` in an example
  block is otherwise counted as a witness, and a `//`-comment containing the word `impl` bleeds
  into the next match.
- **Join wrapped `impl` headers.** `impl<C> Monad<ManifoldWitness<C>>\n    for ManifoldWitness<C>`
  is invisible to a line-oriented regex. Matching on the whole file text with `re.S` and splitting
  the header on the top-level ` for ` recovers `Adjunction`, `Monad` on `ManifoldWitness`, and
  `Pure`/`Applicative` on four topology witnesses — six rows that a line regex drops silently.
