<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Tensor-Network module specification — `deep_causality_tensor`

**Status:** design locked for staged implementation. Nothing here is implemented yet.
**Home:** a module **inside `deep_causality_tensor`** — *no separate crate*. Traits in
`deep_causality_tensor/src/traits/`, types in `deep_causality_tensor/src/types/causal_tensor_network/`,
tested in the mirror `deep_causality_tensor/tests/...`.
**Goal:** a **fully fledged, correct, and performant** tensor-network implementation serving SURD, CFD,
and UQ from one set of primitives, with MPO and DMRG included. Built and tested over multiple **complete**
stages — each stage is fully implemented, fully tested (100 % coverage), and correct before the next
begins. **No MVP-now/full-later; no shortcuts; no corner-cutting.** Staging is dependency ordering, not
partial delivery.

Honesty convention (as elsewhere in `openspec/notes`): **[holds]**, **[holds under precondition]**,
**[open]**, **[speculative]**.

---

## 0. Decisions locked (this revision)

| # | Decision | Resolution |
|---|---|---|
| Naming | Trait vs concrete-type split | **Trait `TensorTrain<T>` + type `CausalTensorTrain<T>`**; **trait `TensorTrainOperator<T>` + type `CausalTensorTrainOperator<T>`** — mirrors the existing `Tensor` trait / `CausalTensor` type pattern. |
| Packaging | Module vs crate | **Module only, inside `deep_causality_tensor`.** No `deep_causality_tensor_network` crate at this time. |
| SVD | Robustness | **Robust truncated SVD is Stage 0, a hard prerequisite.** No fallback onto the existing power-iteration SVD. |
| HKT/Arrow | Compositional shape | **Confirmed:** `Functor`/`Foldable`/`Pure` storage functor only; **no** `Monad`/`CoMonad`/`Applicative` on the state; operator composition is **`EndoArrow<CausalTensorTrain<T>>`**. `Promonad`/`ParametricMonad` explicitly **declined** — they buy nothing while bond/physical dims are runtime values (see §3.6). |
| Precision | Scalar genericity | **Precision is a parameter at every stage.** Every type, trait, constructor, algorithm, and test is generic over the scalar (§3.5). No concrete `f32`/`f64` anywhere in lib code — including tolerances, seeds, and constants. |
| Elementwise | `map_elementwise` | **Resolved — option (b).** No `map_elementwise`. Three honestly-named tools: `scale` (exact linear), `add_scalar` (exact affine), `apply_nonlinear(f, &CrossConfig) -> (Self, residual)` (explicit cross re-approximation). See §6.1, §9, §13. |
| Scalars | real / complex / dual | **In scope, via a layered bound (§3.7), not `RealField`-only.** Real path tested at **`f32`, `f64`, and `Float106`** (Stage 1–3); **complex** `Complex<T>` and **forward-mode AD** `Dual<T>` are first-class targets — bound designed for them from Stage 0 (**Option A, resolved §13.2**), instantiated/tested in Stage 4. Reverse-mode AD out of scope (needs a tape, not a scalar). |
| Bounds | crate-native abstractions | **Resolved (§3.7, §3.8):** norm/SVD layer bounds on **`Normed`** (`type Real`); algebra/AD layer on **`Scalar`** (`Real + Div + FromPrimitive`); complex conjugation via `ComplexField`. The TT types implement **`Module`/`AddGroup`/`Ring`** (and optional `Normed`) via the same marker-trait + num blanket-impl pattern as `CausalTensor`. |
| Delivery | Staging | **Stage 0 prep → Stage 1 basics → Stage 2 MPO/solvers → Stage 3 DMRG/advanced → Stage 4 scalar generality (complex + dual).** Each stage complete + fully tested. |

All design decisions are now **locked**; no open items remain (the elementwise and scalar-aggressiveness
questions are resolved in §13).

---

## 1. Scope and non-goals

### 1.1 In scope

A **one-dimensional tensor-network** stack — matrix-product states (MPS) and matrix-product operators
(MPO), known in numerical analysis as the **tensor-train (TT)** and **TT-operator** formats — built on
top of `CausalTensor<T>` and its existing `svd`, `reshape`, `permute_axes`, `slice`, `matmul`,
`ein_sum`, and `tensor_product` operations, plus the Stage-0 linear-algebra additions (robust truncated
SVD, Householder QR).

All three target consumers are 1-D-expressible:

| Consumer | What it needs | TT/MPO maps to it as |
|---|---|---|
| **SURD** (`deep_causality_algorithms`) | compress + marginalize high-order joint distributions; evaluate information terms | TT of a joint PMF/PDF; partial contraction = marginalization; TT-cross to *build* the joint without forming it dense |
| **CFD** (`deep_causality_cfd`) | compress a flowfield; apply differential operators; advance in time | QTT of the field; differential operator as MPO; explicit step = MPO·MPS + round; implicit step = ALS linear solve |
| **UQ** (`deep_causality_uncertain` / downstream) | represent a high-dim response surface; integrate; fit from samples | TT response surface; integration = contraction with quadrature vectors; TT-completion via ALS |

### 1.2 Explicit non-goals

- **General tensor networks of arbitrary graph topology** (PEPS, MERA, arbitrary contraction-order
  optimization) — a different, much larger problem (NP-hard contraction ordering, 2-D environments).
  Out of scope. **[speculative]**
- **GPU / distributed contraction.** CPU only; an opt-in `parallel` feature is a Stage-3+ performance
  item (§12), not core.
- **Reverse-mode autodiff (adjoint/tape).** Out of scope — it is not a scalar type but a graph-recording
  mechanism. **Forward-mode AD is in scope** via the `Dual<T>` scalar (§3.7), which is a different and
  supported thing.

The two scalar generalizations I had previously listed as out of scope — **complex scalars** and
**autodiff through the network** — are **in scope** after investigating `deep_causality_num`. Both were
mis-scoped; see §3.7 for the corrected analysis and the bound that admits them.

---

## 2. Module layout (trait/type split, mirrors the crate convention)

The crate puts **operation traits in `src/traits/`** (e.g. `Tensor` in `src/traits/tensor.rs`) and
**concrete types in `src/types/`** (e.g. `CausalTensor` in `src/types/causal_tensor/`). The
tensor-network module follows that split exactly. One type per folder module; `mod.rs` holds the type +
inherent constructors; each operation/trait-impl group in its own file; tests mirror the tree with the
`_tests` suffix.

```
src/traits/
  tensor_train.rs              # trait TensorTrain<T>            (behaviour of the MPS/TT state)
  tensor_train_operator.rs     # trait TensorTrainOperator<T>    (behaviour of the MPO)

src/types/causal_tensor_network/
  mod.rs                       # module wiring + re-exports
  truncation/
    mod.rs                     # Truncation<T> value type + constructors
  cross_config/
    mod.rs                     # CrossConfig<T> (TT-cross controls)
  solve_config/
    mod.rs                     # SolveConfig<T> (ALS/DMRG sweep controls)
  canonical_form/
    mod.rs                     # CanonicalForm enum (None | LeftAt | RightAt | Mixed)
  causal_tensor_train/         # the MPS / TT state  (type CausalTensorTrain<T>)
    mod.rs                     # struct def (private fields) + inherent constructors
    getters/mod.rs             # cores(), order(), phys_dims(), bond_dims(), max_bond(), canonical_form()
    construct/
      from_dense.rs            # TT-SVD constructor
      from_fn.rs               # from index->value closure (guarded element-count cap)
      special.rs               # zeros, ones, random_seeded (dev)
      cross.rs                 # TT-cross constructor from an oracle (Stage 2)
    tensor_train_impl/mod.rs   # impl TensorTrain<T> for CausalTensorTrain<T> (the trait surface)
    canonical/mod.rs           # left/right/mixed canonicalization via QR; gauge moves
    round/mod.rs               # SVD rounding to a Truncation target
    ops/
      add.rs                   # TT + TT (exact bond growth)
      scale.rs                 # scalar * TT
      hadamard.rs              # elementwise TT * TT
      inner.rs                 # <a|b> -> scalar; norm()
      contract_full.rs         # to_dense() (guarded element cap)
      marginalize.rs           # partial contraction over a subset of sites
      eval.rs                  # single-entry evaluation
      integrate.rs             # contraction against per-site weight vectors (Stage 2)
      qtt.rs                   # quantization reshape helpers n=2^L <-> L binary sites (Stage 2)
  causal_tensor_train_operator/ # the MPO  (type CausalTensorTrainOperator<T>)
    mod.rs                     # struct def + inherent constructors (identity, from_dense)
    getters/mod.rs
    tensor_train_operator_impl/mod.rs  # impl TensorTrainOperator<T> for the type
    apply/mod.rs               # MPO . MPS -> MPS ; MPO . MPO -> MPO
    round/mod.rs               # operator rounding
    arrow/mod.rs               # impl EndoArrow<CausalTensorTrain<T>> (operator category)
  solve/                       # shared sweep engine (private module; surfaced via free fns)
    mod.rs                     # alternating one-/two-site sweep driver
    linear.rs                  # solve A x = b in TT form           (Stage 2)
    fit.rs                     # TT fit / completion from samples   (Stage 2)
    eigen.rs                   # ground-state DMRG3S eigensolve      (Stage 3)

src/extensions/                # crate-level HKT witnesses (next to CausalTensorWitness)
  ext_hkt_tensor_train.rs      # CausalTensorTrainWitness: HKT + Functor + Foldable + Pure (§3.6)
```

Tests mirror this under `tests/traits/...` and `tests/types/causal_tensor_network/...`, registered in each
`mod.rs` and in `tests/BUILD.bazel`. Shared fixtures go in `src/utils_tests/` (Bazel cannot read helpers
from the tests tree, and they must be covered).

---

## 3. Core types and traits

### 3.1 Trait `TensorTrain<T>` and type `CausalTensorTrain<T>` (MPS / TT)

A rank-`d` tensor `A[i₀,…,i_{d-1}]` factored into a chain of rank-3 **cores** `G_k` of shape
`[r_k, n_k, r_{k+1}]`, boundary bonds `r₀ = r_d = 1`:

```
A[i₀,…,i_{d-1}] = G₀[:, i₀, :] · G₁[:, i₁, :] · … · G_{d-1}[:, i_{d-1}, :]
```

```rust
/// A tensor-train (matrix-product-state) factorization of a rank-`order` tensor.
/// Cores are rank-3 `CausalTensor<T>` of shape `[r_k, n_k, r_{k+1}]`, boundary bonds == 1.
/// Fields are private; access is through getters. Precision is the scalar `T`.
#[derive(Debug, Clone, PartialEq)]
pub struct CausalTensorTrain<T> {
    cores: Vec<CausalTensor<T>>,   // len == order; core k has shape [r_k, n_k, r_{k+1}]
    phys_dims: Vec<usize>,         // [n_0, …, n_{order-1}]  (cached)
    canonical: CanonicalForm,      // tracked orthogonality center
}
```

The **trait `TensorTrain<T>`** (in `src/traits/tensor_train.rs`) declares the *behaviour* — the
transformation/query surface — exactly as `Tensor` does for `CausalTensor`. Inherent constructors stay
on the type. Split:

- **Inherent on `CausalTensorTrain<T>`** (constructors): `from_dense`, `from_fn`, `zeros`, `ones`,
  `random_seeded`, `cross` (Stage 2).
- **On trait `TensorTrain<T>`** (behaviour): `round`, `left_canonicalize`, `right_canonicalize`,
  `canonicalize_at`, `norm`, `inner`, `add`, `scale`, `add_scalar`, `hadamard`, `apply_nonlinear`
  (Stage 2; needs `cross`), `to_dense`, `marginalize`, `eval`, `integrate` (Stage 2). Getters live in
  `getters/mod.rs` as inherent methods (matching `CausalTensor`). The elementwise trio
  (`scale`/`add_scalar`/`apply_nonlinear`) is the resolved option (b) — see §6.1, §13.

### 3.2 Trait `TensorTrainOperator<T>` and type `CausalTensorTrainOperator<T>` (MPO)

An operator mapping one TT index space to another; cores rank-4 `W_k` of shape
`[r_k, n_out_k, n_in_k, r_{k+1}]`.

```rust
/// A matrix-product operator over the same site structure as a `CausalTensorTrain`.
/// Cores are rank-4 `[r_k, n_out_k, n_in_k, r_{k+1}]`. Fields private.
#[derive(Debug, Clone, PartialEq)]
pub struct CausalTensorTrainOperator<T> {
    cores: Vec<CausalTensor<T>>,
    out_dims: Vec<usize>,
    in_dims: Vec<usize>,
}
```

- **Inherent constructors:** `identity(dims)`, `from_dense`.
- **Trait `TensorTrainOperator<T>`:** `apply` (MPO·MPS), `compose` (MPO·MPO), `round`, `to_dense`,
  `transpose`/`adjoint`.

### 3.3 `CanonicalForm`

```rust
/// Which orthogonality sweep (if any) is currently materialized. Lets round/inner/solve
/// skip redundant re-canonicalization. Copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CanonicalForm {
    #[default] None,
    LeftAt(usize),    // cores [0..k] left-orthonormal
    RightAt(usize),   // cores [k..] right-orthonormal
    Mixed(usize),     // orthogonality center on core k
}
```

### 3.4 `Truncation<T>`

```rust
/// Truncation policy for any SVD-based step. A singular value is kept iff its index < max_bond
/// AND it passes both tolerance gates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Truncation<T> {
    max_bond: usize,   // hard cap on every bond dimension r_k
    rel_tol: T,        // drop σ_i once σ_i / σ_0 < rel_tol
    abs_tol: T,        // drop σ_i once σ_i < abs_tol
}
```

Constructors: `by_bond`, `by_tol`, `new`. Threaded **explicitly** into every lossy op — no hidden
global default, matching the crate's configuration-is-a-parameter stance. A `Truncation` with
`max_bond == 0` or negative tolerance is rejected with `InvalidParameter`.

### 3.5 Scalar bound — precision is a parameter at every stage

**Binding invariant.** Every type (`CausalTensorTrain<T>`, `CausalTensorTrainOperator<T>`,
`Truncation<T>`, `CrossConfig<T>`, `SolveConfig<T>`), every trait (`TensorTrain<T>`,
`TensorTrainOperator<T>`), every constructor, every algorithm (Stage 0 → 3), and every test is generic
over the same real-scalar parameter. Compiling at `f32` and `f64` must be the *only* difference between
two precisions; nothing downstream may pin a concrete float.

The **real-scalar instantiation** uses the crate's existing SVD/Cholesky surface:

```rust
T: Clone + RealField + Zero + One + Sum + PartialEq
```

but this is **not** the kernel bound — see §3.7. The actual bounds are **layered** so the same code admits
complex (`Complex<T>`) and dual (`Dual<T>`, forward-mode AD) scalars: the TT-algebra/AD layer takes
`Real + Div` (never gratuitously `Field`), and the norm/SVD layer takes a norm-bearing bound with an
associated real field. `RealField` here is the *convenience* instantiation for `f32`/`f64`, not a ceiling.

`RealField` **is** `Real + Field`, so all transcendental/real machinery the algorithms need —
`sqrt`, `abs`, `epsilon`, `pi`, `exp`, `ln`, comparison — comes from the `Real` half and stays
precision-generic. `FromPrimitive` is added only where integer→scalar conversion arises (cross-pivot
counters, quadrature node counts, the `i` in `1/(i+1)` style series).

**Rules that enforce the invariant (no exceptions in lib code):**

1. **No `f32`/`f64` literals.** Numeric constants come from `T::zero()`, `T::one()`, `T::epsilon()`,
   `T::pi()`, or `<T as FromPrimitive>::from_f64(..)` / `from_usize(..)` — exactly as `ext_stats.rs`
   already does (`T::from_f64(0.5)`, `variance_floor::<T>()`).
2. **Tolerances are `T`.** `Truncation`'s `rel_tol`/`abs_tol`, every sweep residual threshold, and every
   convergence gate are values of `T`, never hard-coded floats. Default constructors derive them from
   `T::epsilon()` (e.g. `abs_tol = T::epsilon().sqrt()`), so they scale with the chosen precision.
3. **Seeds/randomness produce `T`.** `random_seeded` draws via the crate's dev RNG and maps into `T`
   through `FromPrimitive`; no `f64` intermediate that would silently fix precision.
4. **A private sealed marker trait `TtScalar`** (blanket impl over the bound above) documents the bound
   in one place and keeps static dispatch — **not** a public abstraction, adds no `dyn`.

This is the same discipline `CausalTensor`'s `svd_impl`/`cholesky` already follow; the tensor-network
layer must not regress it.

### 3.6 HKT extension and the operator `Arrow` — done honestly

The crate gives `CausalTensor` a full HKT stack (`CausalTensorWitness`: `Functor`/`Foldable`/`Pure`/
`Monad`/`CoMonad`/`Applicative`). Mirroring all of it onto a compressed format is a category error.
This section fixes exactly which instances are lawful and where composition lives. **Decision locked.**

**3.6.1 Lawful — the storage/scalar functor.** `CausalTensorTrainWitness` (loose, `NoConstraint`, in
`extensions/ext_hkt_tensor_train.rs`) implements:

- **`Functor::fmap<A,B>(CausalTensorTrain<A>, f: A→B) -> CausalTensorTrain<B>`** — applies `f` to every
  stored **core entry**, leaving bond/physical structure identical. Lawful (identity + composition
  preserved), structure-preserving. Uses: precision conversion `f64↔f32`, lifting `real→dual` for
  autodiff seeding, scalar-wrapper newtypes. **NOT** a map over logical dense entries `A[i₀,…]`.
- **`Foldable::fold`** — folds over all **core** entries (parameter count, Frobenius accumulation,
  finiteness). Folds the *factors*, not logical entries; documented as such.
- **`Pure::pure(v)`** — the rank-1 boundary TT representing the scalar `v`.

A strict `TensorConstraint`-style witness compiles on **stable** here precisely because we implement
only these three — it never reaches the `Monad`/`CoMonad` GAT blocker that gates
`StrictCausalTensorWitness`.

**3.6.2 Deliberately absent (bias check).**

- **`Monad::bind`:** List/Vec flatten has no counterpart in a factored chain; realizing it requires
  decompression and destroys the format. Aligns with the repo's Causal-Arrow positioning: tensor-network
  structure is the **non-Kleisli, static** fragment, *not* the causal-monad (Kleisli) side. A state monad
  here is a category error.
- **`CoMonad::extend`:** the only structure-preserving neighborhood op on a TT is a local linear operator
  = an **MPO**. A comonadic `extend` would either decompress or be redundant with
  `TensorTrainOperator::apply`. The MPO *is* the comonadic stencil in compressed form.
- **`Applicative::apply`:** needs a TT of closures — not a sensible compressed payload, no consumer.

**3.6.3 Composition lives in the operator `Arrow`.** The compositional ("monadic") structure for tensor
networks is categorical: the value-level **`Arrow`** algebra in `deep_causality_haft` (`Id`, `Compose`
`>>>`, `First`/`Split`/`Fanout`) — the realization of the Causal Arrow generalization.

- `CausalTensorTrainOperator` implements **`EndoArrow<CausalTensorTrain<T>>`**
  (`Arrow<In = Out = CausalTensorTrain<T>>`): `compose` = `>>>` (`Compose`), `identity()` = `Id`,
  `apply` = the arrow's action on a state. Laws: `(A∘B)·x = A·(B·x)`, `I·x = x`.
- MPO chains therefore drop into the existing `arrow(...)` fluent builder / `>>>` wiring used elsewhere.

**Why not `Promonad`/`ParametricMonad` (declined).** Those earn their keep only when the index space is
lifted to the **type** level (const generics) so the compiler tracks `S → S'` transitions. Here bond and
physical dims are **runtime** `Vec<usize>`; a dim-changing MPO is still
`CausalTensorTrain<T> → CausalTensorTrain<T>` at the Rust type level, so the parametric machinery is pure
ceremony with no compile-time guarantee gained. **Trigger to revisit:** if a future design lifts dims to
const generics, `ParametricMonad` becomes the correct interface. Recorded, not built.

**Lax-category caveat (tested).** Bond growth under `compose`/`apply` is bounded by **rounding**, so the
category/action laws hold **exactly only without truncation**, and **up to the `Truncation` tolerance**
otherwise. Tests assert the laws to `tol`. The category is *lax* under truncation; do not claim exact
laws on a rounded chain.

---

## 3.7 Scalar generality — real, complex, and dual (forward-mode AD)

> **As built (Stage 4).** The layered-bound *intent* below was realized through a single bridge trait
> rather than a per-layer split. `deep_causality_num` gained two sibling traits —
> **`ConjugateScalar`** (ring/field arithmetic + `conjugate` + real `modulus_squared`/`real_part`/
> `from_real`, with an associated `Real: Scalar`) and **`NormedScalar`** (the clean `Field + Normed`
> composition). The whole tensor-train layer (kernels, states, MPO operators, TT-cross, and the
> `linear`/`fit`/`tdvp`/`eigen` solvers) is bound on **`ConjugateScalar`**, which is implemented for
> `f32`/`f64`/`Float106`, `Dual<T>`, and `Complex<T>` over disjoint type constructors (a blanket
> `impl<T: Scalar>` would collide with `Complex` under coherence — Rust cannot prove `Complex: !Scalar`).
> Truncation thresholds and singular values live in `T::Real`. The complex path is the genuine Hermitian
> stack (conjugated inner, real singular values, complex Givens/Householder, a complex Hermitian DMRG
> eigensolver). Only `apply_nonlinear` stays `Scalar` (it needs `T::nan`). The original `Normed` +
> `ComplexField` analysis that follows is the design *rationale*; `ConjugateScalar` is the concrete bound
> that delivers it without forcing callers to thread a second type parameter.

**Correction of an earlier mistake.** Prior revisions of this spec listed *complex scalars* and *autodiff*
as out of scope, on the assumption that the layer is `RealField`-only. Reading `deep_causality_num`
reverses that: the crate's scalar tower was **built to support exactly these two extensions**, and a flat
`T: RealField` bound is the only thing that excludes them. The corrected design uses a **layered bound**.

### 3.7.1 What `deep_causality_num` actually provides

- **`Real`** = analytic scalar (`sqrt`/`exp`/`ln`/`sin`/…, ordering, constants) **without field
  invertibility**. Supertraits include `CommutativeRing + PartialOrd + Neg + Copy`. Its own doc says:
  *"Bound generic numeric code on `Real` rather than `RealField` when it needs only the analytic
  operations; such code then also accepts non-field analytic types like dual numbers."*
- **`RealField` = `Real + Field`** — adds the *total* multiplicative inverse / division.
- **`Dual<T>: Real`, but explicitly NOT `Field`/`RealField`** (ε is a zero divisor). It **does** implement
  `Div`, and `(a+bε)/(c+dε)` is well-defined whenever `c ≠ 0`. Every elementary function carries its
  derivative by the chain rule; `Dual<Dual<T>>` gives second derivatives.
- **`Complex<T>: Field` and `ComplexField<R>`** — a field, **not** ordered (so not `RealField`), carrying
  `conjugate()`, `norm() -> R`, `norm_sqr() -> R`, `from_re_im`, with an **associated ordered real field
  `R`**.

So neither complex nor dual is a `RealField`: complex is *beside* it (a field, but unordered), dual is
*below* it (analytic, but not a field). One bound cannot name both; the layer must be split by **what each
operation actually needs**.

### 3.7.2 The layered bound

| Operation layer | Needs | Narrowest bound | Admits |
|---|---|---|---|
| **TT algebra** — `add`, `scale`, `hadamard`, contraction, `marginalize`, `eval`, MPO `apply`/`compose` | ring/field arithmetic only | `Field` (or even `Ring` + `Div` where used) | real, **complex**, dual |
| **Norm / orthogonality** — `norm`, `inner`, `round`, SVD, QR, canonicalization, truncation | a norm into an **ordered real** field (+ conjugation for the Hermitian transpose) | **`T: Normed`** (`Normed::Real = R`) — the crate's own trait; real case `R = T`, complex case `R = T`'s real | real, **complex** |
| **Truncation/pivot decisions** — which σ to keep, pivot choice | compare magnitudes (real) | `Normed::Real: RealField` (ordered) on the norm output | all |
| **AD path** — same kernels over a dual scalar | analytic ops + division-as-operation, **no** total inverse | **`T: Scalar`** (`= Real + Div + FromPrimitive`, NOT `Field`) | real, **dual** |

The discriminating layer is **norm/SVD**, and `deep_causality_num` already supplies its exact abstraction:
**`Normed`** — `type Real: RealField`, `modulus_squared() -> Self::Real`, `scale_by_real()`. Its own doc
notes `Real = f64` for *both* `f64` and `Complex<f64>`, and that `Complex` is unordered hence not a
`RealField`. So the norm/SVD kernels bound on `T: Normed` and use `T::Real` for singular values and
tolerances; the magnitude comparisons run on `T::Real` (ordered), never on `T` (complex has no order). The
Hermitian transpose additionally needs `conjugate()` (from `ComplexField`; identity in the real case) —
the only piece `Normed` alone does not carry. No invented trait is required: **`Normed` + `ComplexField`
(complex) / `Scalar` (real+dual)** are the crate-native bounds.

### 3.7.3 Complex — feasible, real kernel work

The TT **algebra** is `Field`-generic and works over `Complex<T>` unchanged. Only the **norm/SVD/QR/inner**
kernels need the complex-aware form: conjugate (Hermitian) transpose `Aᴴ` instead of `Aᵀ`, singular
values that are **real** (live in `R`), and inner products `⟨a|b⟩ = Σ aᵢ* bᵢ`. This is genuine work in the
Stage-0 SVD/QR (a complex Golub–Kahan / Householder), **not** a free bound relaxation. Payoff: quantum-
faithful MPS (the quantum-EPP direction), and any spectral/Fourier-domain field.

### 3.7.4 Dual / forward-mode AD — nearly free, the headline finding

Because `Dual<T>: Real` and supports `Div`, running the **existing real kernels over `Dual<f64>`** yields
**forward-mode automatic differentiation through the entire network** — TT-SVD, rounding, contraction,
MPO apply, even DMRG sweeps — with derivatives carried by the chain rule through every elementary op. The
*only* obstacle is the bound: `RealField` rejects `Dual`; **`Real + Div<Output=Self>` accepts it.** So the
real-scalar code path must be bounded on the narrower analytic bound, never gratuitously on `Field`.

Honest caveats (all standard for forward-mode AD, all must be documented):

- **It is forward-mode:** one seed/derivative direction per pass (cost ≈ one extra value per scalar). Many
  inputs × few outputs is inefficient — that is what reverse-mode is for, and reverse-mode (a tape) is
  **out of scope** here.
- **Truncation/pivot decisions are piecewise-constant** in the inputs: they branch on the *real part*
  (Dual's `PartialOrd` compares real parts), so the chosen rank/pivots are locally constant and the
  derivative flows through the **retained** subspace. At exact singular-value ties / rank changes the map
  is non-smooth — a measure-zero set, the usual AD caveat; document it, do not pretend it is everywhere
  differentiable.
- **Division safety:** kernels must only ever divide by a **checked-nonzero pivot** (real part ≠ 0), which
  they already must for numerical stability. Never assume `Field` totality. This single discipline is what
  keeps the dual path valid.

### 3.7.5 Consequence for the bounds (design rule, from Stage 0)

To keep both extensions reachable **without a rewrite**:

1. **Do not bound on `Field`/`RealField` where `Scalar` (`Real + Div + FromPrimitive`) suffices.** The
   TT-algebra and AD paths take the analytic-plus-division bound; only the norm/SVD layer takes the
   richer `Normed` (+ `ComplexField` for conjugation) bound.
2. **Route every magnitude comparison through `Normed::modulus_squared(): Normed::Real`**, never through
   an assumed ordering on `T` (complex has none).
3. **Every division is by a checked-nonzero pivot.**

Following these three rules from Stage 0 makes Stage 4 a matter of **adding instantiations + tests**
(`Complex<f64>`, `Dual<f64>`, `Dual<Complex<f64>>` for complex-AD) rather than touching kernel bodies.
This is the "design for it now, instantiate when scheduled" path; it is the no-retrofit choice and the one
recommended in §13.

---

## 3.8 Algebra-trait impls on the TT types

`CausalTensor` participates in the `deep_causality_num` algebra tower by implementing a few **marker
traits** and letting the crate's **blanket impls** derive the rest (`algebra/group.rs`, `ring.rs`,
`module.rs`: it declares `Associative`/`Distributive`/`AbelianGroup`, and `AddGroup`/`Ring`/`Module`
follow for free). A tensor train of a **fixed shape** is mathematically a vector space over the scalar
field, so the same structures are *genuinely* applicable to `CausalTensorTrain` — with two honest caveats
that must be documented at the impl site.

**Directly applicable (worth implementing, mirroring `CausalTensor`):**

| Trait | On the TT type | Realization | Caveat |
|---|---|---|---|
| **`Module<T>`** | `CausalTensorTrain<T>` | scalar–vector multiplication = `scale` (multiply one core) | **none** — exact and rank-preserving; the cleanest fit |
| **`AbelianGroup` / `AddGroup`** | `CausalTensorTrain<T>` | `add` (+ negate via `scale(-one)`) | `add` **grows bonds** (exact, then `round`); laws hold exactly un-rounded, **lax under truncation** |
| **`Ring`** | `CausalTensorTrain<T>` | ring multiply = **`hadamard`** (elementwise), matching `CausalTensor`'s choice | `hadamard` **squares bonds** (exact, then `round`); same lax-under-truncation note |
| **`Normed`** (optional) | `CausalTensorTrain<T>` | `modulus_squared = inner(self,self)`, `scale_by_real = scale`; `type Real = <T as Normed>::Real` | makes a TT a normed space for generic convergence checks; include only if a consumer wants it |

Implementation pattern (identical to `CausalTensor`): declare the **marker** traits
(`Associative`/`Distributive`/`AbelianGroup`) where `T` satisfies them; `AddGroup`, `Ring`, and `Module`
then arrive through the num blanket impls (`Module: AbelianGroup + Mul<R>`; `Ring: AbelianGroup +
MulMonoid + Distributive`). The `Mul<T>`/`Mul<&Self>` operator impls route to `scale`/`hadamard`.

**The two shape caveats (must be handled, not hidden):**

1. **`Zero`/`One` are shape-dependent.** A TT additive zero needs physical dims (which shape?), and the
   `Ring` multiplicative one is the shape-dependent all-ones (rank-1) train. `CausalTensor` resolves this
   with a degenerate empty/`Default` zero plus shape-taking `zeros(shape)`/`ones(shape)` constructors; the
   TT mirrors that — the trait-level `Zero::zero()` is the empty/absorbing identity, and `add`/`hadamard`
   require matching `phys_dims` (else `BondDimensionMismatch`/`ShapeMismatch`). State this explicitly.
2. **Bond growth ⇒ lax laws.** Group/ring axioms hold exactly for the *represented* tensors and the
   *un-rounded* result; under a `Truncation` they hold to tolerance — the same lax caveat as the
   `EndoArrow` operator category (§3.6.3). Tests assert the algebra laws to `tol`.

**Not applicable / declined:** `Field`/`RealField`/`InvMonoid` on the TT type — a tensor train has **no
multiplicative inverse** (neither under Hadamard nor under contraction), so the field/division structures
are mathematically absent. Do not implement them. (These remain *scalar* bounds on `T`, never structures
of the train.) `MulMonoid` is implemented only as the carrier for `Ring` via Hadamard, with the
shape-dependent `One` caveat above.

**Net:** add `Module<T>` (clean), `AbelianGroup`/`AddGroup` and `Ring` (with the documented lax + shape
caveats), and optionally `Normed`, to the Stage-1 surface — implemented the same marker-trait-plus-blanket
way as `CausalTensor`, so the TT slots into the same generic numeric code paths.

---

## 4. Conventions (fixed so all consumers agree)

1. **Row-major index order**, matching `CausalTensor`. Site `k` is physical axis `k`.
2. **Boundary bonds are 1**, never elided — every state core is uniformly rank-3, every operator core
   rank-4.
3. **Canonicalization gauge.** Left-canonical: `Σ Lᵀ L = I` per bond; right-canonical: `Σ R Rᵀ = I`.
   Mixed-canonical at `k` places the orthogonality center on core `k`, where `norm`, `inner`, and the
   sweeps read the singular spectrum. Gauge is **tracked** in `CanonicalForm`, not recomputed.
4. **Rounding is the universal pressure valve.** `add`, `hadamard`, `MPO·MPS`, `MPO·MPO` grow bond
   dimension exactly (sum / product); each returns an exact, un-rounded result, and a paired
   `*_rounded(&Truncation)` wrapper applies `round`. Lossless and lossy steps stay separable and
   independently tested.
5. **QTT ordering.** A physical axis of length `n = 2^L` reshapes to `L` binary sites in **big-endian
   (coarse-to-fine)** order. Multi-axis interleave is a `qtt` helper, not hard-wired (the best interleave
   is field-dependent; the Gourianov scheme is one choice). **[holds under precondition: written]**

---

## 5. Algorithm surface by stage

Stages are dependency ordering of **complete** deliverables. Each row is fully implemented + fully tested
within its stage.

### Stage 0 — Numerical foundation (hard prerequisite)

| Item | Signature (sketch) | Why |
|---|---|---|
| Robust truncated thin-SVD | `svd_truncated(&self, &Truncation<T>) -> Result<(U, S, Vt), _>` | returns only the retained rank with **orthonormal** `U`,`Vt` to working precision; the existing power-iteration deflation does not guarantee this on clustered/degenerate spectra |
| Householder QR | `qr(&self) -> Result<(Q, R), _>` | cheap, stable gauge sweeps for canonicalization (QR, not SVD, is the standard canonicalizer) |
| `Truncation<T>` type | constructors + validation | shared compression control |

**Recommended SVD path:** Golub–Kahan bidiagonalization + implicit-shift QR with reorthogonalization.
This is an **addition** to `ops/tensor_svd*`, not a rewrite of the public `svd`. Both routines are
generic over `T: RealField`; the bidiagonalization/QR shift tolerances derive from `T::epsilon()`
(§3.5 rule 2), never from an `f64` constant, so convergence scales with the working precision. Reference
singular values/vectors are stored as **full-precision `f64` fixtures and asserted at both `f32` and
`f64`** with precision-appropriate tolerances (the crate forbids runtime external deps). **[open — has
its own sub-spec + reference tests; gates everything downstream]**

### Stage 1 — Core TT algebra (state)

| Operation | Signature (sketch) | Semantics / consumer |
|---|---|---|
| TT-SVD | `CausalTensorTrain::from_dense(&CausalTensor<T>, &Truncation<T>)` | left-to-right SVD sweep; canonical constructor |
| from closure | `CausalTensorTrain::from_fn(&[usize], f, &Truncation<T>)` | forms dense then TT-SVD; **guarded** element-count cap |
| specials | `zeros`, `ones`, `random_seeded` | init + tests |
| getters | `cores`, `order`, `phys_dims`, `bond_dims`, `max_bond`, `canonical_form` | introspection |
| canonicalize | `left_canonicalize`, `right_canonicalize`, `canonicalize_at(k)` | QR gauge sweeps |
| round | `round(&self, &Truncation<T>) -> Self` | SVD recompression; idempotent |
| norm / inner | `norm(&self) -> T`, `inner(&self,&Self) -> Result<T,_>` | via mixed-canonical center |
| add / scale | `add(&self,&Self)`, `scale(&self, T)` | exact bond growth (+ `*_rounded`) |
| add_scalar | `add_scalar(&self, T)` | exact affine via rank-1 ones-train (+1 bond, round) — elementwise option (b) |
| hadamard | `hadamard(&self,&Self)` | elementwise product (SURD MI terms) |
| to_dense | `to_dense(&self) -> Result<CausalTensor<T>,_>` | **guarded** element cap; tests + small cases |
| marginalize | `marginalize(&self, sites: &[usize]) -> Result<Self,_>` | sum out a subset of axes — **SURD primitive** |
| eval | `eval(&self, index: &[usize]) -> Result<T,_>` | single entry without going dense |
| HKT | `CausalTensorTrainWitness` Functor/Foldable/Pure | §3.6 |

Stage 1 alone makes SURD's high-order joints tractable.

### Stage 2 — MPO, cross-approximation, ALS solvers

| Operation | Signature (sketch) | Semantics / consumer |
|---|---|---|
| MPO from dense | `CausalTensorTrainOperator::from_dense(&CausalTensor<T>, in_dims, out_dims, &Truncation)` | operator TT-SVD |
| MPO·MPS | `op.apply(&CausalTensorTrain<T>, &Truncation)` | CFD step kernel: operator on field |
| MPO·MPO | `op.compose(&Self, &Truncation)` | operator products (∂ₓₓ = ∂ₓ∘∂ₓ) |
| EndoArrow | `impl EndoArrow<CausalTensorTrain<T>>` | operator category (§3.6.3) |
| TT-cross | `CausalTensorTrain::cross(&[usize], oracle, &CrossConfig<T>)` | build TT from `Fn(&[usize])->T` **without going dense** — curse-of-dimensionality escape (SURD joints, UQ surfaces) |
| apply_nonlinear | `train.apply_nonlinear(f, &CrossConfig<T>) -> Result<(Self, residual),_>` | explicit elementwise nonlinear map via cross re-approximation — **SURD `log p`**; option (b) |
| ALS linear solve | `solve::linear(&CausalTensorTrainOperator<T>, &CausalTensorTrain<T>, &SolveConfig)` | `A x = b` in TT form — **implicit CFD step** `(I − Δt·L)xⁿ⁺¹ = xⁿ` |
| ALS fit/complete | `solve::fit(samples, &SolveConfig)` | least-squares TT from samples — **UQ regression/completion** |
| integrate | `train.integrate(&[CausalTensor<T>]) -> Result<T,_>` | contract each site vs a weight vector — UQ expectations, SURD normalization |
| QTT helpers | `qtt::quantize`, `qtt::dequantize` | `n=2^L` axis ↔ `L` binary sites (CFD/UQ) |

### Stage 3 — DMRG3S eigensolver and advanced dynamics

| Operation | Signature (sketch) | Semantics / consumer |
|---|---|---|
| DMRG3S ground state | `solve::eigen(&CausalTensorTrainOperator<T>, &SolveConfig) -> Result<(T, CausalTensorTrain<T>),_>` | lowest eigenpair via **single-site DMRG with subspace expansion** (DMRG3S); the local eigenproblem is solved by Lanczos/Rayleigh-quotient iteration, and the bond is grown by **residual-subspace enrichment reusing the AMEn engine** before each truncation |
| TDVP step (optional) | `solve::tdvp_step(&op, &mut train, dt, &Truncation)` | **two-site** (or CBE-) time-dependent variational propagation — compressed dynamics for CFD/quantum |

**Why DMRG3S over two-site DMRG.** Classic two-site DMRG grows rank by contracting *two* neighbouring
cores into a `[r·n × n·r]` block, solving the local eigenproblem there, then SVD-splitting — `O(n²)`
larger local systems and an `O(r³n³)` split per site. **DMRG3S** (strictly single-site + subspace
expansion, Hubig–McCulloch–Schollwöck–Wolf 2015) keeps the local problem single-site (`[r·n × r·n]`)
yet still adapts the rank by **augmenting the basis with the projected residual** `P·(H x)` before
truncating — exactly the AMEn enrichment already implemented for `solve::linear`. So Stage 3 reuses the
Stage-2c enrichment machinery rather than introducing a separate two-site code path. **CBE** (controlled
bond expansion, Gleis–Li–von Delft) is the optional frontier refinement of the same idea.

**Why two-site / CBE-TDVP, not one-site TDVP.** One-site TDVP is rank-*static* — it cannot grow the bond
dimension, so it silently under-resolves any dynamics that build entanglement. The two-site variant (or
CBE-TDVP) adapts the rank each step, which is the only honest default for a general dynamics consumer.

**Honest stance:** SURD, CFD, and UQ do not *require* an eigensolver; DMRG3S and TDVP are included for
completeness and the quantum-EPP direction, and are cheap once the shared `solve` sweep driver + AMEn
enrichment (Stage 2c) exist. The reusable artifact is the **alternating single-site sweep engine with
residual enrichment**, shared by `linear`, `fit`, `eigen`, and `tdvp`. TDVP is marked optional pending a
concrete dynamics consumer. **[Stage 3 is genuine, not deferred-indefinitely; built after Stage 2 lands.]**

**Stage 3 references** (also cited at the implementation site + in the crate README):
- C. Hubig, I. P. McCulloch, U. Schollwöck, F. A. Wolf, "Strictly single-site DMRG algorithm with
  subspace expansion," *Phys. Rev. B* 91, 155115 (2015), arXiv:1501.05504 — **DMRG3S**.
- J. Gleis, J.-W. Li, J. von Delft, "Controlled bond expansion for DMRG ground state search at
  single-site costs," *Phys. Rev. Lett.* 130, 246402 (2023), arXiv:2207.14712; CBE-TDVP, *Phys. Rev.
  Lett.* 133, 026401 (2024) — **CBE**.
- S. Paeckel et al., "Time-evolution methods for matrix-product states," *Ann. Phys.* 411, 167998
  (2019), arXiv:1901.05824 — **TDVP** review (two-site vs one-site rank growth).

### Stage 4 — Scalar generality (complex + dual)

| Item | Content | Consumer |
|---|---|---|
| Complex instantiation | complex-aware SVD/QR/inner (Hermitian transpose, real singular values, `⟨a\|b⟩ = Σ aᵢ* bᵢ`); instantiate the whole stack at `Complex<f64>` | quantum-faithful MPS (quantum-EPP), spectral/Fourier fields |
| Dual instantiation (forward-mode AD) | relax kernel bounds to `Real + Div` per §3.7.5; instantiate at `Dual<f64>`; document the piecewise-constant-truncation caveat | sensitivity/gradients through TT-SVD, MPO apply, DMRG |
| Mixed | `Dual<Complex<f64>>` for complex-AD; full dual-precision test pass | both |

Stage 4 is **instantiation + tests**, not a kernel rewrite — *provided* the §3.7.5 design rules are
honored from Stage 0 (no gratuitous `Field` bound; magnitudes via `norm(): R`; divide only by checked
nonzero pivots). The only genuine new kernel work is the **complex** SVD/QR (§3.7.3); the **dual** path is
the existing real kernels under a relaxed bound.

---

## 6. Downstream integration contracts

### 6.1 SURD (`deep_causality_algorithms`)
- Builds a joint distribution as a `CausalTensorTrain<T>` via **`cross`** (the dense joint is
  unrepresentable past ~15 binary variables).
- Information-decomposition terms via **`marginalize`**, **`hadamard`**, and **`integrate`**.
- Unary elementwise transforms (e.g. `log p`) use the resolved **option (b)** trio: `scale` (exact
  linear), `add_scalar` (exact affine), and **`apply_nonlinear(f, &CrossConfig)`** for the genuine
  nonlinear case — explicit cross re-approximation, returning a residual estimate so the cost and
  approximation are visible. No silent-wrong-result `map_elementwise` (§13).

### 6.2 CFD (`deep_causality_cfd`)
- Encodes the flowfield as a **QTT** via `qtt` (step [4] of the Plasma-Blackout-Corridor note).
- Differential operators as **MPOs**; advance via **`apply` + `round`** (explicit) or **`solve::linear`**
  (implicit).
- The field↔QTT encoding (index interleave, Gourianov scheme) is **CFD-specific and stays in the CFD
  crate**; this module supplies generic `qtt` reshape + the MPO/solve machinery only.

### 6.3 UQ (`deep_causality_uncertain` and downstream)
- Response surface from samples via **`cross`** or **`solve::fit`**; moments via **`integrate`**;
  sensitivity via **`marginalize`**.

---

## 7. Performance contract

"Performant" here means **algorithmic optimality and cache-friendly contraction**, *not* a BLAS
dependency — the crate forbids external runtime crates, so all linear algebra reuses the in-crate
`matmul`/`ein_sum`/Stage-0 routines. Held to these targets (d = order, n = max physical dim, r = max
state bond, R = max operator bond):

| Operation | Target complexity |
|---|---|
| TT-SVD `from_dense` | `O(d · n · r² · max(n·r))` over the sweep |
| `round` | `O(d · n · r³)` |
| `norm`, `inner` | `O(d · n · r³)`, via the canonical center (no full re-contraction) |
| MPO·MPS `apply` | `O(d · n² · r² · R²)` |
| `eval` (single entry) | `O(d · n · r²)` — never materializes dense |
| `marginalize` | `O(d · n · r³)` |

Requirements: gauge sweeps run **left-to-right / right-to-left in place** on cores (via `get_mut`),
not by rebuilding the train; `inner`/`norm` exploit the tracked `CanonicalForm` to avoid recomputation;
`to_dense`/`from_fn` are **guarded** by an element-count cap (`RankExceeded`) so a high-order train can
never silently allocate `nᵈ`. `derive(PartialEq)` on trains is for tests, not a hot path. An opt-in
`parallel` feature (per-bond-independent work in `cross`/`fit`) is a Stage-3+ addition, gated and
documented, never on the correctness path.

### 7.1 Benchmark suite — `benches/benchmarks/causal_tensor_network_type/`

Criterion benchmarks mirror the existing `causal_tensor_type/` layout: one bench file per stage,
declared in `benches/benchmarks/causal_tensor_network_type/mod.rs`, with each `criterion_group` added to
the `criterion_main!` set (extend `bench_causal_tensor.rs`, or add a second `[[bench]]` target in
`Cargo.toml`). **Each file is written when its stage's types land** — benchmarks reference real types, so
no placeholder code that cannot compile is committed ahead of the implementation. Benches measure at
`f64` (representative), with an `f32` variant added only for the cache-bound kernels where it matters;
benches are not lib code, so a concrete float here does not violate the §3.5 precision invariant.

Each benchmark is the empirical witness of the matching §7 complexity row — comments record the expected
scaling and the suite includes paired sizes that expose it (e.g. doubling `r` must cost ~8× for an `r³`
op). Expensive groups use `sample_size(10)` as the existing bench does.

| Bench file (stage) | `criterion_group` targets | Parameter sweep | Validates (§7 row) |
|---|---|---|---|
| `bench_tensor_train_core.rs` (Stage 1) | `from_dense`/TT-SVD, `round`, `canonicalize_at`, `norm`, `inner`, `add`(+rounded), `hadamard`, `marginalize`, `eval` | `(order d, phys n, bond r)` over small/medium/large triples | TT-SVD, round, norm/inner, marginalize, eval |
| `bench_tensor_train_operator.rs` (Stage 2) | `mpo_from_dense`, `mpo_apply` (MPO·MPS), `mpo_compose` (MPO·MPO), `integrate` | `(d, n, r, op-bond R)` | MPO·MPS row; operator scaling in `R` |
| `bench_tensor_train_cross.rs` (Stage 2) | `cross_build` from a known low-rank oracle | `(d, target rank)` × oracle cost | cross convergence cost vs rank |
| `bench_tensor_train_solve.rs` (Stage 2→3) | `amen_linear`, `als_fit`, `dmrg3s_eigen`, (opt) `tdvp_step` | `(d, n, r, sweeps)` | sweep-engine cost per stage |
| `bench_svd_qr.rs` (Stage 0) | `svd_truncated`, `qr` | `(rows m, cols n, kept rank k)` | the gating Stage-0 kernels |

Size constants live at the top of each file (the existing bench's pattern), chosen so the full suite runs
in seconds, not minutes — large enough to expose scaling, small enough to stay in the CFD-minutes spirit.

---

## 8. Error handling

Extend the existing `CausalTensorError` enum (the crate has exactly one tensor error type); no second
error type, no `dyn Error` beyond the existing `impl Error`. New variants, each with a `Display` arm:

- `BondDimensionMismatch` — adjacent cores' bonds disagree (constructor / hand-built validation).
- `NotCanonical` — an op needing a gauge received a train not in it (internal guard; tested).
- `RankExceeded` — a guarded `to_dense`/`from_fn` would exceed the element-count cap.
- `SweepDidNotConverge` — ALS/DMRG/TDVP hit `max_sweeps` without meeting the residual target.
- `CrossSampleFailure` — the oracle returned a non-finite value during cross-approximation.

`InvalidParameter(String)` (already present) absorbs bad `Truncation`/`*Config` values.

---

## 9. Open seams (named, not hidden)

- **Nonlinear elementwise maps are not exact in TT form.** `log`, `exp`, reciprocal each need `cross`
  re-approximation and incur error (with possibly large rank growth). Handled honestly by
  `apply_nonlinear` (option b); the API never pretends a general `map_elementwise` is exact. **[resolved]**
- **Complex + dual scalars are in scope (§3.7), not excluded.** Complex needs a complex-aware SVD/QR
  (genuine Stage-4 work); dual (forward-mode AD) is the existing real kernels under a `Real + Div` bound.
  Reverse-mode AD (a tape) remains out of scope. **[holds under precondition: §3.7.5 design rules from
  Stage 0]**
- **TT-cross convergence is heuristic.** ACA/maxvol pivoting can stall on adversarial oracles; expose
  `max_sweeps`, `rank_cap`, and a returned residual estimate; do not claim guaranteed accuracy.
  **[holds under precondition: bounded + reported]**
- **No automatic site-ordering optimization.** TT rank is ordering-sensitive; choosing a good variable
  order (SURD) or axis interleave (CFD) is the caller's responsibility. A `reorder` helper is plausible
  later but out of scope now. **[speculative]**

---

## 10. Testing strategy (100 % coverage of added code, mirror tree)

Per `AGENTS.md`: tests mirror src under `tests/traits/...` and `tests/types/causal_tensor_network/...` with the
`_tests` suffix, registered in each `mod.rs` and `tests/BUILD.bazel`. Shared fixtures in
`src/utils_tests/`.

**Precision coverage.** Every test class below runs at **`f32`, `f64`, and `Float106`** (a generic
`fn check<T: RealField>()` helper instantiated three times), proving precision genericity is real across a
single-, double-, and double-double-precision real and that no path pins a concrete float. Assertion
tolerances are themselves derived from `T::epsilon()`, so the same test body holds at every precision.
Stage 4 adds `Complex<f64>` and `Dual<f64>` instantiations of the applicable classes (class 10 below).

Test classes, applied per stage:
1. **Round-trip identity:** `from_dense → to_dense` reproduces small dense tensors to `ε`.
2. **Compression accuracy:** rank-`r` tensors recover exactly at bond `r`; truncation error tracks the
   discarded singular tail.
3. **Algebraic laws:** `add` commutes; `scale` linear; `hadamard`/`inner`/`norm`/`marginalize` match the
   dense computation (`sum_axes`, elementwise).
4. **MPO:** `apply` matches dense matrix–vector; `compose` matches dense matmul.
5. **Cross:** recovers a known low-rank oracle without going dense; reports residual.
6. **Solve:** `linear` recovers `x` for known `A,b`; `fit` recovers a known TT from samples; `eigen`
   recovers a known smallest eigenpair.
7. **Error paths:** every new `CausalTensorError` variant is provoked and asserted.
8. **Stage-0 SVD/QR quality:** orthogonality residuals `‖UᵀU − I‖ ≤ k·ε`, singular-value reference
   checks against checked-in fixtures.
9. **HKT / arrow laws (§3.6):** `fmap` identity + composition on cores; `Pure`/`Foldable` semantics;
   `EndoArrow` associativity/identity and `apply` action laws asserted **to the truncation tolerance**.
10. **Scalar generality (§3.7, Stage 4):** complex round-trip/unitarity (`‖AᴴA − I‖`), real singular
    values; dual forward-mode AD checked against finite differences (`∂ output / ∂ input` to `√ε`), and
    against an analytic gradient where one exists; `apply_nonlinear` derivative consistency.

---

## 11. Public API / `lib.rs` exports

From the crate root (the crate forbids preludes):

```rust
pub use crate::traits::tensor_train::TensorTrain;
pub use crate::traits::tensor_train_operator::TensorTrainOperator;
pub use crate::types::causal_tensor_network::causal_tensor_train::CausalTensorTrain;
pub use crate::types::causal_tensor_network::causal_tensor_train_operator::CausalTensorTrainOperator;
pub use crate::types::causal_tensor_network::truncation::Truncation;
pub use crate::types::causal_tensor_network::canonical_form::CanonicalForm;
pub use crate::types::causal_tensor_network::cross_config::CrossConfig;
pub use crate::types::causal_tensor_network::solve_config::SolveConfig;
pub use crate::extensions::ext_hkt_tensor_train::CausalTensorTrainWitness;
// solve::{linear, fit, eigen, tdvp_step} re-exported as free functions from the crate root.
```

Internal modules (`canonical`, `round`, `ops`, `apply`, `arrow`, `solve` internals) stay private; their
behaviour is surfaced through the traits and the re-exported free functions.

---

## 12. Staged delivery plan (complete vertical slices)

Each stage ships complete and fully tested before the next starts. Sizes are rough engineering effort,
not a license to cut scope within a stage.

| Stage | Content | Rough size (impl + tests) |
|---|---|---|
| **0 — Prep** | Robust truncated SVD, Householder QR, `Truncation`, reference fixtures | ~1–1.5 weeks; **gates everything** |
| **1 — Basics** | `CausalTensorTrain` + trait, TT-SVD, canonical, round, full algebra, marginalize, eval, HKT witness | ~2–3 weeks; the substantive core (unblocks SURD) |
| **2 — MPO/solvers** | `CausalTensorTrainOperator` + trait, apply/compose, `EndoArrow`, TT-cross, ALS linear+fit, integrate, QTT | ~3–4 weeks (cross + ALS are the hard parts; unblocks CFD + UQ) |
| **3 — DMRG/advanced** | DMRG eigensolve, optional TDVP, opt-in `parallel` perf | ~2 weeks; built on the Stage-2 sweep engine |
| **4 — Scalar generality** | complex-aware SVD/QR + `Complex<f64>` instantiation; `Dual<f64>` forward-mode AD via bound relaxation; mixed `Dual<Complex>` | ~2–3 weeks; complex SVD is the only real new kernel (§3.7) |

---

## 13. Decisions

### 13.1 Resolved — elementwise maps (option b)

**The underlying fact.** An elementwise map `B[i] = f(A[i])` on a tensor train splits into three regimes:
**linear** `f(x)=a·x` (multiply one core — exact, rank-preserving), **affine** `f(x)=a·x+b` (add a
rank-1 ones-train — exact, +1 bond), and **general nonlinear** `f` (`log`/`exp`/`1/x`/indicator), which
has **no local core operation** — `f` of a rank-`r` sum-of-separables is not low-rank, so it can only be
**re-approximated** via TT-cross over `i ↦ f(self.eval(i))`, with potentially large rank growth and
heuristic accuracy. SURD's `p·log p` terms land in the nonlinear regime, so this must be answered.

Three distinct "maps" must stay legible and not be conflated:
1. **storage functor `fmap`** (§3.6) — converts the *scalar type* of cores (`f64→f32`, `real→dual`);
   structural, not a value transform;
2. **exact logical linear/affine** value transforms;
3. **approximate logical nonlinear** value transform.

**Option (a): `map_elementwise(f)` documented "linear only."** *Rejected as the recommendation.* A
closure `f: T→T` cannot be inspected for linearity, so passing a nonlinear `f` compiles, runs, and
returns a TT that decoded `f` over one core's *stored* entries — **silently wrong**, no panic, no error.
The name promises a guarantee the format cannot keep, and it overlaps `scale`.

**Option (b) — RECOMMENDED:** no `map_elementwise`; instead three honestly-named tools so the signature
tells the truth at every call site:
- `scale(a)` — exact linear (already in the trait);
- `add_scalar(b)` — exact affine via the rank-1 ones-train (+1 bond, then round);
- `apply_nonlinear(f, &CrossConfig) -> Result<(Self, residual), _>` — explicit cross re-approximation;
  the `CrossConfig` argument and returned residual make the cost/approximation visible and impossible to
  mistake for exact.

**Resolved: option (b).** `scale` / `add_scalar` / `apply_nonlinear(f, &CrossConfig)`. No
`map_elementwise`. A silently-wrong-result method is the corner-cut the project's standards forbid.

### 13.2 Resolved — Option A (bound-general from Stage 0)

Complex and dual are **in scope**, and the kernels carry the general bound **from Stage 0**: written under
the §3.7.5 rules (bound on `Scalar`/`Normed`, never gratuitously `Field`; magnitudes via
`Normed::modulus_squared(): Normed::Real`; divide only by checked nonzero pivots). The **real path is
tested at `f32`, `f64`, and `Float106`** through Stages 1–3; Stage 4 adds `Complex<f64>` and `Dual<f64>`
as **instantiations + tests**, the complex SVD/QR being the sole genuinely new kernel. No retrofit — the
cost is numerical-hygiene discipline, not extra code, consistent with *zero corner-cutting*.

The whole spec is now **locked** (§0) and ready to implement Stage 0 → 4.
