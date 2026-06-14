## Context

The uncertain crate is a Monte-Carlo sampling engine over a `ConstTree`-backed
computation graph. Its precision floor is f64 at three depths:

1. **`deep_causality_rand`** — `StandardNormal`/`StandardUniform`/`Bernoulli` produce f64
   (f32 by narrowing). But the crate is *already* built on the `Distribution<T>` and
   `SampleUniform`/`UniformSampler` trait pattern, so additional value types are new impls,
   not a redesign. `RngCore::next_u64` is the raw entropy source and is type-agnostic.
2. **`SampledValue`** — `enum { Float(f64), Bool(bool) }`, the leaf carried by every node
   of `UncertainNodeContent`, by the fmap/bind closures, by the cache, and by the sampler.
   This is the pervasive f64.
3. **`Uncertain<T>` / `MaybeUncertain<T>`** — `T: ProbabilisticType` exists as the boundary
   trait but is `PhantomData`; only f64/bool are instantiated.

The platform precedent is `generalize-physics-over-realfield` /
`generalize-topology-over-realfield`: parameterize over `R: RealField`, keep the concrete
instantiations working, add the precision targets. This change applies the same recipe.

## Goals / Non-Goals

**Goals**
- `Uncertain<R>` / `MaybeUncertain<R>` for `R: RealField`, with `R`-native sampling.
- Lossless precision on the certain-value and deterministic-arithmetic paths.
- f64 behavior preserved bit-for-bit; existing tests and downstream f64 consumers untouched.
- An honest, documented sampling boundary: random draws are MC-bounded.

**Non-Goals**
- Reducing Monte-Carlo variance at higher precision (impossible; set by `N`).
- A new RNG algorithm. The `Xoshiro256` core and `next_u64` entropy are unchanged.
- Deterministic (interval/affine) uncertainty arithmetic — a different computational model,
  out of scope.
- Changing any public f64 signature or alias.

## Decisions

### D1: Extend `deep_causality_rand` through its existing trait pattern
Add `SampleUniform` + `UniformFloat` and `Distribution<R> for {StandardUniform,
StandardNormal}` impls for the precision targets, plus `R`-typed `Bernoulli` thresholds.
No change to `RngCore` / `Xoshiro256`. The f64 and f32 impls are left exactly as they are
(bit-identical regression gate D7).

### D2: Double-double uniform `[0,1)` from two draws
`Float106` uniform is assembled as `hi + lo · 2⁻⁵³` where `hi` is a standard 53-bit f64
uniform in `[0,1)` and `lo` is a second independent 53-bit f64 uniform — giving ~106 bits
of mantissa entropy. Normal draws use Box–Muller with `Float106` `sqrt`/`ln`/`cos` (all on
`RealField`). This is a known construction; it is what makes the `Float106` draw *honest*
(genuinely double-double bits) rather than an f64 value widened to double-double.

### D3: `SampledValue<R>` carries the precision; the graph follows
`SampledValue<R> = { Float(R), Bool(bool) }`. `UncertainNodeContent<R>`, the
`SampledFmapFn<R>`/`SampledBindFn<R>` closure traits, `DistributionEnum<R>` and its three
param structs, `SequentialSampler`, `sprt_eval`, and `GlobalSampleCache` all carry `R`.
This is the large mechanical edit; it is bounded (the crate is ~40 files) and purely
type-threading — no algorithm changes.

### D4: `ProbabilisticType` stays the open boundary; ship three float impls
`ProbabilisticType: IntoSampledValue<R> + FromSampledValue<R> + Clone + Send + Sync +
'static` (the `Send + Sync + 'static` already present; `R: RealField` satisfies them).
Ship concrete impls for `f64`, `f32`, `Float106` (and keep `bool` for the presence
channel). The construction/sampling surface stays **generic over `R: RealField` via the
trait** rather than a closed enum of the three floats, so a future flow/field value type
can implement `ProbabilisticType` and ride the engine unchanged — the three floats are the
shipped instantiations, not a ceiling (resolved decision 1). The `is_present` channel of
`MaybeUncertain<R>` remains `Uncertain<bool>` — presence is a Bernoulli fact independent of
value precision.

### D5: SPRT and the sequential sampler are precision-agnostic in logic, `R` in thresholds
The Wald sequential test compares running proportions / means against thresholds; the
control flow is `R`-independent. Thresholds, accumulators, and the `lift_to_uncertain`
gate parameters (`threshold_prob_some`, `confidence_level`, `epsilon`) become `R`. The
presence test still operates on the `Uncertain<bool>` channel.

### D6: The documented sampling boundary (honesty requirement)
A module-level doc comment states plainly: the random *draw* is Monte-Carlo-bounded, so
`Uncertain<Float106>` does not reduce sampling variance versus `Uncertain<f64>`; the value
of precision-genericity is (a) lossless certain-value and post-sample arithmetic
propagation and (b) removal of the platform's `R → f64` cast island. This mirrors the
accepted SURD `R → f64` boundary documented in `3DCausalFluidDynamics.md` §2/§7 — the
program already reasons this way about sampling/IT precision.

### D7: f64 preserved bit-for-bit (the de-risking decision)
`UncertainF64 = Uncertain<f64>`, `MaybeUncertainF64 = MaybeUncertain<f64>`, `UncertainBool`,
and every existing public method keep their exact signatures. A regression battery asserts
that representative f64 computations (distributions, arithmetic, comparison, SPRT, `lift`)
produce bit-identical results before and after the parameterization, under the same seed.
This is what keeps `deep_causality`'s `data_uncertain_*` nodes and the three examples
untouched.

### D8: Stage 4 consumes `MaybeUncertain<R>` after this lands
`add-cut-cells-and-immersed-boundaries` Group C is sequenced after this change; its
inflow zone uses `MaybeUncertain<R>` with `R` = the solver's precision, calling
`lift_to_uncertain` at the sensor patch and collapsing to `R` for assembly with no cast.

## Risks / Trade-offs

- **Pervasiveness of the `SampledValue<R>` thread (D3).** Touches most of the crate.
  Mitigated by D7's bit-identical f64 gate: the change is type-threading with a hard
  behavioral anchor, not a logic rewrite.
- **The `Float106` RNG construction (D2)** must be tested for uniformity/independence of
  the low part. A statistical test (mean/variance/KS against the analytic uniform and
  normal) on the double-double draws is part of the rand spec's acceptance.
- **Over-claiming precision.** Mitigated by D6's mandatory honesty doc and by *not*
  adding any test that asserts variance reduction at higher precision (there is none).
- **Scope vs. payoff.** This is real work for a payoff that is architectural (cast-island
  removal, consistency) plus the two genuinely-lossless paths — not a numerical-accuracy
  win in sampling. Accepted on the user's direction to make precision a true parameter
  end-to-end before Stage 4 builds on it.

## Resolved decisions (review, 2026-06-14)

1. **Precision target set — f64 + f32 + Float106, behind a generic constructor.** All three
   precisions ship as concrete `ProbabilisticType` impls. Crucially, the construction and
   sampling surface stays *generic* over `R: RealField` (the trait, not a closed enum of
   the three floats), so future flow/field value types can implement `ProbabilisticType`
   and flow through the engine without touching it. The three floats are the shipped
   instantiations, not a hard ceiling — future-proofing for new value types is explicit.
2. **`GlobalSampleCache` keying — generic over `R`.** The cache is parameterized over `R`
   (per-`R` storage) rather than keyed by a precision-tagged f64 value; this keeps the
   sample-cache type-honest with the rest of the `R`-threaded engine.
3. **`Display` of `SampledValue<Float106>` — a practical single-decimal pretty-print.**
   Consumer-facing output renders the composite double-double as a single decimal value
   (not both limbs); the two-limb form is reserved for `Debug`. Follows the `Float106
   Display` convention already in `deep_causality_num`.
