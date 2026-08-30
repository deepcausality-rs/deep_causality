<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `deep_causality_homology`: extraction plan

**Scope.** Move the homology material that currently sits split across `deep_causality_linear` and
`deep_causality_topology` into one crate, and give topology a way to keep using it. Nothing else.

**Not in scope.** Integral homology, torsion, Smith normal form, persistence. Those are capabilities
the crate could grow later. None of them is needed to move code that already works.

---

## 1. Inventory

### 1.1 In `deep_causality_topology` — homology, not geometry

| Item | Path | Note |
|---|---|---|
| `ChainComplex` | `src/traits/chain_complex.rs:19` | Mixed. See §1.4 |
| `HomologyField` | `src/types/homology_field/mod.rs:36` | Two arms, `Rational` and `Gf2` |
| `HomologyField::rank_of` | `src/types/homology_field/mod.rs:65` | Dispatches to `rank_exact` / `rank_gf2` |
| `widen_to_dense_i64` | `src/types/homology_field/mod.rs` | Private. A conversion, sitting away from the other conversions |
| `Gf2Chain<W>` | `src/types/gf2_chain/mod.rs:38` | Bit vector plus degree |

### 1.2 In `deep_causality_topology` — homology in name, geometry in fact

| Item | Path | Why it cannot move |
|---|---|---|
| `Chain<T>` | `src/types/chain/mod.rs:18` | Holds `Arc<SimplicialComplex<T>>`; `check_compatibility` uses `Arc::ptr_eq` |
| `cup_product` | `src/types/cup_product/mod.rs:95` | Needs `SplittableCell::split` for the Alexander–Whitney diagonal |

A chain complex has no multiplication. The cup product needs a diagonal approximation, and that comes
from cells. It is an operation of cellular complexes of spaces, so it belongs where the cells are.

### 1.3 In `deep_causality_linear` — present because of homology

| Item | Path | Evidence |
|---|---|---|
| `csr_to_packed_gf2_mod2` | `src/extensions/conversions.rs:112` | Its docstring: *"This is the conversion `deep_causality_topology`'s boundary operators need. Their entries are `{-1, 0, 1}`"* |
| `csr_to_packed_gf2_strict` | `src/extensions/conversions.rs:142` | Same family |
| `PackedGf2Vector<W>` | `src/types/packed_gf2_vector/mod.rs` | Written for `Gf2Chain` |

These are the overlap. Linear carries conversions that name topology's boundary operators as their
reason to exist, and topology carries a private conversion of its own. Neither crate owns the seam.

### 1.4 The `ChainComplex` trait divides cleanly

Eleven items, two disjoint groups:

| Homology | Geometry |
|---|---|
| `num_cells(k)` — this is `dim C_k` | `type CellType: Cell` |
| `max_dim()` | `type CellIter<'a>` |
| `boundary_matrix(k)` | `type Metric` |
| `coboundary_matrix(k)` | `cells(k)` |
| `betti_number_over(k, field)` *(provided)* | `uniform_lattice_layout()` |
| `betti_number(k)` *(provided)* | |

**Measured.** Outside `deep_causality_topology`, the trait is named in `deep_causality_cfd` (10 files)
and `deep_causality_physics` (2 files). The only methods they call are `num_cells` (18 call sites)
and `uniform_lattice_layout` (1). **No crate outside topology calls `boundary_matrix`,
`coboundary_matrix`, `betti_number` or `betti_number_over`.**

The two halves have disjoint audiences. That is what makes the split cheap.

### 1.5 Stays in `deep_causality_linear`

`CsrMatrix`, `DenseMatrix`, `PackedGf2`, `rank_exact`, `rank_gf2`, `kernel_basis_gf2`,
`image_basis_gf2`. General linear algebra. Homology is currently their only consumer, which is a fact
about today's workspace rather than about the mathematics.

---

## 2. Extraction difficulty

**Easy — no geometry in the dependency set.**

1. `HomologyField` and `rank_of`. Depends on `CsrMatrix`, `rank_exact`, `rank_gf2`,
   `csr_to_packed_gf2_mod2`, all from linear.
2. `Gf2Chain<W>`. Depends on `PackedGf2`, `PackedGf2Vector`, `Gf2`, `NaturalNumber`.
3. The homology half of `ChainComplex`, as a trait. The implementations stay in topology.

**Easy, and it tidies the seam.**

4. `widen_to_dense_i64` moves to `deep_causality_linear/src/extensions/conversions.rs` beside the
   other five conversions, as `csr_i8_to_dense_i64`. It is a matrix conversion and has no homological
   content.

**Do not move.**

5. `Chain<T>` and `cup_product`, for the reasons in §1.2.

**Consider moving later, not now.**

6. `csr_to_packed_gf2_mod2` and `csr_to_packed_gf2_strict`. They could move up into homology, since
   the `{-1, 0, 1}` reduction is a statement about boundary operators. Leaving them in linear costs
   nothing and moving them is a published-API break for no gain today.

---

## 3. The design

```
deep_causality_num  ─▶  deep_causality_algebra  ─▶  deep_causality_linear
                                                            │
                                                            ▼
                                                deep_causality_homology
                                                            │
                                                            ▼
                                                deep_causality_topology
                                                            │
                                             ┌──────────────┼──────────────┐
                                          physics         cfd          algorithms
```

### 3.1 What `deep_causality_homology` exports

```rust
/// A graded family of chain groups with boundary and coboundary operators.
/// Says nothing about cells, metrics or geometry.
pub trait ChainComplex {
    fn num_cells(&self, k: usize) -> usize;                       // dim C_k
    fn max_dim(&self) -> usize;
    fn boundary_matrix(&self, k: usize) -> Cow<'_, CsrMatrix<i8>>;
    fn coboundary_matrix(&self, k: usize) -> Cow<'_, CsrMatrix<i8>>;

    fn betti_number_over(&self, k: usize, field: HomologyField)
        -> Result<usize, HomologyError> { /* provided */ }
    fn betti_number(&self, k: usize) -> usize { /* provided */ }
}

pub enum HomologyField { Rational, Gf2 }
pub struct Gf2Chain<W> { /* bit vector + degree */ }
```

### 3.2 How topology reuses it

```rust
// deep_causality_topology
pub use deep_causality_homology::{ChainComplex, Gf2Chain, HomologyField};

/// A chain complex that comes from a space: it has cells, and may have a metric.
pub trait CellComplex: ChainComplex {
    type CellType: Cell;
    type CellIter<'a>: Iterator<Item = Self::CellType> where Self: 'a;
    type Metric;

    fn cells(&self, k: usize) -> Self::CellIter<'_>;
    fn uniform_lattice_layout(&self) -> Option<(Vec<usize>, Vec<bool>)> { None }
}
```

`SimplicialComplex`, `CellComplex<C>` and `LatticeComplex` each implement both. The impls stay in
topology, because they are where the geometry is.

### 3.3 Why the re-export matters

`deep_causality_cfd` and `deep_causality_physics` call `num_cells`, which lands on the homology
trait. Re-exporting `ChainComplex` from topology means their existing `use deep_causality_topology::ChainComplex`
keeps compiling. **Zero downstream edits.** Without it, 12 files need an import change.

### 3.4 The one open choice

`boundary_matrix` returns `Cow<'_, CsrMatrix<i8>>` — the coefficient type is fixed. `betti_number_over`
takes the field at runtime. The trait and the method disagree about whether `R` is a parameter.

**Option A — leave it.** The `i8` is an incidence number, which for a cell complex genuinely is in
`{-1, 0, 1}`. `HomologyField` picks the field at the call site. No break, and the extraction lands.

**Option B — parameterise.** `trait ChainComplex<R>` with `boundary_matrix(k) -> Cow<CsrMatrix<R>>`.
This is the breaking change, and its size is the same whether the trait sits in topology or in
homology.

**Recommendation: A now, B as its own decision later.** The extraction is worth doing on its own.
Bundling it with a breaking generalisation turns a move into a migration.

---

## 4. Fix while moving

Four defects found while surveying, all in the blast radius:

1. **The GF2 basis orientation is documented backwards.** `kernel_basis_gf2` allocates
   `zeros(cols, free.len())`; `image_basis_gf2` allocates `zeros(rows, pivots.len())`. **The bases
   are columns.** Four docstrings say rows: `packed_gf2_vector/mod.rs:107-108`,
   `gf2_chain/mod.rs:73`, `packed_gf2_vector_tests.rs:177`, `gf2_chain_tests.rs:100`. `from_row` is a
   contiguous word-slice copy and cannot read a column; `from_column` does not exist. Add it, correct
   the four docstrings.
2. **Degenerate grades return shape `(0,0)`.** `betti_number_over` survives via
   `n_k.saturating_sub(rank_k)`; anything reading a kernel basis would not.
3. **`∂∘∂ = 0` is stated nowhere** — not in the trait, not in the conformance harness.
4. **`lattice_complex_test.rs:183-193` passes by coincidence.** It asserts `cols(∂₂) == rows(∂₁)`,
   comparing `n₂` with `n₀`. Composability of `∂₁ ∘ ∂₂` is `cols(∂₁) == rows(∂₂)`.

`Gf2Chain` also carries no complex identity, so `add`, `intersect` and `inner` guard on degree and
length alone and would accept a chain from a different complex with the same cell count. `Chain<T>`
in the same crate decided the opposite with `Arc::ptr_eq`. Worth settling as the type moves.

---

## 5. Cost

- Two new version bumps: `deep_causality_homology` 0.1.0, `deep_causality_topology` 0.7.4 (additive,
  given the re-export in §3.3).
- `deep_causality_linear` 0.1.2 for `from_column` and `csr_i8_to_dense_i64`.
- Zero dependent `Cargo.toml` edits. All four dependents pin `version = "0.7"`.
- A new crate costs: `BUILD.bazel` and `tests/BUILD.bazel`, SBOM pair, `[lints] workspace = true`,
  an AGENTS.md index and tier entry, and two hand-maintained CI lists —
  `.github/workflows/formalization.yml` (omission gives a false `MISSING Rust witness`) and
  `.github/workflows/rust_deps.yml` (omission is silent).

---

## 6. Papers

In `deep_causality_homology/papers/`:

| File | Why |
|---|---|
| `arxiv_math_0701146.pdf` | Barakat & Robertz, *homalg: a meta-package for homological algebra*. Layers a homology package over "computable rings", which is §3.4's question asked and answered elsewhere |
| `arxiv_2202.01629.pdf` | Baanen, *Use and abuse of instance parameters in the Lean mathematical library*. Generic-over-a-ring in a language with typeclasses; bears directly on Option A versus Option B |
| `arxiv_2108.08831.pdf` | Hang, Giusti, Ziegelmeier, Henselman-Petrusek, *U-match factorization*. The mathematical basis of `oat_rust`, the largest Rust computational-topology library |
| `integralHomology2605.04944v1.pdf` | Pre-existing. Out of scope for this extraction; relevant if the crate later grows integral homology, where `H_i ≅ ℤ^{β_i} ⊕ (ℤ₂)^{T_i}` is the return shape |

A wider literature sweep produced 36 candidates on Smith normal form, discrete Morse reduction,
persistence and formalization. **None of them bears on this task.** They are algorithms for
capabilities the crate does not have and does not need in order to exist. Recorded, not retrieved.

---

## 7. Order of work

1. Fix §4.1, the column/row defect, in `deep_causality_linear`. It is a live error today.
2. Create `deep_causality_homology` with `ChainComplex`, `HomologyField`, `Gf2Chain`.
3. Add `CellComplex: ChainComplex` in topology; re-export the three names.
4. Move `widen_to_dense_i64` into linear's conversions.
5. Settle `Gf2Chain`'s complex identity.
6. Fix §4.2, §4.3, §4.4.
7. Verify: `bazel test //...`, clippy, fmt. No dependent should need an edit.
