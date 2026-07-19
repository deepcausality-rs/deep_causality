# Tensor-network acceleration — survey the SOTA *before* optimizing

**Status.** Forward-looking engineering note, 2026-06-30. Gating discipline for any future
performance work on `deep_causality_tensor` (tensor networks / tensor trains, QTT, MPS/MPO, AMEn).

---

## The rule

**No parallelism, SIMD, or GPU work on the tensor crate begins until a literature survey of the
state of the art in tensor-network acceleration has been done and written up here.** The order is
*correct → measure → survey → optimize*, not *correct → optimize*. The survey is not optional
diligence; it is what tells us *which* operation to optimize. Skipping it risks pouring effort into
the kernel that looks expensive (dense contraction / GEMM) while the real cost sits elsewhere.

## Why the survey must come first (the bottleneck is non-obvious)

Naive intuition says "tensor-train contraction is batched GEMM, GEMM loves GPUs, so port the
contraction." That is probably the *wrong* first target:

- **Truncation, not contraction, is often the hot spot.** Every contraction in a TT/MPS sweep is
  followed by an SVD/QR that re-orthogonalizes and re-ranks. SVD parallelizes far less cleanly than
  GEMM and frequently dominates wall-clock (an Amdahl ceiling). Randomized / truncated SVD and
  structured re-orthogonalization are their own research thread.
- **Sweeps are partly sequential.** DMRG/AMEn rank-adaptation carries state site-to-site, capping
  naive data-parallelism. The literature on *parallel* DMRG (real-space decomposition, etc.) exists
  precisely because the obvious parallelization does not work.
- **Contraction *order* can beat hardware.** For tensor *networks* (not just trains), choosing the
  contraction path is often a larger lever than the per-contraction kernel — a hypergraph
  optimization problem with a mature toolchain. The right path can change the asymptotic cost; no
  amount of GPU saves a bad order.
- **The project's own finding tempers the goal.** Per [`../cfd/tensor_network_cfd.md`](../archive/cfd/tensor_network_cfd.md)
  §0, at the project's Reynolds numbers TN compression buys *accuracy affordability*, not raw speed,
  and any wall-clock advantage appears only above Re ≈ 9.5×10³. So "make the tensor backend fast"
  must be aimed at the regimes/operations where speed actually matters, which the survey defines.

## Measured corroboration — the SRP compressible marcher (2026-07-17)

The SRP momentum-jet de-risk (`deep_causality_cfd/studies/srp_momentum_jet/`;
[`../cfd-plasma-retropulsion/derisk-verdict.md`](../cfd-plasma-retropulsion/derisk-verdict.md)
addendum) produced a first **measured** profile of the 2-D compressible QTT marcher, and it
confirms this note's central thesis (truncation-dominated, contraction is not the target).
Directly answers Deliverable #1 for one real marcher, though a profiler run still owes the
exact percentages:

- **Where the step's time goes.** Each `CompressibleMarcher2d` step (`marcher_2d.rs`):
  dequantizes all four conserved components to dense, evaluates the flux/EOS pointwise, then
  **re-quantizes eight flux fields — a TT-SVD each** — and passes the result through the
  gradient + closed-form acoustic-inverse chain, which carries **~20 cap-limited rounding
  passes** per step (each an SVD-truncation).
- **Two consequences.** (a) The **nonlinear round-trip defeats the TT asymptotics**: the
  dequantize→dense→requantize path is dense `O(n²)` every step, so the compression buys no
  step-time asymptotic at this scale (consistent with the Re-crossover finding above — QTT is
  accuracy-affordability here, not raw speed). (b) **Rounding dominates the TT side** — exactly
  the "truncation, not contraction, is the hot spot" claim, now with a marcher behind it.
- **Compression is not the accuracy limit either.** Bond cap 24 → 32 (exact at 2⁵ — truncation
  off) left every drag observable unchanged to displayed precision. So on this workload the
  rounding is a *cost* concern, not a *correctness* one.
- **The scaling wall.** Going up one `L` doubles the steps (convective CFL) and raises per-step
  cost; the domain-widened 2⁷–2⁸ runs the SRP collapse needs sit near **a hundred single-core
  hours** today — the concrete number the acceleration work exists to attack.

The two levers this most directly motivates are already anchored in the seed bibliography:
TT **cross-interpolation** for the nonlinear flux (removes the dense round-trip — cf. the
TT-cross entry, [arXiv:2407.11290](https://arxiv.org/abs/2407.11290)) and **randomized/sketched
TT rounding** for the dominant truncation cost (cf. the randomized-SVD entries). Both stay
behind the survey rule below; the CFD-*solver*-level levers that need no tensor-crate change
(fused rounding, component/roster `scoped_map`, local dissipation scaling) are recorded in the
CFD roadmap and are not gated by this note. Full ladder, with expected magnitudes and the
CFD-vs-tensor-crate split:
[`../cfd-roadmap/cfd-industry-scaling.md`](../cfd-roadmap/cfd-industry-scaling.md) §4.

## What the survey should cover (candidate starting points — to be verified, not assumed)

Anchor the search here; treat every claim as unverified until checked against the source:

- **GPU contraction frameworks:** NVIDIA cuQuantum / `cuTensorNet` (tensor-network contraction +
  path optimization on GPU); `cuTENSOR` for the primitive contractions; `cuSOLVER` batched SVD/QR.
- **Contraction-path optimization:** `opt_einsum`, `cotengra` (hyper-optimized contraction trees) —
  the algorithmic lever above the kernel.
- **TT/MPS-specific parallelism:** parallel and real-space DMRG; parallel AMEn; shared- vs
  distributed-memory sweep schemes; where the rank-adaptive sequential dependency actually binds.
- **Truncation acceleration:** randomized SVD, Nyström / sketching for low-rank re-compression,
  GPU batched small-matrix SVD performance characteristics.
- **Mature backends to learn from (not necessarily depend on):** ITensor / ITensorNetworks, `quimb`,
  Google `TensorNetwork`, JAX/XLA-backed TT libraries — read for *where they put the speedups* and
  what they report as the bottleneck.
- **Quantics/QTT-specific:** any acceleration work particular to the quantics encoding the CFD
  marchers use, since the rank/structure profile differs from generic MPS.

## Seed bibliography (arXiv / publication survey, 2026-06-30)

Entry points found via an arXiv/web sweep, grouped by the lever each informs. **Speedup figures are
as-reported in abstracts — verify against the source before relying on any number.** This is the
*seed* for the mandated survey, not the survey itself.

### Contraction-order optimization — often the largest lever, and needs no new hardware
- Gray & Kourtis, *Hyper-optimized tensor network contraction*, [arXiv:2002.01935](https://arxiv.org/abs/2002.01935)
  (Quantum 5, 410, 2021). The `cotengra` method; paths "orders of magnitude" better than established
  approaches (claimed ~10⁴× on Sycamore-class circuits). Read first — a good order can change the
  asymptotic cost no GPU can recover.
- Gray & Chan, *Hyper-optimized approximate contraction of tensor networks with arbitrary geometry*,
  [arXiv:2206.07044](https://arxiv.org/abs/2206.07044). Extends the above to approximate contraction.
- `cotengra` ([github.com/jcmgray/cotengra](https://github.com/jcmgray/cotengra)) — hypergraph
  partitioning (KaHyPar) for contraction trees; reference implementation to study.

### Truncation / batched SVD–QR — the bottleneck flagged above (Amdahl ceiling)
- Boukaram, Turkiyyah, Ltaief, Keyes, *Batched QR and SVD Algorithms on GPUs with Applications in
  Hierarchical Matrix Compression*, [arXiv:1707.05141](https://arxiv.org/abs/1707.05141) (Parallel
  Computing 2017). One-sided Jacobi; ~20× over `cuSOLVER` `gesvd` for tiny matrices; quantifies the
  kernel-launch-overhead bound that limits naive batched SVD.
- *Efficient GPU implementation of randomized SVD and its applications*,
  [arXiv:2110.03423](https://arxiv.org/abs/2110.03423). Randomized truncated SVD reformulated onto
  BLAS-3 — directly applicable to TT re-compression.

### CPU multicore parallelism — the project's near-term "correct-then-fast" lever
- Röhrig-Zöllner, Becklas, Thies, Basermann, *Performance of linear solvers in tensor-train format on
  current multicore architectures*, [arXiv:2312.08006](https://arxiv.org/abs/2312.08006) (IJHPCA 2025).
  **Most directly relevant to this crate's AMEn linear solve** — measures where TT solvers actually
  spend time on multicore CPUs. Start here for the CPU path.
- Shi, Ruth, Townsend, *Parallel algorithms for computing the tensor-train decomposition*,
  [arXiv:2111.10448](https://arxiv.org/abs/2111.10448) (SIAM J. Sci. Comput.). Parallel-TTSVD, PSTT,
  Tucker2TT, TT-fADI — the parallelizable TT-construction family.
- *Distributed memory parallel adaptive tensor-train cross approximation*,
  [arXiv:2407.11290](https://arxiv.org/abs/2407.11290). TT-cross at scale (relevant if cross /
  interpolation enters the marcher path).

### GPU tensor-network engines / frameworks to evaluate (not necessarily depend on)
- Bayraktar et al., *cuQuantum SDK: A High-Performance Library for Accelerating Quantum Science*,
  [arXiv:2308.01999](https://arxiv.org/abs/2308.01999). `cuTensorNet` (contraction + path optimization
  + automatic multi-GPU/MPI) and `cuStateVec`.
- Lyakh et al., *ExaTN: Scalable GPU-Accelerated High-Performance Processing of General Tensor Networks
  at Exascale*, Frontiers Appl. Math. Stat. 8:838601 (2022).
- *State of practice: evaluating GPU performance of state vector and tensor network methods*,
  [arXiv:2401.06188](https://arxiv.org/abs/2401.06188). Empirical backend comparison — which method
  actually wins, where.

### GPU DMRG / MPS sweeps — addresses the sequential-sweep concern head-on
- *Parallel implementation of the DMRG method achieving a quarter petaFLOPS performance on a single
  DGX-H100 GPU node*, [arXiv:2407.07411](https://arxiv.org/abs/2407.07411). 246 TFLOPS; ~80× over a
  128-core OpenMP CPU — evidence that the rank-adaptive sweep *can* be parallelized, and how.
- *A distributed multi-GPU ab initio DMRG algorithm … P-cluster of nitrogenase*,
  [arXiv:2311.02854](https://arxiv.org/abs/2311.02854). Multi-GPU sweep decomposition at scale.

### Rust-native / task-based implementations to learn from
- **TNC** — distributed tensor-network contractions library in **Rust** (JOSS, 2026). Closest prior
  art in our language; study its parallelism and data layout.
- **QuantRS2** (`quantrs2-sim`) — Rust MPS simulation with SIMD-accelerated ops and parallel TN
  optimization.
- Vincent et al., *Jet: Fast quantum circuit simulations with parallel task-based tensor-network
  contraction*, [Quantum 6, 709 (2022)](https://quantum-journal.org/papers/q-2022-05-09-709/). C++,
  but the task-based parallel-contraction design transfers to a Rust `rayon`/task model.

### Foundational — the algorithm being accelerated
- Dolgov & Savostyanov, *Alternating minimal energy methods for linear systems in higher dimensions*,
  Part I (SIAM J. Sci. Comput. 36(5), 2014) / Part II
  [arXiv:1304.1222](https://arxiv.org/abs/1304.1222). The AMEn solver itself — local (one/two-core)
  operations and the residual-enrichment step define what is and isn't parallelizable.

## Deliverable of the survey

Before writing any optimization code, this note (or a successor) should answer:

1. For our actual operations (QTT marchers, AMEn linear solves), **which kernel dominates wall-clock**
   under profiling — measured, not assumed.
2. What the SOTA does to accelerate *that* kernel, and whether it is a Rust-native lever (rayon,
   `portable_simd`, BLAS threading), a GPU lever (which framework), or an *algorithmic* lever
   (contraction order, randomized truncation) that needs no new hardware at all.
3. The expected ceiling (Amdahl) so we don't chase a kernel that is 20% of the time.

Cross-reference:
- [`../archive/tensor-network/Tensor-Network-Spec.md`](../archive/tensor-network/Tensor-Network-Spec.md)
  — the build history and design of the MPS/MPO/AMEn stack being accelerated (start here for *what
  the kernels are*).
- [`../cfd/tensor_network_cfd.md`](../archive/cfd/tensor_network_cfd.md) — where TN actually helps the CFD
  axis (accuracy affordability vs raw speed; the Re crossover that should aim the optimization).
- The tensor-train specs under `openspec/specs/tensor-train*` (the live contracts the optimization
  must not break).
