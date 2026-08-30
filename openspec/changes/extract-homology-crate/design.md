## Context

`deep_causality_topology` is 27,317 lines. Of those, 419 are the chain-complex layer:
`traits/chain_complex.rs`, `types/homology_field/mod.rs` and `types/gf2_chain/mod.rs`. The rest is
geometry — cut cells, gauge fields, Regge calculus, Hodge stars, manifolds, point clouds.

The homology layer computes nothing itself. `HomologyField::rank_of` dispatches to
`deep_causality_linear`'s `rank_exact` and `rank_gf2`. Linear reciprocates by carrying
`csr_to_packed_gf2_mod2`, whose docstring reads *"This is the conversion `deep_causality_topology`'s
boundary operators need."* Topology keeps a private `widen_to_dense_i64` doing the same class of job
in the other direction. The seam is real and it is split.

Measured, and this is what makes the split cheap: outside topology, `ChainComplex` is named in
`deep_causality_cfd` (10 files) and `deep_causality_physics` (2). The only methods they call are
`num_cells` (18 sites) and `uniform_lattice_layout` (1). **No crate outside topology calls
`boundary_matrix`, `coboundary_matrix`, `betti_number` or `betti_number_over`.** The half that moves
has no external consumer; the half that stays is what everyone uses.

`deep_causality_quantum` depends on ten crates, and neither topology nor linear is among them. QCL
gaps G-07 and G-09 force that edge to be created. This change decides what is on the other end.

## Goals / Non-Goals

**Goals:**
- One crate owns the chain-complex layer, so the seam stops being split across two.
- `deep_causality_quantum` can reach homology without depending on geometry.
- No downstream crate needs an edit. The change is additive at every published boundary.
- The four defects in the blast radius are fixed as the code moves, not after.

**Non-Goals:**
- Integral homology, torsion, Smith normal form. The crate could grow these; none is needed for it
  to exist, and the field-coefficient path already works.
- Persistence, discrete Morse reduction, coreduction.
- G-04 (homology representatives) and G-08 (Poincaré duality). This change gives them a home; it
  does not build them.
- Moving `Chain<T>` or `cup_product`.
- Parameterising the coefficient ring. See Decision 3.

## Decisions

### 1. Split `ChainComplex` by what it talks about, not by who calls it

The trait's eleven items divide into two disjoint groups:

| Homology — moves | Geometry — stays |
|---|---|
| `num_cells(k)`, which is `dim C_k` | `type CellType: Cell` |
| `max_dim()` | `type CellIter<'a>` |
| `boundary_matrix(k)` | `type Metric` |
| `coboundary_matrix(k)` | `cells(k)` |
| `betti_number_over(k, field)` *(provided)* | `uniform_lattice_layout()` |
| `betti_number(k)` *(provided)* | |

`num_cells` sits on the homology side because it is the dimension of the chain group, even though
its external callers want it as a cell count. The supertrait relation gives them both readings from
one method.

**Alternative rejected:** splitting by caller, leaving `num_cells` in topology. That would duplicate
it, since the homology trait needs `dim C_k` to compute anything.

### 2. Topology re-exports the three moved names

```rust
pub use deep_causality_homology::{ChainComplex, Gf2Chain, HomologyField};
```

Existing `use deep_causality_topology::ChainComplex` keeps compiling. Without this, 12 files across
CFD and physics need an import change for no benefit.

**Alternative rejected:** requiring downstream crates to import from the new crate. It converts an
additive change into a coordinated one and buys nothing.

### 3. `boundary_matrix` keeps `Cow<'_, CsrMatrix<i8>>`

The entries are incidence numbers. For a cell complex they lie in `{-1, 0, 1}` by construction —
this is a property of the boundary operator, not a storage choice — so `i8` is the honest type and
the trait needs no coefficient parameter. The coefficient field is a property of the *computation*,
not of the complex, and `HomologyField` already carries it at the call site where the choice is
made.

**Alternative rejected:** `trait ChainComplex<R>` with `boundary_matrix(k) -> Cow<CsrMatrix<R>>`.
It is breaking, its size is identical wherever the trait lives, and it would put a parameter on the
object to express a choice that belongs to the operation.

### 4. `Chain<T>` and `cup_product` stay in topology

`Chain<T>` holds an `Arc<SimplicialComplex<T>>` and its `check_compatibility` uses `Arc::ptr_eq`. It
is a simplicial object wearing a homological name.

The cup product needs `SplittableCell::split` for the Alexander–Whitney diagonal. A chain complex
has no multiplication; a diagonal approximation comes from cells. The cup product is an operation of
cellular complexes of spaces.

### 5. `Gf2Chain` gains the identity of its complex

Today `add`, `intersect` and `inner` guard on degree and length alone, so a chain from a different
complex with the same cell count is accepted and answered. `Chain<T>` in the same crate already
decided the opposite. Settle it as the type moves, before anything is built on it.

### 6. Conversions consolidate in linear

`widen_to_dense_i64` becomes `deep_causality_linear::csr_i8_to_dense_i64`, beside the five
conversions already there. It has no homological content.

`csr_to_packed_gf2_mod2` and `csr_to_packed_gf2_strict` stay where they are. They could rise into
homology on the argument that the `{-1, 0, 1}` reduction is a statement about boundary operators.
Moving them is a published-API break for no gain today.

### 7. Fix four defects while moving

1. **Basis orientation.** `kernel_basis_gf2` allocates `zeros(cols, free.len())`;
   `image_basis_gf2` allocates `zeros(rows, pivots.len())`. The bases are **columns**. Four
   docstrings say rows: `packed_gf2_vector/mod.rs:107-108`, `gf2_chain/mod.rs:73`,
   `packed_gf2_vector_tests.rs:177`, `gf2_chain_tests.rs:100`. `from_row` is a contiguous word-slice
   copy and cannot read a column; `from_column` does not exist. A caller following the documented
   path gets a wrong-length vector. This is live today.
2. **Degenerate grades return `(0,0)`.** `betti_number_over` survives via
   `n_k.saturating_sub(rank_k)`; a kernel basis would not, and G-04 will need one.
3. **`∂∘∂ = 0` is stated nowhere** — not in the trait, not in the conformance harness.
4. **A test passes by coincidence.** `lattice_complex_test.rs:183-193` asserts
   `cols(∂₂) == rows(∂₁)`, comparing `n₂` with `n₀`. Composability of `∂₁ ∘ ∂₂` is
   `cols(∂₁) == rows(∂₂)`.

## Risks / Trade-offs

- **A 30th publishable crate carries permanent overhead** → Two hand-maintained CI lists must be
  edited: `.github/workflows/formalization.yml` (omission gives a false `MISSING Rust witness` on
  every new theorem id) and `.github/workflows/rust_deps.yml` (omission is silent, cargo-machete
  simply never inspects the crate). `build/scripts/crates.sh` reads the member list from
  `Cargo.toml`, so SBOM, Miri, check and format follow for free.

- **Crate creation is effectively irreversible here** → `deep_causality_sparse` is retired, still a
  workspace member, still shipping, because AGENTS.md forbids deletion. Mitigated by the measured
  scope: 419 lines with one external consumer pending, not a speculative boundary.

- **The `num_cells` move could surprise a downstream caller** → The re-export makes it invisible.
  Verify by building `deep_causality_cfd` and `deep_causality_physics` with no `Cargo.toml` or
  `use` edits; a required edit means the re-export is wrong.

- **Fixing the basis orientation may change behaviour, not just docs** → No caller crosses that
  boundary yet, which is why no test caught it. Add `from_column` and a test that round-trips a
  kernel basis through it before touching the docstrings.

- **Splitting a trait can break inference at call sites that relied on one bound** → All four
  implementors are inside topology and implement both halves. Method resolution through a supertrait
  is unchanged. Verified by `bazel test //...`.

## Migration Plan

1. `deep_causality_linear` 0.1.2: add `from_column` and `csr_i8_to_dense_i64`; correct four
   docstrings. Independently useful and independently releasable.
2. Create `deep_causality_homology` 0.1.0 with the three moved items and full Bazel, SBOM, lint and
   CI registration.
3. `deep_causality_topology` 0.7.4: add `CellComplex: ChainComplex`, re-export the three names,
   delete nothing.
4. Fix defects 2–4 and settle `Gf2Chain`'s complex identity.
5. Verify: `bazel test //...`, clippy, fmt, and a build of CFD and physics with zero edits.

**Rollback.** Steps 1 and 3 are additive and revert cleanly. Step 2 is the irreversible one; run it
after step 1 has proved the seam.

## Open Questions

- Should `Gf2Chain` carry `Arc<Complex>` like `Chain<T>`, or a cheaper complex identity such as a
  generation counter or a shape tuple? `Chain<T>`'s `Arc::ptr_eq` requires a concrete complex type,
  which a generic `Gf2Chain<W>` does not have.
- Does the new crate need a `LEAN_HOMOLOGY.md` and a theorem-map namespace at creation, or only when
  it first carries a theorem? `∂∘∂ = 0` is a candidate first entry.
