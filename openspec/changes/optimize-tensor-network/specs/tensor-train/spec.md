## ADDED Requirements

### Requirement: Adaptive randomized rounding policy

The `round` operation SHALL support an **opt-in adaptive randomized rounding** strategy selectable
through the `Truncation` policy, in addition to the default deterministic truncated-SVD rounding, and
the same strategy SHALL apply to the `add_rounded` / `hadamard_rounded` variants and the AMEn
enrichment recompression. The randomized
strategy SHALL use the randomize-then-orthogonalize scheme with Khatri-Rao-structured Gaussian sketches
(applied core-by-core without forming the full sketch) and SHALL choose the retained rank adaptively
from the `Truncation` tolerance via a residual estimator. Randomness SHALL come from the crate's
existing deterministic seeded generator (no new external dependency). The deterministic strategy SHALL
remain the default.

#### Scenario: Randomized rounding matches deterministic to tolerance
- **WHEN** a train is rounded with the randomized policy at relative tolerance `τ`
- **THEN** the result reproduces the deterministic-rounded train (and the original, for a train of true
  rank within the cap) to within `τ` in the represented tensor

#### Scenario: Default rounding is unchanged
- **WHEN** `round` is called with a `Truncation` that does not select the randomized strategy
- **THEN** the deterministic truncated-SVD rounding is used and its result is bit-for-bit the prior
  behaviour

#### Scenario: Adaptive rank tracks the tolerance
- **WHEN** the randomized policy rounds a train whose true rank exceeds the seed sketch size
- **THEN** the sketch is grown until the estimated residual is at or below the tolerance, and the
  retained bond dimension matches the deterministic rank for that tolerance (up to the oversample)

#### Scenario: Asymptotically cheaper, benchmarked against deterministic
- **WHEN** the benchmark suite rounds a train under both the deterministic and randomized policies
- **THEN** both timings are recorded; the randomized kernel is `O(d·n·r²·ℓ)` versus the deterministic
  `O(d·n·r³)`, so it wins only once the rounding unfoldings are large and low-rank — at small bench
  scales it is on par with or slightly slower than deterministic, which the benchmark documents
  honestly (the deterministic kernel therefore stays the default)

### Requirement: Fused Hadamard-then-truncate

The crate SHALL provide a fused Hadamard-product-then-round path so that `hadamard_rounded` does not
materialize the full bond-`r²` intermediate **train** before compressing. The fused path SHALL build
and left-orthonormalize the squared-bond cores one site at a time — holding at most a single
squared-bond core, not the whole squared train — and its result SHALL equal `hadamard(other).round(trunc)`
to within the truncation tolerance.

#### Scenario: Fused result matches build-then-round
- **WHEN** `hadamard_rounded` is computed on two trains
- **THEN** the represented tensor matches `hadamard(other).round(trunc)` to the truncation tolerance,
  while only one squared-bond core (not the full `d`-core squared train) is materialized at a time

### Requirement: Allocation-reusing transfer-matrix contractions

The `inner` and `norm` transfer-matrix contractions SHALL reuse scratch buffers across sites rather
than allocating per site, preserving their results exactly. This is a constant-factor optimization with
no observable behavioural change.

#### Scenario: Results unchanged after buffer reuse
- **WHEN** `inner`/`norm` are computed before and after the buffer-reuse change
- **THEN** the returned scalars are identical (bit-for-bit for the same inputs), at every supported
  scalar type
