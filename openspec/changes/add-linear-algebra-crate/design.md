## Context

The workspace has 29 crates and no linear-algebra crate. Matrices exist in five representations:
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
| a dense matrix type has 46 call sites | 118 constructions across seven crates: 60 rank-1, 46 rank-2, 12 rank ≥ 3 |
| three crates never construct anything above rank 2 | physics, quantum, topology: 56 two-dimensional ops, **zero** N-d ops |

Constraints this repository imposes: MSRV 1.93.0 pinned to Kani's toolchain, so no specialization
and no negative impls; `unsafe_code = "forbid"` workspace-wide; macros barred from `src`;
`bazel test //...` as the primary gate alongside `cargo`; files are moved aside, never deleted.

## Goals / Non-Goals

**Goals:**

- One crate that owns matrix representations and the algorithms over them, with sparse, dense and
  bit-packed 𝔽₂ side by side, plus the vector type the census showed is the larger need.
- A crate that is genuinely generic over the tower — integers and floats — rather than a float
  library with type parameters. Operations banded by the structure they need, not by convenience.
- The minimum surface expected of a linear algebra library: solve, a reusable factorisation,
  triangular substitution, inner products and norms.
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

### Integers are a first-class scalar, not a special case

The tower distinguishes ℕ, ℤ, ℚ, ℝ and ℂ, and linear algebra respects that distinction rather than
flattening it to `f64`. The decisive fact is that **the determinant is a polynomial in the entries**
— it needs no division — so it is well defined over any commutative ring, ℤ included. Gaussian
elimination divides by its pivot and therefore leaves ℤ on its first step.

That is not academic here. `deep_causality_topology`'s boundary matrices are `CsrMatrix<i8>`
(`cell_complex/boundary_operator.rs:19`), and their rank is currently obtained by densifying to
`f64`, running an SVD and thresholding at `1e-5`. The rank of an integer matrix is an exact question.
Bareiss fraction-free elimination answers it in cubic time without leaving ℤ, using the `div_euclid`
and `normalize` that `EuclideanDomain` already supplies.

So the crate carries three ranks — exact over `EuclideanDomain`, exact over 𝔽₂, numerical over
`RealField` — as three separate calls. They disagree on real inputs, and `qcl-gaps.md` G-02 records
what conflating two of them already cost.

### The scalar layer is the tower's, not the crate's

`deep_causality_algebra` already publishes `CommutativeSemiring`, `CommutativeRing`,
`EuclideanDomain`, `Field`, `RealField`, `Normed`, `NormedScalar`, `ConjugateScalar` and `Scalar`,
and `deep_causality_num` publishes `NaturalNumber`. The crate defines none of its own.

This is the discipline the `deep_causality_num` split was about. A linear-algebra crate with its own
scalar hierarchy would fork the tower, and the E0119 traps that split produced are the evidence for
what that costs. `NormedScalar` in particular does more work than it looks: `modulus_squared` lands
in an ordered `Real`, which is what lets partial pivoting work over ℂ without requiring the scalar
itself to be ordered.

### Composition is part of the contract, not an extension

Every container gets a `deep_causality_haft` witness matching the trait set `CsrMatrixWitness`
already implements — `HKT`, `Functor`, `Foldable`, `Pure`, `Applicative`, `Monad`, `CoMonad`,
`Adjunction`. A container that stopped short would compose in some pipelines and not others, which is
worse than being uniformly absent, and the workspace has cross-crate examples that would break
silently.

### The crate is built test-first, and the suite is verified before it is trusted

The order is: declare the API with unimplemented bodies, write the complete suite against it, observe
every test fail with the unimplemented panic, prove the suite rejects each defect class this change
already knows is reachable, implement until green, and only then repoint a consumer.

Two of those steps are not ceremony. **Observing the failure** distinguishes a test that will catch a
defect from one that passes vacuously — a suite written after the implementation tends to encode what
the implementation does. **Verifying against deliberate defects** is the only way to know the suite
is worth gating on: the research already produced four concrete defect classes (an unpivoted
elimination, an off-by-one in the packed word index, a loosened 𝔽₂ tolerance, a rank off by one), so
each is a thing the suite must be shown to catch rather than a hypothesis.

**Gating migration on a green suite** separates two failure sources. Repointing 102 import sites
against an implementation that is still moving means a build failure could be either a broken
consumer or a broken library. Sequencing them makes any post-migration failure unambiguously a
migration failure.

The repository's standing rules apply unchanged: full coverage of added files, tests mirror `src`
file for file, every test module registered upward and declared in `tests/BUILD.bazel`, shared
helpers under `src/utils_tests/` because Bazel cannot reach helpers inside `tests/`, and a failing
test changes the implementation or the API — never the test.

### `gaussian_determinant` is not a general determinant

It is correct where it is used and wrong as a shared implementation. `lazy_hodge_star.rs:81` feeds it
a Gram matrix — `vectors[i]·vectors[j]`, symmetric with a strictly positive diagonal — where taking
`mat[i][i]` as the pivot always works. The two Laplace determinants are fed Cayley-Menger matrices,
whose `(0,0)` entry is zero by construction, where it always fails.

So the consolidation is not "pick the O(n³) one and delete the two O(n!) ones". The shared
implementation pivots by search, which none of the three does today, and the closed-form cutoff at
order three is a separate matter of speed. `deep_causality_physics` carries five more fixed-size
closed forms (`invert_4x4`, `invert_3x3`, `inverse_spatial_metric`, `symmetric_3x3_eigenvalues`, and
an inline 3×3 determinant) which are correct as written and stay where they are — evidence for the
small-n rule rather than targets for it.

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

### 𝔽₂ is bit-packed in storage, and a tower scalar as an element

G-01 argued for packing and the prototype now prices it. At n=2048, packed `u64` runs 3.2× faster
than a `Gf2` scalar stored one byte per bit, on 8× less memory, and the gap widens with n as cache
pressure grows.

That measurement decides the **storage** and says nothing about the **element type**. A packed
matrix still has to answer `get` with something, and the prototype is explicit about what:
`prototype/consumer_b/src/packed_gf2.rs:76` sets `type Scalar = Gf2`, where `Gf2(pub bool)` is
defined at `prototype/linear_a/src/gf2_scalar.rs:15`. Both facts hold at once — pack the bits,
name the element.

`linear-scalar-contract` forbids this crate from defining a scalar newtype, and the tower has none
to offer, so `Gf2` moves into `deep_causality_num` alongside every other primitive and its law
markers into `deep_causality_algebra`. That is the same arrangement `i8` already has: the type is
foreign to the algebra crate, and `impl IntegralDomain for i8 {}` is written there regardless.

The generic seam costs nothing: 0.92–0.95× the hand-written non-generic loop at every size, slightly
faster because the trait's `from_col` argument lets the implementation skip the eliminated prefix
that the hand-written loop re-reads. G-01's "roughly 200 lines over `u64`" and a generic algorithm
behind a row-operation trait run at the same speed, so the generic one is taken.

The word type is generic over `NaturalNumber` (`deep_causality_num/src/integer/natural.rs`) rather
than fixed to `u64`, which the algebra tower work made possible.

### The tower separates fields by characteristic, not by finiteness

Admitting 𝔽₂ needs a guard, because `Field` is blanket-implemented (`field.rs:41`). A type becomes a
field the moment it satisfies `CommutativeRing + InvMonoid + Div + DivAssign`, with no per-type
opt-in, so every `T: Field` bound in the workspace widens the day `Gf2` lands. The tower has had
this failure once already: a blanket over `Float` widened to `Num` and silently admitted integers to
`Field`.

The obvious guard is a finite-versus-infinite split, and it is the wrong one. What the exposed code
depends on is that `n · 1 ≠ 0`, which is characteristic zero. Finiteness neither implies it nor
follows from it: 𝔽₃ is finite and halves perfectly well, 𝔽₄ is finite and does not, and the rational
function field 𝔽ₚ(x) is infinite and does not.

Measured rather than assumed, and measured twice. Sixteen sites in the workspace compute
`T::one() + T::one()` as two. **Three sit under a `Field` bound**, all in
`deep_causality_multivector` — `commutator_geometric` at `types/multifield/algebra/mod.rs:163`, whose
`let half = T::one() / (T::one() + T::one());` would be a division by zero over 𝔽₂;
`types/multifield/ops/conversions.rs:139`; and `types/multivector/ops/ops_product_impl.rs:316`. Those
three are the migration and they are the whole of it.

The other thirteen are excluded by a bound 𝔽₂ cannot reach: `RealField` (nine), `ConjugateScalar`
(two) and `Real` (three). The count matters less than how it was reached — a file-level reading
over-counted, because a file containing a `Field` bound somewhere is not a file whose halving site is
under it. Each exclusion is a compile probe against the bound itself.

So the tower gains `DivisibleByIntegers` and `FiniteField` as separate refinements of `Field`. They
are disjoint by definition — every finite field has prime characteristic — but they do not partition
the fields, and the documentation says so: 𝔽ₚ(x) is in neither. Stating the cut as
finite-against-infinite would claim a partition that does not exist while guarding the wrong
property.

### Every container implements the tower, not only the HKT surface

An HKT witness makes a container composable through `deep_causality_haft`. It does not make it
composable through the tower: a function bounded on `Ring` or `Module` cannot take a container that
never declares them.

The crate inherits an unfinished instance. `CsrMatrix<f64>` reaches `AbelianGroup` and stops —
compile-probed — because `Distributive` and `Annihilating` are not implemented for it. Everything
else `Ring` needs is present: `One`, `Mul`, and `Associative<Multiplicative>`. A matrix ring over a
ring is a ring, and the tower is two marker impls away from saying so. Because `Ring` is missing,
`Module<S>` is unreachable, so the scalar multiplication `arithmetic/mod.rs:283` already implements
is invisible to the tower as well.

The move finishes this rather than carrying it across, and the dense matrix, the vector and the
packed matrix are built with it from the start. The vector in particular is a `Module<R>` over its
scalar ring — the tower's name for a vector space, and the general notion that admits ℤ where
`Field` would not.

### Integer admission is a sweep, not a feature

Bounding each operation at its lowest correct level is what yields integer support; 
. The sweep is therefore a review of every bound, and the code
being moved shows what it will find: `mat_mult_impl` takes
`T: Copy + Clone + Mul<Output = T> + Zero + PartialEq + Default`, which is a semiring spelled out
longhand, stating no algebraic claim and leaving a reader unable to tell whether ℕ is admitted
deliberately.

Rewriting those as tower traits says what was already meant. 
The genuine loosenings are the operations that never divide — addition, subtraction,
negation, scaling, matrix multiplication, matrix–vector, transpose, trace, dot product, and the
determinant, which is a polynomial in the entries. Each one loosened names the number set it newly
admits and is instantiated at it, because a loosened bound nothing exercises is indistinguishable
from an untested one.

### 𝔽₂ linear algebra is owned by `deep_causality_linear`, superseding G-01's owner field

G-01 assigns 𝔽₂ to `deep_causality_topology` "because that is where chain complexes live and
topology must not learn about codes." The same reasoning places it better in a linear-algebra crate,
which knows about neither chain complexes nor codes. Topology then consumes it, and
`deep_causality_quantum` consumes it without depending on topology.

The ownership splits cleanly along the scalar boundary. `Gf2` — the two-element field itself — is a
number and lives in `deep_causality_num` with the other numbers. The elimination, the rank, the
kernel and image bases and the packed representation are linear algebra and live here.

### The retired crate re-exports rather than freezes

`deep_causality_sparse` publishes one final version whose `src/lib.rs` re-exports
`deep_causality_linear` and whose README carries a retirement notice. It stays in the workspace and
on crates.io for a few months. Nothing is yanked.

Freezing the implementation instead would make `deep_causality_sparse::CsrMatrix` and
`deep_causality_linear::CsrMatrix` distinct types. Any crate depending on both — which is exactly
what a partially-migrated dependent looks like — would fail to typecheck. Re-exporting keeps one
type, so a dependent can migrate module by module.

### Archived openspec changes are not rewritten

34 of the 203 files naming `deep_causality_sparse` are under `openspec/changes/archive/`. They
record what was proposed at the time. Rewriting them would falsify the record.

## Risks / Trade-offs

**Consolidating topology's determinants onto an unpivoted elimination returns zero volumes.**
Researched rather than assumed. Both Laplace determinants are fed Cayley-Menger matrices, which have
`m[0][0] = 0` by construction; `gaussian_determinant` at `lazy_hodge_star.rs:97` takes `mat[i][i]` as
its pivot and returns zero when it is small. Measured on a regular unit tetrahedron: Laplace gives
`det = 4.0` → `vol = 0.117851130198` (exactly √2⁄12), the unpivoted elimination gives `det = 0.0` →
NaN, and elimination **with partial pivoting** reproduces Laplace exactly. *Mitigation:* the shared
determinant pivots by search, which `linear-dense-algorithms` now requires with those scenarios
attached. With pivoting there is no numerical change on the shapes topology uses. Run the topology
suite before and after and diff regardless.

**Changing `betti_number` from f64 SVD to exact 𝔽₂ rank changes answers.** That is the point — G-02
records that the current answer can be wrong — but it changes committed test expectations wherever
the two ranks differ. *Mitigation:* the two agree for the toric code and for every complex currently
under test, per G-02. Keep the ℝ-rank path available for complexes that are not being read as codes,
and make the choice explicit at the call site rather than global.

**Deduplication alone does not justify a published crate.** Topology's five helpers are all inside
`deep_causality_topology`, and `deep_causality_tensor` has its own internal copies — six `matmul`
definitions, two SVD paths (`svd_impl` power-iteration and `jacobi_svd`), two Choleskys
(`cholesky_decomposition_impl` and `cholesky_in_place`). Every one of those is fixable with a
`pub(crate) fn` and no crate boundary. The crate rests on holding the representations side by side
and on giving 𝔽₂ a home that is neither a chain-complex crate nor a tensor crate; the duplication is
what gets tidied on the way, not the reason. *Mitigation:* none needed — but the Why section should
not be read as claiming otherwise, and the last workspace split cost 828 files and 7,537 insertions,
so the bar is real.

**A new crate boundary can trap coherence.** The `deep_causality_num` split produced E0119 when
marker traits over a now-foreign `Float` could not admit integers. *Mitigation:* the traits this
change introduces are defined in `deep_causality_linear` and implemented by crates above it, which
is the direction the orphan rule permits; the prototype compiles both the permitted and the
forbidden direction and shows which fails.

**Delegation adds a call layer to the most-used numerical code in the workspace.** `matmul` alone has
13 call sites in the physics Kalman filter and 18 across that crate. *Mitigation:* the delegating methods are generic and
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

## Resolved

- **The retirement window never ends in a yank.** `deep_causality_sparse` stays published
  indefinitely; the window governs how long it receives a re-export, not how long it exists.
- **All 1,088 relocated lines move in this change.** Nothing stays in `deep_causality_tensor` but the
  delegating method shells, and none is deferred to a follow-up.
- **Topology's determinants do not change numerically**, provided the shared implementation pivots.
  See Risks.
- **A dense matrix type has 46 real call sites**, concentrated in three crates that call no N-d
  operation at all. `DensityMatrix` carries `dim: usize` beside its tensor precisely because
  `CausalTensor` cannot express squareness.
- **Physics' five small-matrix helpers do not consolidate here.** One genuine pair exists
  (`invert_3x3` and `inverse_spatial_metric`, same math, thresholds 100× apart), and merging it is a
  `pub(crate) fn` inside `deep_causality_physics` needing no crate boundary. Recorded as task 8.1.

## Open Questions

1. **Does the dense matrix type replace rank-2 `CausalTensor` at any call site?** Phase 4 makes both
   viable. Physics and quantum use rank-2 tensors heavily and switching them is a separate decision
   with its own blast radius.
2. **Does `Matrix3` fold in later?** It has two consumers and lives below this crate. Folding it in
   would need `deep_causality_num` to depend on `deep_causality_linear`, which inverts the tower.
3. **How is the 𝔽₂ rank chosen at the call site in topology?** A parameter on `betti_number`, a
   separate method, or a property of the complex. Phase 5 decides; the risk section requires only
   that it not be a silent global switch.
