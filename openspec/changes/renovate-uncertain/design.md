<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Design — renovate the uncertain crate

## Context

`deep_causality_uncertain` is 3,285 lines of source and 247 tests.
Its value type is an id from a global atomic counter plus a `ConstTree<UncertainNodeContent>` and a
`PhantomData<T>`. The tree is untyped; the scalar lives in three node arms
(`DistributionF64`, `DistributionF106`, `DistributionBool`) and in a three-variant `SampledValue`.

Five globals exist today: `NEXT_UNCERTAIN_ID`, `GLOBAL_SAMPLE_CACHE`, the thread-local
`SAMPLER_SEED`, the sample-index source, and `rand`'s thread RNG. After this change `uncertain` owns
none; the thread RNG stays in `rand`, as it does for the standard library.

Three constraints shape everything below.

- **A `static` cannot be generic.** This is the one true clause in `SampledValue`'s docstring, and
  removing the cache removes it as a constraint.
- **haft's container traits carry no `'static`.** Measured: zero occurrences across `Functor`,
  `Pure`, `Applicative`, `Monad`, `Traversable` and `LaxMonoidal`. `Profunctor`, which stores its
  functions, carries `'static` on every parameter. A carrier holding a stored closure therefore
  cannot implement the container traits — `error[E0276]: impl has stricter requirements than trait`.
  This is already ratified in `sampling-hkt-composition` and is not reopened.
- **AGENTS.md forbids `dyn` in library code.** The crate has six `Arc<dyn ...>` sites today.

## Goals / Non-Goals

**Goals:**

- Zero global state owned by `uncertain`, and reproducibility that holds by construction.
- Zero concrete scalar types named in `src/`; a scalar added to `deep_causality_num` works with no
  line changed here.
- Zero `dyn` sites.
- One `materialize` signature that serves every witnessed rank-1 container.
- The lazy graph composes as a value-level `Arrow`.

**Non-Goals:**

- Replacing the lazy graph with eager particle propagation. The graph carries QMC's
  static-structure analysis and the adaptive SPRT; materialisation is a second stage, not a
  substitute.
- Giving the lazy graph an HKT container witness. Settled and recorded.
- Lifting Sobol's 32-bit coordinate resolution. Unchanged, and already documented at
  `SobolSequence::coordinate`.
- Threading a session through `deep_causality`'s context nodes. The zero-argument `sample()` keeps
  taking its index from `rand`'s thread RNG; see Open Questions.
- New distributions.

## Decisions

### D1. A leaf's draw is addressed by a deterministic ordinal, never by its `Arc` address

Every leaf draw is a pure function of three inputs: the session seed, the sample index, and the
leaf's ordinal. The generator for one draw is `Xoshiro256::from_seed(mix(seed, index, ordinal))`.

The ordinal is assigned by one deterministic pre-pass over the tree, which dedupes by node identity
so a shared leaf gets one ordinal and `x + x` draws `x` once.

**The pointer is a map key inside that pre-pass and never an input to the generator.**
`ConstTree`'s node id is `Arc::as_ptr(&self.node) as usize`, a heap address: it differs between runs
and between two structurally identical trees. Mixing it into a seed would make a seeded run
irreproducible — the exact property the session exists to provide. This corrects the sketch in
`openspec/notes/unified_math/hkt_uncertain.md` §B5, which proposed deriving the draw from the leaf
address directly.

*Alternative considered:* keep a cache keyed by `(id, index)`. Rejected — it is what is being
removed, and it does not deliver the property anyway: it is consulted only at the root, so two
different roots sharing a leaf disagree today. The ordinal scheme is strictly stronger.

*Precedent:* `QmcSampler::new` already runs exactly this pre-pass, assigning each non-point
distribution leaf a Sobol dimension and keying a map by node id within one traversal. Monte Carlo
joins the scheme rather than inventing one.

### D2. The scalar bound is `RandScalar`

`RandScalar: RealField + FromPrimitive`, blanket-implemented in `deep_causality_rand` and
re-exported by `deep_causality_stats`. The graph becomes `ConstTree<Node<R>>` with
`Sample<R> { Real(R), Bool(bool) }` and a single `Distribution(DistributionEnum<R>)` arm.

The three `DistributionEnum` sample impls collapse to one: `DistributionEnum<f64>::sample` and
`DistributionEnum<Float106>::sample` are byte-identical bodies today, and the `bool` arm becomes the
`Bernoulli` branch of the generic body.

*Alternative considered:* a `MANTISSA_DIGITS`-style associated constant per scalar. Rejected for the
reason the sampling layer already established — a table keyed by type is the anti-pattern the
convention exists to remove, and every per-type fact this crate needs is derivable.

### D3. Carrier-generic materialisation needs one new haft capability

`materialize` is generic over the witness:

```rust
pub fn materialize<W>(&self, session: &mut SampleSession, n: usize)
    -> Result<W::Type<R>, UncertainError>
where
    W: Collectable<W> + HKT;
```

All sixty-odd haft traits were checked and none can build a container from a sequence. `Foldable`
consumes; `Pure` builds a one-element container; `Semigroupal::zip_with` pairs but does not extend.
So one capability is added beside `Foldable`:

```rust
pub trait Collectable<F: HKT> {
    fn collect<T, I: IntoIterator<Item = T>>(items: I) -> F::Type<T>;
}
```

`deep_causality_haft` implements it for its own `VecWitness`, `deep_causality_linear` for
`DenseVectorWitness`, and `deep_causality_tensor` for `CausalTensorWitness` (rank 1). `uncertain`
therefore depends on **`haft` only** and stays at tier 5.

*Alternatives considered:*

- *`uncertain` depends on both `linear` and `tensor` and writes two inherent methods.* Rejected: two
  dependencies, a duplicated signature, and `uncertain` moves to tier 6, which moves
  `deep_causality` to tier 7.
- *Return `Vec<R>` and let the caller wrap it.* Rejected: it drops the witness exactly at the seam
  where composition begins, and pushes the shape decision to every call site.
- *Name the trait `Unfoldable`.* Rejected: an unfold is the anamorphism from a seed; this is the
  sequence-to-structure direction, which `collect` names accurately in Rust.

This follows the precedent set when `DiagonalTraversable` was added: a capability several witnesses
need, and none can express, belongs in `haft` beside its dual rather than in the crate that noticed.

### D4. Correlated draws use the zip witnesses; the cartesian traversal is not used for sampling

Composition after materialisation is `Semigroupal::zip_with` on `ZipDenseVectorWitness` /
`ZipTensorWitness`, and `DiagonalTraversable::sequence_zip` for turning a structure of ensembles
inside out. Draw *i* of the viscosity belongs with draw *i* of the inflow.

The cartesian `Traversable::sequence` is a correctness error here, not a performance one: measured
on a 2×2 field of 50 draws per cell it returns 50⁴ = 6 250 000 entries. This is already ratified;
it is restated because the new `materialize` is the function that hands a caller the container.

Neither zip witness has `Pure` — the unit of a positional zip is the infinite repeat — so
`sequence_zip` takes its accumulator as a parameter. For sampling that means the ensemble size is
declared rather than inferred, which is correct.

### D5. `Arrow` for the graph: `In = SampleIndex`, `Out = Result<R, UncertainError>`

```rust
impl<R: RandScalar> Arrow for Uncertain<R> {
    type In = SampleIndex;
    type Out = Result<R, UncertainError>;
    fn run(&self, at: SampleIndex) -> Self::Out { ... }
}
```

This is possible **because of D1**: once a leaf draw is a pure function of (seed, index, ordinal),
evaluating the graph at an index is a pure function, which is what `Arrow::run(&self, ..)` requires.
The three goals are not independent — goal 1 is what makes goal 3's Arrow instance well-typed.

The combinators (`compose`, `first`, `second`, `split`, `fanout`) come from haft as concrete generic
structs, so composition stays static — the same shape `deep_causality_calculus` uses for `Euler`,
`Rk4` and `Diff`, each of which holds its function as a type parameter rather than a `dyn`.

The four unreachable node arms — `PureOp`, `FmapOp`, `ApplyOp`, `BindOp` — are removed rather than
left as dead arms. They asked for `Send + Sync + 'static` on a stored closure, which is what the
container traits withhold; the Arrow layer is where a stored function belongs, and it is now
occupied.

*Alternative considered:* `type In = (&mut SampleSession, u64)`. Rejected: an associated type that
borrows forces a lifetime onto every composite, and `run(&self, ..)` cannot mutate through `&self`
anyway.

### D6. Dimensionless probabilities take the caller's scalar

`bernoulli(p: R)`, the five comparison thresholds, `probability_exceeds`,
`estimate_probability -> R`, and both `MaybeUncertain` probabilities. A probability is a ratio of
two counts, so it is `lift_count::<R>(hits) / lift_count::<R>(n)`, and it reaches `println!` through
`lower`. The SPRT log-likelihood arithmetic runs in `R`.

One bound does not move and is documented where it lives rather than here: `Bernoulli::new` in
`stats` holds `p` as 64-bit fixed point, which is what makes `p = 0` and `p = 1` exact, so a
`Float106` caller stating 106 bits of probability keeps 64 of them. That is the representation's
bound, not the scalar's, and widening the scalar does not move it.

### D7. `MaybeUncertain<R>` keeps its name

Twelve CFD sites and the alias discipline depend on the name, so it stays a named type rather than
becoming `Uncertain<Option<R>>` at the surface. Internally the four per-type files (306 lines)
become one impl block over the generic tree.

Its presence and value channels are sampled today at two separately drawn global indices, which was
the cache's hardest job. Under the session they are one tree sampled at one index, and there are no
two channels to reconcile.

### D8. The flaky test is fixed by seeding, never by relaxing the assertion

`uncertain_maybe_f106_tests::test_lift_to_uncertain_success` failed 2 of 12 consecutive runs: an
unseeded SPRT at true p = 0.9 against a 0.8 threshold with a 100-sample budget can fail to accept
within budget. The fix is a seeded session, which the renovation makes the ordinary way to write the
test. The assertion stays exact.

### D10. The session carries no scalar parameter and no Sobol sequence

`SampleSession` holds a seed, a sample counter and a mode. That is all a draw's address needs, and
both of the things the proposal originally put in it turned out not to belong.

**No `<R>`.** Seed, counter and mode are integers; the scalar appears nowhere in the type, so the
parameter would be `PhantomData` for its own sake. Dropping it buys something real rather than
merely costing nothing: one session can drive an `Uncertain<f64>` and an `Uncertain<Float106>` and
correlate them at the same index, which is exactly the mixed-precision input case the ensemble
carrier exists for. Under `SampleSession<R>` those two would need separate sessions and could not
share an index.

**No Sobol sequence.** `SobolSequence::new(dim)` needs the dimension count, and that is a property
of the tree rather than of the session: `QmcSampler::new` computes it by a pre-pass that assigns
each stochastic leaf a dimension, so one session driving two differently shaped trees would need
two sequences. The session holds the mode and the seed; a QMC session's seed is what the per-tree
digital shift is derived from, which is what `QmcSampler::new(uncertain, Some(seed))` already takes.

The counter also replaces today's *random* sample index with a sequential one, so `take_samples(n)`
draws at indices `0..n` and a fresh session replays exactly rather than approximately.

## Risks / Trade-offs

- **Reproducibility regresses silently** → a golden-value test: a seeded session reproduces a
  recorded vector of draws, asserted against literal values rather than a count, and asserted across
  two structurally identical trees built separately, which is what catches a pointer leaking into
  the seed.
- **The ordinal pre-pass changes every recorded draw** → it does; no seeded sequence from before the
  change survives. Stated as breaking rather than hidden, and the `f64` bit-for-bit requirement in
  `uncertain-realfield-generic` is withdrawn for exactly this reason.
- **Breaking the public API of a published crate** → the four aliases are kept, so the 45 core sites
  and 12 CFD sites need no edit; removing `+ ProbabilisticType` from a bound never breaks a caller.
  The removed items are the cache, the seed functions and the closed dispatcher, none of which a
  consumer should have been holding.
- **A new haft trait ripples** → one trait, two impls, no existing signature changed. It is additive
  in the same sense `DiagonalTraversable` was.
- **An ensemble at field size does not fit in memory** → a `CausalTensor<DenseVector<f64>>` over a
  million cells at 10 000 draws is 80 GB. The working pattern is to materialise only the inputs that
  carry uncertainty, traverse them diagonally into one ensemble of input records, and solve per
  draw. The lazy graph is why one does not materialise a field, and the documentation says so.
- **QMC's dimension budget is unchanged** → the pre-pass still rejects `BindOp` and
  branch-divergent `ConditionalOp`, and `MAX_SOBOL_DIM` still caps the count. Removing the cache
  does not touch this.

## Migration Plan

1. Session and ordinal addressing land first, with the cache still present but unused, so the
   reproducibility tests can be written against the new path before the old one is removed.
2. The cache, the seed slot and `SamplerKind` are removed; the 24 `rusty_fork_test!` invocations
   become ordinary tests and `rusty-fork` leaves the dev-dependencies.
3. The tree is made generic and `SampledValue` / `ProbabilisticType` are removed; `CFD`'s ten bounds
   drop `+ ProbabilisticType` in the same step, since they cannot compile between steps 3 and 4.
4. `Collectable` lands in haft with its two impls; `materialize` follows.
5. `Arrow` lands and the four dead node arms are removed.

Steps 1–2 and 3 are each independently shippable; 4 and 5 are additive. There is no rollback beyond
reverting, and no persisted state to migrate — the only durable artefact is a recorded seed, and
step 1 is where recorded seeds change meaning.

## Open Questions

- Should `deep_causality`'s context nodes thread a `SampleSession` so that a causal model's draws
  are reproducible from one seed? Out of scope here — the zero-argument `sample()` keeps taking its
  index from `rand`'s thread RNG, which is the one global left in the path and is `rand`'s, not
  `uncertain`'s. Worth a later change.
- `Collectable` is the proposed name for the new haft capability. `FromSequence` and `Rank1` were
  the other candidates; the name is cheap to change before it ships and expensive after.
