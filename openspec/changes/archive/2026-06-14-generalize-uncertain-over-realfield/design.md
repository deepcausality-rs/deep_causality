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

### D1: Generalize `deep_causality_rand` from `Float` to `Real` (precision-as-a-parameter)
`deep_causality_rand` predates the num crate's `Real` / `RealField` separation and is
over-coupled to `Float` — `Normal<F: Float>`, `UniformFloat<F: Float>`. But the distribution
*math* never needs `Float`: `Normal::new` / `sample_from_zscore` use only `is_finite` +
arithmetic, and `UniformFloat` uses `is_finite` / `one` / `epsilon` / arithmetic — all
declared on `Real` (`impl<T: Float> Real for T`). Re-bound the consumer-facing wrappers
`Float → Real`: `Normal<F: Real>`, `UniformFloat<F: Real + RandFloat>`. The **only**
genuinely `Float`-level seam is `RandFloat::rand_float_gen` (mantissa-bit assembly), which
stays a per-type impl — the legitimate home for the "specific implementation detail" the
num-crate paradigm keeps low. `Xoshiro256` / `RngCore` and the f32 / f64 paths are
untouched (bit-identical regression gate D7). A small `RealRng` convenience bound
(`Real + SampleUniform` with `StandardNormal: Distribution<Self>`, blanket-impl'd for every
qualifying type) lets downstream thread a single bound instead of three.

### D1a: Why this matters — the forward-compat mechanism
Because `RealField` is `impl<T: Float> RealField for T` (Float ⇒ RealField, never the
reverse), a `RealField` consumer cannot reach a `Float`-bounded API without illegally adding
`R: Float`, which would re-couple every downstream crate to the bit-level trait. Generalizing
rand over `Real` is what lets `deep_causality_uncertain` (and any future consumer) abstract
over `RealField` only. A new float type then needs `impl Float` (num) + the rand per-type
seam; uncertain and all other downstream code are untouched.

### D2: Double-double uniform `[0,1)` from two draws
`Float106` uniform is assembled as `hi + lo · 2⁻⁵³` where `hi` is a standard 53-bit f64
uniform in `[0,1)` and `lo` is a second independent 53-bit f64 uniform — giving ~106 bits
of mantissa entropy. Normal draws use Box–Muller with `Float106` `sqrt`/`ln`/`cos` (all on
`RealField`). This is a known construction; it is what makes the `Float106` draw *honest*
(genuinely double-double bits) rather than an f64 value widened to double-double.

### D3: A closed `SampledValue` enum is the precision dispatcher (revised 2026-06-14)
The first plan threaded `R` through the whole engine (`SampledValue<R>`,
`UncertainNodeContent<R>`, …). That foundered on the **global sample cache**: it lives in a
`static` (`OnceLock`/`thread_local`), and Rust has no generic statics, so `SampledValue<R>`
would force a `TypeId`-keyed `Any` registry.

Revised, simpler design (chosen): make `SampledValue` a **closed enum carrying the
precision as variants** — `{ Float(f64), DoubleFloat(Float106), Bool(bool) }` — so it is
precision-complete without a type parameter. Then the engine internals stay **non-generic**:
the cache stays a plain `static` (its role is only top-level `sample_with_index(i)`
reproducibility, keyed by `(id, index)`; within-expression correlation is the sampler's
local memo map), `UncertainNodeContent` and `SequentialSampler` keep their shapes, and the
operators dispatch per variant (`ArithmeticOperator::apply` made generic over `RealField`,
called on the matched variant's inner type). Only the boundary types — `Uncertain<R>`,
`MaybeUncertain<R>`, `DistributionEnum<R>` and its params — carry `R`, converting `R ↔
SampledValue` through the per-type `ProbabilisticType` impls.

This delivers the two load-bearing precision paths: **certain values** (`Value`/`PureOp`
carry `DoubleFloat(Float106)` directly) and **deterministic arithmetic** (`ArithmeticOp`
dispatches per variant, applying at full precision). `ComparisonOp.threshold` is promoted
from `f64` to `SampledValue` so comparisons keep precision; the user-supplied `.map()`
closures (`FunctionOpF64: Fn(f64)->f64`) stay f64-typed — a narrow, documented boundary
(mirroring the accepted SURD `R→f64` boundary), generalizable as a follow-up.

Trade-off vs. the fully-generic plan: `SampledValue` is a closed set, so a new float type
adds a variant + a `ProbabilisticType` impl (both in this crate). The **public** API
(`Uncertain<R>`/`MaybeUncertain<R>`) stays generic over `RealField`; only the internal
dispatcher enum is closed. No generic statics, no `Any`, far less churn than the 40-file
thread.

### D4: `ProbabilisticType` stays the open boundary; ship three float impls
`ProbabilisticType: IntoSampledValue<R> + FromSampledValue<R> + Clone + Send + Sync +
'static` (the `Send + Sync + 'static` already present; `R: RealField` satisfies them).
Ship concrete impls for `f64`, `f32`, `Float106` (and keep `bool` for the presence
channel). The **public** construction/sampling surface stays generic over `R: RealField`
via the trait, so downstream code abstracts over `RealField`; internally each impl maps its
type to/from the closed `SampledValue` dispatcher variant (D3). Adding a future value type
is a `ProbabilisticType` impl plus a `SampledValue` variant, both local to this crate — the
public API is untouched. The `is_present` channel of `MaybeUncertain<R>` remains
`Uncertain<bool>` — presence is a Bernoulli fact independent of value precision.

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
