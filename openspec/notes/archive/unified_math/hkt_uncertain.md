<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Lifting `uncertain` and `rand` into the unified math stack

**Scope.** `deep_causality_uncertain` and `deep_causality_rand`, read on `main` at `7c63468d1` on
2026-09-14. Two questions: can each crate take precision as a parameter the way the rest of the
stack does, and can each join the HKT composition surface. Breaking the public API is allowed.

**`rand` is done.** `blanket-scalar-sampling` landed at `191281cae`, and the sampling layer now
names zero concrete scalars. §1.2, the `rand` rows of §2, stage 1 of §4 and the commands in §6
describe what is in the tree. The `uncertain` half of the note is untouched by it and still states
a plan.

**Not in scope.** New distributions. Performance beyond the arithmetic shown. The physics, CFD,
quantum and discovery consumers, except as migration cost.

**Method.** Every public type, trait and global in both crates was read from source. The
`haft` trait signatures the retrofit has to satisfy were read from `haft/src`. Consumer call
sites were counted by grep across the workspace. Effort figures are the author's estimates and
say so.

**Companion.** `openspec/notes/archive/unified_math/hkt_gaps.md` §3.1 and §3.4 name these two crates as
gaps. §0 and B2 of this note say what that note should now say instead, and stage 4 in §4
updates it.

---

## 0. Verdict

- **Precision as a parameter: done in `rand`, feasible in `uncertain` at moderate effort.** `rand`
  names no concrete scalar anywhere: one blanket sampler per algebraic tower, and `BFloat16` — a
  type the crate mentions in no implementation — draws from a range on the strength of the algebra
  alone. `uncertain` is the opposite: its whole engine runs through a closed three-variant enum
  that exists, by its own
  docstring, so that a global static cache can stay non-generic. That assumption predates unified
  math and no longer holds anywhere else in the stack; every other crate is generic in its scalar
  and holds no scalar in a static. The crate has already half-abandoned it at its own boundary,
  it costs 56 variant match arms today, and it pins CFD's generic scalar to `f64` and `Float106`
  through the bound it forces. Making the graph generic in its scalar is the retrofit, and the
  generic version is smaller than what it replaces.
- **Decision: the global sample cache is removed, not made generic.** It caches only the root,
  it leaks one entry per draw with nothing ever clearing it, and its production branch is the one
  branch the test suite never runs. The one property it provides, that the same index returns the
  same value, is had without storage by deriving every leaf draw from the session seed, the sample
  index and the leaf's identity. B5 records the decision and what it removes.
> **Stage 2 shipped, and settled two of the items below differently.** The lazy graph did get a
> witness — not a container one, an **`Arrow`** — and it is well-typed precisely because every draw
> became a pure function of (seed, index, ordinal). And no `Particles<T>` was written: the ensemble
> carrier is a *type parameter*, so a materialised ensemble is whatever container the caller names.
> The paragraph below is left as written, because its diagnosis of the container traits is correct
> and is why `Arrow` is the right surface rather than `Functor`.

- **An HKT witness on the lazy computation graph: not feasible against `haft` as written.**
  `Functor::fmap` hands the witness an `FnMut(A) -> B` with no `'static`, `Send` or `Sync`
  bound and no bound on `B`, and `Pure::pure` hands it one `T` with no `Clone`. A lazy graph has
  to store the closure and produce `T` on every sample; it cannot do either under those
  signatures. This is why the `FmapOp` and `BindOp` arms exist and no builder feeds them. The
  archived note's "the coding is small" was wrong; the block is at the trait signature, and it
  blocks every lazily evaluated container, not this one.
- **A strict particle carrier is feasible, lawful, and is what `Traversable` needs.** A
  `Particles<T>` holding `n` draws is an ordinary container: `fmap` maps, `apply` zips with
  broadcast, `bind` runs per particle, and the laws hold by equality. It is also the shape a
  `CausalTensor<Uncertain<R>>` has to take before it can become an `Uncertain<CausalTensor<R>>`,
  so it is the payoff the whole exercise is for. The lazy graph stays, precision-generic, as
  the thing that builds distributions, drives quasi-Monte Carlo and adaptive testing, and
  materialises particles on demand.
- **`rand` needs no witness.** Its `Map<D, F, T, S>` is a correct zero-cost functor over a
  trait, and the only carrier worth having is the particle type in `uncertain`. §3.4 of the gap
  note resolves as "no witness in `rand`".
- **`MaybeParallel` replaces the nine hardcoded `Send + Sync` bounds** and trims the signature
  blocker to its irreducible part, `'static` on a stored closure and `Clone` on a lazy constant.
  It also gives the particle carrier a parallel `fmap` through `scoped_map` for free.

---

## 1. The two crates today

### 1.1 `deep_causality_uncertain`

3,273 lines of source, 46 test files, 253 tests. Depends on `stats`, `rand`, `ast`, `num`,
`algebra`; not on `haft`.

**The value type.** `Uncertain<T: ProbabilisticType>` is an id from a global atomic counter, a
`ConstTree<UncertainNodeContent>` from `deep_causality_ast`, and a `PhantomData<T>`
(`types/uncertain/mod.rs:32-36`). The tree is untyped. The scalar lives in the node arms:
`DistributionF64`, `DistributionF106`, `DistributionBool` (`uncertain_node_content/mod.rs:46-49`).

**The closed precision dispatcher.** `SampledValue` is `Float(f64) | DoubleFloat(Float106) | Bool(bool)`
(`types/cache/sampled_value.rs:19-27`), and its docstring states the design decision:
the precision is "a variant rather than a type parameter, so the computation graph, the global
sample cache (a `static`), and the sampler all stay non-generic". `ProbabilisticType` is
implemented for exactly `bool`, `f64`, `Float106` and `SampledValue`. There is no `f32` and no
`BFloat16`. Adding a scalar today means a new enum variant, a new `DistributionEnum` arm, new
sampler match arms in two samplers, a new `ProbabilisticType` impl, and new alias files.

**The HKT arms nothing feeds.** `PureOp`, `FmapOp { func: Arc<dyn SampledFmapFn>, .. }`,
`ApplyOp`, `BindOp { func: Arc<dyn SampledBindFn>, .. }` (`uncertain_node_content/mod.rs:51-66`).
`SampledFmapFn` is `Fn(SampledValue) -> SampledValue + Send + Sync + 'static`. No public
constructor produces any of the four. The only `map` is a method on `Uncertain<f64>`
(`uncertain_f64.rs:48`), and there is no `bind` on any instantiation.

**What is already generic.** `UncertainReal` gives one generic impl for `point`, `normal` and
`uniform` over `f64` and `Float106` (`uncertain_real.rs`), and the `Float106` file is thirteen
lines. The statistics go through `deep_causality_stats`. That part of the crate was retrofitted
already and shows the shape the rest should take.

**Where `f64` leaks.** Every comparison takes an `f64` threshold regardless of `T`:
`greater_than`, `less_than`, `equals`, `approx_eq`, `within_range`
(`uncertain_op_comparison.rs:14-80`) and the `ComparisonOp { threshold: f64 }` node. The function
nodes are `Fn(f64) -> f64` and `Fn(f64) -> bool` (`uncertain_node_content/mod.rs:83-87`), and the
sequential sampler narrows a `DoubleFloat` to `f64` to feed them (`sequential_sampler.rs:256`).
`bernoulli(p: f64)`, `probability_exceeds(threshold: f64, confidence: f64)`,
`estimate_probability -> f64` (`uncertain_bool.rs:23-113`), and `MaybeUncertain`'s
`prob_some: f64` and `threshold_prob_some: f64`. A `Float106` value compared against an `f64`
threshold is the precision leak the stack's alias discipline exists to prevent.

**Global state.** `NEXT_UNCERTAIN_ID` (atomic), `GLOBAL_SAMPLE_CACHE` (a `OnceLock` static,
`thread_local!` under test, `global_cache.rs:112-116`), `SAMPLER_SEED` (thread-local
`Option<Xoshiro256>`, `sampler_seed.rs:23-25`), the sample-index counter, and `rand`'s thread
RNG. Every `sample()` goes through the cache keyed by `(id, sample_index, SamplerKind)`
(`uncertain_sampling.rs:21-53`). The cache is what makes `x.sample_with_index(3)` and
`(x + 1).sample_with_index(3)` consistent across two calls, and it is why the value type must be
non-generic: a `static` cannot be generic in `R`.

**Sharing semantics.** Node identity is the `Arc` allocation address
(`ast/const_tree/accessors.rs:60`), and both samplers memoize by it within one sample, so `x + x`
draws `x` once. Any redesign has to keep that.

**Quasi-Monte Carlo.** `QmcSampler::new` runs a pre-pass that assigns every non-point
distribution leaf a Sobol dimension and rejects `BindOp` and branch-divergent `ConditionalOp`
(`qmc_sampler.rs:10-16, 327-332`). The rejection is correct: QMC needs a static number of
stochastic dimensions.

**`MaybeUncertain`.** A second struct with its own per-type files (`mod.rs` 54 lines, `bool` 62,
`f106` 70, `f64` 120) rather than `Uncertain<Option<T>>`.

**Consumers.** `deep_causality` core (context nodes `DataUncertainF64` and `DataUncertainBool`),
`deep_causality_cfd` (inflow uncertainty and flow config), `deep_causality_quantum` (one QPU
file), three examples. How they spell the type: `UncertainBool` 32 sites, `Uncertain<f64>` 29,
`UncertainF64` 18, `MaybeUncertain<R>` 12, `Uncertain<bool>` 4, `Uncertain<R>` 2. CFD is already
generic in `R`; core pins `f64` and `bool`. Methods called: `normal`, `uniform`, `conditional`,
`bernoulli`, `from_samples`, `sample`, `expected_value`, `expected_value_qmc`,
`standard_deviation`, `estimate_probability`, `probability_exceeds`, `lift_to_uncertain`,
`implicit_conditional`, `to_bool`, `is_some`, `is_none`.

### 1.2 `deep_causality_rand`

1,387 lines of source, 112 tests. Depends on `num` and `algebra`; not on `haft`.

**What the crate holds.** Entropy, and the draws that are facts about bits rather than about a
density: a machine word (`StandardWord`), a Boolean (`StandardBool`), a value uniform over a range
(`Uniform`), the generators (`Xoshiro256`, a thread handle, an OS-entropy source) and a Sobol
sequence. No mean, no variance, no moment. The shaped distributions — normal, exponential, Cauchy,
Weibull, log-normal, Poisson, categorical, the unit-interval draws and their inverse-CDF transforms
— are in `deep_causality_stats` beside the densities that define them, and `stats` re-exports the
generator traits so a caller needs one dependency to spell a bound.

**The traits.** `RngCore` (three methods), `Rng: RngCore` (default methods only; the sole impl is
`impl<T: Rng> Rng for &mut T`), `Distribution<T>` with `sample`, `sample_iter` and `map`,
`SampleUniform<Kind>` / `UniformSampler`, `SampleRange<T, Kind>`, `Fill`, `SampleBorrow`, and two
blanket capability traits: `RandScalar: RealField + FromPrimitive`, which carries the unit draw,
and `RandUnsigned: NaturalNumber + Num + FromPrimitive + Debug`. `RandScalar` is the bound the rest
of the stack asks for.

**Zero concrete scalars.** No implementation in `src/` names `f32`, `f64`, `Float106`, `BFloat16`
or any unsigned type. Two blanket implementations cover both towers:

```rust
impl<T: RealField + FromPrimitive> SampleUniform<FloatKind> for T { type Sampler = UniformFloat<T>; }
impl<T: RandUnsigned> SampleUniform<UnsignedKind> for T { type Sampler = UniformUnsigned<T>; }
```

`FloatKind` and `UnsignedKind` are type parameters and nothing else; neither is ever constructed.
They exist because one trait cannot carry two blanket impls over disjoint towers — coherence cannot
prove a real field will never also be a natural number — and parameterising makes the two impls
distinct items. `Rng::random_range<T, K, R>` keeps one signature, because `K` is inferred from the
range and no call site names it.

**Per-type facts are derived, not declared.** The unit draw accumulates 53-bit generator words and
stops when the next word would land entirely below the scalar's own `epsilon`, so `f32`, `f64` and
`BFloat16` take one word and `Float106` takes two with no table saying so.

**Where `f64` still appears.** Nine non-comment lines, every one of them a conversion pivot
(`from_f64`, `to_f64`, an `as f64` scaling) or a panic message; none is an implementation target.
Two fixed widths remain, both properties of a representation rather than of a scalar and both
documented at the item that has them: `SobolSequence::coordinate<T>` returns at the caller's scalar
but resolves `2^-32`, fixed by its direction-number table (`sobol.rs`); and `Bernoulli::new<T>` in
`stats` quantises `p` to `2^-64`, which is what makes `p = 0` and `p = 1` exact.

**`BFloat16`.** Drawn, and asserted. `tests/types/dist/uniform/parity_probe_tests.rs` runs one
generic function at `f32`, `f64`, `Float106` and `BFloat16`, requiring every draw inside
`[10, 20)` and zero draws on the bound. The bound needed work to hold: a narrow significand rounds
the affine map `u * scale + low` up onto `high` — 14 draws in 2 000 at `BFloat16`, none at `f32`
and wider — so `UniformFloat` carries an `exclusive_high` and rejects a result that reaches it.

**The functor.** `Map<D, F, T, S>` (`types/map/mod.rs`) is a struct of a distribution and a
closure, monomorphised, and `Distribution<S>` for it is four lines. It is a functor written by
hand and a good one. It cannot be an `HKT` witness because a witness binds one type parameter
and `Map` has four, and because `Distribution` is a trait rather than a type constructor.

**Global state.** A thread-local `Xoshiro256` behind `rng()` (`lib.rs`), the same shape as the
standard library's.

**Consumers.** Runtime: `stats`, `uncertain`, `algorithms` (BRCD, DAG sampling) and `physics`
(Lund, feature-gated). Dev-only: `data_structures`, `discovery`, `tensor`, `ultragraph`. Three
example crates. They use `Rng`, `Xoshiro256`, `Uniform`, `SobolSequence` and `Distribution::sample`;
the named distributions now come from `stats`. `topology` does not depend on this crate directly;
it reaches the generator traits through the `stats` re-export. No call site broke in the retrofit,
and the bounds got shorter: two sites in `topology` and one example ask for `RandScalar` and
nothing more. No consumer uses `Map`.

---

## 2. Blockers and resolutions

| # | Blocker | Crate | Class | Resolution in one line |
|---|---|---|---|---|
| B1 | ~~The closed `SampledValue` enum~~ | uncertain | **done** | `ConstTree<Node<R>>` with `Sample<R> { Real, Bool }`; 72 variant arms gone, the enum and its four traits removed |
| B2 | ~~`haft`'s container signatures cannot feed a lazy graph~~ | uncertain, haft | **closed, differently** | the lazy graph is an `Arrow` (`In = SampleIndex`, `Out = Result<R, _>`), which is well-typed exactly because B5 made evaluation pure. No `Particles<T>` was written — see §4 |
| B3 | ~~Struct bound `T: ProbabilisticType`~~ | uncertain | **done** | the bound is `R: RandScalar` exactly — no `'static`, no `Send`/`Sync`, because no node stores a trait object |
| B4 | ~~`f64` thresholds, function nodes and probabilities~~ | uncertain | **done** | everything in `R`. Two `f64` mentions survive in live code, both documented boundaries: `BernoulliParams::p` (the fixed point the draw honours) and `QmcSampler::coordinate` (a Sobol address, not a value) |
| B5 | ~~Five globals~~ | uncertain, rand | **done** | index-addressed draws from (seed, index, **ordinal** — not leaf id, see the correction below); a `SampleSession` value; zero globals owned by `uncertain` |
| B6 | QMC needs static structure, and Sobol resolves 32 bits | uncertain, rand | inherent | a structure descriptor; the 32-bit cap is a limit `Float106` cannot lift, and `SobolSequence::coordinate` now states it |
| B7 | ~~Six per-type files in `rand`~~ | rand | **done** | two blanket impls, one per tower, kept apart by a kind type parameter; zero concrete scalars named |
| B8 | ~~No `BFloat16` in `rand`~~ | rand, stats | **done** | nothing to add for it: the blanket impls cover it, its word count comes from `epsilon`, and a probe test draws at it |
| B9 | ~~`MaybeUncertain` as a parallel type~~ | uncertain | **closed, differently** | it stays a named type: `MaybeUncertain<R>` over an `UncertainBool<R>` presence channel and an `Uncertain<R>` value channel, drawn at one index. `Uncertain<Option<R>>` was not taken — see the correction under §4 |
| B10 | ~~Core pins `f64` and `bool`; CFD inherits the `ProbabilisticType` bound~~ | consumers | **done** | CFD compiles at `f32`, with a test that instantiates the boundary source and inflow zone there. The aliases were *not* kept in `uncertain` — they moved to `deep_causality`, which is the crate that picks the scalar |

**The bound the rows above ask for is `RandScalar`**: `RealField + FromPrimitive`,
blanket-implemented in `deep_causality_rand` and re-exported by `deep_causality_stats`. Neither
`RealRng` nor the `RandWidth` width table exists; a scalar joins the sampling layer by satisfying
the algebra and by nothing else.

### B1. The closed precision dispatcher

**Block.** `SampledValue` carries precision as a variant so that `GLOBAL_SAMPLE_CACHE`, a
`static`, can hold it. A `static` cannot be generic, so as long as the cache is global the value
type cannot be a type parameter, and every scalar costs a variant in three enums and two samplers.

**Why the assumption no longer holds.** Only one of its three clauses is a Rust constraint: a
`static` cannot be generic. The other two, that the graph and the sampler stay non-generic, follow
from choosing a global static cache in the first place, and that choice predates unified math.
Nothing else in the stack holds a scalar in a static; every other crate lets the program's alias
reach the bottom. The crate itself has moved on at its edges: `UncertainReal` gives generic
constructors, `Sampler<T>` is generic in name, and the docstring concedes the boundary types are
generic and convert at the edge. The enum survives on the inside only because the static needs it.
It has a cost today: 56 match arms on its variants across `src/`, 58 of them inside the two
samplers, which a tree generic in `R` reduces to `Real` and `Bool`. And it has a consumer cost:
`deep_causality_cfd` bounds its uncertain march config on `CfdScalar + ProbabilisticType`
(`types/flow_config/uncertain_march_config.rs:21`), so a CFD run at `f32` with uncertain inflow
does not compile, at the one crate that models uncertainty. Once the cache is gone (B5) the
enum has no reason left.

**Resolution.** Make the tree generic: `ConstTree<Node<R>>` with
`enum Sample<R> { Real(R), Bool(bool) }` and one `Distribution(DistributionEnum<R>)` arm in place
of three. The distributions come from `stats` and the range draw from `rand`, both over
`R: RandScalar`, which serves every real field — `f32`, `Float106` and `BFloat16` included — today.
`Uncertain<f32>` then exists on the day the tree compiles, with no new files.

The cache does not move; it goes, and B5 says why and what replaces it. A `SampleSession<R>`
the caller owns holds the seed, the sample counter and, for QMC, the Sobol sequence. `sample()`
on an `Uncertain<R>` takes `&mut SampleSession<R>`; the zero-argument `sample()` the core context
nodes call builds a session on the thread RNG that `rand` already keeps, so `uncertain` owns no
global of its own.

**Breaks.** `SampledValue`, `ProbabilisticType`, `IntoSampledValue`, `FromSampledValue`,
`with_global_cache`, `GlobalSampleCache`, `SamplerKind` leave the public API. `seed_sampler` and
`clear_sampler_seed` become `SampleSession::seeded(seed)`.

### B2. The trait signatures and the lazy graph

**Block.** The signatures in `haft/src`:

```rust
fn fmap<A, B, Func>(m_a: F::Type<A>, f: Func) -> F::Type<B> where Func: FnMut(A) -> B;
fn pure<T>(value: T) -> F::Type<T>;
fn apply<A, B, Func>(f_ab: F::Type<Func>, f_a: F::Type<A>) -> F::Type<B> where A: Clone, Func: FnMut(A) -> B;
fn bind<A, B, Func>(m_a: F::Type<A>, f: Func) -> F::Type<B> where Func: FnMut(A) -> F::Type<B>;
```

An impl may not add bounds the trait does not have. A lazy `Uncertain<B>` has to store `f` and
call it once per sample, which needs `Fn + Send + Sync + 'static`; it has to return a `T` from
`pure` on every sample, which needs `Clone`; and it has to memoize a `B` by node id, which needs
`'static`. None is available. The existing `FmapOp` arm asks for exactly the bounds the trait
withholds, which is why it has no builder. Every witness on the surface today is eager: tensors,
vectors, matrices, multivectors, manifolds, `Option`, `Result`. The surface is a surface for data,
and the lazy graph is a program.

Three ways out were weighed.

1. **Change `haft`.** Add the bounds to the four traits. Every witness in the stack and every
   caller would carry `'static + Send + Sync` closures for the benefit of one crate. Rejected.
2. **A second trait family for lazy containers.** `LazyFunctor` and friends with the stronger
   bounds. Principled, but it is a second surface, and the point of the stack is one. Rejected
   for now; the `Arrow` trait already covers the function-shaped case and is the right home for
   the graph's own composition (see the design in §3).
3. **A strict carrier.** `Particles<T>` holds `n` draws as a `Vec<T>` and takes the witness.
   `fmap` maps the vector. `pure` is one particle that broadcasts, the way a scalar broadcasts
   against a tensor. `apply` zips with broadcast. `bind` runs the continuation per particle and
   takes the diagonal, so `n` in gives `n` out. Right identity, left identity and associativity
   then hold by equality, not by sampling, and `fold` is the ordinary fold. **Recommended.**

The particle representation keeps the sharing semantics that the id-memo gives the graph:
`x + x` on particles zips index against index, so the two uses of `x` are the same draw, and two
independent materialisations are two independent draws. Correlation is by particle index, which
is what the cache key `(id, sample_index)` encodes today.

**What `MaybeParallel` changes here.** `deep_causality_par` makes thread safety a feature: with
`parallel` the trait is a `Send + Sync` alias, without it the trait is vacuous. Topology and fft
already bound their scalars on it. The nine hardcoded `Send + Sync` sites in `uncertain` (the
`ProbabilisticType` supertrait, `SampledFmapFn`, `SampledBindFn`, both function-node arms, the
two `map` bounds) take `MaybeParallel` instead, so serial builds carry no thread-safety bound on
the graph at all. That removes two of the bounds a stored closure would need. It does not remove
the other two. `'static` stays, because `HKT` binds one type slot and no lifetime slot, so a
borrowed closure cannot live in the container. `Clone` on `pure` stays, because a lazy constant
has to hand out its value on every sample. The `FnMut` against `Fn` mismatch is removable by
interior mutability, a `RefCell` in serial builds and a `Mutex` under the feature, which is
exactly the split `MaybeParallel` expresses. Two of four bounds gone, and the two that remain are
enough to keep the lazy graph off the surface. The carrier stands.

What the carrier gives up: laziness, the QMC pre-pass, and the adaptive sample count that
`probability_exceeds` uses through the sequential probability ratio test. None of those is lost,
because the lazy graph keeps them and the carrier is what the graph produces on request:
`Uncertain<R>::materialize(&mut session, n) -> Particles<R>`. The two types divide the work.
The graph composes distributions; the carrier composes samples.

**The memory arithmetic.** A `Particles<Float106>` at `n = 10 000` is 160 KB. A
`CausalTensor<Particles<R>>` over a million cells at `f64` is 80 GB, and after `sequence` it is
ten thousand tensors of 8 MB, the same 80 GB. Particles at field size do not fit, and the lazy
graph is why one does not materialise a field. The working pattern is: materialise the inputs
that carry uncertainty (an inflow speed, a material constant, a handful of numbers), traverse
them into one `Particles<Input>`, run the solve per particle, and reduce with `stats` as the
particles stream. `sequence` and `bind` on `Particles` are what make the per-particle solve one
`fmap`.

### B3. The struct bound

`Uncertain<T: ProbabilisticType>` must become `Uncertain<T>`, with the bounds on impl blocks. The
`Constraint` slot that would have carried the bound no longer exists on `HKT` (`hkt_gaps.md` §7),
and `Dual` made the same move. `ProbabilisticType` then has no job: real values ask for
`R: RandScalar`, booleans are `bool`, and the `Into`/`From` conversions through `SampledValue` go
with the enum.

### B4. The `f64` leaks

All of `uncertain_op_comparison.rs`, the `ComparisonOp` threshold, both `FunctionOp` arms,
`bernoulli`, `probability_exceeds`, `estimate_probability`, and the two `MaybeUncertain`
probabilities take `R` or return `R`. A probability is a ratio of two counts, so it is
`lift_count::<R>(hits) / lift_count::<R>(n)`, and it reaches `println!` through `lower`. `Bernoulli::new` in `stats` already takes the caller's scalar, so
`bernoulli` takes `R` like everything else. What it does not widen is resolution: the parameter is
held as 64-bit fixed point, which is what makes `p = 0` and `p = 1` exact, so a `Float106`
probability keeps 64 of its bits. That is the representation's bound, not the scalar's, and the
constructor says so.

### B5. Global state, and the decision to remove the cache

Five globals today: `NEXT_UNCERTAIN_ID`, `GLOBAL_SAMPLE_CACHE`, the thread-local `SAMPLER_SEED`
slot, the sample-index source, and `rand`'s thread RNG. After the change, `uncertain` owns none;
the thread RNG in `rand` remains, as it does for the standard library.

**What the cache was read to do, against what it does.** Three findings from reading
`types/cache/global_cache.rs` against its callers.

- It caches only the root. The two samplers never consult it below the root
  (`types/sampler/*.rs` contains no reference to it); within one draw they memoize by node
  identity in a per-call map, and that map is what keeps `x + x` on one draw. The global cache
  therefore does not make `x.sample_with_index(3)` and `(x + 1).sample_with_index(3)` agree,
  which is what its shape suggests. Two roots are two keys.
- It leaks by design. `sample()` draws its index as a random `u64` from the RNG
  (`sampler_seed.rs:49-55`), `sample_with_index` inserts one entry under it, and nothing in
  `src/` calls `clear`. Every plain draw adds an entry that is never read again and never freed;
  `expected_value` over ten thousand samples adds ten thousand of them for the life of the process.
- Its production branch is untested by construction. Under `cfg(test)` the `OnceLock` static
  becomes a `thread_local!` so the suite can run in parallel (`global_cache.rs:112-116`), so the
  global that ships is the one branch the 253 tests never execute. Thirty-eight call sites across
  six test files exist to manage that state.

**What it buys.** One property: `sample_with_index(i)` on the same root returns the same value
on a second call. The sequential probability ratio test (`sprt_eval.rs:57`) and the QMC
estimators (`uncertain_bool.rs:109`) rely on it, and `SamplerKind` is in the key so Monte Carlo
and QMC draws at the same `(id, i)` are never cross-served.

**Decision.** Remove the cache and obtain the property by construction. Every leaf draw is
derived from three inputs and nothing else: the session seed, the sample index, and the leaf's
**ordinal**. A leaf's generator for one draw is `Xoshiro256::from_seed(mix(seed, index, ordinal))`,
a few nanoseconds of hashing per leaf per draw.

> **Correction, made when stage 2 shipped.** This paragraph originally named the leaf's identity as
> "the `Arc` address `ConstTree` already exposes". That is a heap address: it differs between two
> runs of the same program and between two structurally identical trees, so a draw derived from it
> could not be replayed from a recorded seed — which is the whole property the cache was being
> removed to obtain. The identity is instead an **ordinal**: the position a drawing leaf occupies
> in one fixed traversal, assigned by a pre-pass (`LeafOrdinals`). Two trees built by the same
> sequence of constructor calls agree on it however their memory was laid out. The address is still
> used, but only to recognise a node already seen *within* one traversal, and it never reaches the
> generator. A test builds two structurally identical graphs separately and asserts they draw alike;
> reintroducing the pointer makes it fail. The QMC path already works this way, since a Sobol point is a function of its index and
dimension alone; the Monte Carlo path joins it.

What that gives, in order of weight:

1. The same index gives the same draw for every tree that shares the leaf, not only for the same
   root. That is stronger than today, and it is the correlation-by-index rule `Particles`
   uses, so the graph and the carrier agree on what an index means.
2. No lock, no static, no `RwLock` double-check, no `cfg(test)` split, no leak. The `SampleSession`
   is a value: constructed per test, dropped at the end, nothing shared.
3. Parallel materialisation partitions by index range and shares nothing, which is what lets
   `Particles::materialize` fan out under `scoped_map` with no synchronisation in the sampler.
4. `SamplerKind` leaves the key because there is no key; a session is either Monte Carlo or QMC
   by construction.
5. The `MaybeUncertain` difficulty dissolves. Its presence and value channels are sampled today
   at two separately drawn global indices (`uncertain_maybe_f64.rs:45-50`), so keeping them in
   step was the cache's hardest job. Under the generic tree the pair is one tree over an `Option`,
   sampled at one index, and there are no two channels to reconcile.

**What is removed from the public API.** `GlobalSampleCache`, `with_global_cache`, `SamplerKind`,
`seed_sampler`, `clear_sampler_seed`. `SampleSession::seeded(seed)` and `SampleSession::qmc(seed)`
replace the last two. The per-call memo inside the samplers stays; it is a cost saving, not a
correctness device.

**What the simplification removes from `src/`.** The cache module, the seed slot, the
`SamplerKind` enum, the `cfg(test)` duplication, and the 56 variant arms that the enum in B1
carried. The generic tree, the session, and the index-addressed generator replace them with less
code than they take out, which is the author's expectation and the first thing stage 2 should
measure.

### B6. Quasi-Monte Carlo

Two facts, one design and one inherent.

The design one: QMC needs a static number of stochastic dimensions, so `bind` is unsupported
there, and the pre-pass says so today. Under the generic tree nothing changes; the pre-pass runs
over `Node<R>` the same way. Under `Particles`, QMC is a materialisation strategy:
`materialize_qmc(n, seed)` fills the particles from Sobol points once, and the carrier's
`bind` is ordinary per-particle evaluation with no static-structure question at all. The fork
the archived note worried about dissolves: `bind` lives on the carrier, and the graph keeps its
guard.

The inherent one: `SobolSequence::coordinate` is 32-bit fixed point (`sobol.rs:27, 91`). A
`Float106` QMC estimate therefore carries at most 32 bits of stratification per coordinate.
That is not a defect, since QMC's value is in the low-discrepancy layout and not in the
per-coordinate resolution, but it means the QMC path gains nothing from `Float106` until the
direction numbers move to 64 bits. State it in the docstring; do not fix it in this change.

### B7. The per-type sampler files in `rand`

**Done**, `191281cae`. The crate names zero concrete scalars. Two independent things had to
happen, and the first alone was not enough.

**The crate boundary took the shaped distributions out.** A blanket
`impl<T: RealField> Distribution<T> for StandardUniform` is `error[E0119]` against the `u64`, `u32`
and `bool` implementations a generator must also provide: coherence cannot prove `u64` will never
be a real field. Moving normal, exponential, Cauchy, Weibull, log-normal, Poisson, categorical and
the unit draws to `deep_causality_stats` — which has no reason to sample a machine word — removes
the overlap rather than working around it. `rand` kept the word draw, the Boolean draw, the
generators, Sobol and the range sampler; `Uniform<X, K>` stays because `SampleUniform` can only be
implemented in the crate that owns it.

**The kind parameter took the per-type range samplers out.** The boundary does not reach them,
because both towers need a range sampler in the same crate. What stood in the way was a circular
argument in the crate's own comments: the float bindings said they could not be generic because a
blanket would collide with the integers, and the integer bindings said the same about the floats.
One trait cannot carry two blanket impls over disjoint towers — but two *parameterised* impls of
one trait can. `SampleUniform<FloatKind>` and `SampleUniform<UnsignedKind>` are distinct items, so
each is written once: `UniformFloat<T>` over `RealField + FromPrimitive`, `UniformUnsigned<T>` over
`RandUnsigned`, and one `SampleRange<T, K> for Range<T>` in place of five. `u8`, `u16` and `u128`
gained samplers they never had, and the `usize` sampler that drew from `next_u32` and so returned
only the bottom `2^32` of a wider range cannot recur: one body has nothing to disagree with.

**The `Rng` blanket proposed above was not done, and should not be.**
`impl<R: RngCore + ?Sized> Rng for R {}` exists to make a `&mut dyn RngCore` an `Rng`. That is a
trait object, and AGENTS.md forbids `dyn` in this workspace.

**Breaks.** None at a call site. The consumer edits the change made were bound simplifications.

### B8. `BFloat16`

**Done**, `191281cae`. Neither crate names it, and it draws.

It needs no implementation and no table row. The unit draw accumulates 53-bit words and stops when
the next would land below the scalar's own `epsilon`, so `BFloat16` takes one word because its
`epsilon` says one is enough, and a scalar added later takes however many it needs on the day it
arrives.

The half-open edge this row anticipated is real, and it bit in two distinct places, both measured
rather than argued.

- A unit draw can round onto exactly `1.0` at a narrow significand: 6 draws in 2 000 at
  `BFloat16`'s 8 bits, none at `f32` and wider. That leaves the interval every inverse-CDF
  transform assumes, so the rejection lives inside `RandScalar::rand_float_gen` where the promise
  is made, not in each caller.
- The affine map `u * scale + low` can round up onto `high` even when `u` is strictly below one:
  14 draws in 2 000 at `BFloat16`. `UniformFloat` therefore carries an `exclusive_high` and rejects
  a result that reaches it; `new_inclusive` leaves it `None`, because `high` is a legitimate result
  there.

Both counts are zero now at every scalar the probe test runs. Clamping was the alternative in both
cases and was rejected on measurement: it piles an atom of probability mass on one value.

### B9. `MaybeUncertain`

Under the generic tree, `MaybeUncertain<R>` is `Uncertain<Option<R>>` with `is_some` as a map
and `lift_to_uncertain` as a conditional against a threshold in `R`. Under the carrier it is
`Particles<Option<T>>` for free. The 306 lines across four files become one impl block. The alias
`MaybeUncertain<R>` stays as a type alias so CFD's twelve sites compile unchanged.

### B10. Consumers

| Consumer | Sites | Change |
|---|---|---|
| `deep_causality` core context nodes | 79 pinned to `f64` and `bool` | keep `UncertainF64`, `UncertainBool`, `MaybeUncertainF64` as aliases; `sample()` uses the default session |
| `deep_causality_cfd` | 14, already `Uncertain<R>` and `MaybeUncertain<R>` | none beyond the alias |
| `deep_causality_quantum` | 1 file | alias |
| three examples | constructors and `expected_value` | alias; one example gains a `Particles` demonstration |
| `rand` consumers | `topology`, `algorithms`, `physics`, `tensor`, `discovery`, `data_structures`, `ultragraph` | none; B7 is additive at the call site |

---

## 3. The target design

Three layers, each with one job.

```
rand        entropy: RngCore, Rng, Distribution<T>, StandardWord, StandardBool, Uniform,
            Sobol. Generic in the scalar over RandScalar and RandUnsigned. No haft.
            No density. In the tree today.
stats       the distributions and their densities, generic in the scalar over RandScalar.
            In the tree today.
uncertain   Uncertain<R>: the lazy graph. Builds distributions, draws every leaf from
            (seed, index, leaf id), drives QMC and adaptive testing, materialises
            particles. Arrow-shaped. No globals.
            SampleSession<R>: a value the caller owns; seed, counter, optional Sobol.
            Particles<T>: the strict carrier. Witness: Functor, Foldable, Pure,
            Applicative, Monad. Laws by equality. fmap fans out through scoped_map
            under the parallel feature; bounds are MaybeParallel, never Send + Sync.
linear,     Traversable on DenseVectorWitness and CausalTensorWitness, so a
tensor      container of Particles becomes Particles of a container.
```

The path a consumer walks:

```rust
type FloatType = f64;

let inflow: Uncertain<FloatType> = Uncertain::normal(lift(1.0), lift(0.05));
let viscosity: Uncertain<FloatType> = Uncertain::uniform(lift(0.9e-3), lift(1.1e-3));

let mut session = SampleSession::<FloatType>::seeded(7);
let inputs: DenseVector<Particles<FloatType>> = DenseVector::from_vec(vec![
    inflow.materialize(&mut session, 10_000),
    viscosity.materialize(&mut session, 10_000),
]);

// linear: Traversable. particles of a vector, not a vector of particles.
let per_particle: Particles<DenseVector<FloatType>> =
    DenseVectorWitness::sequence::<FloatType, ParticlesWitness>(inputs);

// one fmap runs the solve ten thousand times; stats reduces the result.
let drag: Particles<FloatType> = ParticlesWitness::fmap(per_particle, |v| solve(&v));
println!("{:.3e}", lower(drag.mean()));
```

Every type in that block takes the alias. Switch it to `Float106` and the draws, the solve and
the mean widen together, which is the claim the rest of the stack already makes and these two
crates cannot make today.

`Uncertain<R>` keeps its own composition as an `Arrow`, the way `calculus` composes `Diff`,
`Euler` and `Rk4`: `run(&self, session) -> R`, `compose`, `first`, `split`. That is static
dispatch over structs holding closures with whatever bounds they need, which is what a lazy graph
is. The `FmapOp`, `ApplyOp` and `BindOp` arms either become the `Arrow` combinators or are deleted;
they should not stay as dead arms.

---

## 4. Staging

Four changes, each shippable alone. Effort is the author's estimate for one engineer.

| Stage | Change | Breaks | Tests to carry | Effort |
|---|---|---|---|---|
| 1 | ~~`rand` generic cleanup~~ — **done**, `191281cae`. B7 and B8 by a crate boundary plus two blanket impls over a kind parameter. The `Rng` blanket was dropped: it needs `dyn` | none at call sites; two bounds in `topology` and one example got shorter | 112 in `rand`, the rest moved to `stats` | shipped |
| 2 | ~~`uncertain` precision as a parameter and the cache removal~~ — **done**, as change `renovate-uncertain`. B1, B3, B4, B5 and B9 all closed; `Uncertain<f32>` and `Uncertain<BFloat16>` both work and CFD compiles at `f32` | the public enum, the traits, the cache and seed functions, `SamplerKind`, and — not anticipated here — `Uncertain<bool>` itself | 304 in the crate, 1395 across the workspace | shipped |
| 3 | ~~`Particles<T>` and its witness~~ — **not built.** The carrier is a *type parameter* instead: `materialize::<W>` returns `W::Type<R>`, so the ensemble is a `DenseVector` or a rank-1 `CausalTensor` and this crate declares no container. `Collectable` in `haft` is the one trait that had to be added | none; additive | ensemble tests asserting values rather than counts | shipped |
| 4 | ~~Consumers~~ — **done**. The aliases went to `deep_causality` rather than staying in `uncertain` | none after the aliases | consumer suites | shipped |

Stage 2 before stage 3, because `Particles<R>` materialised from a graph that is still `f64`
under the hood would be generic in name only. Stage 3 is where the `Traversable` item M1 from the
archived note lands, and it needs nothing from stage 2 except the carrier's scalar.

---

## 5. What is not solved

- QMC resolution stays at 32 bits per coordinate (B6). `SobolSequence::coordinate<T>` now returns
  at the caller's scalar and states the cap at the item, rather than returning `f64` and leaving it
  implicit.
- A `Particles` field at simulation size does not fit in memory (B2, the arithmetic). The lazy
  graph is the answer and the note says how to use it.
- The zero-argument `sample()` the core context nodes call still needs an index and a seed from
  somewhere, and it takes them from `rand`'s thread RNG. That is the one global left in the
  path, it is `rand`'s and not `uncertain`'s, and a later change should thread a session through
  the context so that a causal model's draws are reproducible from one seed.
- `Uncertain<R>` gets no `haft` witness. That is a statement about lazy containers and the
  surface, not a deferral, and §2 B2 records why.

---

## 6. Reproducing

From the workspace root:

```bash
# the closed dispatcher and the arms nothing feeds
grep -n "enum SampledValue" -A8 deep_causality_unified_math/deep_causality_uncertain/src/types/cache/sampled_value.rs
grep -rn "PureOp\|FmapOp\|BindOp" deep_causality_unified_math/deep_causality_uncertain/src --include=*.rs

# the cache: root-only, never cleared, random index per draw, cfg(test) split
grep -n "global_cache\|GLOBAL_SAMPLE_CACHE" deep_causality_unified_math/deep_causality_uncertain/src/types/sampler/*.rs
grep -rn "\.clear()" deep_causality_unified_math/deep_causality_uncertain/src --include=*.rs
grep -n "next_sample_index" -A6 deep_causality_unified_math/deep_causality_uncertain/src/types/sampler/sampler_seed.rs
grep -n "cfg(test)\|cfg(not(test))" deep_causality_unified_math/deep_causality_uncertain/src/types/cache/global_cache.rs
grep -rn "with_global_cache\|seed_sampler\|clear_sampler_seed" deep_causality_unified_math/deep_causality_uncertain/tests --include=*.rs | wc -l

# the Send + Sync sites MaybeParallel replaces
grep -rn "Send + Sync" deep_causality_unified_math/deep_causality_uncertain/src --include=*.rs

# the f64 leaks
grep -rn "f64" deep_causality_unified_math/deep_causality_uncertain/src --include=*.rs | grep -v "Float106\|uncertain_f64\|f64_probabilistic"

# the haft signatures the retrofit has to satisfy
for f in functor/functor_base.rs pure/mod.rs applicative/mod.rs monad/mod.rs traversable/mod.rs; do
  awk '/^pub trait/{p=1} p' deep_causality_unified_math/deep_causality_haft/src/$f | grep -v "^\s*///" | head -12
done

# rand names no concrete scalar: this prints nothing
grep -rn "for f32\|for f64\|for Float106\|for BFloat16\|for u8\|for u16\|for u32\|for u64\|for u128\|for usize" \
  deep_causality_unified_math/deep_causality_rand/src --include=*.rs

# the f64 that remain: nine lines, each a conversion pivot or a panic message
grep -rn "f64" deep_causality_unified_math/deep_causality_rand/src --include=*.rs \
  | grep -v "^[^:]*:[0-9]*: *//"

# the two blanket impls and the kind parameter that keeps them distinct
grep -rn "SampleUniform<" deep_causality_unified_math/deep_causality_rand/src --include=*.rs

# a scalar the crate never names, drawn and asserted
cargo test -p deep_causality_rand --test mod parity_probe

# consumer spellings
grep -rhoE "(Maybe)?Uncertain<[A-Za-z0-9_]+>|UncertainF64|UncertainBool|MaybeUncertainF64" \
  deep_causality/src deep_causality_cfd/src deep_causality_quantum/src examples/causal_uncertain_examples --include=*.rs | sort | uniq -c
```
