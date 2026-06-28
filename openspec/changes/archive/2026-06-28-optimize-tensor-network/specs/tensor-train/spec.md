## ADDED Requirements

### Requirement: Adaptive randomized rounding policy

The `round` operation SHALL support an **opt-in adaptive randomized rounding** strategy selectable
through the `Truncation` policy, in addition to the default deterministic truncated-SVD rounding, and
the same strategy SHALL apply to the `add_rounded` / `hadamard_rounded` variants and the AMEn
enrichment recompression. For TT `round` the randomized strategy SHALL use the
**randomize-then-orthogonalize** scheme (sketch the train with a structured Khatri-Rao Gaussian
contraction, then orthogonalize only the small sketched basis — never canonicalizing the full high-bond
train), so its cost is `O(d·n·r²·ℓ)`. The randomized
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

#### Scenario: Randomized TT round is faster on large compressible trains
- **WHEN** the study suite rounds a sum of `k` copies of a low-rank train (large input bond, low output
  rank) under both strategies
- **THEN** the randomize-then-orthogonalize round is faster, and increasingly so as the input bond
  grows — measured 1.1× / 6.6× / 33× / 58× at input bond 24 / 48 / 96 / 144 (its time stays ~flat at
  4–6 ms while deterministic grows cubically), matching the 20–50× the literature reports for rounding
  sums of TT-tensors; the deterministic kernel stays the default and both agree to tolerance

#### Scenario: Direct large-matrix SVD speedup
- **WHEN** `svd_truncated` with the randomized strategy factors a large low-rank matrix directly
- **THEN** it is far faster than the deterministic Jacobi SVD (measured 38×–935× at 100²–1000², rank 20),
  the gap widening with size because Jacobi is `O(S³)` while the range-finder is `O(S²·ℓ)`

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
