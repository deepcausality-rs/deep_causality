# uncertain-realfield-generic Specification

## Purpose
`Uncertain` and `MaybeUncertain` must be precision-parametric over `R: RealField`, removing
the f64 island that forced a lossy `R → f64` cast at every boundary with the rest of the
`RealField`-generic platform. The two paths that genuinely carry precision — certain values
and deterministic arithmetic on sampled values — must propagate `R` losslessly; random
draws remain Monte-Carlo-bounded by design and are documented as such. Internally a closed
`SampledValue` dispatcher (`{ Float(f64), DoubleFloat(Float106), Bool(bool) }`) keeps the
global sample cache, the computation graph, and the sampler non-generic (Rust has no generic
statics), while the public `Uncertain<R>` / `MaybeUncertain<R>` surface stays generic. f64
behavior is preserved bit-for-bit, and constructors stay source-compatible (a single generic
impl, so `Uncertain::normal(0.0, 1.0)` still infers f64). Unblocks the CFD Stage-4
`MaybeUncertain<R>` inflow zone.

## Requirements
### Requirement: Precision-generic uncertain types
`Uncertain<R>` and `MaybeUncertain<R>` SHALL be generic over `R: RealField`, with the
computation graph (`SampledValue<R>`, `UncertainNodeContent<R>`, the fmap/bind closure
traits), the distribution carriers, the sequential sampler, the SPRT evaluator, and the
sample cache all carrying `R`. Random draws SHALL use the crate's `R`-native distribution
sampling. The presence channel of `MaybeUncertain<R>` SHALL remain `Uncertain<bool>`.

#### Scenario: Uncertain is instantiable at multiple precisions
- **WHEN** an `Uncertain<R>` is built from a normal distribution at `R ∈ {f64, Float106}` and sampled
- **THEN** the sample is of type `R` and is drawn through the `R`-native distribution path, not via an f64 round-trip

### Requirement: Lossless precision on the certain and arithmetic paths
A certain value SHALL propagate `R` without narrowing, and deterministic arithmetic on
sampled values SHALL compose at full `R` precision. The crate SHALL document, at the
sampling boundary, that random draws are Monte-Carlo-bounded — higher precision does not
reduce sampling variance — so that the precision-genericity claim is not over-stated.

#### Scenario: Certain Float106 value round-trips without loss
- **WHEN** `MaybeUncertain::<Float106>::from_value(x)` is created for a `Float106` `x` whose low limb is non-zero and is read back through the present-value path
- **THEN** the recovered value equals `x` exactly, with no narrowing through f64

#### Scenario: Sampling boundary is documented as MC-bounded
- **WHEN** the crate documentation is inspected
- **THEN** it states that random draws are Monte-Carlo-bounded and that the value of precision-genericity is lossless certain/arithmetic propagation plus removal of the `R → f64` cast island, not reduced sampling variance

### Requirement: f64 behavior preserved bit-for-bit
The crate SHALL preserve existing f64 behavior bit-for-bit. The `UncertainF64 =
Uncertain<f64>`, `MaybeUncertainF64 = MaybeUncertain<f64>`, and `UncertainBool` aliases and
every existing public method MUST retain their signatures and numerical behavior; existing
f64 computations MUST produce bit-identical results under a fixed seed before and after the
parameterization; and downstream f64 consumers MUST require no source changes.

#### Scenario: Existing f64 computation is unchanged
- **WHEN** a representative f64 pipeline (distribution construction, arithmetic, comparison, SPRT, `lift_to_uncertain`) runs under a fixed seed before and after this change
- **THEN** every result is bit-identical and the public f64 API is source-compatible

