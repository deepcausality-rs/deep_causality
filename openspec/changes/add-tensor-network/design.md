## Context

The full design is locked in `openspec/notes/tensor-network/Tensor-Network-Spec.md`; this document
captures the load-bearing technical decisions and their rationale so the specs and tasks have a single
reference. The module lands inside `deep_causality_tensor`, which already provides `CausalTensor<T>` with
`svd`, `reshape`, `permute_axes`, `slice`, `matmul`, `ein_sum`, `tensor_product`, and the
`deep_causality_num` scalar tower (`Real`/`RealField`/`Field`/`Complex`/`Dual`/`Normed`/`Scalar`) and the
`deep_causality_haft` HKT/`Arrow` traits. A tensor-train (MPS) and tensor-train-operator (MPO) layer
factors a rank-`d` tensor into a chain of small cores, turning `nᵈ` storage into `O(d·n·r²)` and making
high-order SURD/CFD/UQ tractable.

Hard constraints from the workspace (`AGENTS.md`, root `Cargo.toml`): static dispatch only (no `dyn`),
`unsafe_code = "forbid"`, no macros in lib code, no external runtime crates, one-type-one-module layout,
tests mirroring the src tree with 100 % coverage, and precision generic over the scalar.

## Goals / Non-Goals

**Goals:**
- A complete 1-D tensor-network stack (TT state + MPO) serving SURD, CFD, and UQ from one primitive set.
- Precision a parameter at every stage (`f32`/`f64`/`Float106`), with complex and forward-mode-AD scalars
  first-class.
- Correctness and performance held to explicit complexity targets and reference-checked numerics; built
  in complete, fully-tested stages.

**Non-Goals:**
- Arbitrary-topology tensor networks (PEPS/MERA, NP-hard contraction ordering) — out of scope.
- Reverse-mode AD (a tape, not a scalar) — out of scope; forward-mode via `Dual<T>` is in scope.
- GPU/distributed contraction; a separate crate (this is a module inside `deep_causality_tensor`).

## Decisions

**D1 — Module inside `deep_causality_tensor`, not a new crate.** All consumers (physics/CFD, algorithms,
uncertain) already depend on the tensor crate, so a module keeps the dependency graph clean. *Alternative:*
a `deep_causality_tensor_network` crate — deferred until arbitrary-topology networks justify it.

**D2 — Trait/type split mirroring `Tensor`/`CausalTensor`.** Behaviour lives in traits `TensorTrain<T>` /
`TensorTrainOperator<T>` (in `src/traits/`); concrete types `CausalTensorTrain<T>` /
`CausalTensorTrainOperator<T>` (in `src/types/causal_tensor_network/`) carry inherent constructors. This is
the established crate pattern; *alternative* (inherent-only methods) would diverge from `CausalTensor`.

**D3 — Robust truncated SVD is Stage 0, a hard gate with no fallback.** TT-SVD and rounding call SVD per
bond per sweep and compound its error; the existing power-iteration deflation loses orthogonality on
clustered spectra. Stage 0 adds a truncated thin-SVD (Golub–Kahan bidiagonalization + implicit-shift QR,
reorthogonalized) and Householder QR as *additions* to `ops/tensor_svd*`, validated against checked-in
reference fixtures at every precision. *Alternative* (ship on the existing SVD with caveats) was rejected
as a correctness corner-cut.

**D4 — Layered scalar bounds (Option A, from Stage 0), using crate-native traits.** The norm/SVD layer
bounds on `T: Normed` (`type Real`, `modulus_squared`, `scale_by_real`); the TT-algebra/AD layer on
`T: Scalar` (`Real + Div + FromPrimitive`, **never** gratuitously `Field`); complex conjugation via
`ComplexField`. Three rules enforce no-retrofit generality: no gratuitous `Field` bound, magnitudes via
`Normed::modulus_squared(): Normed::Real` (never an assumed order on `T`), divide only by checked-nonzero
pivots. *Alternative* (D4′: `RealField`-only through Stage 3, generalize later) was rejected — relaxing the
bound later touches every kernel signature and risks silent total-inverse/ordering assumptions.

**D5 — Forward-mode AD is nearly free; reverse-mode is out.** Because `Dual<T>: Real + Div` (but not
`Field`), the real kernels run over `Dual<f64>` and carry derivatives by the chain rule — *provided* the
bound is `Scalar`, not `RealField`. Truncation/pivot decisions branch on the real part (Dual's
`PartialOrd`), so gradients flow through the retained subspace; rank-change ties are measure-zero and
documented. Reverse-mode needs graph recording, not a scalar — excluded.

**D6 — Composition is `EndoArrow`, the HKT stack is the storage functor only.** `CausalTensorTrainOperator`
implements `EndoArrow<CausalTensorTrain<T>>` (`compose` = `>>>`, `identity` = `Id`, `apply` = the action),
the repo's Causal-Arrow algebra. The HKT witness implements only `Functor`/`Foldable`/`Pure` over **core
storage** (scalar-type conversion, e.g. `f64→f32`, `real→dual`). `Monad`/`CoMonad`/`Applicative` are
deliberately **absent** (List-flatten has no counterpart in a factored chain; a comonadic stencil is just
an MPO). `Promonad`/`ParametricMonad` are declined while dims are runtime values. *Alternative* (full HKT
mirror) is a category error on a compressed format.

**D7 — Elementwise maps: option (b), three honest methods.** `scale` (exact linear), `add_scalar` (exact
affine via a rank-1 ones-train), `apply_nonlinear(f, &CrossConfig) -> (Self, residual)` (explicit cross
re-approximation). No `map_elementwise`. *Alternative* (a single `map_elementwise` "linear only") was
rejected: a closure cannot be checked for linearity, so a nonlinear `f` would silently return a wrong
tensor.

**D8 — Rounding is the universal, lax pressure valve.** `add`/`hadamard`/`apply`/`compose` grow bond
dimension exactly (sum/product), then a paired `*_rounded(&Truncation)` recompresses. Algebra and category
laws hold exactly un-rounded and **to tolerance** under truncation; tests assert them to `tol`.

**D9 — Algebra-trait impls mirror `CausalTensor`.** `CausalTensorTrain` implements `Module<T>` (clean,
exact `scale`), `AbelianGroup`/`AddGroup` (via `add`), and `Ring` (via `hadamard`) through marker traits +
the `deep_causality_num` blanket impls, with documented bond-growth (lax) and shape-dependent `Zero`/`One`
caveats. `Field`/`InvMonoid` are **not** implemented — a tensor train has no multiplicative inverse.

**D10 — Staged delivery, each stage complete.** Stage 0 numerics → Stage 1 TT state → Stage 2 MPO + cross
+ ALS solvers → Stage 3 DMRG/advanced → Stage 4 complex/dual instantiation. Staging is dependency ordering,
not partial delivery; every stage is fully implemented and 100 %-tested before the next.

## Risks / Trade-offs

- **SVD robustness is the single gating dependency.** [Compounded SVD error degrades every TT op] →
  Stage 0 reference fixtures (full-precision `f64`, asserted at `f32`/`f64`/`Float106`) and orthogonality
  residual checks `‖UᵀU − I‖ ≤ k·ε` gate Stage 1.
- **TT-cross convergence is heuristic.** [ACA/maxvol pivoting can stall on adversarial oracles] → expose
  `max_sweeps`/`rank_cap`, return a residual estimate, never claim guaranteed accuracy.
- **Nonlinear elementwise inflates rank.** [`log`/`exp`/indicator of a TT can be high-rank] → confine to
  `apply_nonlinear` with explicit `CrossConfig` + returned residual; never a silent `map_elementwise`.
- **Forward-mode AD non-smoothness.** [Rank changes at singular-value ties are non-differentiable] →
  measure-zero, documented; gradients validated against finite differences in Stage 4 tests.
- **Performance without BLAS.** [No external linear-algebra crate] → reuse in-crate `matmul`/`ein_sum`,
  in-place gauge sweeps via `get_mut`, tracked `CanonicalForm`, guarded `to_dense`/`from_fn`
  (`RankExceeded`); each §7 complexity target has a matching criterion benchmark as its empirical witness.
- **Shape-dependent `Zero`/`One` in the algebra impls.** [Parameterless `Zero::zero()` is ambiguous for a
  TT] → mirror `CausalTensor` (degenerate empty/`Default` identity; shape-taking `zeros`/`ones`
  constructors; `add`/`hadamard` require matching `phys_dims`).
