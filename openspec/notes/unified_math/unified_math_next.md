<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Unified math: what is missing, measured against its consumers

**Status.** Assessment, 2026-09-03. Not a change proposal; the ranked work list at the end is
the input to one.

**Question answered.** Which areas of mathematics does `deep_causality_unified_math/` lack
relative to what its four downstream consumers do today: CFD and physics, the causality engine
with the algorithms and discovery crates, the quantum crate, and the Deep Brain project?

**Method.** Four independent sweeps of the consumers, each reading source for mathematics the
consumer implements itself or reaches for and cannot find, followed by verification greps
against the stack. A hand-rolled solver or a workaround inside a consumer is the footprint of
the missing piece; the same footprint in three consumers is the ranking signal. Every finding
carries a `file:line` so it can be re-checked. Paths are relative to the workspace root.

**Scope of the stack at the time of writing.** Sixteen crates: `num`, `metric`, `algebra`,
`haft`, `num_rational`, `rand`, `linear`, `num_complex`, `num_dual`, `uncertain`, `calculus`,
`fft`, `homology`, `tensor`, `multivector`, `topology`. `deep_causality_ast` lives in
`deep_causality_utils/` and is not a member. The stack ships, among other things: LU, QR,
Cholesky, SVD, Hermitian eigendecomposition, conjugate gradient, least squares and norms in
`linear`; Euler, RK4, forward differences and a fixed-step quadrature in `calculus`; FFT and
real FFT in `fft`; Normal, Uniform, Bernoulli, Sobol and a ziggurat in `rand`; sample mean,
covariance, log-sum-exp, Gaussian log-density and conditional variance in `tensor`'s stats
extension; `erf` and `erfc` in `num`; the precision-boundary lifts in `num::lift`.

---

## 1. Verdict

The findings fall into three classes.

1. **Missing outright.** Five areas have no equivalent anywhere in the stack, and each shows up
   downstream as the same code written several times: statistics and information theory, root
   finding and optimisation, interpolation, time integration beyond Euler and RK4, and graph
   algorithms with mathematical content. Under them sit three smaller holes: special functions
   beyond `erf`, exact big integers, and general combinatorics.
2. **Present but unreachable or unused.** The `Real` bound lacks `cbrt`, `max`, `min` and
   `mul_add`, and `RealField` carries no `ToPrimitive`; `rand`'s distributions are implemented
   per concrete float type with no blanket over `RealField`; the spectral Poisson construction
   in `topology` is private; three consumers lack the dependency edge to the crate that has what
   they hand-roll; and the causality engine pins its numeric aliases to `f64`. These cost more
   downstream lines than the missing areas do and are cheaper to fix.
3. **Domain-owned, correctly downstream.** Channel algebra, partial trace, tensor-train
   operator assembly, SURD, BOSS, clique-picking sampling. Two generic pieces inside them could
   move down.

Counted across the four consumers: 7 independent root finders, 6 linear or multilinear
interpolators, 3 log-sum-exp copies, 2 SURD entropy implementations, 3 combinations routines,
2 MEC-size algorithms, 3 cofactor matrix inverses, 6 Euclidean norms, 3 literal-lift helpers in
physics plus 15 lift shims in algorithms, and about 118 sites of open-coded complex arithmetic in
quantum.

---

## 2. Missing outright

### 2.1 Statistics and information theory

Nothing in the stack computes entropy, mutual information, a divergence, a correlation, a
regression, a histogram, or a hypothesis test. `tensor`'s `ext_stats` holds sample mean,
covariance, log-sum-exp, Gaussian log-density and conditional variance, shaped for tensors.

| Consumer | Where | What it wrote |
|---|---|---|
| algorithms, SURD | `deep_causality_algorithms/src/causal_discovery/surd/surd_utils/mod.rs:111,152` | Shannon entropy over marginalised axes; conditional entropy `H(X\|Y) = H(X,Y) − H(Y)` |
| algorithms, SURD | `.../surd/surd_utils/surd_utils_cdl.rs:135,189` | the same two functions again for `CausalTensor<Option<T>>` |
| algorithms, SURD | `.../surd/surd_algo.rs:145-200,497`; `surd_algo_cdl.rs:85,501` | specific mutual information, information leak, `p(t,i,j)·log(p(t\|i)/p(t\|j))`; two near-identical 580 and 640-line implementations, ~450 differing lines after normalisation, differing only in `T` versus `Option<T>` |
| algorithms, SURD | `.../surd/surd_utils/mod.rs:52` | tolerance-quantised stable argsort, `tol = 1e-9`, so sub-resolution noise cannot reorder tied specific informations |
| algorithms, mRMR | `.../feature_selection/mrmr/mrmr_utils.rs:29,127` | Pearson correlation with pairwise deletion; `F = (n−2)r²/(1−r²)` with a `1e12` sentinel. Both take generic `T` and compute in `f64` |
| algorithms, BRCD | `.../brcd/brcd_gaussian.rs:86,481` | ridge least squares by normal equations, twice: materialised and streaming |
| algorithms, BRCD | `.../brcd/brcd_gaussian.rs:561,579` | 1-D normal log-density, inlined for allocation reasons; the doc comment names `ext_stats::gaussian_log_density` as the function it duplicates |
| algorithms, BRCD | `.../brcd/brcd_gaussian.rs:631`, `brcd_algo.rs:540`, `brcd_boss_bootstrap.rs:325` | log-sum-exp three times; `ext_stats.rs:46,154` has the same max-shift form with the same non-finite guard |
| algorithms, BRCD | `.../brcd/brcd_gaussian.rs:661,670` | mean and Bessel-corrected variance on slices |
| algorithms, BRCD | `.../brcd/brcd_gate.rs:91,202,214` | L2-penalised logistic regression by Newton IRLS; stable sigmoid and clamped logit. No logistic regression exists anywhere else in the workspace |
| algorithms, BRCD | `.../brcd/brcd_dirichlet.rs:43` | prequential Dirichlet posterior predictive |
| algorithms, BRCD | `.../brcd/brcd_boss_bootstrap.rs:91,162,214,232` | bootstrap resampling with frequency-corrected log-weights and posterior marginalisation |
| discovery | `deep_causality_discovery/src/.../data_discretizer.rs:71,117` | equal-width and equal-frequency binning, the joint-density estimator SURD consumes; no KDE anywhere |
| discovery | `.../missing_value_imputer.rs:28` | column-mean imputation |
| quantum | `deep_causality_quantum/src/types/qpu/shot_estimate.rs:41-61,122,131` | `p = k/n`, `√(p(1−p)/n)`, Bhattacharyya bits per shot, separation bits. No statistical distance exists in the stack; `deep_causality_metric` is Clifford signatures, not distances |
| quantum | `.../qpu/bridge.rs:76-109` | mean and unbiased variance accumulated in `f64` beside the generic sibling |
| physics | `deep_causality_physics/src/kernels/thermodynamics/stats.rs:154,216,240` | Shannon entropy kernel, partition function with exponent clamping |
| CFD | `deep_causality_cfd/src/types/flow/frequency.rs:23` | dominant frequency by mean-crossing counting; `fft` is a dependency and unused here |
| CFD | `deep_causality_cfd/src/types/flow/operator_study.rs:109-119` | observed convergence order as pairwise `log₂(eₙ/e₂ₙ)`; no regression or fitting utility exists |
| Deep Brain | `deep_brain/deepbrain.md`, see §5.4 | needs rank fusion, BM25, MMR; KL divergence would serve drift measurement; none exists |

Two SURD helpers already leaked upward into the stack: `surd_log2` and `safe_div` at
`deep_causality_unified_math/deep_causality_tensor/src/extensions/ext_math.rs:92,100`.

### 2.2 Root finding and optimisation

No bisection, Newton, Brent, secant or fixed-point routine exists in the stack, and no
minimiser sits on top of the derivative machinery that `num_dual` and `calculus` provide.

| Where | Method | Notes |
|---|---|---|
| `deep_causality_physics/src/kernels/propulsion/nozzle.rs:111` | bisection, fixed 200 iterations | inverse area–Mach; bracket by doubling with a `1e9` cap |
| `deep_causality_physics/src/kernels/propulsion/plume.rs:206` | bisection, same constant | terminal-shock Mach |
| `deep_causality_physics/src/kernels/propulsion/plume.rs:583` | bisection, same constant | mass-flow scaling parameter |
| `deep_causality_physics/src/kernels/astro/two_body.rs:180` | Newton, 100 iterations, absolute `1e-15` | Kepler's equation |
| `deep_causality_physics/src/kernels/astro/ks_propagator.rs:184` | Newton, 100 iterations, relative tolerance | KS fictitious-time inversion |
| `deep_causality_physics/src/theories/electroweak/radiative.rs:110-158` | fixed point, 20 iterations, hard-coded `1e-6` and `80.30` start | uses unchecked `T: From<f64>`; returns the last iterate on non-convergence without signalling |
| `deep_causality_physics/src/kernels/hypersonic/finite_rate.rs:220` | closed-form quadratic root | |
| `deep_causality_algorithms/src/causal_discovery/brcd/brcd_gate.rs:91` | Newton IRLS | logistic gate |
| `.../brcd/brcd_boss_search.rs:50,197,208` | permutation hill-climbing to a fixpoint with a visited-order cycle guard; argmax; Fisher–Yates shuffle | `rand` has no `shuffle`, `choose` or weighted sampling |
| `.../brcd/brcd_mapconfig.rs` | greedy `O(du)` coordinate walk replacing a `2^du` enumeration | approximate by construction |

### 2.3 Interpolation and tables

| Where | What | Out-of-range policy |
|---|---|---|
| `deep_causality_cfd/src/types/keyed_table.rs:123` | linear, O(n) scan | clamp, with a `clamped` provenance flag |
| `deep_causality_physics/src/kernels/propulsion/srp.rs:23` | linear, O(n) `windows(2)` scan | reject |
| `deep_causality_cfd/src/types/flow_config/compressible_march_config.rs:147-168` | inline `lerp` closure over an altitude table | clamp |
| `deep_causality_cfd/src/solvers/dec/surface_force.rs:188,372` | D-dimensional multilinear sampling over `2^D` corners, twice | |
| `deep_causality_cfd/src/solvers/dec/diagnostics.rs:142` | third multilinear sampler | |

None uses binary search. The divergence in out-of-range policy is the risk, more than the
duplication. No spline exists anywhere.

### 2.4 Time integration beyond Euler and RK4

`calculus` ships `Euler`, `Rk4`, `Diff` and a fixed-step `quadrature(f, a, b, n)`.

| Where | What | Stack equivalent |
|---|---|---|
| `deep_causality_physics/src/kernels/relativity/gravity.rs:203-235` | hand-rolled RK4 for the geodesic system, `k1..k4` over `Vec<T>`, with `<T as From<f64>>::from(0.5)` | `calculus::Rk4` is the same scheme and is used at `deep_causality_cfd/src/types/flow/verify.rs:98` and `deep_causality_cfd/src/solvers/dec/marcher.rs` |
| `deep_causality_physics/src/kernels/astro/ks_propagator.rs:252` | Strang half-kick, exact drift, half-kick | none; the only splitting scheme in the workspace |
| `deep_causality_cfd/src/solvers/qtt/compressible/imex.rs:123` | 1-D IMEX with a closed-form implicit core | none |
| `.../compressible/marcher_2d.rs:231`, `marcher_3d.rs:233`, `marcher_3d_fitted.rs:192` | 2-D and 3-D IMEX predictor with ADI implicit inverse | none |
| `deep_causality_cfd/src/solvers/qtt/incompressible_2d.rs:159`, `immersed_2d.rs:144`, `qtt/mod.rs:120` | explicit Euler plus projection with per-step tensor-train recompression; the doc states `Rk4` is unusable because train stages must round between operations | `calculus::Euler` cannot round between stages |
| `deep_causality_cfd/src/types/flow/blackout.rs:41` | closed-form exponential relaxation, unconditionally stable stiff source step | none |
| `deep_causality_cfd/src/navigation/eskf.rs:230` | first-order process-noise discretisation; Van Loan declined explicitly | none |
| `deep_causality_physics/src/kernels/propulsion/plume.rs:537-571` | midpoint quadrature over a tabulated curve with central-difference geometry derivatives | `quadrature(f, a, b, n)` takes a function, not a table |

No matrix exponential, logarithm or square root exists in the stack; the quantum crate does not
yet need one, and a Lindblad or Hamiltonian channel would.

### 2.5 Graph algorithms with mathematical content

`topology`'s mixed graph owns acyclicity: `topological_sort`, `has_cycle`, `find_cycle` at
`deep_causality_unified_math/deep_causality_topology/src/types/mixed_graph/acyclicity/mod.rs:34,56,65`.
`ultragraph`, outside the stack, owns betweenness, shortest paths, reachability, strongly
connected components, articulation points and bridges. Everything below exists in neither.

| Need | Who needs it | Where it lives today |
|---|---|---|
| Meek rules R1 to R3, CPDAG from DAG, unshielded-collider validity | discovery, algorithms | `deep_causality_algorithms/src/causal_discovery/brcd/brcd_meek.rs:46`, `brcd_boss_cpdag.rs:38`, `brcd_validity.rs:26-81`. R4 omitted deliberately for parity with the reference (`brcd_meek.rs:13-24`) |
| d-separation, Markov blankets, backdoor and frontdoor adjustment, do-calculus, structural-equation solving, conditional-independence tests | causality engine | nowhere in the workspace. `deep_causality_core/src/types/causal_flow/intervene.rs:13` states that at that level there is no graph to mutilate, so intervention is value substitution |
| PageRank | Deep Brain | nowhere; delegated to grafeo (`deep_brain/deepbrain.md:643`) |
| community detection, Louvain | Deep Brain | nowhere; delegated to grafeo |
| spectral clustering | Deep Brain, implied | eigensolvers exist in `linear` and `tensor`; no graph Laplacian from an adjacency structure and no k-means anywhere |
| transitive closure, reverse reachability sets | Deep Brain blast radius (`deepbrain.md:427`) | `ultragraph` has pairwise `is_reachable` only |
| structural similarity of two causal chains | Deep Brain's primary cross-domain signal (`deep_brain/Deep Brain — Technical Architecture.txt:969`) | undefined in every document; nothing in the workspace |
| MEC size | algorithms | computed two ways: polynomial clique-picking at `deep_causality_algorithms/src/dag_sampling/mod.rs:107`, exact AMO enumeration capped at 100 000 at `brcd/brcd_mec.rs:71`, with different signatures and failure modes |
| chordal-graph machinery: maximum-cardinality search, clique tree, separators | algorithms | `dag_sampling/chordal.rs:20`, `clique_tree.rs:45-174`; chordality is assumed and never checked (`dag_sampling/mod.rs:56`, `brcd_mec.rs:44`) |
| a third graph type with its own BFS and components | algorithms | `dag_sampling/graph.rs:37-84`, decoupled from `topology::MixedGraph` by design |

### 2.6 Special functions, exact integers, combinatorics

- **Special functions.** `num` ships `erf` and `erfc` only. No gamma, log-gamma, beta,
  digamma or Bessel function exists. BRCD carries its own Dirichlet density because `rand`
  has no Beta, Gamma or Dirichlet; there is no Student-t or chi-square for any test.
- **Exact integers.** No big integer exists in the stack; `num_rational` is exact over the
  primitive integers only. `deep_causality_algorithms/src/dag_sampling/mod.rs:44-53`
  documents that the AMO count is carried in a float, exact to 2⁵³ at `f64` and 2¹⁰⁶ at
  `Float106`, and beyond that the inclusion–exclusion "rounds and may cancel, and the returned
  integer can be off by one or more with no error or saturation signal". The reference
  implementation used `BigUint`.
- **Combinatorics.** `num`'s combinatorics module holds two functions, `stirling_second` and
  `stirling_first_unsigned`. Factorials, combinations and `choose3` are written locally:
  `dag_sampling/combinatorics.rs:34,57`, `surd_utils/mod.rs:76`, `brcd_algo.rs:509`,
  `deep_causality_quantum/src/types/qcm/hypothesis.rs:689` and
  `.../pipeline/validate.rs:558` (identical `choose3` twice),
  `.../qgates/gates_haruna.rs:91,100,264-292` (tuple counts in `u128`, a hand-rolled odometer
  with quadratic dedup). The odometer walk also appears in CFD's spectral diffusion and
  topology's spectral Poisson (§3.3).

---

## 3. Present but unreachable or unused

### 3.1 The `Real` bound

`Real` at `deep_causality_unified_math/deep_causality_algebra/src/algebra/real.rs:29` exposes
`sqrt`, `abs`, `floor`, `round`, `exp`, `ln`, `powf`, the trigonometric family, `clamp` and
`epsilon`. Verified absent: `cbrt`, `max`, `min`, `mul_add`, `hypot`. `RealField: Real + Field`
carries no `ToPrimitive`; `Scalar: Real + Div + FromPrimitive` carries `FromPrimitive` only.
`Float::cbrt` exists at `deep_causality_num/src/float/mod.rs:657` and is unreachable through
the `RealField` bound.

The CFD and physics sweep counted about 575 sites of resulting boilerplate. The representative
ones:

| Where | Workaround |
|---|---|
| `deep_causality_physics/src/kernels/fluids/coherent_structures.rs:199` | `signed_cbrt` as `powf(1/3)` with a sign branch |
| `.../coherent_structures.rs:272` | a three-element sorting network because `R: PartialOrd` |
| `deep_causality_cfd/src/navigation/eskf.rs` (`max_abs`), `deep_causality_physics/src/kernels/relativity/spacetime.rs:99` | manual `max`; the comment reads "RealField has no `max`, so do it…" |
| `deep_causality_cfd/src/solvers/dec/surface_force.rs:196-206` | integer floor found by a bounded linear scan, because there is `FromPrimitive` and no `ToPrimitive` |
| `deep_causality_quantum/src/types/verdict/projection.rs:157-168` | `Tr(P)` rounded by a `while r + half < tr` counting loop, with `R::from_f64(0.5).unwrap_or_else(R::zero)`; `Real::round` exists at `real.rs:131`, and the fallback degenerates the loop instead of erroring |
| `deep_causality_cfd/src/solvers/qtt/compressible/marcher_2d.rs:98,132,290`, `marcher_3d.rs:86,123`, `marcher_3d_fitted.rs:86`, `types/flow/coupling.rs:567`, `types/flow/corridor/branch.rs:156`, `types/flow/corridor/regime.rs:183-185`, `types/flow/blackout.rs:358` | `R::from_f64(0.5).unwrap_or_else(R::one)`, twelve sites: a failed conversion silently makes ½ into 1, and at `blackout.rs:358` makes `1e30` into 1. The physics kernels use `.ok_or_else(NumericalInstability)` instead |
| `deep_causality_quantum/src/types/verdict/projection.rs:235`, `decision/tolerance.rs:162`, `qcm/markov_freeze.rs:54,98`, `qpu/shot_estimate.rs:132` | `unwrap_or_else(R::one)` and `R::zero` on `from_usize`/`from_u64`; sibling sites `qgates/operator_linalg.rs:437` and `qcode/css_code.rs:194` return `CalculationError` |
| `deep_causality_physics/src/constants/mod.rs:54`, `kernels/propulsion/plume.rs:55`, `kernels/hypersonic/finite_rate.rs:42` | three literal-lift helpers with three error behaviours |
| fifteen files under `deep_causality_algorithms/src/causal_discovery/brcd/` and `dag_sampling/` | `from_f64`, `from_usize` and `t_usize` `.expect(...)` shims |

`deep_causality_num::lift` now covers the literal and count crossings (`lift`, `lift_count`,
`lift_<primitive>`, `lower`, `to_count`); the shims above predate it and can be replaced without
touching the bound. The bound itself is the remaining work.

### 3.2 `rand` distributions

Verified: `StandardUniform`, `Open01`, `OpenClosed01` and `StandardNormal` are implemented for
`f32`, `f64` and `Float106` separately; there is no blanket implementation over `RealField`.
Consequences: `deep_causality_physics/src/kernels/nuclear/lund/flavor.rs:13-16` and
`kinematics.rs:15-19` sample at `f64` and lift;
`deep_causality_algorithms/src/dag_sampling/sample.rs:126` scales a `Float106` cumulative weight by
an `f64` variate; and the `Float106` sampler consumes a different number of random bits per draw, so
it walks a different stream at the same seed, which the README's mixed-precision table notes.

`rand` also has no `shuffle`, `choose`, weighted choice, Beta, Gamma or Dirichlet.

### 3.3 Visibility and dependency edges

| Consumer | Missing edge or visibility | What it hand-rolls as a result |
|---|---|---|
| CFD | `deep_causality_topology/src/types/manifold/differential/spectral_poisson.rs:49` `spectral_poisson_solve` is `pub(super)` | `deep_causality_cfd/src/tensor_bridge/projection.rs:146` reimplements the dense periodic Poisson solve with a different eigenvalue convention, and `deep_causality_cfd/src/solvers/dec/spectral_diffusion.rs:96-125` duplicates the eigenvalue weight tables and odometer walk of `spectral_poisson.rs:70-115` line for line |
| CFD | no dependency on `linear` | `deep_causality_cfd/src/navigation/eskf.rs:31-51`: `mat_mul`, `mat_transpose`, `mat_add`, `mat_vec`, `dot`, `diag` for the 17-state filter; `types/flow/mms.rs:224` a 3-vector norm |
| physics | depends on `linear` and `tensor`, uses neither for these | `deep_causality_physics/src/theories/general_relativity/gr_utils.rs:12,114`, `adm_state.rs:126`: three cofactor inverses with absolute determinant floors `1e-12` and `1e-14`; `kernels/mhd/ideal.rs:204,236`: two CSR matrix-vector products, the first silently skipping out-of-range columns where `CsrMatrix::vec_mult` (`deep_causality_linear/src/types/csr_matrix/mod.rs:260`) returns an error; `kernels/fluids/coherent_structures.rs:178,188,211`: 3×3 products and a closed-form symmetric 3×3 eigensolver beside `CausalTensor::eigen_hermitian`; `theories/general_relativity/gr_ops_impl.rs:46-70`: `g^μν R_μν` by nested loops beside `EinSumOp` used at `kernels/dynamics/estimation.rs:145` |
| algorithms | no dependency on `linear` | `deep_causality_algorithms/src/causal_discovery/brcd/brcd_linalg.rs:29`: dense Gaussian elimination with partial pivoting; the module doc keeps it local so consumers "do not reach into another crate's internals"; `brcd_gaussian.rs:654`, `brcd_gate.rs:195`: dot products |
| quantum | no dependency on `rand` | `deep_causality_quantum/src/types/qpu/prng.rs:11-31`: splitmix64 from scratch; `qpu/sim.rs:216-243`: inverse-CDF sampling by `partition_point`; `qpu/born_sampler.rs:34`: Bernoulli by comparison; `qpu/evidence.rs:151-156`: seed derivation. `rand` ships `Xoshiro256::from_seed`, `Bernoulli`, `UniformFloat<F: RealField>` and both inverse CDFs. No document states why `rand` was declined |
| quantum | depends on `num_complex` and `linear`, uses neither for these | about 118 sites in 10 files open-code complex arithmetic on `.re`/`.im`: private `cmul`/`conj` at `qgates/channel.rs:33,37` and again at `carriers/qubit_operator.rs:40-48`, a private `struct C` with its own `add`/`mul`/`norm_sq` at `qpu/sim.rs:57-79`, trace and modulus accumulations at `qgates/operator_linalg.rs:57-88,235`, `density_matrix.rs:100-260`, `verdict/projection.rs:83-252`, `verdict/born.rs:47-48`, `qcm/hypothesis.rs:459-464`, `qgates/gates.rs:56`, `qgates/bridge.rs:44-51`. Every hand-rolled `(dr*dr + di*di).sqrt()` uses the direct form; `num_complex`'s `Normed::modulus` at `deep_causality_num_complex/src/complex/complex_number/normed.rs:10-38` is written in the scaled form to avoid overflow, so the local copies return `inf` for a representable modulus. `frobenius_norm` at `qgates/operator_linalg.rs:64` duplicates `deep_causality_linear::matrix_norm_frobenius` (`linear/src/algorithms/norms.rs:123`), directly applicable because `CausalTensor<T>: MatrixView` (`tensor/src/extensions/ext_linear.rs:42`). The entrywise max-modulus residual idiom is written five times: `projection.rs:79-84,177-183,198-205`, `channel.rs:370-379`, `qubit_operator.rs:176-189` |
| quantum | `PackedGf2Vector` is a transitive dependency | `deep_causality_quantum/src/types/qcode/clifford_action.rs:59-126` unpacks symplectic vectors into `Vec<bool>` for the Aaronson–Gottesman tableau update and repacks, one byte per bit and two conversions per call; the only place the qcode layer leaves the packed representation |
| quantum | its own `Tolerance` family (`decision/tolerance.rs:48`) | bypassed with hard-coded `R::epsilon().sqrt()` at `verdict/born.rs:52`, `qgates/mechanics.rs:100`, `qcm/hypothesis.rs:469`; `default_tolerance()` defined twice at `projection.rs:48` and `density_matrix.rs:40` |
| causality engine | depends on `algebra` for `Verdict` only; on none of `metric`, `linear`, `tensor`, `num_complex`, `physics` | six Euclidean norms at `deep_causality/src/types/context_node_types/space/{euclidean,ecef,ned}_space/metric.rs:13`, `space_time/euclidean_spacetime/metric.rs:13`, `tangent_spacetime/getters.rs:38,51`; a Minkowski interval with the speed of light inline at `traits/contextuable/space_temporal.rs:73,76` and a 4×4 metric literal at `space_time/tangent_spacetime/mod.rs:112-120` beside `deep_causality_metric`'s Lorentzian signatures and `deep_causality_physics::SPEED_OF_LIGHT`; a quaternion of four loose `f64` fields at `space/quaternion_space/metric.rs:32` beside `num_complex`'s `Quaternion<F>`; a `ScalarValue` marker trait at `traits/scalar/scalar_value.rs:30` with seven manual impls duplicating `algebra::Scalar`; Haversine with a hard-coded Earth radius at `space/geo_space/metric.rs:9` |

### 3.4 The causality engine is not precision-parametric

`deep_causality_core/src/alias/mod.rs:21,26` pins `NumericalValue = f64` and `FloatType = f64`.
The reasoning, inference, observation and context layers are `f64` only while every crate above
`num` in the stack is generic over `RealField`. The discovery loaders at
`deep_causality_discovery/src/types/data_loader/{csv,parquet}.rs` parse into
`CausalTensor<f64>` and up-cast afterwards, so higher precision is unreachable from file input;
mRMR computes in `f64` behind a generic signature. Given the stack's thesis that precision is a
parameter, this is the largest single inconsistency in the workspace.

### 3.5 `uncertain` is unused where it fits best

`deep_causality_uncertain` is a complete uncertain-programming type: arithmetic and comparison
operators, expected value and standard deviation with QMC variants
(`types/uncertain/uncertain_statistics.rs:17-86`), a sequential probability ratio test
(`algos/hypothesis/sprt_eval.rs`), a Sobol sampler. Deep Brain's design puts a real-valued
`confidence` on every edge (`deep_brain/Deep Brain — Technical Architecture.txt:372`) and leaves
the rule for composing confidence along a multi-hop chain unspecified in every document. No
Deep Brain document mentions `uncertain`. The causality engine does use it, through the CSM
uncertain parameter at
`deep_causality/src/types/csm_types/csm_parameter/uncertain_parameter/mod.rs:12-14`.

---

## 4. Domain-owned mathematics, correctly downstream

These are the consumers' proper subject matter and the sweeps found them in the right place.

- **Quantum.** Choi from Kraus, Kraus from Choi with the Hermitian-part guard, CPTP checks,
  Choi identity and composition (`deep_causality_quantum/src/types/qgates/channel.rs:55-491`);
  partial trace and `embed_on_legs` with hand-rolled strides
  (`qgates/operator_linalg.rs:174,251`); the partial-trace preservation boundary, documented,
  tested and Lean-backed as a bound rather than a preservation claim
  (`operator_linalg.rs:147-167,407`); the symplectic Pauli representation and stabilizer spans
  over `PackedGf2` and `Gf2Chain` (`qcode/logical_equivalence.rs`, `logical_pauli.rs`);
  the C₃ search by degree sequence (`qcm/faithfulness.rs:160-200`); the `CommutatorTolerance`
  family (`qcm/markov_freeze.rs:45,96`); Haruna gate builders. Two remarks: `choi_compose` is
  raw six-deep loops where einsum could express it, and Schmidt decomposition exists nowhere
  although `linear::svd` and `CausalTensor::svd_truncated` supply the machinery.
- **CFD.** Tensor-train operator assembly: `build_core`, `shift_plus` as a ripple-carry
  increment, gradient and Laplacian stencils, leg lifts
  (`deep_causality_cfd/src/tensor_bridge/operators.rs:19-290`); the closed-form Helmholtz
  resolvent by binary doubling with ADI splitting, a documented replacement of the AMEn solve
  (`tensor_bridge/acoustic_inverse.rs:54-251`); the positivity-preserving mask clamp
  (`tensor_bridge/mask.rs:63`); `conservation_round` and `positivity_floor`, both marked "not
  on a shipped path" (`solvers/qtt/compressible/imex.rs:187,216`); the Lax–Friedrichs flux
  family, first order by design, with no Roe, HLLC, MUSCL or WENO anywhere
  (`types/flow/duct_march_run.rs:380`, `solvers/qtt/compressible/euler_1d.rs:112,147`); the
  17-state error-state Kalman filter with sequential scalar updates
  (`navigation/eskf.rs:204,285`), whose doc at `:249-265` records that no test bounds the
  covariance asymmetry or checks `vᵀPv ≥ 0` after a long fold. The DEC path delegates
  correctly to `topology`'s Leray projection and stencil tables and to `linear::cg_solve`;
  no CG, multigrid, Jacobi or Gauss–Seidel is reimplemented.
- **Algorithms.** SURD, BOSS (search, grow-shrink tree, SEM-BIC score with the documented sign
  divergence from the reference at `brcd_boss_score.rs:26-34`), the clique-picking sampler and
  its `WeightedChoice` replacing the reference's alias table, F-node augmentation with the
  `2^du` cap. `brcd_boss_learn.rs:63` and `brcd_boss_score.rs:150` delegate covariance and
  conditional variance to `tensor`'s stats extension, the one place the stack is used for
  statistics.

Two generic pieces inside these could move down: partial trace and Kronecker embedding onto a
leg union belong in `tensor` beside `kronecker`; Meek rules and CPDAG closure belong in
`topology`'s mixed graph beside acyclicity.

Idle-code and stated-approximation inventory, for completeness. None of the four consumer sets
contains a `TODO`, `FIXME`, `todo!` or `unimplemented!` marker. Approximations are stated in doc
comments: Yeo–Johnson deferred (`brcd_gaussian.rs:21,53`); Meek R4 omitted; RKHS and BDeu
scores out of scope (`brcd_boss_score.rs:10`); chordality assumed; the Wilson loop as a
quadratic expansion (`deep_causality_physics/src/kernels/nuclear/qcd.rs:248-305`); Föppl–von
Kármán nonlinearity as an element-wise square (`kernels/condensed/moire.rs:263-316`);
Sweet–Parker simplified (`kernels/mhd/resistive.rs:41`); the Leray projector passing a
collocated checkerboard unchanged (`deep_causality_cfd/src/tensor_bridge/projection.rs:110-119`);
Van Loan omitted in the filter (`navigation/eskf.rs:185-186`); `hypothesis.rs:483` in quantum
naming the instrument-choice probe as approximated by factor replacement.

---

## 5. Per-consumer detail

### 5.1 CFD and physics

Neither crate has a crates.io math dependency. `deep_causality_cfd` depends on `algebra`,
`calculus`, `core`, `fft`, `file`, `haft`, `num`, `num_complex`, `num_dual`, `par`, `physics`,
`tensor`, `topology`, `uncertain`, and not on `linear`. Findings by recurrence:

1. Generic-scalar workarounds, about 575 sites (§3.1).
2. Small fixed-size dense linear algebra (§3.3): the filter's own matrix kit, three cofactor
   inverses, 3×3 products, a KS 4×4 action at `kernels/astro/ks_propagator.rs:214-232` that is
   domain-specific.
3. Spectral solvers duplicated across the topology boundary (§3.3).
4. Tensor-train operator assembly, crate-owned (§4). `deep_causality_tensor`'s
   `CausalTensorTrainOperator` docs at
   `causal_tensor_network/causal_tensor_train_operator/mod.rs:177,202`
   anticipate callers assembling `(S₊+S₋−2I)/Δx²` from shifts and ship no shift operator.
5. Seven root finders (§2.2).
6. Time integrators (§2.4).
7. Three linear and three multilinear interpolators (§2.3). Stencils delegate correctly to
   `topology::DecStencilTables` at `solvers/dec/dec_ns_rate.rs:287-305`.
8. Shock kernels: the ideal-gas equation of state restated four times across `euler_1d.rs:37`,
   `marcher_2d.rs:290`, `marcher_3d.rs`, `marcher_3d_fitted.rs` with different signatures, and
   not routed through the physics crate's compressible kernels; the shock-fitting path does
   call `vibrational_relaxation_kernel` and `rankine_hugoniot_temperature_kernel`.
9. Eigen and characteristic polynomials: closed-form symmetric 3×3 eigenvalues by the Smith
   trigonometric method (`coherent_structures.rs:211`), Cardano on the velocity-gradient cubic
   (`:131`), its discriminant (`:73`); the quantum geometric tensor at
   `kernels/condensed/qgt.rs:45` consumes eigenpairs from the caller, which is the right
   shape.
10. Sparse matvec in the MHD kernels (§3.3) and a cup-product wedge at `kernels/mhd/ideal.rs:267`
    beside `topology`'s wedge operator.
11. Two Kalman filters: the tensor-based Joseph-form linear filter at
    `kernels/dynamics/estimation.rs:110` delegates to `CausalTensor::inverse` and einsum; the
    ESKF avoids the inverse by sequential scalar updates. Two Joseph-form covariance updates
    with independent guard logic.
12. Reductions and diagnostics: `l2_residual`, energy-budget inner products, the QTT observe
    family at `solvers/qtt/observe.rs:25-268`, `convective_skew_generic` at
    `dec_ns_rate.rs:691` as the deliberately slow O(n²) equivalence oracle.

### 5.2 Causality engine, algorithms, discovery

No crates.io math dependency in any of the four crates; `discovery` has `csv` and `parquet`.
Several ports state that the reference used `numpy`, `sklearn` or `num-bigint` and were ported
in-tree to stay dependency-free. Findings by recurrence:

1. Statistics, regression and likelihoods in BRCD (§2.1).
2. Information theory in SURD, implemented twice (§2.1).
3. Combinatorics, MEC counting and uniform DAG sampling, computed two ways (§2.5, §2.6), with
   `mec_size` returning `T` in one crate module and `Result<usize>` in the other.
4. Structure-score search in BOSS (§2.2).
5. Graph algorithms (§2.5). Kahn scheduling and shortest paths in `deep_causality` delegate to
   `ultragraph` (`deep_causality/src/traits/causable_graph/graph/mod.rs:48-60,420`).
6. Elementary statistics and density estimation (§2.1), plus a hand-rolled FNV-1a at
   `deep_causality_discovery/src/.../cpdag_cache.rs:57` and a 4-decimal truncation equality at
   `deep_causality/src/traits/inferable/mod.rs:85` beside `num`'s epsilon machinery.
7. Robustness remarks worth keeping: `surd_algo.rs:355-364` widens the zero-increment floor
   under Miri because `log2` moves by a few ULP there; `surd_algo.rs:153,164` clones a tensor
   as a workaround for `CausalTensor::sum_axes(&[])` returning a scalar.
8. Generic-scalar workarounds: float-as-integer counting in the DAG sampler; `f64` internals
   behind generic APIs in mRMR; the `f64` ingest bottleneck; the fifteen lift shims; the `f64`
   aliases in `core`; the `ScalarValue` marker trait.
9. Geometry and metric duplication in `deep_causality` (§3.3). The time types
   `DiscreteTime`, `EuclideanTime`, `LorentzianTime`, `EntropicTime`, `SymbolicTime` are
   containers with no interval or temporal arithmetic; `TimeScale` has a `Display` impl and
   no unit conversion.

### 5.3 Quantum

10 836 lines across 55 files. Depends on `algebra`, `core`, `haft`, `homology`, `linear`,
`metric`, `multivector`, `num`, `num_complex`, `num_rational`, `tensor`, and optionally
`uncertain` and `deep_causality`; not on `rand`, `calculus`, `fft` or `num_dual`. Findings by
recurrence:

1. Open-coded complex arithmetic, about 118 sites (§3.3). The root of the next two.
2. The entrywise max-modulus residual idiom, five copies.
3. `frobenius_norm` duplicated from `linear`, one definition, six call sites.
4. splitmix64, inverse-CDF sampling and Bernoulli draws duplicated from `rand`.
5. Tolerance family bypassed at three sites and the counting-loop rank (§3.1).
6. `choose3` twice and a quadratic tuple dedup.
7. The GF(2) tableau on `Vec<bool>`.
8. Crate-owned domain mathematics, about fifteen functions (§4).

GF(2) linear algebra otherwise delegates correctly: `image_basis_gf2`, `rank_gf2`,
`csr_to_packed_gf2_mod2` from `linear`, `Gf2Chain` and `betti_number_over` from `homology`. The
stabilizer-span membership test at `qcode/logical_equivalence.rs:467-476` uses a scratch trailing
column because `linear` exports no column concatenation; `openspec/notes/quantum/qcl-gaps.md:376`
records that gap, and `:852-854` records that Bhattacharyya appears in no other `.rs` file.

### 5.4 Deep Brain

The material is six documents under `deep_brain/`: `deepbrain.md` (current architecture, August
2026, primary source), `Deep Brain — Technical Architecture.txt` (Rev 3; mechanics superseded,
taxonomy and §7–§10 retained), `The Philosophical Foundation of Deep Brain.txt`,
`Deep Brain — Next Generation.txt` (zones, deferred at `deepbrain.md:661-667`),
`deep_brain_mvp_plan.md` (superseded; the only enumeration of grafeo's native algorithm surface at
`:69-110`), `knowledge-propagation-process.md`. Negative findings: no `openspec/notes/deep_brain/`
directory, no `deep_docs` crate and no occurrence of that string, no Rust crate for Deep Brain yet,
and `grafeo` appears in no `Cargo.toml` or `MODULE.bazel`.

The framing fact: `deepbrain.md:692` removes `ultragraph` from the crate stack, and
`deepbrain.md:643-645` states that grafeo's own algorithms cover PageRank, shortest path, Louvain
and centrality in the meantime. Stage 1 (`deepbrain.md:616-624`) requires no mathematics beyond
integer comparison and traversal; Stages 2 and 3 delegate to grafeo; Stage 3
(`deepbrain.md:636-649`) revives an in-memory projection for gap centrality, dissolution chains
and domain hierarchy traversal. Nothing Deep Brain currently plans is blocked by the stack.

| Area | Item | Source | Verdict |
|---|---|---|---|
| graph | PageRank | `deep_brain_mvp_plan.md:80`, `deepbrain.md:643` | missing everywhere |
| graph | betweenness centrality | `deep_brain_mvp_plan.md:83`, `Technical Architecture.txt:566` | in `ultragraph/src/traits/graph_algo_centrality.rs:11,18`, the crate the design removed |
| graph | Louvain | `deep_brain_mvp_plan.md:82`, `deepbrain.md:643` | missing everywhere |
| graph | shortest path, Dijkstra | `deep_brain_mvp_plan.md:81`, `Technical Architecture.txt:564,1184` | in `ultragraph/src/traits/graph_algo_pathfinder.rs:14-28` |
| graph | reachability, transitive closure | `Technical Architecture.txt:565`, `deepbrain.md:427` | pairwise `is_reachable` only |
| graph | cycle detection, topological sort | `Technical Architecture.txt:567-568`; property-test invariant at `deep_brain_mvp_plan.md:627-629` | provided in `ultragraph` and `topology` |
| graph | components, articulation points, bridges | `deep_brain_mvp_plan.md:83` | provided in `ultragraph/src/traits/graph_algo_structural.rs:11-37` |
| graph | spectral clustering | implied by `Technical Architecture.txt:599` | eigensolvers exist; no graph Laplacian from adjacency, no k-means |
| graph | zone-scoped traversal | `Next Generation.txt:702-720` | missing, deferred |
| graph | domain-distance ranking | `Technical Architecture.txt:623` | expressible via shortest-path length; no named metric |
| graph | structural similarity of causal chains | `Technical Architecture.txt:969`; `Philosophical Foundation.txt:441` | undefined and missing; the primary cross-disciplinary signal |
| similarity | cosine similarity | `deepbrain.md:195,306` | `dot` and `norm_l2` on `DenseVector` (`deep_causality_linear/src/types/dense_vector/mod.rs:97,236`) compose it; no named function. `num`'s `cos` at `deep_causality_num/src/float/mod.rs:689-701` is trigonometry |
| similarity | inner product | `deepbrain.md:195` | provided |
| similarity | L1, L2, L∞ distances | `deepbrain.md:195` | norms provided at `dense_vector/mod.rs:229-252` and `linear/src/algorithms/norms.rs`; distances not named |
| similarity | ANN index, HNSW | `deepbrain.md:195,306,333` | missing; delegated |
| similarity | LSH, MinHash, Jaccard | not named; the dedup problem is open at `deepbrain.md:744-745` | missing |
| similarity | maximal marginal relevance | `deepbrain.md:339-344` | missing; needs pairwise cosine |
| similarity | 384-d embeddings | `deepbrain.md:253,306,756-757` | external model by design |
| retrieval | BM25 | `deepbrain.md:55-56,196,307,546-548`; the lexical leg measured at 97 ms against 3202 ms for the vector leg | missing everywhere |
| retrieval | TF-IDF | implied | missing |
| retrieval | reciprocal rank fusion | `deepbrain.md:197,547` | missing; no top-k utility either |
| retrieval | extraction method as a ranking term | `deepbrain.md:160-162`; taxonomy at `:147-152` | Deep Brain's own; combination rule unspecified |
| retrieval | topology boost | `deep_brain_mvp_plan.md:355-357,420-428` | formula never written down |
| retrieval | Bayesian provenance score | absent under that name | the corpus uses a categorical confidence lattice, `Observed \| Inferred \| Asserted` and `Hard \| Soft \| Provisional` (`Technical Architecture.txt:152,207`), plus a real `extraction_confidence` (`:149`) and edge `confidence` (`:372`); `Philosophical Foundation.txt:379` states these are policy, not tunable parameters. A Bayesian proposal contradicts that document |
| statistics | entropy, mutual information | not named | provided in `algorithms`, not the stack |
| statistics | KL divergence | not named | missing; `divergence` in `topology` is the vector-calculus operator |
| statistics | descriptive statistics | `Technical Architecture.txt:1001` | provided in `tensor`'s stats extension |
| statistics | threshold policy table | `Technical Architecture.txt:795-816`, `deepbrain.md:559-566` | a lookup table |
| statistics | bridge-condition ratio | `Technical Architecture.txt:1019-1023`, `deepbrain.md:653-655`; conceded a hypothesis at `:1280` | counting; the Stage 4 gate |
| probability | Bayesian updating | not named; revision is structural, `deepbrain.md:575-582` | a Bayesian engine exists in BRCD for structure discovery only |
| probability | uncertainty propagation along chains | conceptually at `Philosophical Foundation.txt:222` | `uncertain` fits and is unmentioned (§3.5) |
| probability | distributions | not named | provided in `rand`; no Beta or Dirichlet |
| other | bitemporal validity windows | `deepbrain.md:200,348-356` | delegated to grafeo epochs; no interval logic in the stack |
| other | optimisation, time series | absent from every document | not needed |

Three structural mismatches: the retrieval mathematics is wholly absent from the stack and
wholly delegated, with `deepbrain.md:747-748` already flagging grafeo's release cadence; the
named graph algorithms are split across a crate the design removed, and PageRank and Louvain
are the entire in-house gap for Stage 3; and the two most important measures, chain similarity
and near-duplicate contextoids, are undefined rather than unimplemented.

---

## 6. Ranked work list

Ordered by consumers served per line written. Tiers refer to the stack's dependency tiers.

| # | Work | Target | Consumers served | Notes |
|---|---|---|---|---|
| 1 | Widen `Real` with `cbrt`, `max`, `min`, `mul_add`, `hypot`; add `ToPrimitive` to `RealField`; blanket `Distribution` impls over `RealField + FromPrimitive` in `rand`, plus `shuffle` and weighted choice | `algebra`, `rand` | all four | Breaking across sixteen crates; `num` has just taken a breaking bump. Lifts the 575 workarounds, the twelve silent `½ → 1` fallbacks, the `f64` sampling in physics and the DAG sampler, and the divergent Float106 stream |
| 2 | A statistics crate at tier 3 over `linear` and `rand`: descriptive statistics on slices, Pearson, ridge and logistic regression, log-sum-exp, entropy and conditional entropy, mutual information, KL and Jensen–Shannon, Bhattacharyya and Hellinger, equal-width and quantile binning | new crate | algorithms, discovery, quantum, CFD diagnostics, Deep Brain | Absorbs three log-sum-exp, two entropy, two Gaussian log-density and two ridge copies; makes mRMR precision-parametric; supplies the KL that Deep Brain drift would use |
| 3 | Root finding and minimisation in `calculus`: bisection, Brent, Newton with a supplied or dual-number derivative, fixed point with a convergence signal, a line-search minimiser; Strang and IMEX scaffolds; an adaptive Runge–Kutta | `calculus` | physics, CFD, algorithms | Absorbs seven solvers and the hand-rolled RK4; the fixed-point that returns silently on non-convergence gets an error path |
| 4 | Interpolation with an explicit out-of-range policy enum: linear with binary search, cubic spline, multilinear over `2^D` corners; a tabulated-curve quadrature | `calculus` | CFD, physics | Absorbs six interpolators and unifies three policies |
| 5 | Make `spectral_poisson_solve` public with a selectable eigenvalue convention; add `partial_trace` and Kronecker embedding onto a leg union to `tensor`; add Meek rules and CPDAG closure to `topology`'s mixed graph; export column concatenation from `linear` | `topology`, `tensor`, `linear` | CFD, quantum, discovery | Each removes a verbatim duplicate |
| 6 | Graph mathematics with a home: PageRank, Louvain, spectral clustering over `linear`'s eigensolver with k-means, transitive closure and reverse reachability sets; `cosine_similarity`, a distance family and top-k on `DenseVector` | `topology` or a new tier-4 crate; `linear` | Deep Brain, discovery, causality engine | The similarity functions are a few lines over existing primitives and are the cheapest insurance against a grafeo dependency |
| 7 | The gamma family in `num`; Beta, Gamma, Dirichlet, Student-t in `rand`; a general combinatorics surface beside the Stirling numbers; a bounded exact counting type with a saturation signal | `num`, `rand` | algorithms, quantum | The DAG sampler's documented silent-wrongness ceiling needs the last item |
| 8 | Add the missing dependency edges and use them: CFD on `linear` for the filter's matrix kit, algorithms on `linear` for the LU solve, quantum on `rand` and on `num_complex`'s operators; replace the fifteen lift shims and the three physics helpers with `num::lift` | consumers | CFD, algorithms, quantum, physics | Consumer discipline; no new mathematics |
| 9 | Make the causality engine precision-parametric: lift the `f64` pins in `core`'s aliases, load files into `CausalTensor<T>`, replace the `ScalarValue` marker with `algebra::Scalar`, route geometry through `metric`, `num_complex` and the physics constants | `core`, `deep_causality`, `discovery` | causality engine | The largest inconsistency with the stack's thesis; also the largest diff |

Decisions the maintainer owns:

- The shape of item 1, since widening `Real` is a breaking change everywhere.
- Whether item 2 is a crate or modules inside `tensor` and `rand`; a crate keeps `tensor` from
  growing statistics it does not need for its own purpose.
- Where graph mathematics lives, given that `ultragraph` is infrastructure and Deep Brain has
  removed it from its stack.
- Whether the causality engine's `f64` pins are a decision or an omission; the note treats them
  as an omission.

---

## 7. Traps for the next reader

- `deep_causality_metric` is Clifford-algebra signatures `Cl(p, q, r)` and Lorentzian sign
  conventions; it has nothing to do with distance metrics.
- `divergence` in `deep_causality_topology` is the vector-calculus operator, not a statistical
  divergence.
- `cos` in `deep_causality_num` is trigonometry, not cosine similarity.
- `mec_size` exists twice with different return types and different failure modes.
- The three interpolators disagree on what happens out of range.
- `R::from_f64(x).unwrap_or_else(R::one)` is a silent substitution, not a fallback.

---

## 8. Sources

Four sweeps on 2026-09-03, each reading the consumer's source and grepping the stack for an
equivalent before recording a finding: CFD and physics; `deep_causality`, `core`, `algorithms`,
`discovery`; `quantum` with `openspec/notes/quantum/qcl-design-note.md` §3.3, §5.1, §6.4 and
`qcl-gaps.md`; the six Deep Brain documents. Verification greps against the stack for the
combinatorics surface of `num`, the method set of `Real`, the `Distribution` implementations of
`rand`, and the aliases in `core`. Companion documents:
`openspec/notes/archive/unified_math/unified_math_gaps.md` for the categorical layer's gaps, and
`deep_causality_unified_math/README.md` for the stack's current shape and its precision
sections.
