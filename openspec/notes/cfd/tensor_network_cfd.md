# Tensor-network CFD — applicability to the DeepCausality DEC + HKT stack

**Status.** Research synthesis, 2026-06-16. Companion to `quantum_fluid_dynamics_survey.md`
(Strand 2) and to `causal_cfd.md` (§0, the uncertainty and counterfactual moat). It answers one
question: where, if anywhere, do tensor-network (MPS / tensor-train / quantics) methods help the
existing DEC Navier–Stokes solver, judged by the §0.5 metric (wall-clock to within 1–2% of a
published reference).

**Provenance.** A deep-research run (`wf_44faadee-7cf`, 102 agents) executed but its synthesizer
stalled before writing a report, so this note is reconstructed from the recovered verified claim
ledger: 117 claims confirmed and 24 refuted under 3-vote adversarial verification. The claims are
verified; the connective narrative and the §2 inferences are editorial and flagged as such.

---

## 0. Bottom line

For a single laminar flow, the wall-clock upside is weak and the survey over-stated it. The
compression that makes tensor networks famous buys *accuracy affordability*, not speed, at the
project's Reynolds numbers. Two facts settle it:

- At laminar/transitional Re the bond dimension χ stays small and is bounded by geometry, not
  mesh resolution. That is real and useful.
- But the measured wall-clock advantage over a standard solver appears only above Re ≈ 9.5×10³.
  Below that crossover a standard solver is faster. The accuracy sweet spot (laminar) and the
  speed sweet spot (high Re) do not coincide.

So the headline "compressed field backend that runs faster" does not hold for cylinder St/C_d at
Re=100. The genuine value is elsewhere, and it is non-obvious (see §2). The most important
reframing: tensor networks help the project's **uncertainty and counterfactual** axis far more
than its single-flow speed.

---

## 1. The five-angle findings (verified)

**1.1 Encoding.** MPS/quantics solves incompressible NS with memory and runtime
poly-logarithmic in mesh size, reading the solution straight from the compressed encoding without
forming the dense vector (Kiffner & Jaksch, arXiv:2303.03010). The scaling law is χ = O(poly(1/ε)),
polynomial in inverse accuracy. The crucial regime fact: at laminar Re, χ is bounded by *geometry
complexity*, not resolution. A cylinder on a 2³⁰-cell mesh needs χ=30 (~24,200 parameters, over
44,000× compression); an airfoil needs χ=45; object masks need χ≤30 largely independent of
bit-count. QTT is formally a renormalization-group method, and the bond dimension is a measure of
length-scale entanglement, so quantics ordering encodes scale separation. Operator range and
locality set χ, which bears on whether the DEC `d` and Hodge star stay cheap under quantics order.

**1.2 Solvers in compressed form.** AMEn (Dolgov & Savostyanov) is the tensor-train-native linear
solver: it solves directly in TT form, applies to SPD systems (the Leray projection is SPD),
grows χ adaptively to meet accuracy, and has proven geometric convergence under truncation. The
pressure-Poisson / projection stays the dominant per-iteration cost even compressed; authors name
it the bottleneck explicitly. This matches the FFT audit of the repo: the projection is the cost
center. The adversarial finding that matters: every demonstrated TT Navier–Stokes result is
time-marching (RK4, or explicit Euler with Chorin projection per step). No paper demonstrates a TT
steady-state fixed-point solve. The "skip the transient" idea has no literature support and is
treated here as speculation, not a plan.

**1.3 Structure preservation under truncation (the make-or-break question).** SVD truncation does
not preserve incompressibility or no-slip. After truncation the velocity violates both, and they
are re-imposed by re-projection in the next step. Divergence-free is not preserved through the TT
manifold; it is re-enforced per step. Truncation error concentrates at high wavenumbers and grows
as χ shrinks, which is the mechanism by which compression eats the accuracy budget. The
encouraging counter-evidence: a TN fractional-step method reached field errors below 0.3% while
compressing fields 20× and operators 1000× (PhysRevResearch 7.013112), and MPS truncation
reproduced the energy spectrum and two-point correlations. So you can stay inside the 1–2% budget,
but by re-projecting each step, not by preserving structure through truncation. A genuine
structure-preserving-truncation class exists: tangent-space / TDVP truncation as constrained
optimization on the MPS manifold (Riemannian), with better cost scaling than brute-force SVD.
`torchTT` ships a Riemannian fixed-rank module. It is not yet shown to enforce div-free for fluids.

**1.4 Categorical / HKT fit.** The composition half is established mathematics. Tensor networks
are string diagrams in a monoidal category (FinVect, tensor product as monoidal product); Penrose
notation equals traced symmetric monoidal categories; the diagrammatic calculus is sound and
complete (Biamonte & Bergholm, "Tensor Networks in a Nutshell"). A `TensorTrainWitness` with
contraction as the monoidal product is principled, not cosmetic. The truncation half is sound but
novel for this use: graded monads can track accumulated numerical error with the grade carrying a
quantitative bound (a graded monad on the category of metric spaces, type-level error
guarantees), so "truncation as a graded-monadic effect carrying the ε / discarded-singular-weight
budget" has real grounding. That precedent is for general floating-point roundoff, not tensor
networks, and truncation is lossy and not functorial. Treat it as a bridge to build, not cited
prior art. Contraction ordering is #P-hard, so only heuristics are practical.

**1.5 Evidence at the regime, and the Rust landscape.** Laminar evidence is good: the lid-driven
cavity at Re=1000 reproduces the Ghia reference with χ saturating at 38 (Comms Phys 2024,
doi 10.1038/s42005-024-01623-8); a cylinder QIS solver matches finite-difference references for
Strouhal number and forces and was validated against Ansys Fluent at Re=1.7–2230 with runtime
~N·χ^4.1 (PhysRevResearch 7.013112). One caveat: a separate cylinder study validated only
qualitatively (streamlines, vortex phases), not C_d/St to 1–2%. The speed caveat is the main one:
MPS beats DNS only above Re ≈ 9.5×10³ (5.8× at Re=24000); below that a standard solver is faster,
and reported GPU speedups are internal (12.1× is QIS-GPU vs QIS-CPU, not vs DNS). Rust landscape:
no Rust TT library exists; `torchTT` (PyTorch) is the reference. Beyond the existing CausalTensor
SVD, a from-scratch layer needs TT-rounding, AMEn / DMRG linear solve, TT-cross (adaptive cross
interpolation), a Riemannian fixed-rank module, and contraction-ordering heuristics.

---

## 2. Non-obvious upsides (where the real value is)

The single-flow speed story is limited. These five are not, and several are better aligned with
the project's actual differentiators than "go faster on one cylinder." Each is tagged by epistemic
status: **[verified]** rests directly on a confirmed claim; **[inference]** is an editorial
extrapolation from a verified claim onto the repo's architecture.

**2.1 The uncertainty / counterfactual axis is where tensor networks actually win. [inference,
from a verified result]** The famous O(10³)–O(10⁶) reductions are in *high-dimensional parameter
and PDF space*, not in compressing a single 2D/3D velocity field. The verified case is a 5+1-D
turbulence PDF compressed by O(10⁶) memory and O(10³) compute (arXiv:2407.09169): the win is
beating the curse of dimensionality in parameter space. That is exactly what the project's
`MaybeUncertain` inflow and counterfactual `*_with_config` marches generate: a solution as a
function of many uncertain inputs or boundary parameters. Encoding the *ensemble* over those
parameter dimensions as a tensor train is the high-value use, and it maps onto `causal_cfd.md §0`
capability one (selective probabilistic typing). This reframes tensor networks from a Strand-2
speed play into an enabling technology for the moat. It is an inference because no paper applies
TT to CFD uncertainty quantification specifically, but the PDF-transport result is the same
mathematics.

**2.2 The compression sweet spot is geometry and operators, not the velocity field. [verified]**
Object masks compress to χ≤30 largely independent of bit-count, and differential operators
compress up to 1000× in TT form. The project's hard geometric problem (cut-cell no-slip,
aperture-resolved boundaries; see `aperture_resolved_noslip`) is representing complex boundary
geometry on a fine grid. A TT-encoded mask decouples geometric resolution from cost. Storing the
DEC operators (the grade-0/grade-1 Laplacian, Hodge star) as matrix-product operators is where
χ is smallest and most bounded. The near-term, low-risk use is compressing the mask and operators,
not the flow.

**2.3 A QTT field is its own multigrid hierarchy. [verified]** QTT is a renormalization-group
method, and each quantics bit is one coarsening level, so a QTT-encoded field carries a multigrid
V-cycle structure for free. This is a direct synergy with the Phase-1 Poisson work in §0.5: the
compression format and the optimal Poisson preconditioner are the same object. A QTT/AMEn solve of
the grade-0 Laplacian could *be* the geometric multigrid the cylinder case needs, on the immersed
geometry that rules out a direct FFT/DCT solve.

**2.4 χ(t) is a free multiscale-complexity and under-resolution diagnostic. [inference]** Because
the bond dimension needed to hold the current field at tolerance ε measures its length-scale
entanglement, χ(t) is a cheap global signal of how much fine structure the flow is developing.
Rising χ flags transition, instability, or that the mesh is about to under-resolve. That is a
ready-made trigger for adaptive refinement, and it connects to the project's interest in
vortex-core and transition detection. It costs almost nothing if a TT copy of the field is
maintained alongside the dense solve.

**2.5 TT-cross builds manufactured-solution fields and sources cheaply. [verified capability]**
TT-cross constructs a tensor train from a function sampled at a few points, without ever forming
the dense field. The project's verification backbone is the method of manufactured solutions,
whose fields and source terms are analytic functions. TT-cross can encode MMS reference fields,
initial conditions, and forcing at extreme resolution at low cost, which serves the convergence
ladder directly.

**Net answer to "is the upside as limited as you describe it?"** For single-flow wall-clock at
laminar Re, yes. Reframed onto uncertainty ensembles (2.1), geometry and operators (2.2), the
Poisson preconditioner (2.3), and verification tooling (2.4, 2.5), no. The strongest of these,
2.1, is also the best aligned with what makes the project distinct.

---

## 3. Conclusions

- **Near-term win:** a compressed *operator and geometry* layer (2.2) and a QTT-multigrid Poisson
  preconditioner (2.3), not a compressed velocity-field time-march. The field-compression speedup
  is not present at the project's Re.
- **Highest-risk assumption:** that a wall-clock win exists at laminar Re at all. χ stays bounded,
  so accuracy is affordable, but the measured speedup over a standard solver only appears above
  Re~10⁴. The Poisson/projection also stays the bottleneck even compressed. Any plan that promises
  speed from field compression at Re=100 is betting against the published crossover.
- **Steady-state TT solve:** speculative. No literature support; every demonstrated TT-NS result
  is time-marching. Not on the roadmap until something demonstrates it.
- **HKT fit:** genuine for composition (contraction is a lawful monoidal product; a
  `TensorTrainWitness` is a principled Functor/Applicative citizen alongside `CausalTensorWitness`).
  The truncation-as-graded-monadic-effect idea is theoretically sound but novel and not functorial;
  it is a research contribution to attempt, not established art.

---

## 4. If a Rust TT layer is built — kernels required

Beyond the existing `CausalTensor` SVD: TT-SVD rounding; TT-cross / adaptive cross interpolation
(field and source compression); AMEn or DMRG linear solve in TT form (the SPD Poisson/Leray
operator); a Riemannian fixed-rank manifold step (tangent-space truncation); and a
contraction-ordering heuristic (the optimal order is #P-hard). The lawful path into the existing
HKT layer is a `TensorTrainWitness` whose monoidal product is contraction and whose `fmap` lifts
elementwise scalar maps; truncation sits outside the functor as an explicit, error-budget-carrying
operation.

---

## 5. GPU, precision, and the candle backend option

The bottleneck operations are tensor operations (contraction and SVD), so GPU is the eventual
scale lever for the high-dimensional ensemble work. Three facts decide where that lever can reach,
and a prior attempt already settled two of them.

**The MLX revert was a precision wall, and it is hardware, not framework.** The reverted macOS
MLX backend (`openspec/changes/reverted/revert_mlx_backend/`) downcast every tensor f64 → f32
because Metal does not support f64. For a stack built on `Float106` and verified to 1e-30, that is
fatal, and reverting was correct. The limit is Apple-Silicon Metal, not MLX, so it persists under
any Metal framework. In practice this does not constrain the project: scientific CFD is
NVIDIA-dominated, A100-class rentals are inexpensive for normal workloads, and avionics research
centers run their own NVIDIA clusters. The f64 target is NVIDIA datacenter hardware, where f64 is
a first-class fast capability (A100/H100), not the consumer cards that throttle f64 to 1/32–1/64.

**Candle (web-checked, June 2026) clears the two problems that sank MLX.** Its backend is
separated: `candle-core` has no default features, the `cuda` feature pulls a distinct kernel crate
(`candle-kernels`, PTX) plus `cudarc`, and `metal` pulls `candle-metal-kernels`; hardware sits
behind `BackendDevice` / `BackendStorage` traits and a `Device` enum (Cpu / Cuda / Metal). f64 is
a first-class `DType`, the default float literal, supported on CPU and CUDA. So candle-on-CUDA
keeps f64 end to end, and the f32 wall that killed MLX does not apply on NVIDIA. This also retires
the mixed-precision split floated in an earlier discussion (f32-GPU for ensembles, f64-CPU for the
core): that split was a Metal artifact. On an A100 via candle you keep f64 throughout and pay only
the f64 throughput penalty, which datacenter cards absorb.

**The honest catch is the SVD, not precision.** Candle is an ML-inference framework. Its CUDA
kernels cover matmul/gemm (cuBLAS Dgemm in f64), elementwise, and reductions, which is the
contraction half of tensor-network work. It does not provide SVD, QR, or eig, and SVD is the
recurring tensor-train bottleneck (the truncation step). Not every op has an f64 kernel on every
backend either: a reported `rmsnorm F64` gap shows specialized fused kernels can be f32-only, so
the exact op set must be checked. A candle backend would therefore accelerate f64 contraction on
NVIDIA, but the SVD/truncation kernel stays an open problem: cuSOLVER via `cudarc`/FFI, a
hand-written kernel, or CPU SVD with transfer cost. The repo's existing `CausalTensor` SVD is CPU.

**Pathway: contribute the missing decomposition kernels to candle (investigated, 2026-06).** The
gap is tractable, and most of the binding already exists. cuSOLVER provides the f64 routines
(`gesvd` and the Jacobi `gesvdj` for SVD, `geqrf`/`orgqr` for QR, `syevd`/`syevj` for symmetric
eig) and is actively maintained (release 13.3, 2026). The key finding: `cudarc`, which candle
already depends on, **already ships cuSOLVER bindings** behind a `cusolver` feature, in its usual
`sys` / `result` / `safe` layering, covering `gesvd`, `geqrf`, and `syevd`. So contributing
SVD/QR/eig to candle's CUDA backend is mostly plumbing those existing `cudarc` calls into candle's
`BackendStorage` op layer, not writing CUDA. The `baracuda-cusolver` crate is independent prior
art wrapping the same routines. Candle has no decomposition support at all, so this is a
broadly-wanted upstream contribution, and it would serve the whole deep_causality stack, not only
tensor-network CFD.

The performance crux is now concrete, not vague. cuSOLVER's fast batched Jacobi path,
`gesvdjBatched`, is capped at 32×32 matrices. Tensor-train truncation SVDs are χ-by-χd cores, and
at this project's laminar bond dimensions (χ=30 for the cylinder, 45 for the airfoil) the cores
sit right at or above that cap. So the batched fast path covers the χ≤32 regime cleanly, and above
it you fall back to per-core `gesvd`/`gesvdj` (still on GPU, but launched individually, less
batch-efficient). That boundary, χ≈32, is the thing to benchmark: batched below it, per-matrix
above it, against the existing CPU SVD plus transfer cost. If the batched cap proves limiting,
recent research on efficient small-batched GPU SVD (arXiv:2601.17979) is the higher-value kernel
to port instead of cuSOLVER's.

Action: prototype a `cudarc`-cuSOLVER SVD path (feature-available today), benchmark batched
(χ≤32) vs per-matrix (χ>32) f64 SVD at TT-core sizes against the CPU path plus transfer cost, and,
if the numbers justify it, upstream SVD/QR/eig into candle. Landing it makes candle a full f64
NVIDIA backend for the entire tensor-network kernel set, not just contraction.

**Placement and policy.** Candle slots behind the reverted `TensorBackend` seam as an opt-in,
non-default backend, with `CpuBackend` f64 the default. The MLX lessons hold: opt-in, never
default, never silent downcast. On CUDA the no-downcast rule is satisfied for free, since f64 is
native. The cost is a large external dependency tree (`cudarc`, CUDA FFI, hence `unsafe`), which
tensions with the workspace `unsafe_code = "forbid"` and minimal-dependency policy and would need a
documented exemption and strict feature-gating so default builds stay pure. Strand fit:
candle-on-NVIDIA is a credible deferred path to accelerate the f64 contraction-heavy ensemble and
operator work (§2.1, §2.2), with the SVD kernel as the one genuine piece of new numerics, and
Apple GPUs correctly out of scope.

### Decision (2026-06): candle is the backend path; Warp is not

Warp was evaluated and ruled out as an integration target. It was open-sourced only about a year
ago, has no Rust bindings, and is architecturally a Python JIT framework: `libwarp` is a
codegen-and-runtime engine, and the solvers are Python-authored `@wp.kernel` functions compiled at
runtime, so there is no callable C solver symbol to bind. FFI would mean embedding CPython (PyO3),
harvesting generated PTX, or running a Python sidecar with DLPack buffer sharing, all of which pull
in Python and fight the repo's pure-Rust policy. Warp does now support f64 (including `warp.fem`)
and is a useful reference and competitor to differentiate against, but it is not a dependency.

Candle is the chosen pluggable backend. It is Rust-native, has a feature-gated, separated CUDA
backend, supports f64 end to end on NVIDIA, and reaches the CUDA-X C libraries (the real clean-ABI
FFI surface) through its existing `cudarc` dependency. The one genuine engineering item is the
missing decomposition kernels (SVD/QR/eig): contribute them upstream via `cudarc`-cuSOLVER, with
the batched-small-SVD 32×32 cap as the part to benchmark, or port the research kernel
(arXiv:2601.17979) if the cap limits. The work stays opt-in and non-default behind the reverted
`TensorBackend` seam, keeps `CpuBackend` f64 the default, needs a documented `unsafe`/external-dep
exemption, and is scoped to the high-dimensional ensemble and operator work where it pays off, not
the single-flow march.

## 6. Caveats and refuted directions

- 24 of the verified claims were refuted (3-vote). The recurring refutations: that TT field
  compression yields a guaranteed wall-clock win at laminar Re (it does not below the Re~10⁴
  crossover); that headline GPU speedups are versus a standard solver (they are internal QIS-GPU
  vs QIS-CPU); and that a steady-state TT-NS solve has been demonstrated (it has not).
- The O(10⁶) figure belongs to PDF-space (5+1-D), not velocity-field compression. Do not quote it
  as a field-compression ratio.
- Truncation breaking the divergence-free constraint is confirmed, not hypothetical. Any TT field
  backend must re-project each step; it cannot rely on the TT manifold to hold incompressibility.

---

## 7. Citations (fold into `references.md`)

- **[Gourianov-2022]** *A quantum-inspired approach to exploit turbulence structures.* Nature
  Computational Science 2 (2022). doi:10.1038/s43588-021-00181-1.
- **[Kiffner-2023]** Kiffner & Jaksch. *Tensor network reduced order models for computational
  fluid dynamics.* arXiv:2303.03010 (2023).
- **[CommsPhys-2024]** *Tensor-network solver for incompressible Navier–Stokes (lid-driven
  cavity); χ saturates at 38 vs the Ghia reference.* Commun. Phys. 7 (2024).
  doi:10.1038/s42005-024-01623-8.
- **[QIS-Cylinder]** *Quantum-inspired tensor-network solver for laminar flows; cylinder vs Ansys
  Fluent at Re=1.7–2230, runtime ~N·χ^4.1; fields/operators compressed 20×/1000× at <0.3% error.*
  Phys. Rev. Research 7, 013112 (2024). doi:10.1103/PhysRevResearch.7.013112.
- **[TTPDF-2024]** *Tensor-train parameterization of high-dimensional turbulence PDFs; O(10⁶)
  memory / O(10³) compute in 5+1-D parameter space.* arXiv:2407.09169 (2024).
- **[MPS-GP-2025]** *Matrix-product-state solver for the Gross–Pitaevskii equation; cuQuantum
  12.1× GPU (internal).* arXiv:2508.12191 (2025).
- **[AMEn]** Dolgov & Savostyanov. *Alternating minimal energy methods for linear systems in
  higher dimensions.* SIAM J. Sci. Comput. 36(5), A2248 (2014). (TT-native SPD linear solver.)
- **[Biamonte-Nutshell]** Biamonte & Bergholm. *Tensor Networks in a Nutshell.* arXiv:1708.00006
  (2017). (Tensor networks as monoidal-category string diagrams.)
- **[torchTT]** ion-g-ion/torchTT. PyTorch tensor-train library: AMEn, DMRG, TT-cross
  (`torchtt.interpolate`), Riemannian fixed-rank manifold. Reference implementation, not Rust.
- **[Candle]** huggingface/candle (web-checked 2026-06). Backend separated via feature-gated
  kernel crates (`candle-kernels`/CUDA, `candle-metal-kernels`/Metal) behind `BackendDevice`/
  `BackendStorage`; f64 is a first-class `DType` on CPU and CUDA; no SVD/QR/eig. Feature mapping:
  https://lib.rs/crates/candle-core/features. Architecture: https://deepwiki.com/huggingface/candle.
  f64 fused-kernel gap example: https://github.com/huggingface/candle/issues/2355.
- **[cudarc-cusolver]** coreylowman/cudarc (web-checked 2026-06). Safe Rust CUDA wrapper, `sys`/
  `result`/`safe` layering; `cusolver` feature exposes `gesvd`/`geqrf`/`syevd`. candle's existing
  CUDA dependency. https://docs.rs/cudarc, https://github.com/coreylowman/cudarc.
- **[baracuda-cusolver]** Independent Rust cuSOLVER wrapper (geqrf, cholesky, gesvd, syevd/heevd)
  — prior art for the binding. https://docs.rs/baracuda-cusolver/.
- **[cuSOLVER]** NVIDIA cuSOLVER (release 13.3, 2026). `gesvdjBatched` batched Jacobi SVD capped at
  32×32. https://docs.nvidia.com/cuda/cusolver/ ; sample:
  https://github.com/NVIDIA/CUDALibrarySamples/tree/master/cuSOLVER/gesvdjBatched.
- **[BatchSVD-2026]** *An Efficient Batch Solver for the Singular Value Decomposition on GPUs.*
  arXiv:2601.17979 (2026). (Higher-value small-batched-SVD kernel if cuSOLVER's 32-cap limits.)
</content>
