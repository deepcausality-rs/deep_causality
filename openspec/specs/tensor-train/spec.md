# tensor-train Specification

## Purpose
TBD - created by archiving change add-tensor-network. Update Purpose after archive.
## Requirements
### Requirement: Tensor-train state type and trait

The crate SHALL provide a concrete type `CausalTensorTrain<T>` (private fields, accessed through getters)
storing a chain of rank-3 cores of shape `[r_k, n_k, r_{k+1}]` with boundary bonds equal to 1, and a trait
`TensorTrain<T>` (in `src/traits/`) declaring its behavior, mirroring the `Tensor`/`CausalTensor` split.
All operations SHALL use static dispatch.

#### Scenario: Core and boundary invariants
- **WHEN** a `CausalTensorTrain` is constructed by any constructor
- **THEN** adjacent cores share bond dimensions, the left and right boundary bonds are 1, and the cached
  `phys_dims` equal the cores' middle dimensions

#### Scenario: Introspection getters
- **WHEN** `cores`, `order`, `phys_dims`, `bond_dims`, `max_bond`, or `canonical_form` is queried
- **THEN** the returned metadata reflects the current factorization

### Requirement: TT-SVD construction from a dense tensor

`CausalTensorTrain::from_dense` SHALL factor a dense `CausalTensor<T>` into a tensor train via a
left-to-right truncated-SVD sweep under a `Truncation<T>`, producing a canonical train.

#### Scenario: Exact recovery at sufficient bond
- **WHEN** a tensor of exact TT-rank `r` is converted with a bond cap `≥ r`
- **THEN** `to_dense` of the result reproduces the original to `‖·‖ ≤ tol`

#### Scenario: Truncation error tracks the discarded tail
- **WHEN** the bond cap is below the exact rank
- **THEN** the reconstruction error is bounded by the norm of the discarded singular values

### Requirement: Closure and special constructors with guards

The type SHALL provide `from_fn` (build from an index→value closure), `zeros`, `ones`, and a dev-seeded
`random_seeded`. Constructors that would materialize a dense buffer SHALL be guarded by an element-count
cap and fail with `RankExceeded` rather than allocate `nᵈ`.

#### Scenario: Guard prevents silent blow-up
- **WHEN** `from_fn` (or `to_dense`) is asked for a configuration exceeding the element-count cap
- **THEN** it returns `CausalTensorError::RankExceeded`

### Requirement: Canonical forms and rounding

The trait SHALL provide left/right/mixed canonicalization (QR-based gauge sweeps that update the tracked
`CanonicalForm`) and a `round(&Truncation)` SVD recompression. Rounding SHALL be idempotent at a fixed
policy.

#### Scenario: Canonical gauge established
- **WHEN** `canonicalize_at(k)` is called
- **THEN** cores left of `k` are left-orthonormal, cores right of `k` are right-orthonormal, and
  `canonical_form` reports the center at `k`

#### Scenario: Rounding reduces bond without exceeding tolerance
- **WHEN** `round` is applied to a train with redundant bond dimension
- **THEN** bond dimensions do not exceed the policy and the represented tensor changes by at most the
  truncation tolerance

### Requirement: Norm, inner product, and exact linear algebra

The trait SHALL provide `norm`, `inner`, `add`, `scale`, `add_scalar`, and `hadamard`. `add` and
`hadamard` SHALL be exact (growing bond by sum / product respectively) and SHALL be paired with
`*_rounded(&Truncation)` recompressing variants. `scale` SHALL be exact and rank-preserving; `add_scalar`
SHALL realize an exact affine offset via a rank-1 ones-train.

#### Scenario: Operations match the dense computation
- **WHEN** `add`, `scale`, `add_scalar`, `hadamard`, `inner`, or `norm` is computed on small trains
- **THEN** the result equals the corresponding dense computation to `‖·‖ ≤ tol`

#### Scenario: Inner and norm use the canonical center
- **WHEN** `inner`/`norm` is computed on a mixed-canonical train
- **THEN** the result is obtained from the orthogonality center without a full re-contraction

### Requirement: Marginalization, evaluation, and dense contraction

The trait SHALL provide `marginalize` (sum out a subset of sites), `eval` (single-entry evaluation without
materializing dense), and a guarded `to_dense`.

#### Scenario: Marginalize matches dense reduction
- **WHEN** `marginalize(sites)` is applied
- **THEN** the result equals dense `sum_axes(sites)` to `‖·‖ ≤ tol`

#### Scenario: Single-entry evaluation
- **WHEN** `eval(index)` is called for a valid multi-index
- **THEN** it returns the logical entry `A[index]` without forming the dense tensor

### Requirement: QTT reshape helpers

The crate SHALL provide quantization helpers reshaping a physical axis of length `n = 2^L` to `L` binary
sites in big-endian (coarse-to-fine) order and back, as a generic primitive (multi-axis interleave left to
the caller).

#### Scenario: Quantize/dequantize round-trip
- **WHEN** an axis of length `2^L` is quantized then dequantized
- **THEN** the original tensor is recovered exactly

### Requirement: Algebra-trait impls

`CausalTensorTrain<T>` SHALL implement `Module<T>` (via exact `scale`), `AbelianGroup`/`AddGroup` (via
`add`), and `Ring` (via `hadamard`) through marker traits plus the `deep_causality_num` blanket impls,
mirroring `CausalTensor`. It SHALL NOT implement `Field`/`InvMonoid` (a tensor train has no multiplicative
inverse). Group/ring laws SHALL hold exactly without truncation and to tolerance under rounding.

#### Scenario: Module scaling is exact
- **WHEN** a train is scaled by a scalar through the `Module` interface
- **THEN** the represented tensor is exactly scaled and bond dimensions are unchanged

#### Scenario: Lax laws under truncation
- **WHEN** an algebra law (e.g. distributivity) is checked on rounded results
- **THEN** it holds to the truncation tolerance

### Requirement: Storage HKT witness

The crate SHALL provide `CausalTensorTrainWitness` implementing `Functor`, `Foldable`, and `Pure` over
**core storage** (the scalar type of the cores). It SHALL NOT implement `Monad`, `CoMonad`, or
`Applicative` for the tensor train.

#### Scenario: fmap converts the scalar type structurally
- **WHEN** `fmap` is applied with a scalar conversion `A → B`
- **THEN** every core entry is converted and the bond/physical structure is unchanged

#### Scenario: Functor laws on cores
- **WHEN** identity and composed maps are applied via `fmap`
- **THEN** the functor identity and composition laws hold on the core storage

### Requirement: Error variants for tensor-train operations

The existing `CausalTensorError` enum SHALL be extended with `BondDimensionMismatch`, `NotCanonical`, and
`RankExceeded`, each with a `Display` arm. No second error type SHALL be introduced.

#### Scenario: Mismatched bonds rejected
- **WHEN** a train is assembled from cores whose adjacent bond dimensions disagree
- **THEN** validation returns `CausalTensorError::BondDimensionMismatch`

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

