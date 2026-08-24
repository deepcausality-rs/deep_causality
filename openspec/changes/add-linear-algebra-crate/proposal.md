# Establish `deep_causality_linear` as the workspace's linear algebra crate

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
- Add a dense row-major matrix, a dense vector, and a bit-packed 𝔽₂ matrix alongside the sparse one.
  The vector answers the larger half of the census: **60 rank-1 constructions against 46 rank-2**.
- Band every operation on the tower trait it actually needs, so the crate is generic over **integers
  and floats** rather than floats with generics bolted on. The determinant needs no division and so
  works over ℤ; elimination divides and so does not. ℕ admits less again — no subtraction, no
  determinant.
- Add exact integer linear algebra: a fraction-free determinant and an exact rank over
  `EuclideanDomain`, neither leaving ℤ. This is what `deep_causality_topology`'s `CsrMatrix<i8>`
  boundary matrices need and currently reach by densifying to `f64` and running an SVD.
- Add the operations that make it a library rather than a consolidation: `solve(A, b)` by LU with
  partial pivoting, the factorisation exposed for reuse, triangular substitution, the Hermitian inner
  product, and the vector and matrix norms defined once.
- Give every container a `deep_causality_haft` witness matching the existing ones, so the new types
  compose with `CausalTensor` and the other mathematical crates rather than stopping the pipeline.
- Give every container its **tower** impls too, not only the HKT witness. A witness makes a container
  composable through `deep_causality_haft`; it does not make it composable through the tower, and a
  function bounded on `Ring` or `Module` cannot take a container that never declares them. The crate
  inherits an unfinished case: `CsrMatrix<f64>` reaches `AbelianGroup` and stops, because
  `Distributive` and `Annihilating` are missing, which also puts `Module<S>` out of reach even though
  `arithmetic/mod.rs:283` already implements the scaling. The vector is a `Module<R>` — the tower's
  name for a vector space, and the general notion that admits ℤ where `Field` would not.
- **Add 𝔽₂ to the tower.** The packed representation needs an element type; the crate is forbidden
  from defining a scalar; the tower had none. `Gf2` moves into `deep_causality_num` alongside every
  other primitive, with its law markers in `deep_causality_algebra`. Packing decides the storage, not
  the element — the prototype packs bits and still names `type Scalar = Gf2`.
- **Separate fields by characteristic, not by finiteness.** `Field` is blanket-implemented, so
  admitting 𝔽₂ widens every `T: Field` bound in the workspace at once. Sixteen sites compute
  `T::one() + T::one()` as two; twelve are bounded on `RealField` and safe; **four sit under a
  `Field` bound**, including `commutator_geometric`, whose `one / (one + one)` is a division by zero
  over 𝔽₂. Finiteness is the wrong guard — 𝔽₃ is finite and halves, 𝔽₄ is finite and does not, 𝔽ₚ(x)
  is infinite and does not — so the tower gains `CharacteristicZero` and `FiniteField`, disjoint by
  definition but not a partition, and the four sites are rebounded.
- **Sweep the bounds down.** Integer admission is what bounding each operation at its lowest correct
  level yields, not a feature beside the field work. The code being moved bounds on ad-hoc operator
  bundles — `mat_mult_impl` takes `T: Copy + Clone + Mul<Output = T> + Zero + PartialEq + Default`,
  a semiring spelled longhand — and each becomes a tower trait. Every bound loosened off `Field`
  names the number set it newly admits and is instantiated at it.
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
- `linear-scalar-contract`: no new scalar traits; every operation banded on the weakest tower trait
  that makes it correct, from `CommutativeSemiring` up to `RealField`, with exact and approximate
  scalars distinguished at the signature.
- `linear-integer-algebra`: fraction-free integer determinant, exact integer rank, the ring
  operations without a field bound, and the three distinct ranks kept distinct.
- `linear-vector`: the dense vector, dot and outer products, the Hermitian inner product,
  matrix–vector for all three representations, and norms defined once.
- `linear-solve`: `solve`, LU with partial pivoting, a reusable factorisation, triangular
  substitution, and solving preferred over inverting.
- `linear-hkt-composition`: an HKT witness per container, the laws tested, and composition with the
  neighbouring mathematical crates preserved.
- `linear-tower-integration`: every container implementing the tower traits its structure supports,
  the vector as a `Module<R>`, law markers that name their operator, bounds that state algebraic
  structure rather than operator bundles, and the lowering sweep with each admission instantiated.
- `num-finite-field`: 𝔽₂ as a tower scalar owned by `deep_causality_num`, the characteristic-based
  separation of fields, the `CharacteristicZero` bound on everything that divides by an integer, and
  the rungs 𝔽₂ deliberately does not reach.
- `linear-consumer-migration`: the retirement of `deep_causality_sparse`, the type identity a
  re-export preserves, the duplicated linear algebra in `deep_causality_multivector` marked with its
  successor, and what the two build systems and the documentation must agree on.
- `linear-test-first-development`: the crate is built test-first — API declared with unimplemented
  bodies, the full suite written and observed failing against it, the suite verified against
  deliberate defects, then implementation, and only then downstream migration.

### Modified Capabilities

- `neumann-poisson`: `openspec/specs/neumann-poisson/spec.md:34` requires the preconditioned CG
  variant of `deep_causality_sparse`. The requirement moves to `deep_causality_linear`; the
  behaviour is unchanged.

## Deferred to subsequent changes

- **Tier 2**: QR-based least squares alongside the existing Cholesky path; pseudo-inverse and
  condition number, both cheap once the SVD is here; non-symmetric eigendecomposition; the matrix
  exponential promoted out of `causal_tensor_network/solve/local.rs:880`; Hermite and Smith normal
  forms — Smith is what integral homology **with torsion** would need.
- **Tier 3**: BLAS or LAPACK bindings, SIMD, GPU offload; iterative solvers beyond conjugate
  gradient; sparse direct factorisation with fill-reducing ordering.

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
- **`deep_causality_num`**: gains `Gf2`, the tower's first finite field.
- **`deep_causality_algebra`**: gains `Characteristic`, `CharacteristicZero` and `FiniteField`, and
  the law-marker impls for `Gf2`. Both crates take a version bump before phase 1 begins.
- **`deep_causality_multivector`**: four sites rebounded on `CharacteristicZero`, and three
  duplicates marked — `MultiVectorL2Norm::norm_l2` and `CausalMultiField::squared_magnitude` routed
  through the shared norm, `BatchedMatMul` decided explicitly. `ScalarEval` is left alone: it is
  already a facade over `Normed` rather than a second definition.
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
