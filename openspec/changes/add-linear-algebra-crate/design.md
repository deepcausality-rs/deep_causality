## Context

The workspace has 28 crates and no linear-algebra crate. Matrices exist in five representations:
`CausalTensor<T>` (strided, rank-2 serves as dense), `CsrMatrix<T>`, `Matrix3<F> = [[F;3];3]`, and
two ad-hoc shapes inside `deep_causality_topology` — `&[Vec<R>]` and `&mut [T]` with a stride
argument. `AdjacencyMatrix`, `IncidenceMatrix` and `LaplacianMatrix` are aliases of `CausalTensor`;
`AbcdMatrix` and `DensityMatrix` are newtypes over it.

The consequences are measured in `openspec/notes/linear/deep-causality-linear.md`:

| finding | evidence |
|---|---|
| dense linear algebra lives in a tensor crate | 1,088 lines of svd/qr/eigen/inverse against 2,069 lines of N-d ops and 3,881 of tensor-train |
| topology carries three determinants | `curvature.rs:275` O(n!), `geometry/mod.rs:145` O(n!), `lazy_hodge_star.rs:97` O(n³) |
| topology carries two near-identical ranks | `chain_complex_impl.rs:94` documents itself as a mirror of `cell_complex/mod.rs:172` |
| homology is computed by thresholded f64 SVD | both helpers densify `CsrMatrix<i8>` and count singular values above `1e-5` |
| no 𝔽₂ linear algebra exists | `qcl-gaps.md` G-01, severity S1 |

Constraints this repository imposes: MSRV 1.93.0 pinned to Kani's toolchain, so no specialization
and no negative impls; `unsafe_code = "forbid"` workspace-wide; macros barred from `src`;
`bazel test //...` as the primary gate alongside `cargo`; files are moved aside, never deleted.

## Goals / Non-Goals

**Goals:**

- One crate that owns matrix representations and the algorithms over them, with sparse, dense and
  bit-packed 𝔽₂ side by side.
- Exact 𝔽₂ rank, kernel basis and image basis, closing G-01 and removing the `1e-5` tolerance from
  homology, closing G-02.
- The duplicated determinants and ranks in `deep_causality_topology` replaced by shared
  implementations.
- `deep_causality_tensor`'s public surface unchanged, so its 8 in-workspace and 7 example dependents
  need no edit.
- `deep_causality_sparse` retired without breaking already-published dependents.

**Non-Goals:**

- Sparse elimination. `axpy_rows` changes a CSR row's non-zero pattern, which means reallocating
  every row after it; sparse elimination needs a fill-reducing ordering and symbolic factorisation.
  That is a separate proposal.
- Moving `CausalTensor` itself. An N-d strided tensor is not a matrix, and `ein_sum`, `broadcast`
  and the tensor-train stack stay where they are.
- Moving `Matrix3<F>`. It is a 3×3 array alias with two consumers, both inside `num_complex`
  quaternions. Folding it in would put a dependency edge on `deep_causality_num` for no gain.
- A BLAS or LAPACK binding, SIMD intrinsics, or GPU offload. `unsafe_code = "forbid"` and the
  acceleration survey in `openspec/notes/tensor-network/ACCELERATION-SOTA-FIRST.md` govern that
  question separately.
- Renaming `deep_causality_sparse` in place. crates.io cannot rename a crate.

## Decisions

### The dependency runs tensor → linear

For linear's generic algorithms to accept a `CausalTensor`, some crate has to write
`impl MatrixView for CausalTensor<f64>`. The orphan rule permits that in exactly two places, and each
forces a dependency direction:

| where the impl lives | legal | why | forces |
|---|---|---|---|
| `deep_causality_linear` | yes | `MatrixView` is local | linear `use`s `CausalTensor` → linear → tensor |
| `deep_causality_tensor` | yes | `CausalTensor` is local | tensor `use`s `MatrixView` → tensor → linear |
| any third crate | no | neither is local | E0117; no impl exists |

`openspec/notes/linear/prototype/tensor_impl/` compiles the third row and confirms E0117.

The two legal rows are mutually exclusive — taking both closes a cycle — so the orphan rule narrows
the field without choosing. What chooses is the relocation of the decompositions: `CausalTensor::svd`
has to call into `deep_causality_linear`, and a crate can only call into what it depends on. That
fixes tensor → linear, which makes "the impl lives in linear" the forbidden direction and leaves
`deep_causality_tensor` as the only home for the impl.

Two consequences follow. `deep_causality_linear` cannot depend on `deep_causality_tensor` under any
feature, so the `tensor-iso` conversion moves up into tensor — and stops being a feature, because the
dependency it was gating is now unconditional. And the 𝔽₂ layer lands below tensor, so
`deep_causality_quantum` reaches mod-2 rank without pulling in the tensor crate.

This is affordable because `deep_causality_sparse → deep_causality_tensor` is already optional:

```toml
tensor-iso = ["dep:deep_causality_tensor"]
[dependencies.deep_causality_tensor]
optional = true
```

confined to `extensions/ext_iso.rs` behind `#[cfg(feature = "tensor-iso")]`. The core — `CsrMatrix`,
`solver/cg.rs` — is tensor-free, so the absorbing crate can sit below tensor with no contortion.

### Delegate the decompositions, do not relocate them

`svd`, `svd_decomp`, `svd_truncated`, `qr`, `eigen` and `inverse` are inherent methods on
`CausalTensor` and members of the `Tensor` trait (`traits/tensor.rs:435,439`). Removing them breaks
`deep_causality_physics` (GRMHD, the Kalman filter), `deep_causality_quantum` (channels,
projections), `deep_causality_multivector`, `deep_causality_topology` and seven example crates.

The bodies move; the methods stay and call through. The public surface, the error type and the
return shapes are unchanged, which makes this a patch-level change for tensor rather than a major
one. It also gives the 1,088 lines a home where a dense matrix type can use them without going
through a rank-2 tensor.

### Sparse implements the read side only

The prototype records the constraint in `tensor_impl/src/lib.rs`: `swap_rows` is fine on CSR,
`axpy_rows` is not. Adding a multiple of one sparse row to another changes that row's non-zero
pattern, which in CSR means reallocating every row after it.

So the seam splits. A read trait — shape and element access — that CSR, dense and bit-packed all
implement. A row-operation trait that only the dense-layout representations implement. Elimination
is generic over the second. A CSR matrix reaches elimination by converting to dense, which is what
topology's rank helpers already do by hand.

This bounds the "side by side" claim honestly: one crate owning both representations and the
algorithms appropriate to each, rather than one algorithm covering everything.

### 𝔽₂ is bit-packed, not a tower scalar

G-01 argued this and the prototype now prices it. At n=2048, packed `u64` runs 3.2× faster than a
`Gf2` scalar satisfying `Field` stored one byte per bit, on 8× less memory, and the gap widens with
n as cache pressure grows.

The generic seam costs nothing: 0.92–0.95× the hand-written non-generic loop at every size, slightly
faster because the trait's `from_col` argument lets the implementation skip the eliminated prefix
that the hand-written loop re-reads. G-01's "roughly 200 lines over `u64`" and a generic algorithm
behind a row-operation trait run at the same speed, so the generic one is taken.

The word type is generic over `NaturalNumber` (`deep_causality_num/src/integer/natural.rs`) rather
than fixed to `u64`, which the algebra tower work made possible.

### 𝔽₂ is owned by `deep_causality_linear`, superseding G-01's owner field

G-01 assigns 𝔽₂ to `deep_causality_topology` "because that is where chain complexes live and
topology must not learn about codes." The same reasoning places it better in a linear-algebra crate,
which knows about neither chain complexes nor codes. Topology then consumes it, and
`deep_causality_quantum` consumes it without depending on topology.

### The retired crate re-exports rather than freezes

`deep_causality_sparse` publishes one final version whose `src/lib.rs` re-exports
`deep_causality_linear` and whose README carries a retirement notice. It stays in the workspace and
on crates.io for a few months. Nothing is yanked.

Freezing the implementation instead would make `deep_causality_sparse::CsrMatrix` and
`deep_causality_linear::CsrMatrix` distinct types. Any crate depending on both — which is exactly
what a partially-migrated dependent looks like — would fail to typecheck. Re-exporting keeps one
type, so a dependent can migrate module by module.

### Archived openspec changes are not rewritten

36 of the 203 files naming `deep_causality_sparse` are under `openspec/changes/archive/`. They
record what was proposed at the time. Rewriting them would falsify the record.

## Risks / Trade-offs

**Replacing topology's determinants perturbs floating-point output.** Regge geometry uses
`det_recursive` on Cayley-Menger determinants of small simplices, where Laplace expansion is exact in
a different way than elimination and, at n ≤ 3, faster. `deep_causality_topology` has 29,474 lines of
tests, some of which may encode expected values to full precision. *Mitigation:* keep the closed
forms for n ≤ 3 and dispatch to elimination only at n ≥ 4, which is what
`manifold/geometry/mod.rs:145` already does for its base cases. Run the topology suite before and
after and diff, rather than assuming.

**Changing `betti_number` from f64 SVD to exact 𝔽₂ rank changes answers.** That is the point — G-02
records that the current answer can be wrong — but it changes committed test expectations wherever
the two ranks differ. *Mitigation:* the two agree for the toric code and for every complex currently
under test, per G-02. Keep the ℝ-rank path available for complexes that are not being read as codes,
and make the choice explicit at the call site rather than global.

**A new crate boundary can trap coherence.** The `deep_causality_num` split produced E0119 when
marker traits over a now-foreign `Float` could not admit integers. *Mitigation:* the traits this
change introduces are defined in `deep_causality_linear` and implemented by crates above it, which
is the direction the orphan rule permits; the prototype compiles both the permitted and the
forbidden direction and shows which fails.

**Delegation adds a call layer to the most-used numerical code in the workspace.** `matmul` alone has
15 call sites in the physics Kalman filter. *Mitigation:* the delegating methods are generic and
monomorphise; the prototype's 0.93× seam measurement is evidence that a trait boundary in this
position does not cost, but the tensor benchmarks are re-run before and after regardless.

**The two build systems already disagree.** `deep_causality_cfd/BUILD.bazel:30` declares a
`deep_causality_sparse` dependency that `deep_causality_cfd/Cargo.toml` does not. Migrating one
without the other would hide it. *Mitigation:* resolve the discrepancy explicitly rather than
carrying it forward.

**The deprecation window is a promise with no enforcement.** Nothing fails if the retired crate stops
building. *Mitigation:* keep it a workspace member so `bazel test //...` and `cargo test` cover it
for as long as it exists.

## Migration Plan

Five phases, each independently green under `bazel test //...`.

1. **Stand up the crate.** `deep_causality_linear` with `CsrMatrix`, the CG solvers, the HKT witness
   and the errors moved over from `deep_causality_sparse`. `deep_causality_sparse` becomes a
   re-export facade with a retirement notice; every in-workspace consumer switches its import.
   The `tensor-iso` conversion moves to `deep_causality_tensor` and the feature is deleted. At the
   end of this phase the workspace builds against the new name and the old name still works.
2. **Add the representations.** A dense row-major matrix and a bit-packed 𝔽₂ matrix, plus the read
   and row-operation traits, plus conversions among the three.
3. **Add elimination.** RREF, rank, kernel basis, image basis, determinant and solve, written once
   against the row-operation trait, with the 𝔽₂ implementation carrying word-parallel XOR.
4. **Relocate the decompositions.** `svd`, `svd_decomp`, `svd_truncated`, `qr`, `eigen`, `inverse`
   move their bodies; `CausalTensor` keeps its methods and delegates. Tensor benchmarks re-run.
5. **Retire the duplication.** Topology's three determinants and two ranks route through the shared
   implementations; `betti_number` routes through exact 𝔽₂ rank; G-01 and G-02 are marked closed in
   `openspec/notes/quantum/qcl-gaps.md`.

Publication order: `deep_causality_linear` 0.1.0 first, then the final `deep_causality_sparse`, then
the dependents. release-plz strips path dependencies when verifying publish tarballs, so each
dependent resolves the *published* API of the crate below it and the order is load-bearing.

## Open Questions

1. **Does the retirement window end in a yank?** The stated intent is a few months of availability so
   that already-published dependents keep resolving. Yanking afterwards would break the dependents
   the window exists to protect, so the default is that it does not.
2. **Does the dense matrix type replace rank-2 `CausalTensor` at any call site?** Phase 4 makes both
   viable. Physics and quantum use rank-2 tensors heavily and switching them is a separate decision
   with its own blast radius.
3. **Does `Matrix3` fold in later?** It has two consumers and lives below this crate. Folding it in
   would need `deep_causality_num` to depend on `deep_causality_linear`, which inverts the tower.
4. **How is the 𝔽₂ rank chosen at the call site in topology?** A parameter on `betti_number`, a
   separate method, or a property of the complex. Phase 5 decides; the risk section requires only
   that it not be a silent global switch.
