# Consolidate the workspace's linear algebra into `deep_causality_linear`

## Why

The quantum gap register asks for 𝔽₂ linear algebra (`openspec/notes/quantum/qcl-gaps.md` G-01,
severity S1, blocking three requirements). Deciding where to put it surfaced the prior question:
the workspace has no linear-algebra crate, and its linear algebra is spread across three places.

Measured, not inferred (`openspec/notes/linear/deep-causality-linear.md` carries the full research):

- **`deep_causality_tensor` is two libraries.** 1,088 lines of 2-D matrix operations — `svd` 117,
  `svd_decomp` 170, `svd_truncated` 375, `qr` 145, `eigen` 158, `inverse` 123 — sit alongside 2,069
  lines of N-d tensor operations and 3,881 lines of tensor-train. The matrix half reached a tensor
  crate because `CausalTensor` was the only dense container available.
- **`deep_causality_topology` carries three determinants over three representations** — `&[Vec<R>]`
  Laplace at `regge_geometry/curvature.rs:275`, `CausalTensor` Laplace at
  `manifold/geometry/mod.rs:145`, and flat-slice Gaussian elimination at
  `simplicial_complex/lazy_hodge_star.rs:97` — and two rank helpers that are near-identical copies.
  `chain_complex_impl.rs:94` says so in its own doc comment.
- **Those rank helpers compute homology by thresholding f64 singular values at `1e-5`**, on matrices
  whose entries are `{-1, 0, 1}`. G-02 records the consequence: rank over ℝ is not rank over 𝔽₂, so
  a complex with even-weight dependencies would report a wrong `k` with no error raised.
- **The seam costs nothing and packing pays.** A prototype runs one generic elimination over four
  representations of the same 𝔽₂ matrix. Through a row-operation trait, bit-packed `u64` runs at
  0.92–0.95× the hand-written non-generic loop at every size from n=128 to n=2048 — slightly faster —
  while beating a `Field`-satisfying `Gf2` byte scalar by 1.7× rising to 3.2×, on 8× less memory.
- **The dense type has real call sites: 46.** Taking the rank of every constructed shape across the
  seven consumer crates gives 118 constructions — 60 rank-1, 46 rank-2, 12 rank ≥ 3. Physics, quantum
  and topology call 56 two-dimensional operations and **zero** N-d ones between them; topology
  constructs 46 tensors and not one has rank above 2.
- **`deep_causality_linear` is free on crates.io** (404 against a 200 control).

## What Changes

- Add `deep_causality_linear`, a crate that owns matrix representations and the algorithms over
  them. It sits **below** `deep_causality_tensor`, because `CausalTensor::svd` has to call into it.
  The orphan rule then settles where the access-trait impl for `CausalTensor` lives: a third crate
  cannot write it at all (E0117, probed), and writing it in linear would need the reverse edge, so
  it lives in `deep_causality_tensor`.
- Move `CsrMatrix`, the CG solvers and the sparse HKT witness into it from `deep_causality_sparse`.
- Add a dense row-major matrix and a bit-packed 𝔽₂ matrix, generic over `NaturalNumber` word types,
  alongside the sparse one.
- Build the crate **test-first**: declare the whole public API with `todo!()` bodies, write the
  complete suite against it, observe every test fail for the right reason, prove the suite rejects
  each known defect class, then implement, then migrate consumers. No consumer is repointed until the
  crate's own suite is green at full coverage.
- Add elimination — RREF, rank, kernel basis, image basis, determinant, solve — written once against
  a row-operation trait, with dense and bit-packed implementations. Sparse implements the read side
  only: `axpy_rows` changes a CSR row's non-zero pattern, so sparse elimination is a different
  algorithm and is out of scope here.
- Move the bodies of `svd`, `svd_decomp`, `svd_truncated`, `qr`, `eigen` and `inverse` into
  `deep_causality_linear`. **`CausalTensor`'s methods stay and delegate**, so its public surface is
  unchanged and its 8 in-workspace and 7 example dependents are untouched.
- Route topology's five duplicated helpers through the shared implementations, and route
  `betti_number` through exact 𝔽₂ rank, closing G-01 and G-02.
- **BREAKING** for `deep_causality_sparse`: the crate is retired. It publishes one final version
  that re-exports `deep_causality_linear` and carries a retirement notice in its README, then stays
  available on crates.io and in the workspace for a deprecation window of a few months so that
  already-published dependents keep resolving. Nothing is yanked.
- Drop the `tensor-iso` feature. The `CausalTensor ↔ CsrMatrix` conversion moves into
  `deep_causality_tensor` and stops being optional: the gate exists so that sparse users do not pay
  for a dependency on tensor, and once tensor depends on linear that dependency is already paid. No
  library crate enables the feature today, so this changes one manifest line in
  `examples/mathematics_examples`.

## Capabilities

### New Capabilities

- `linear-crate-identity`: what the crate owns, its position in the tier graph, the acyclicity the
  orphan rule forces, and its `no_std` obligations.
- `linear-matrix-representations`: sparse, dense and bit-packed 𝔽₂ side by side — construction,
  conversion, and which operations each supports.
- `linear-dense-algorithms`: the row-operation seam, the elimination family written once against it,
  and the decompositions relocated behind `CausalTensor`'s unchanged methods.
- `linear-f2-algebra`: exact mod-2 rank, kernel basis and image basis; word-parallel elimination;
  and the exactness that removes the `1e-5` tolerance from homology.
- `linear-consumer-migration`: the retirement of `deep_causality_sparse`, the type identity a
  re-export preserves, and what the two build systems and the documentation must agree on.
- `linear-test-first-development`: the crate is built test-first — API declared with unimplemented
  bodies, the full suite written and observed failing against it, the suite verified against
  deliberate defects, then implementation, and only then downstream migration.

### Modified Capabilities

- `neumann-poisson`: `openspec/specs/neumann-poisson/spec.md:34` requires the preconditioned CG
  variant of `deep_causality_sparse`. The requirement moves to `deep_causality_linear`; the
  behaviour is unchanged.

## Impact

- **New crate `deep_causality_linear`**: absorbs 3,107 lines of `src` and 1,916 of tests from
  `deep_causality_sparse`, plus 1,088 lines relocated from `deep_causality_tensor`, plus the new
  dense and 𝔽₂ representations.
- **`deep_causality_sparse`**: retired to a re-export facade with a README notice. Stays in the
  workspace and published.
- **`deep_causality_tensor`**: gains a dependency on `deep_causality_linear`; its 2-D method bodies
  become delegations; it takes over the `CausalTensor ↔ CsrMatrix` conversion. Public surface
  unchanged.
- **`deep_causality_topology`**: the largest consumer — 61 import sites across 73 files (282 `CsrMatrix` mentions). Five
  duplicated helpers are replaced and `betti_number` changes from f64 SVD to exact 𝔽₂ rank.
- **`deep_causality_physics`**, **`examples/mathematics_examples`**, **`examples/physics_examples`**:
  import paths follow.
- **Build**: 35 Bazel label references across 8 `BUILD.bazel` files. `deep_causality_cfd/BUILD.bazel:30`
  declares a sparse dependency its `Cargo.toml` does not, so the two disagree today and the
  discrepancy is resolved as part of this change.
- **Documentation**: `AGENTS.md` §Project Structure and §Project Dependencies, `README.md:268`,
  `website/web/src/pages/overview/index.astro`, `website/docs/…/install.md` and
  `…/concepts/uniform-math.md`.
- **New-crate checklist**: `build/scripts/sbom.sh` enumerates the publishable crates, and
  `deep_causality_num_rational` was added without being registered there. This change adds its own
  crate to that list and generates the artifacts, rather than becoming the second omission.
  (`deep_causality_quantum` is absent by design — it is `publish = false`.)
- **Not touched**: the 34 files under `openspec/changes/archive/` that name `deep_causality_sparse`.
  Archived changes record what was proposed at the time and are not rewritten.
