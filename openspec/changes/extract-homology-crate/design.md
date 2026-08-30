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

### 5. `Gf2Chain` identifies its chain group structurally, by `(degree, len)`

The workspace has already answered this four times. An element carries the *defining data* of its
ambient object, by value, and operations compare that data:

| element | ambient object | what it carries |
|---|---|---|
| `CausalTensor<T>` | a tensor space | `shape` |
| `CausalMultiVector<T>` | a Clifford algebra | `Metric` — the signature |
| `DensityMatrix<R>` | `B(H)` | `dim` |
| `Chain<T>` | a simplicial complex | `Arc<SimplicialComplex<T>>`, compared by `ptr_eq` |

`Chain<T>` is the one nominal case among four structural ones. Two identically built complexes fail
its `Arc::ptr_eq` though they are the same complex, which is a claim homology does not make.
Following it would spread the outlier.

The rule asks what the operations need. `Gf2Chain` has `add`, `intersect`, `inner`, `weight`,
`support`, `support_pairs`, `support_triples` — every one an operation of the chain *group* `C_k`,
none of them mentioning a boundary. And `C_k = 𝔽₂^{n_k}` is fixed by `n_k` alone: two complexes with
twelve 1-cells have the same `C₁`, and a sum of two of its elements is right whichever complex
produced them. The defining data is `(degree, len)`, and the type already carries both.

So the carrier stays as it is, and one guard changes. Today `add` checks the degree in
`same_degree` and the length inside `PackedGf2Vector::add`, so two halves of one condition surface
as two different error types. `same_group` checks both and names `C_k` in the message.

The complex enters with `∂`, and there only. When `boundary` arrives — G-04 needs it — it reads

```rust
fn boundary(&self, c: &impl ChainComplex) -> Result<Self, HomologyError>
```

and checks `c.num_cells(self.degree) == self.len()` against the complex being applied, which a
remembered token cannot do once it is stale.

**Alternative rejected: `Arc<C>` and `ptr_eq`.** `ChainComplex` is a trait, so there is no complex
type to hold. `Gf2Chain<W>` becomes `Gf2Chain<W, C>`, and the parameter reaches `from_row` and
`from_column`, whose input is a `PackedGf2` kernel basis with no complex in it. That constructor is
the Haruna path.

**Alternative rejected: a generated `ComplexId`.** A global atomic counter with no mathematical
content, and it makes a chain unconstructible without a complex to issue a token — the kernel-basis
path again.

**Alternative rejected: a phantom brand `Gf2Chain<W, Tag>`.** The right approximation to the
dependent type, and out of reach here: the invariant-lifetime form needs `unsafe`, which the
workspace forbids repo-wide, and the marker-type form puts a parameter on every signature to
express a distinction no current operation can act on.

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

### 8. The crate carries `LEAN_HOMOLOGY.md` from creation, discharging an open hypothesis

`lean/DeepCausalityFormal/Linear/RankNullity.lean` proves `gf2_betti_from_ranks`: that
`(n_k − rank ∂_k) − rank ∂_{k+1}` is `dim H_k`. It proves it under a hypothesis it never supplies.

```lean
(hchain : LinearMap.range dk1.mulVecLin ≤ LinearMap.ker dk.mulVecLin)
```

That hypothesis is `∂ₖ ∘ ∂ₖ₊₁ = 0`. It is unproved in Lean, unstated in the `ChainComplex` trait,
and unasserted in the conformance harness — defect 3 above. Every Betti number the workspace
computes rests on an assumption written down in one place: as an argument to a theorem.

`deep_causality_homology` is where that assumption becomes an obligation on implementors, so it is
where the proof belongs. Two theorems, and the first is the bridge:

- `homology.chain.dd_zero_implies_range_le_ker` — `∂ₖ ⬝ ∂ₖ₊₁ = 0 → range ∂ₖ₊₁ ≤ ker ∂ₖ`, turning
  `hchain` into a matrix identity a Rust test can check.
- `homology.chain.betti_from_dd_zero` — `gf2_betti_from_ranks` restated with `∂ₖ ⬝ ∂ₖ₊₁ = 0` in
  place of the subspace inclusion, so the Betti identity stands on a hypothesis the Rust side
  discharges rather than assumes.

This meets the bar `LEAN_LINEAR.md` sets — *load-bearing and invisible* — for the same reason
rank–nullity did: it is the step the source performs without stating.

The Rust witnesses are what task 7.2's harness already computes, tagged with the two ids and moved
to `deep_causality_homology/tests/formalization_lean/`, which is where
`.github/workflows/formalization.yml` looks.

**Not made a theorem: that `C_k` depends only on `n_k`.** It would formalize Decision 5, and in
Lean `Fin n → F2` depends on `n` definitionally, so the statement is true by `rfl`. Neither
load-bearing nor invisible.

Four registrations follow from adding the crate and the namespace, each silent if missed:
`_NAMESPACES` in `lean/BUILD.bazel` (no `Homology` entry means the proofs are never type-checked by
`bazel test //lean:proofs`), the grep list in `.github/workflows/formalization.yml`, two rows in
`lean/THEOREM_MAP.md`, and `cache_roots` in `MODULE.bazel` for any import the Linear file does not
already pull — `Matrix.mulVecLin_mul` lives in `Mathlib.LinearAlgebra.Matrix.ToLin`, which is not
in that list today.

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

Each step that produces API runs mock → suite → suite audit → implementation, with a root-cause
diagnosis before any failing test is touched. `tasks.md` states the protocol and names the phase on
each task.

0. `openspec/notes/homology/reference/reference.py`: published Betti numbers and exact 𝔽₂ bases,
   importing nothing from this workspace. Every later expectation resolves here, so it comes first.
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

Both questions this document opened are now decided — the carrier in Decision 5, the formalization
in Decision 8. What remains open is downstream of them:

- Does `Chain<T>`'s `Arc::ptr_eq` want the same structural treatment? It is the workspace's one
  nominal identity and it refuses sums that are mathematically fine. Out of scope here: `Chain<T>`
  stays in topology (Decision 4), and changing its compatibility rule is a published-API change
  with its own blast radius.
- Does `homology.chain.dd_zero_implies_range_le_ker` need `∂` over a general field, or is 𝔽₂
  enough? `RankNullity.lean` fixes `ZMod 2` and the two files must compose, so 𝔽₂ is the answer for
  the bridge. A ℚ statement would serve `HomologyField::Rational`, which has no formalization
  today and is not opened by this change.
