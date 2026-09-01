<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# QCL: the gap register

**What this is.** Every gap found while assessing whether QCL can be built as designed, with the
evidence for each, a proposed closure, and the crate that owns it. Written so a spec can be derived
from it directly.

**How the gaps were found.** Four sources: reading `deep_causality_quantum/src` in full (3980
lines), reading `LEAN_QUANTUM.md`, reading Haruna (arXiv:2511.15224) end to end, and six parallel
feasibility assessments of which two ran the shipped API in scratch crates rather than reading it.

**Status of the assessment.** Two of eight feasibility lenses did not complete. G-15 and G-16 are
therefore marked unresolved rather than confirmed. Everything else carries evidence.

---

## 0. Status

Verified against the tree on **2026-08-31**. **All eighteen gaps are closed.**

`add-linear-algebra-crate` closed G-01 and G-02. The category-A sweep then closed G-03, G-10, G-11,
G-12, G-15 and G-17: everything that was unblocked, self-contained and small. `Gf2Chain<W>` closed
G-05, which this register had not recorded until today. G-06 closed on 2026-08-31: the gate alphabet
now covers Table 1, and the coincident-index defect it uncovered is fixed.

**`deep_causality_homology` now exists, and it is where G-04 and G-08 land.**
The `extract-homology-crate` change moved the chain-complex layer out of `deep_causality_topology`
into its own crate at tier 4: `ChainComplex` over boundary matrices alone, `HomologyField`, and
`Gf2Chain<W>`. The geometric half stayed behind on `CellularComplex: ChainComplex`, and topology
re-exports all three moved names, so nothing downstream changed except two `use` lines in
`deep_causality_cfd`.

What this changes for this register is the *dependency edge*. G-07 and G-09 need
`deep_causality_quantum` to reach homology, and until now that meant depending on
`deep_causality_topology`, 27,102 lines of geometry with a Hodge star and a metric, to use the 927
lines of chain-complex machinery. The edge now goes to `deep_causality_homology`, whose *direct*
dependencies are `deep_causality_linear` and `deep_causality_num`. Read that as direct and not
transitive: `deep_causality_linear` pulls `deep_causality_algebra` and `deep_causality_haft` in
behind it, so the resolved graph is four crates. None of them is geometry, which is the point. A
CSS code is a chain complex with no cells; it can now be typed as one.

**`Gf2Chain<W>` closed G-05, and this register missed it.** The type landed on 2026-08-26 in
`fb295f9dd`, one day after the verification stamp this file carried until today, and moved into
`deep_causality_homology` with the extraction. It supplies everything G-05 asked for. The evidence
is at G-05. Two consequences: §4's head node was `G-05 + G-18`, and only the G-05 half closed, so
**G-18 is still at the head**; and G-04, whose one stated blocker was G-05's return type, is
unblocked.

Two further things came with the move and are worth recording here, because both were assumptions
this register's Betti-number work rested on:

- **`∂ₖ ∘ ∂ₖ₊₁ = 0` is now stated and proved.** It was the unproved hypothesis of
  `linear.gf2.betti_from_ranks`. `homology.chain.dd_zero_implies_range_le_ker` discharges it, and
  the conformance harness asserts it at every grade of every shipped complex. See
  `deep_causality_unified_math/deep_causality_homology/LEAN_HOMOLOGY.md`.
- **The degenerate grades carry a shape.** `∂₀` is `(0, n₀)` and `∂_{max+1}` is `(n_max, 0)`, in
  place of the empty matrix all three implementors returned. `betti_number_over` survived that on
  `saturating_sub`; the kernel basis G-04 needs would not have.

**Publication is not a gate on anything here.** `deep_causality_linear` 0.1.0 is on crates.io, and
the whole workspace has since been patch-bumped and republished. It makes no difference to this
register: every closure consumes its crate through a workspace path dependency, so what unblocked
G-03 and G-04 was the crate existing in the tree. Publication gates consumers outside this
workspace and gates no gap in this file.

**What the sweep found that the register had wrong.** Four things, each recorded at its own gap.
G-17's third clause asked to gate `predict`, which does not exist. G-15 asked what `fork` produces,
and neither `fork` nor `adjudicate` exists in QCL — the only `fork` in the workspace is
`deep_causality_cfd`'s, unrelated. G-10 said "Stirling numbers" without saying which, and A.12 and
A.14 need both kinds, the first of them unsigned. G-18's cited signature is stale: `cup_product`
takes two cochains and two degrees at `cup_product/mod.rs:95`, not one cochain at line 62.

**A fifth, at G-06, changes its size.** The register says "the work is in the gate enum". `GateOp`
has an execution path — `sim.rs:162-223` dispatches every variant into a state-vector simulator — so
each new gate needs a simulator arm too, and `C^{m-1}Z` needs a generic multi-control apply rather
than a copy of `apply_cz`.

**One open, verified today.** `homology_representatives` and `dual_representative` exist nowhere
in the workspace, and a sweep of every `fn` name containing homolog, representat, cycle, cocycle,
generat, basis or dual returns nothing that yields a homology class. There is no `Cochain`, and an
active spec forbids adding one (see G-18). `logical_z` is still typed on `CausalMultiVector`
and `deep_causality_quantum` now depends on `deep_causality_homology`, which G-07 added. `Gf2Chain`
has its first first-party caller: the Haruna gate layer. The 𝔽₂ stack is wired end to end for
counting Betti numbers and for emitting logical gates, and nowhere wired for representatives.

### An open question about the layering, raised 2026-08-25

`deep_causality_topology` imports from `deep_causality_linear` in **36 files**, and the dominant
import is `CsrMatrix` — the representation of the boundary operators. That coupling has a name in
mathematics, and the register has been treating its symptoms without naming it.

The bridge between topology and linear algebra is a functor composition:

```
Top  ──C_•──▶  Ch(R)  ──H_n──▶  R-Mod
```

A chain complex is a graded family of `R`-modules with differentials satisfying `∂∘∂ = 0`. Chain
complexes over a ring form a category, and homology is a functor out of it. The categorical reason
the same construction serves vector spaces, modules and complexes alike is that all three are
**abelian categories**, which is where kernel-modulo-image is definable.

Two consequences for this register.

**G-02's finding is the universal coefficient theorem.** "Rank over ℝ is not rank over 𝔽₂" is the
statement that `0 → H_n(C) ⊗ F → H_n(C; F) → Tor(H_{n-1}(C), F) → 0` has a vanishing Tor term over a
field, so the two coefficient choices see different parts of the integral homology's torsion.
`HomologyField::{Rational, Gf2}` is a coefficient-change functor with the theorem left implicit.

**The middle layer now exists as a crate.** `ChainComplex`, `HomologyField` and `betti_number_over`
live in `deep_causality_homology`; `deep_causality_topology` keeps `pub use` re-export shims for all
three (`traits/chain_complex.rs:38`, `types/homology_field/mod.rs:25`, `types/gf2_chain/mod.rs:27`),
so no consumer moved. Under the layering above they are homological algebra, and so are G-04's
representatives, G-08's duality, and `Gf2Chain`, which landed in homology rather than in topology as
this register once asked. The earlier suggestion of a `deep_causality_packed` crate splits on the
wrong axis, because bit-packing is a representation choice inside 𝔽₂ linear algebra rather than a
branch of the mathematics.

### What the numeric-tower work changed

The algebra tower was completed after this register was first written — ℕ and ℚ added, ℤ admitted,
three correctness bugs fixed. It closed no gap here, and the branch touched
`deep_causality_quantum` and `deep_causality_topology` in one line each, the
`deep_causality_algebra` requirement moving from `0.2` to `0.3`. The tower work was orthogonal to
this register. Three of its consequences still bear on how a spec is written, and they follow.

**G-01's aside is now stale.** It reads *"`Integer` … is not wired into the algebra tower. Wiring it
in is a reasonable thing to do for its own sake, but it is not what closes this gap."* The first half
no longer holds — the signed integers reach `CommutativeRing` and `EuclideanDomain`, the unsigned
reach `CommutativeSemiring`. **The second half holds exactly as written**, and the register was right
to say so: 𝔽₂ as a tower scalar would still store one bit per element and still lose the
word-parallel XOR. The closure is unchanged.

**The 𝔽₂ layer now has a generic home.** It was never blocked — `Integer` has supplied `count_ones`,
`BitXor` and the rest all along, so the "roughly 200 lines" was always writable. What is new is that
it no longer has to be written against a concrete `u64`. `NaturalNumber`
(`deep_causality_unified_math/deep_causality_num/src/integer/natural.rs`) is blanket-implemented for every unsigned width and
carries `gcd`, `lcm`, `monus`, `div_rem`, `succ`/`pred` on top of `UnsignedInt`'s bit surface, and
`deep_causality_topology` already depends on `deep_causality_num`. A `Gf2Matrix` generic over
`W: NaturalNumber` gets its word width as a parameter rather than a hard-coded assumption.

**Law markers are no longer handed out by inference, which changes the shape of the rejected
alternative.** `Commutative`, `Associative` and `Distributive` were blanket-implemented over `Num`;
they are now written out per type, and `Ring` additionally requires a new `Annihilating` marker.
So the `Matrix<F2>`-over-a-`Field`-impl route this gap rejects is now also more work than it was: a
`Gf2` scalar would have to state every marker explicitly rather than acquiring them from `Num`.
That reinforces the recommendation rather than changing it — but a spec that reaches for the tower
route should know the cost has gone up, not down.

`EuclideanDomain` also gained `normalize`, `checked_normalize` and `checked_gcd`. None of them bear
on 𝔽₂ work, where the only arithmetic is XOR and popcount; they are noted so a reader diffing the
trait against this register is not surprised.

**G-05 had a name collision, and it is resolved.** `deep_causality_topology` already exports a
`Chain<R, G>` (`src/types/chain/mod.rs:32`), a *weighted* chain holding an
`Arc<SimplicialComplex<R>>`, a `grade`, and a `CsrMatrix<G>` of weights. G-05 asked for a different
object: a bit-packed chain with a support, an inner product and an intersection. Two distinct
objects, one name. The bit-packed one shipped as `Gf2Chain<W>` in `deep_causality_homology`, and the
weighted `Chain` kept its name.

---

## 1. The requirement this register is measured against

Haruna's construction needs nine things and nothing more. The full paper was read to establish this;
the earlier belief that a gauge-field carrier was needed does not survive it (see C-3).

| # | Requirement | Paper |
|---|---|---|
| R1 | Chain complex `C₂ --∂₂=H_Zᵀ--> C₁ --∂₁=H_X--> C₀` **over 𝔽₂**, `∂₁∂₂ = 0` | 2.5, 2.6 |
| R2 | A 1-chain `γ ∈ C₁` as a bit vector, `γⁱ ∈ {0,1}` | 2.14 |
| R3 | `supp(γ)`, and enumeration of its pairs and triples | 3.17, 3.51, 3.59 |
| R4 | **𝔽₂ homology with representatives**: `H₁ = ker ∂₁/im ∂₂`, `H¹ = ker ∂₂ᵀ/im ∂₁ᵀ` | 2.9 |
| R5 | `⟨γ₁,γ₂⟩ = Σγ₁ⁱγ₂ⁱ` and `γ₁∩γ₂`, mod 2 | after 2.15 |
| R6 | Poincaré-dual representative: given `γ`, find `γ̃` with `⟨γ,γ̃⟩ = 1` | 2.22, 2.23 |
| R7 | A physical-gate circuit over `S, CZ, H, T, CS†, CCZ, C^{m-1}Z` | Table 1 |
| R8 | Logical equivalence `O₁ ~ O₂ ⟺ O₁O₂⁻¹ ~ I`, decided by commuting with logical Paulis | A.1, B.1–B.3 |
| R9 | Integer identities on `a(γ)^m` via Stirling numbers, only if the compact form is wanted | A.12–A.14 |

**Not required:** multivectors, metric signatures, gradings, Clifford algebras, matrix exponentials,
Taylor series, `2ⁿ × 2ⁿ` matrices, or a gauge-field type. Every `zᵢ` is diagonal, so `a(γ)` is
diagonal with integer eigenvalues, and every gate reduces to combinatorics on a bitset. The
gauge-field expressions in Table 1 are the compact forms that make the Appendix B invariance proofs
tractable; they are not the computational path.

---

## 2. Gaps

Severity: **S1** produces a wrong answer with no error raised. **S2** blocks a designed consumer.
**S3** is ergonomics or documentation.

### G-01 — No 𝔽₂ linear algebra anywhere in the workspace — **CLOSED**

**Severity S1.** Blocks R1, R4, R6. **Closed** by `add-linear-algebra-crate`.

Searched `deep_causality_topology`, `deep_causality_sparse` and `deep_causality_algebra` for GF(2),
mod-2 elimination or binary-field arithmetic. There is none. The only `mod 2` reference is a comment
in `lattice_complex/cell_splitting.rs:48` about the cup-product sign rule.

**Closure.** A bit-packed 𝔽₂ matrix with Gaussian elimination returning rank, kernel basis and image
basis. Roughly 200 lines over `u64` words using XOR and `count_ones`.

**Owner (as closed):** `deep_causality_linear`, not `deep_causality_topology` as proposed.

The placement argument here was that topology must not learn about codes. It holds, and the crate
that was created satisfies it without owning chain complexes at all: `PackedGf2<W>` is a bit-packed
matrix over a `NaturalNumber` word, and `rank_gf2` / `kernel_basis_gf2` / `image_basis_gf2` are
elimination over it. Neither knows what a complex is. `linear-f2-algebra` restates the placement as
a **separability** property for that reason — the requirement is that the matrix be usable without
chain complexes, which is a stronger and more checkable claim than which crate the file sits in.

~~Moving it removed no dependency edge either way: `qcl-gaps` records G-07 and G-09 as needing G-04
and G-05, both owned by `deep_causality_topology`, so quantum takes a topology dependency for the
𝔽₂ work regardless.~~ **Superseded.** G-04, G-05 and G-08 are owned by `deep_causality_homology`
now, and G-05's type shipped there. Quantum takes a *homology* dependency for the 𝔽₂ work, which is
four crates of arithmetic rather than a geometry stack. Every remaining "topology dep" in this
file means `deep_causality_homology`.

**What was built.** `deep_causality_unified_math/deep_causality_linear/src/types/packed_gf2/` — the representation, over
`W: NaturalNumber` rather than a fixed `u64`, which is what the aside below asked for.
`deep_causality_unified_math/deep_causality_linear/src/algorithms/gf2.rs` — rank, kernel basis, image basis. `Gf2` itself is a
`deep_causality_num` scalar reaching `Field` through the tower's blanket, and is confirmed by
compile probe not to reach `RealField`, `NormedScalar` or `ConjugateScalar`.

**On the algebra tower.** `deep_causality_num` has an `Integer` trait supplying exactly the needed
primitives (`count_ones`, `trailing_zeros`, `checked_*`, `wrapping_*`,
`deep_causality_unified_math/deep_causality_num/src/integer/mod.rs:41`). 𝔽₂ as a tower scalar would store one bit per element and
lose the word-parallel XOR that makes mod-2 elimination fast; the right shape is packed bitsets using
`Integer`'s bit operations, not `Matrix<F2>` over a new `Field` impl. **This still holds after the
tower work** — see §0.

~~and it is not wired into the algebra tower~~ — superseded. The integer types are now in the tower
(`CommutativeSemiring` for unsigned, `EuclideanDomain` for signed), which does not close this gap but
does mean the bit-packed layer can be written generically: bound the word type on `NaturalNumber`
(`src/integer/natural.rs`) rather than fixing it to `u64`.

### G-02 — Homology rank is computed by `f64` SVD, not over 𝔽₂ — **CLOSED**

**Severity S1.** **Closed** by `add-linear-algebra-crate`.

```rust
// deep_causality_unified_math/deep_causality_topology/src/types/simplicial_complex/topology/chain_complex_impl.rs:94
fn rank_of_csr(matrix: &CsrMatrix<i8>) -> usize {
    // build an f64 dense tensor, then
    let (_, s, _) = tensor.svd().expect("SVD failed");
    s_vec.iter().filter(|&x| x.abs() > 1e-5).count()
}
```

Used at `chain_complex_impl.rs:85-86` and mirrored by `CellComplex::rank_of_matrix`
(`cell_complex/mod.rs:158-160`). **Both citations are historical.** Neither `rank_of_csr` nor
`rank_of_matrix` exists anywhere in the workspace today; the quoted code is the pre-closure state,
kept because it is the evidence for the gap. `cell_complex/mod.rs` no longer contains the string
`rank` at all.

Rank over ℝ is not rank over 𝔽₂. The two agree for the toric code, which is why the geometric-QEC
example's `[[32,2,4]]` comes out right, but that is a property of that code family. A qLDPC code with
even-weight dependencies has a smaller 𝔽₂ rank, so the reported `k` would be wrong and no error
would be raised.

**Closure as built.** Neither helper survives. `HomologyField`
(`deep_causality_unified_math/deep_causality_topology/src/types/homology_field/mod.rs`) is an enum with one method,
`rank_of(&CsrMatrix<i8>)`, and `ChainComplex::betti_number_over(k, field)` is the one body both
`SimplicialComplex` and `CellComplex` now inherit — each had an identical override, and both are
gone along with their rank helpers.

The register proposed keeping "the real-valued path". It was not kept, and the reason is that the
real-valued path was never needed: rank over ℚ **is** rank over ℝ for an integer matrix, so the
characteristic-zero case is served exactly by fraction-free integer elimination
(`deep_causality_linear::rank_exact`) with no tolerance and no float. `HomologyField::Rational` is
that path; `HomologyField::Gf2` is the mod-2 one. A manifold discretisation asks for the former and
gets the same number it got before — exactly, rather than to `1e-5`.

`betti_number(k)` remains, defined as `betti_number_over(k, HomologyField::Rational)`. It is an
alias rather than a default: there is no setting, feature or global that changes which field a call
lands in.

**Measured, not assumed.** The register predicted the two ranks agree for every complex currently
under test. Confirmed by running the whole topology suite with `betti_number` bound to
`HomologyField::Gf2`: 1471 tests, none failed. Injecting a wrong rank fails 7, so the suite does
discriminate. Both were run rather than reasoned about.

**Owner (as closed):** `deep_causality_topology` for `HomologyField` and the trait method;
`deep_causality_linear` for the two ranks underneath. **Note:** this was a pre-existing correctness
risk independent of QCL.

### G-03 — `LatticeComplex::betti_number` never reads the boundary matrices — **CLOSED**

**Severity S2.** **Closed** as a checked fast path.

```rust
// deep_causality_unified_math/deep_causality_topology/src/types/lattice_complex/mod.rs:570
fn betti_number(&self, k: usize) -> usize {
    let all_periodic = self.periodic.iter().all(|&p| p);
    if all_periodic { /* returns the binomial C(D, k) */ }
```

For a fully periodic lattice it returns a closed form. It is a lookup for the torus, not a homology
computation, and it cannot produce representatives even in principle.

**Consequence for tests:** any test asserting Betti `[1,4,6,4,1]` on `T⁴` currently tests the
binomial formula, not the complex.

**Closure.** Compute from the boundary matrices via G-01. Keep the closed form as a fast path only if
a test asserts the two agree.

**Closure as built: the closed form stays, and is now checked.** The decision was made by running
the comparison rather than by argument. `LatticeComplex` implements `ChainComplex`, so it already
inherited `betti_number_over(k, field)`, which reads the boundary matrices. Measured across `T²` at
two extents, `T³`, a cylinder, `T² × I`, a 2D disk and a 3D block, at every grade, over both ℚ and
𝔽₂: the binomial agrees everywhere. So it is kept, and
`tests/types/lattice_complex/betti_agreement_tests.rs` asserts the agreement rather than assuming
it. Injecting a wrong binomial fails the suite, so the check discriminates.

The partially periodic arm is covered too, which the register did not ask for. A lattice with `p`
periodic dimensions returns `C(p, k)`, and that arm had no test at all.

**Owner:** `deep_causality_topology`.

### G-04 — No homology representatives — **CLOSED**

**Severity S2.** Blocks R4, and therefore R6 and the whole gate layer.

`ChainComplex` exposes six methods and none of them returns a basis:
`num_cells`, `max_dim`, `boundary_matrix`, `coboundary_matrix`, `betti_number_over` and
`betti_number`, at lines 55, 58, 65, 68, 97 and 119 of
`deep_causality_unified_math/deep_causality_homology/src/traits/chain_complex.rs`. Cite that path in
full: `deep_causality_topology`'s file of the same name is a 38-line re-export shim, and the old
line numbers land inside its doc comment. `betti_number_over` is rank
arithmetic: no kernel is built and no quotient is formed. A count, not a basis. A sweep of every
`fn` name in the workspace containing homolog, representat, cycle, cocycle, generat, basis or dual
returns nothing that yields a homology class.

The paper needs actual `γ ∈ ker ∂₁ \ im ∂₂` to build any logical gate.

**Closure.** `fn homology_representatives(&self, k: usize) -> Vec<Gf2Chain<W>>` off the G-01 kernel
and image bases.

**State on 2026-08-31: unblocked.** G-04's one stated blocker was G-05's return type, and G-05
closed. The pieces all ship: `kernel_basis_gf2` and `image_basis_gf2`
(`deep_causality_unified_math/deep_causality_linear/src/algorithms/gf2.rs:35,67`),
`csr_to_packed_gf2_mod2` (`extensions/conversions.rs:136`) to convert a `CsrMatrix<i8>` boundary
matrix into what they take, and `Gf2Chain` to carry the result.

**Take the generators off columns, not rows.** `kernel_basis_gf2` allocates
`PackedGf2::zeros(cols, free.len())` and `image_basis_gf2` allocates `zeros(rows, pivots.len())`.
Both write basis vectors **down columns**, so a generator becomes a chain through
`Gf2Chain::from_column` (`deep_causality_homology/src/types/gf2_chain/mod.rs:110`). Reading the
same basis with `from_row` returns a
vector whose length is the *number of generators* rather than the dimension they live in, which is a
silent dimension error rather than a failure. `deep_causality_linear` carries a regression test for
exactly this, `test_reading_a_kernel_basis_as_rows_gives_the_wrong_length`
(`tests/types/packed_gf2_vector/packed_gf2_vector_tests.rs:483`).

**The quotient step itself has no primitive.** Extracting `ker ∂ₖ / im ∂ₖ₊₁` needs `[im | ker]`
stacked before an rref or a rank sweep, and `deep_causality_linear` exports no column-concatenation
helper: a search for `hstack`, `concat_cols`, `augment` and `extend_basis` finds nothing. The
stacking must be built from `MatrixBuild::zeros` plus `set`. This is the work; the bases underneath
it are done.

**Closure as built.** `ChainComplex::homology_representatives(k) -> Vec<Gf2Chain<W>>`, with
`cohomology_representatives(k)` as its dual. Both go through one `quotient_basis` helper, which
takes the kernel basis of the numerator's operator and the image basis of the denominator's,
column-stacks them as `[B | Z]` with the image first, reduces, and keeps the kernel columns holding
a pivot. A pivot at or beyond `B`'s width is a kernel vector the image and the earlier kernel
columns together do not reproduce. The count that falls out is `rank[B | Z] − rank B`, which is
`dim ker ∂ₖ − rank ∂ₖ₊₁`, which is `β_k`. The `[im | ker]` stacking was built from
`MatrixBuild::zeros` plus `set`, as this register predicted, because `deep_causality_linear` has no
column-concatenation primitive.

**Generators come off columns, and that is load-bearing.** `kernel_basis_gf2` allocates
`zeros(cols, free.len())` and `image_basis_gf2` allocates `zeros(rows, pivots.len())`, so both write
down columns and `Gf2Chain::from_column` is the reader. `from_row` would return a vector whose
length is the number of generators rather than the dimension they live in.

**No field parameter, unlike `betti_number_over`.** A representative is a vector, and a vector over
ℚ is not a vector over 𝔽₂. The bit-packed path is the one with a chain type to return, so this
method names its field in its return type. Where both fields yield the same *kind* of answer, as
they do for a dimension, the choice belongs in a parameter; where they do not, it belongs in the
signature.

**Tested by invariant, because a quotient basis is not canonical.** Which vectors come back depends
on the elimination order, so there is nothing to diff against a stored answer, and the register's
usual demand for an external oracle is met differently: the *count* is `β_k` over 𝔽₂, which the ten
fixtures carry from Hatcher. Three properties are asserted at every grade of all ten: the count
equals `β_k` and agrees with the rank-nullity path that builds no kernel; every representative is a
cycle; and adjoining the representatives to a basis of `im ∂ₖ₊₁` raises the rank by exactly their
number, which is independence modulo boundaries. The sharp case is `S²`, where `ker ∂₁` is
non-trivial and `H₁` is not, so the basis must come back empty.

Four mutations, four failures: an off-by-one in the pivot cut fails 3 tests, ignoring the image
entirely fails 5, placing the kernel columns before the image fails 2, and accepting a zero pairing
in `dual_representative` fails 2.

**Owner:** `deep_causality_homology`, not `deep_causality_topology` as first proposed. The trait it
extends and the type it returns both live there.

### G-05 — No `Chain` type — **CLOSED**

**Severity S2.** Unblocks R2, R3, R5. **Closed** by `Gf2Chain<W>`.

There was no 1-chain type at all: nothing carried a degree with its data, and nothing supplied the
support, the inner product or the intersection.

**Closure.** A bit-packed `Chain { bits, degree }` with `supp()`, `inner(&self, &Chain) -> u8`,
`intersect(&self, &Chain) -> Chain`, and pair and triple iterators over the support.

**Closure as built.** `Gf2Chain<W>`
(`deep_causality_unified_math/deep_causality_homology/src/types/gf2_chain/mod.rs:51`) is
`{ bits: PackedGf2Vector<W>, degree: usize }` and carries every member the closure named:
`support` (`:155`), `support_pairs` (`:160`), `support_triples` (`:165`), `intersect` (`:188`) and
`inner` (`:204`), with `add` (`:175`), `weight` (`:150`), `zeros`, `from_support`, `from_row` and
`from_column` besides. Nine tests in `tests/types/gf2_chain/`.

**Four deviations from the closure as written, all deliberate.** The signatures do not match what
this gap specified, and an implementor reading the old text would get them wrong:

- The method is `support()`, not `supp()`.
- `inner` returns `Result<Gf2, HomologyError>`, not `u8`. `Gf2` is a `bool` newtype
  (`deep_causality_num/src/gf2/mod.rs:53`).
- `intersect` returns `Result<Self, HomologyError>`. Every binary operation is fallible behind
  `same_group` (`:219`), which compares the pair `(degree, len)` and raises `ChainGroupMismatch`.
  This gap did not ask for that guard. It is what makes the topology re-export a breaking change,
  `TopologyError` to `HomologyError`, released as topology 0.8.0.
- `support_pairs` and `support_triples` return `Vec::into_iter`, not lazy iterators. Both collect
  the support and materialize the full tuple list first, so a weight-`w` chain allocates `C(w,2)`
  and `C(w,3)` up front. Correct, and the wrong shape for a large support.

**The bit arithmetic is not in this type.** `Gf2Chain` delegates all five operations to
`PackedGf2Vector<W>` in `deep_causality_linear`
(`src/types/packed_gf2_vector/mod.rs:240,271,287,299,313`). What homology adds is the degree and the
chain-group guard. The 𝔽₂ word arithmetic is tested where it lives, in 17 further tests.

**Name collision, as resolved.** The crate already exported a `Chain<R, G>`
(`src/types/chain/mod.rs:32`): a *weighted* chain carrying an `Arc<SimplicialComplex<R>>`, a
`grade`, and a `CsrMatrix<G>` of weights. A different object. The name went to `Gf2Chain`, with the
reasoning recorded at `deep_causality_homology/src/types/gf2_chain/mod.rs:19`, and the weighted
`Chain` kept its name.

**Owner (as closed):** `deep_causality_homology`, not `deep_causality_topology` as proposed.
Topology re-exports the name (`src/types/gf2_chain/mod.rs:27`), so the proposed path still resolves.

**What this does not close.** §4 booked this gap as `G-05 + G-18` on one line, and only this half
shipped. `Gf2Chain` is 𝔽₂-only, so it cannot serve the ring-generic cup product G-18 needs, and
G-18's instruction to "fold into G-05" is void: the two need different types. G-18 stays open and
stays at the head. R2, R3 and R5 are *available* now rather than *exercised*: `Gf2Chain` has no
first-party caller outside its own tests and the topology shim.

### G-06 — No circuit type — **CLOSED**

**Severity S2.** Blocks R7.

The gate kernels exist (`gates_haruna.rs`, `logical_z/x/s/hadamard/cz/t`) but there is no
representation of a physical-gate sequence, which is what every Table 1 decomposition produces.

**Closure.** `Circuit` as an ordered list of gate applications over named qubit indices, covering
`S, CZ, H, T, CS†, CCZ, C^{m-1}Z`.

**The shortfall, as it stood.** `GateOp` carried `H, X, Y, Z, S, T, Cnot, Cz`: four of Table 1's
seven, plus four it does not ask for. Missing were `CS†`, `CCZ` and `C^{m-1}Z` for general `m`,
and no adjoint existed for any gate. `QuantumCircuit` itself was a register width plus an ordered
program and needed no change, which held.

**Closure as built.** Five variants added: `Sdg`, `Tdg`, `Csdg`, `Ccz { q0, q1, q2 }` and
`Cmz { qubits: Vec<usize> }`. `Cmz` is the general `C^{m-1}Z` over a symmetric list; `Cz` and `Ccz`
are its two- and three-qubit cases, kept as fixed-arity variants because they carry no allocation
and clone without one. `qubits()` moved from `match *self` to `match self`, which the `Vec` field
forces.

**One kernel replaced three, and the register had this backwards.** The register warned that
`C^{m-1}Z` "needs a generic multi-control apply rather than a copy of `apply_cz`; there is no
multi-control kernel to vary". True, and the wrong conclusion. Every gate being added is *diagonal
in the computational basis*, so the arity is a runtime value rather than a new kernel per gate:
`apply_diagonal_phase(state, qubits, phase)` folds the qubit list into a mask and multiplies the
amplitude where `i & mask == mask`. It serves `Z`, `S`, `S†`, `T`, `T†`, `CZ`, `CS†`, `CCZ` and
`C^{m-1}Z` alike. `apply_cz` is gone, and `Z`, `S` and `T` gave up their 8-line matrix literals.
The simulator got smaller while gaining five gates.

**The fourth obligation was the correctness one, and it is fixed.**
`QuantumCircuit::new` validated coincident indices with `if qs.len() == 2 && qs[0] == qs[1]`, which
fired for two-qubit gates only. `Ccz{0, 0, 1}` would have been accepted and then acts as `Cz{0, 1}`,
a different gate from the one written, with nothing raised. The check is now a scan over the whole
index list at every arity, plus a rejection of an empty `Cmz`, whose mask would phase the entire
register.

**Tested against published identities, not against this simulator.** A diagonal gate moves no
amplitude, so it is invisible on a basis state; every test conjugates with `H` and lands on a
deterministic basis state fixed by a relation from Nielsen & Chuang: `HZH = X`, `S = T²`, `Z = S²`,
`T⁸ = I`, `(CS†)² = CZ`, and `CNOT = (I ⊗ H)·CZ·(I ⊗ H)` with its Toffoli and `C^{m-1}X` analogues.
18 tests added across `circuit_tests.rs` and `sim_tests.rs`.

**Four mutations, four failures.** Replacing `i & mask == mask` with `i & mask != 0`, the classic
any-control-for-all-controls defect, fails 5 tests. Flipping `S†`'s phase sign fails 2, flipping
`T†`'s fails 1, and reverting the validation fix to its two-qubit form fails 1. The suite
discriminates.

`SimQpu::sample` still caps at 24 qubits, so the one gate whose usefulness scales with register
width is the one the in-process simulator will not exercise far. That is a simulator limit rather
than a gap in the alphabet.

**Owner:** `deep_causality_quantum`.

### G-07 — The Haruna gate layer is typed on the wrong carrier — **CLOSED**

**Severity S1.** This is friction F8 in the design note, and it is the one that produces confident
wrong answers.

```rust
// deep_causality_quantum/src/types/qgates/gates_haruna.rs:137
pub fn logical_z<R>(a_gamma: &CausalMultiVector<Complex<R>>)
    -> Result<CausalMultiVector<Complex<R>>, QuantumError>
```

The paper's `a(γ)` is a diagonal element of `u(2ⁿ)` built from a bitset, and every gate is a product
of physical gates over `supp(γ)`. A `CausalMultiVector` is a graded geometric algebra with a metric;
`exp` on it is a Taylor series. This is why the scratch code produced *"Taylor series did not
converge within 64 terms; the exponent norm is too large."* The complaint about magnitude was a
symptom of the wrong carrier, not a numerical accident.

`deep_causality_multivector` is a genuine universal Clifford algebra and `ℂl_{2n} ≅ M(2ⁿ,ℂ)`, so an
isomorphism exists in principle. It is still the wrong tool: the objects are diagonal, the
pre-configured signatures stop at `Cl_C(10)`, and an `n`-qubit code would carry `2^{2n}` components
to represent `n` bits.

**Closure.** Retype the layer to take a `Chain` and return a `Circuit`. The gate builders become pure
combinatorics per Table 1: `S̄(γ) = Π S_{jₖ} · Π_{k₁<k₂} CZ_{j_{k₁}j_{k₂}}` and so on.

**Closure as built.** `gates_haruna.rs` takes `Gf2Chain<W>` and returns `Vec<GateOp>`. Each builder
is the product Table 1 names, read from the paper rather than from this register:
`Z̄` and `X̄` are transversal (rows 1 and 2); `S̄` is Eq. (3.17), transversal S plus CZ on every pair
of the support; `T̄` is Eq. (3.59), transversal T plus CS† on every pair plus CCZ on every triple;
`CZ̄` is the full Cartesian product of the two supports (§3.3); `H̄` is Eq. (3.27),
`S̄(γ) · ∏H · S̄(γ̃) · ∏H · S̄(γ)`. `logical_multi_cz` is new and covers Table 1's row 6.

**Two things the paper required that this register never recorded.** Both would have shipped as
silent wrong answers.

- **`CZ_{i,i} = Z_i`.** §3.3 defines the coincident case as `exp(iπ z_i z_i) = exp(iπ z_i) = Z_i`.
  When `γ₁` and `γ₂` share a qubit, that factor is a single-qubit Z rather than a CZ. `logical_cz`
  emits it. Without the reduction the gate would be wrong *and* the circuit would be rejected, since
  G-06's validation refuses a gate naming one qubit twice.
- **The `C^{m-1}Z` reduction rule**, from Table 1's caption: repeated controls drop to a
  lower-control gate and a control coinciding with the target drops to Z, so `C³Z_{i,i,j,k} =
  C²Z_{i,j,k}` and `C²Z_{i,i,i} = Z_i`. `logical_multi_cz` deduplicates each product tuple before
  emitting, which is what makes overlapping supports well defined.

**The global phase is returned rather than dropped.** Table 1's Hadamard carries `e^{-iπ/4}`, and a
circuit type has nowhere to put it. `logical_hadamard` returns `(Vec<GateOp>, Complex<R>)`. A global
phase is unobservable under computational-basis measurement and becomes a relative, observable one
the moment the gate is used as a controlled operation, which is what the Appendix B arguments carry.
The causal wrapper drops it and says so.

**The carrier removal cascaded through two further layers.** `mechanics.rs` held six
`haruna_*_gate_kernel` adapters from `CausalMultiVector` to `Operator`, and `wrappers.rs` held six
causal wrappers over those. The kernels are gone, having nothing left to adapt, and the wrappers now
call `gates_haruna` directly and carry `PropagatingEffect<Vec<GateOp>>`. With them went the Taylor
`exp` and its non-convergence error: the paper's gates are combinatorics, so there is no series to
diverge and the *"Taylor series did not converge within 64 terms"* symptom cannot recur.

**The circuit types left the `qpu` feature gate.** `GateOp` and `QuantumCircuit` are plain data with
no dependency of their own, and the always-on gate layer emits them, so gating them would have gated
the logical gates too. `qpu` still gates what it was written for: the `QpuSampler` seam, `SimQpu`,
and the optional `deep_causality_uncertain` edge. Verified with a `--no-default-features
--features no-std` build.

**Breaking, and versioned as such.** Six public functions changed signature and six were removed, so
`deep_causality_quantum` goes 0.1.2 to **0.2.0**.

**Tested against the paper, and mutated.** Every expected program is read off Table 1 or the equation
it summarises, never off this implementation. Four mutations: routing T's pair factor through CZ
instead of CS† fails 1 test; dropping the `CZ_{i,i}` reduction fails 3; conjugating the Hadamard with
`supp(γ)` instead of `supp(γ̃)` fails 3; dropping the `C^{m-1}Z` deduplication fails 2.

**The dependency edge, as built.** It goes to `deep_causality_homology`, in `Cargo.toml`,
`BUILD.bazel` and `tests/BUILD.bazel`. There was no topology edge to remove: this crate never had
one. Before the change `deep_causality_quantum/Cargo.toml` depended on
`deep_causality`, `algebra`, `core`, `haft`, `metric`, `multivector`, `num`, `num_complex`, `tensor`
and `uncertain`. Two of the ten are optional: `deep_causality` is gated on `qcm`, which is in
`default`, and `deep_causality_uncertain` on `qpu`, which is not. So nine by default and ten with
`qpu`. It depends on none of `deep_causality_topology`, `deep_causality_linear` or
`deep_causality_homology`.

**The edge to add is homology, not topology.** This paragraph used to say that taking a `Chain`
parameter adds the topology edge. That is superseded: the carrier is `Gf2Chain<W>`, which lives in
`deep_causality_homology`. Note that homology exports no type named `Chain`, and topology's `Chain`
is the unrelated weighted one, so a spec must name `Gf2Chain` explicitly or it will budget for the
wrong crate.

**It is a two-file edge.** `deep_causality_quantum/BUILD.bazel:21-31` lists the same ten crates, so
adding a dependency means Cargo.toml and BUILD.bazel together.

The carrier finding itself has been stable for six weeks: the last commit touching
`gates_haruna.rs` is `a7d386066` on 2026-07-16, and it was docs-only.

**Owner:** `deep_causality_quantum`.

### G-08 — No Poincaré-dual representative — **CLOSED**

**Severity S2.** Blocks R6, which the logical Hadamard needs (`H̄` requires a `γ̃` with `⟨γ,γ̃⟩ = 1`).

**Closure.** `fn dual_representative(&self, gamma: &Gf2Chain<W>) -> Option<Gf2Chain<W>>` over the
cohomology basis from G-04. `ChainComplex::coboundary_matrix`
(`deep_causality_unified_math/deep_causality_homology/src/traits/chain_complex.rs:68`) supplies the
cohomology side.

**The 𝔽₂ solve cannot be reached by instantiating an existing solver, and that is a tower fact
rather than a missing name.** `deep_causality_linear`'s generic elimination entry points are bounded
on `Field`, which `Gf2` satisfies, so `rref`, `rank`, `kernel_basis` and `image_basis` already accept
`PackedGf2`. The ones that would solve a system do not: `determinant`, `rref_stable` and
`rank_stable` are `NormedScalar`-bounded, `NormedScalar` is `Field + Normed + FromPrimitive + Copy`,
`Normed` is blanket-implemented only for `T: RealField`, and `Gf2` has neither a `Normed` nor a
`FromPrimitive` impl anywhere. So closing G-08 means writing a solver, not parameterizing one.

**`DualLatticeComplex` is not a foundation to build on.** Its `dual_cell` keeps the primal position
and complements only the orientation bitmask, carrying the inline comment *"We might need to adjust
position for exact Poincaré duality geometric interpretation"*
(`dual_lattice_complex.rs:82-84`), and its `coboundary` assumes the map is self-inverse (`:129`).
The surrounding docstring is unresolved deliberation, and the only law-level test is
`dual(dual(cell)) = cell`. The type is public. Treat it as needing rework rather than extension.

**Closure as built, and it needs no solver at all.** `ChainComplex::dual_representative(gamma, k)`
scans the cohomology basis and returns the first class pairing to one, or `None`.

That is the whole algorithm, and it is sound rather than a shortcut. The pairing is 𝔽₂-linear in its
second argument, so for `γ̃ = Σ aᵢcᵢ` over a cohomology basis, `⟨γ, γ̃⟩ = Σ aᵢ⟨γ, cᵢ⟩`. That sum can
be made one exactly when *some* `⟨γ, cᵢ⟩` is one. So the scan both decides the question and produces
the witness, and a single basis element is a valid answer whenever any answer exists.

**This register was wrong to call it a linear solve, twice over.** It first said G-08 "is a linear
solve over 𝔽₂ against the cohomology basis", then that closing it "means writing a solver, not
parameterizing one". Neither holds. There is no system: the linearity of the pairing collapses it to
a scan. And had a solve been needed, `rref` is bounded on `M::Scalar: Field`, `Gf2` reaches `Field`,
and `PackedGf2` implements `RowOps`, so 𝔽₂ elimination was available the whole time; only the
shipped `solve` is out of reach, being `NormedScalar`-bounded.

`DualLatticeComplex` is untouched, so the concern about building on it does not arise: this works
off `coboundary_matrix`, which every `ChainComplex` already owes.

**Tested against a non-degeneracy fact, not against itself.** On a closed orientable surface the
intersection pairing is non-degenerate, so every torus 1-class must have a dual; that is asserted
for both. The converse case is asserted too: a boundary pairs to zero with every cocycle, so it has
no dual, and pushing a 2-cell down gives one to check. Every dual returned anywhere in the ten
fixtures is re-paired against its argument and must give one.

**Owner:** `deep_causality_homology`, not `deep_causality_topology` as first proposed.

### G-09 — No logical-equivalence predicate — **CLOSED**

**Severity S2.** Blocks R8, and therefore `check_class_invariance`.

`Projection::commutes_with` (`verdict/projection.rs:192`) and `matrix_commutator` exist, but nothing
decides `O₁ ~ O₂`.

**Closure.** Per B.1 and B.3, `O ~ I` iff `O` commutes with every logical `Z̄(γ)` for `γ ∈ H₁` and
every `X̄(γ̃)` for `γ̃ ∈ H¹`. With the representatives from G-04 this is a bitset computation, not a
matrix comparison. Theorem A.1 supplies the underlying justification.

**Closure as built.** A new `qcode` module with two types. `LogicalPauli<W>` is a physical Pauli in
symplectic form, `{ x: Gf2Chain, z: Gf2Chain }`, with `compose` adding the two vectors over 𝔽₂.
`LogicalBasis<W>` holds the `H_k` and `H^k` representatives from G-04 and decides both questions:

- `is_logically_trivial(op)` is B.1's criterion. A Pauli `P = (x, z)` anticommutes with
  `Z̄(γ) = ∏_{i∈γ} Z_i` exactly when `|x ∩ γ|` is odd, since only a coincident `X_i` and `Z_i`
  anticommute; that count mod two is `⟨x, γ⟩`. Symmetrically `⟨z, γ̃⟩` against `X̄(γ̃)`. So the test
  is two loops of `Gf2Chain::inner` over the two bases, and nothing forms a matrix.
- `are_logically_equivalent(a, b)` is B.1's reduction `O₁ ~ O₂ ⟺ O₁O₂⁻¹ ~ I`. A Pauli is its own
  inverse up to phase, so the composite is the 𝔽₂ sum.

**Two limits are documented rather than assumed away.** Phase is not carried: `Y = iXZ`, so a
symplectic pair fixes a Pauli only up to phase, while B.1's matrix-element definition is
phase-sensitive. And the criterion decides triviality *of a logical operator*: an operator outside
the normalizer commutes with the logical generators without acting trivially, because it leaves the
code space instead. This crate carries no stabilizer group, so that precondition is stated at the
method rather than checked.

**Tested on the toric code, against two external facts.** The complex is the 2-torus fixture, whose
`β₁ = 2` over 𝔽₂ is Hatcher's and is why the toric code encodes two logical qubits (Kitaev, 2003).
The physics those combine into is that a stabilizer acts trivially and a logical operator does not,
so the predicate must separate exactly the boundaries from the non-trivial cycles. Asserted: every
`∂₂` column is trivial; every `H₁` representative is not; adding any boundary to a class leaves it
equivalent; the two independent classes are not equivalent to each other; an X-type operator is
caught through the homology side, which a predicate testing only the `Z` part would pass vacuously;
and `S²`, encoding nothing, calls every operator trivial rather than dividing by an empty basis.

Four mutations, four failures: checking only the homology side fails 2, swapping which part pairs
against which basis fails 2, composing by intersection instead of sum fails 3, and answering
"trivial" unconditionally fails 3.

**The dependency edge already existed**, paid for by G-07.

**Owner:** `deep_causality_quantum`.

### G-10 — Stirling numbers absent — **CLOSED**

**Severity S3.** Needed only for R9, the compact `a(γ)^m` form of A.12 to A.14.

Not required for any gate in Table 1. Record it so a later spec that wants the general
`O_k(γ₁…γₘ)` family of §3.5 knows the dependency.

**Closure as built.** `deep_causality_unified_math/deep_causality_num/src/combinatorics/` with `stirling_second` and
`stirling_first_unsigned`, both generic over `N: NaturalNumber + FromPrimitive + Copy`.

**Both kinds, not one.** The register said "Stirling numbers" without saying which. Reading A.12 and
A.14 settles it: A.12 expands `a(γ)^m` using `S(m,r)` of the second kind, and A.14 inverts it using
the paper's `s(r,m)`, defined there as *the number of permutations of r elements with exactly m
disjoint cycles*. That is the **unsigned** first-kind number, with the sign carried separately as
`(−1)^{m+r}`. Handing A.14 a signed convention would count the sign twice, so the function is named
for the convention it implements.

**The caller supplies the working row.** `deep_causality_num` allocates nowhere — it contains no
`Vec` and no `String`, and has no `alloc` feature — so both functions take a `scratch: &mut [N]`
rather than building a row. A fixed internal array was considered and rejected: any cap is a real
limit, since `S(n, n−1) = C(n, 2)` makes `S(200, 199) = 19900` representable but 200 wide.

**Verified against published values and against each other.** Rows 0–5 of both triangles against
OEIS A008277 and A132393; the row sums against the Bell numbers and the factorials; the edges
against `S(n,2) = 2^{n−1} − 1` and `c(n,1) = (n−1)!`; overflow reported rather than wrapped at the
exact boundary in each. The sharp one is the inversion `Σ_k (−1)^{n−k} c(n,k) S(k,m) = δ_{n,m}`,
which is what makes A.12 and A.14 a matched pair and which pins the sign convention. Mutating either
recurrence's multiplier fails six of the eight tests.

**Owner:** `deep_causality_num`.

### G-11 — No quantum channel composition — **CLOSED**

**Severity S2.**

`channel.rs` has `choi_from_kraus`, `kraus_from_choi`, `apply_kraus`, `apply_choi` and the two CPTP
checks, and no way to compose two CPTP maps.

**The formula is settled.** Under this crate's input-major Choi convention (`channel.rs:9-14`):

```
J(F∘E)_{(a,c),(a',c')} = Σ_{b,b'} J(E)_{(a,b),(a',b')} · J(F)_{(b,c),(b',c')}
```

A plain double contraction over the shared wire, with no partial transpose and no partial trace.
Verified independently by five agents: max relative Frobenius residual **3.198e-16** over 500 random
CPTP pairs across 10 dimension triples including non-square and degenerate cases; every transposed or
conjugated variant is wrong by O(1), so a single test pins the convention. Associativity holds to
6.5e-16 and the unit laws are exactly zero. One agent reproduced it against a real Rust build linking
the crate's own `choi_from_kraus`/`apply_choi`.

**Closure as built.** `choi_compose(first, then, d_a, d_b, d_c)` in `channel.rs`, bound on
`R: RealField`, dimensions validated through `square_dim`. The parameters are named for the order of
application rather than the order of the `∘` symbol.

**The formula is derived in the doc, not asserted.** It follows from this module's own `apply_choi`:
substituting `E(ρ)[b,b'] = Σ_{a,a'} ρ[a,a']·J(E)[(a,b),(a',b')]` into itself for `F(E(ρ))` and
collecting the coefficient of `ρ[a,a']` gives the double contraction. So the convention is a
consequence of the crate's own definition rather than a choice that happens to agree with it.

**Tested against references that do not use it.** Two unitary channels composing to the product
unitary, checked against one `apply_kraus` with `U = HS`, whose factors do not commute. A `2 → 3 → 2`
composition against an answer computed on paper, `Tr(ρ)·|0⟩⟨0| = 5·|0⟩⟨0|`, because unequal
dimensions are where a wrong stride survives. Both unit laws, associativity, and the dimension
errors. Injecting a transposed `J(F)`, a transposed `J(E)`, and a `b`/`b'` swap each fails.

**Owner:** `deep_causality_quantum`.

### G-12 — No identity channel — **CLOSED**

**Severity S3.**

**Closure as built.** `choi_identity(d)` in `channel.rs`, giving `J(id_d)[(i,j),(k,l)] = δ_ij·δ_kl`.
With `choi_compose` the module now carries a category: dimensions are the objects, Choi operators the
morphisms, and both unit laws are asserted.

**Checked against an independent construction.** `choi_identity(d)` must equal
`choi_from_kraus(&[identity_matrix(d)])`, which reaches the same operator through the Kraus formula
`Σ_κ K_κ[j,i]·conj(K_κ[l,k])`. The two share no code, so the test pins the index layout rather than
restating it.

**Owner:** `deep_causality_quantum`.

### G-13 — The scalar bound table is over-tight in five of eight rows — **CLOSED**

**Severity S3.**

A feasibility lens checked `qcl-design-note.md` §6.4 against what the shipped functions actually
require and found five rows wider than the operations need, all in the same direction. It also
confirmed that **one `QclConfig<R>` parameter suffices**; the bounds do not conflict across stages.

This is the same failure mode as the cup product shipping on `RealField` when `CommutativeRing +
Copy` sufficed.

**Closure.** Correct the five rows against the actual signatures. Not yet re-derived here, so the
spec should re-check rather than copy.

**Closure as built: the premise does not reproduce.** All eight rows were re-derived against the
shipped signatures. The lens's headline, five rows over-tight in one direction, is not what the tree
says:

- **One row is genuinely over-tight and relaxable.** Shot statistics and separation sit at
  `RealField` where `Real + FromPrimitive` suffices, because the Bhattacharyya formula contains no
  ratio and the surface touches no complex carrier. It is also the one row with no shipped signature
  at all: `Bhattacharyya` appears in no `.rs` file in the workspace.
- **Three rows are under-specified rather than over-tight.** Rotations, Born read-out and Verdicts
  omit the `FromPrimitive`, `Default` and `Debug` that ship on every one of them.
- **One row is not implementable as written.** `ComplexField<R>` appears nowhere in
  `deep_causality_quantum`; every signature is a concrete `Complex<R>`. It cannot cheaply become
  generic either, because `re` and `im` are public fields read directly while `ComplexField` exposes
  only methods.
- **One row contradicts §6.5 seventeen lines below it.** The table said costs need no scalar bound
  and use `usize`; `Ledger<R>` carries `device_time`, `cost` and `bits` as `R` and binds
  accumulation on `Real`.
- **Two rows were correct.** Cochains at `CommutativeRing + Copy`, and tolerances at
  `RealField + FromPrimitive`.

**The load-bearing fact, which makes the table derivable rather than memorised.** Every impl for
`Complex<T>` in `deep_causality_num_complex` is written `impl<T: RealField>`, `Zero` and `One`
included, so `Complex<R>` reaches no algebraic structure below `RealField`. And `Real` has exactly
two implementor families: the `Float` blanket, which also reaches `RealField`, and `Dual<T>`, which
does not. So `Real` versus `RealField` on any row asks one question, does this surface admit dual
numbers and stay differentiable, and any surface routed through `Complex` cannot. Four of the eight
rows are pinned by the carrier rather than by their operations, which is why editing their `where`
clauses would not have worked.

**Owner:** the design note. There is no implementation half: the correction is to §6.4's table and
the prose under it.

### G-14 — The `design` complexity claim is wrong — **CLOSED**

**Severity S3.**

`qcl-design-note.md` calls minimum-cost set cover "exponential in k". A feasibility lens found that
framing to be an artifact of choosing exhaustive enumeration, and that the problem has a much cheaper
formulation at the scale QCL operates on.

**Closure.** Re-derive the algorithm, correct §10.3, and revisit §7.5. **Only §10.3 carries the
claim**, at its line 693; §7.5 contains no complexity statement at all, just two
`.design(MinCostCover { .. })` call sites that would change if the algorithm did.

**Closure as built: the exponent is real, and it is in the wrong variable.** §8.8 of the liftback
note formulates the stage as minimum-cost set cover whose universe is the `C(n,2)` hypothesis pairs
and whose sets are the `k` experiments. Enumerating subsets of *experiments* costs `2^k`, which is
where "exponential in k" came from. That is the wrong enumeration. The exact answer is a DP over
subsets of the universe, `dp[S | cover(e)] = min(dp[S], dp[S] + cost(e))`, at `O(2^C(n,2) · k)`:
**linear in k, exponential in n.** So the benchmark plan's "sweep k from 4 to 20 to find the cliff"
sweeps an axis with no cliff on it. The cliff is in n and arrives near n = 7 or 8, where `C(n,2)`
reaches 21 to 28.

**And at this note's own scale the solve is not the cost.** §8.6 sizes the scan at `|E| × |H|` plant
evolutions plus `|E| × C(|H|, 2)` closed-form coefficients, which is 120 and 120 for 40 depths and
three hypotheses. The cover DP over the same numbers is `2^3 × 40 = 320` arithmetic steps against
120 plant evolutions. The coverage-matrix build dominates, and it is what Tier 2 should measure.

**The claim had four sites, not one.** `qcl-design-note.md` §10.3 carried the table row and the
sweep instruction; `qcl-dsl-liftback.md` §8.8 and `example-crosstalk-attribution.md` §6 both said
"exactly solvable by enumeration at the scale this operates on" without saying which enumeration,
which is the ambiguity the wrong exponent grew out of. All four now name the universe.

**There is no implementation half.** `MinCostCover`, `DesignObjective`, `DesignPlan` and a `design`
stage appear in zero `.rs` files, and `deep_causality_quantum` has no `benches/`. This gap was
entirely prose.

**Owner:** the design note.

### G-15 — The orthomodular guard may be misplaced — **CLOSED**

**Severity: was unknown; the guard was over-broad.**

`qcl-design-note.md` §4 requires `adjudicate` to check `Projection::commutes_with` before folding
verdicts from forked worlds. But the forked worlds in the calibration example carry
`Uncertain<FloatType>` from a Born read-out, not `Projection<R, D>`. If that is what flows, the guard
has nothing to apply to and §4's second rule over-reaches, however sound the underlying law is.

The lens assigned to settle this did not complete before the workflow was stopped.

**Closure as resolved.** §4's second rule is narrowed to the projection-valued case, in the note.

**There was nothing to determine.** The register's closure said to find out what `fork` produces.
Searched: the only `fork` in the workspace is `deep_causality_cfd`'s state-fork counterfactual, an
unrelated thing. Neither `fork` nor `adjudicate` exists in QCL. So the rule constrains an API still
to be built.

**The rule was over-broad regardless.** Rule 1 puts the measurement boundary at `observe`, and the
calibration pipeline in §7.5 forks *after* it, on `Spec::at_least(ft(0.999))`. A threshold on a real
quantity is a classical proposition; those form a Boolean algebra where the distributive law holds
unconditionally and no pair fails to commute. Applying the guard there would reject sound folds. The
rule now reads: whichever kind of verdict a world carries, the fold must match — projection-valued
folds check commutation, read-outs against a real-valued spec do not, and must not.

**Owner:** the design note.

### G-16 — `Boundary`'s second hypothesis is not decidable at the stated tolerance — **CLOSED**

**Severity S2 if confirmed.**

The carriers lens found that the `Z ⊗ 1_B` **form** of `partial_trace_preservation_boundary` is
decidable from `FactorSupports` leg data alone, soundly and cheaply. The theorem's other hypothesis,
`[Z⊗1_B, M] = 0`, is not: it needs a numeric commutator, and the crate supplies commutation only to
Q-TOL tolerance while the Lean theorem is stated on exact equality.

**Closure.** Decide whether a Q-TOL-satisfied commutator is sufficient warrant to invoke an
exact-hypothesis theorem, and record the answer. This is a soundness question, not an ergonomics one.

**It also inherits a clause from G-17.** That gap asked to gate `predict` behind this boundary check.
`predict` does not exist in `deep_causality_quantum`, so the gating belongs with whenever it is
built, and it belongs here rather than at G-17 because this is the check it would be gated on.

**The substitution this gap asks about is already being made, unrecorded, at a hardcoded tolerance.**
`tests/formalization_lean/partial_trace_tests.rs:139` discharges the theorem's exact hypothesis
numerically, `assert!(frobenius_norm(&matrix_commutator(&x, &y).unwrap()) < 1e-12)`, then asserts the
conclusion the same way at `:143`. That is precisely the exact-for-approximate trade this gap says
needs a ruling, it sits in a THEOREM_MAP-bound witness, and the constant is an ad-hoc `1e-12` rather
than the crate's `CommutatorTolerance` policy (`src/types/qcm/markov_freeze.rs:44`). The Lean
statement it stands in for is propositional equality over a general ring with no epsilon
(`lean/DeepCausalityFormal/Quantum/PartialTrace.lean:143`).

**There is no Rust `partial_trace_preservation_boundary`.** It exists as a Lean theorem
(`PartialTrace.lean:141`) and as that witness test, nothing more. So this gap gates a check that has
not been written and blocks no shipped code today. Note also that the `partial_trace` doc block that
closed G-17 addresses the `Z ⊗ 1_B` **form**, the decidable half, and says nothing about tolerance.
It is not the ruling this gap asks for.

**Closure as built: the answer is a factor, not a yes or a no.** A tolerance-satisfied commutator
is **not** warrant for the exact conclusion, and **is** warrant for the conclusion at `√(d_B)` times
the tolerance. Both halves ship.

The route around the exact hypothesis is to stop needing it. `quantum.partial_trace.commutator_transport`
(`Quantum/PartialTrace.lean :: partialTraceRight_commutator`) states
`Tr_B([Z ⊗ 1_B, M]) = [Z, Tr_B(M)]` with **no hypothesis at all**, proved from the two bimodule laws
plus a new `partialTraceRight_sub`. It is a pure `CommRing` identity and needed no new Mathlib
import. Combined with `‖Tr_B(E)‖_F ≤ √(d_B)·‖E‖_F`, which is Cauchy-Schwarz on the traced index and
is tight at `E = F ⊗ 1_B`, a residual of `ε` in the hypothesis certifies `√(d_B)·ε` in the
conclusion. Exactly zero in, exactly zero out, so the original theorem is the vanishing case.

The bound is deliberately **not** stated in Lean. A norm-quantified theorem needs
`Mathlib.Analysis`, and this development is over a general ring with no norm; the amplification
lives in Rust, where the norms are.

**The advertised function now exists.** `partial_trace`'s doc block named
`partial_trace_preservation_boundary` as "the sound path" and pointed a caller at a *test*, because
no such Rust function existed. It does now, in `qgates/operator_linalg.rs`, returning a
`BoundaryWarrant` carrying the measured residual, the tolerance, the amplification and the certified
`conclusion_bound`. It returns a bound rather than a boolean because that is the whole content of
the ruling. The `Z ⊗ 1_B` form is *constructed* from the caller's `Z` rather than checked, so the
theorem's other hypothesis holds by construction.

**The severity did not survive, and this register was wrong about why.** It recorded that the
exact-for-approximate substitution "is already being made, unrecorded, at a hardcoded `1e-12` in a
THEOREM_MAP-bound witness". Checked rather than reasoned: in that witness `X = σz ⊗ W` and
`Y = σz ⊗ 1`, so `[X, Y] = (σzσz) ⊗ W − (σzσz) ⊗ W` vanishes algebraically, and every entry is a
small dyadic integer, so kron and matmul are exact in binary64. Measured: the commutator is
bit-exactly `0.0`. The `1e-12` was defensive slack over an exact zero, which is a **traceability**
defect and not a soundness one. That witness now asserts `== 0.0`, because writing `< 1e-12` there
misrepresents an exact premise as an approximate one. The genuinely approximate case has its own
test alongside it.

**Four mutations, four failures:** amplifying by the kept dimension instead of the traced one fails
1 test, dropping the amplification fails 1, making the tolerance comparison strict fails 1, and
building the boundary as `1_B ⊗ Z` fails 2.

**Owner:** `deep_causality_quantum`, and `lean/THEOREM_MAP.md` carries the two new rows.

### G-17 — `partial_trace` does not document its non-preservation — **CLOSED**

**Severity S1.** This is friction F9.

`deep_causality_quantum/src/types/qgates/operator_linalg.rs:176` documents shape errors carefully and says nothing about commutation. The
refutation of `quantum.partial_trace_preservation` lives only in `LEAN_QUANTUM.md` and the Lean tree,
so a caller marginalising a validated factorization would destroy the Markov property that validate
had certified, with nothing at the call site to say so.

**Closure as built.** `partial_trace`'s doc block now carries a *This does not preserve commutation*
section: the refutation, its Lean name `quantum.partial_trace_nonpreservation` and the
`[[0, 4], [−4, 0]]` counterexample, the concrete consequence for a caller marginalising a validated
factorization, and `partial_trace_preservation_boundary` named as the sound path with its witness
test. Both witnesses were confirmed to exist at
`tests/formalization_lean/partial_trace_tests.rs`.

**The third clause was moot.** The register also asked to "gate `predict` behind the boundary check
of G-16". `predict` does not exist in `deep_causality_quantum`. That clause belongs with whenever it
is built, and is recorded under G-16 rather than here.

**Owner:** `deep_causality_quantum`.

### G-18 — No `Cochain` type — **CLOSED**

**Severity S3.** The dual of G-05.

`cup_product` takes two cochains and two degrees as four separate arguments
(`cup_product/mod.rs:95-101`): `(complex, alpha, alpha_degree, beta, beta_degree)`. `cup_product_n`
(line 180) takes `&[(&[R], usize)]`, pairing them by convention only. Nothing binds data to degree,
so a mismatch is caught by a runtime check rather than by the type.

The register previously cited `cup_product/mod.rs:62-70` and a single `cochain: &[R]`. Both are
stale. The real signature strengthens the case: a `Cochain` turns five parameters into three, and a
slice of tuples into a slice of one type.

**The premise "no type binds data to degree" is also wrong, and that matters more.**
`Topology<R, G>` (`types/topology/mod.rs:34`) carries `complex`, `grade`, `data` and `cursor`, and
its `cup_product` method documents itself in cochain language: *"`other`: The q-cochain β. `self` is
the p-cochain α."* A type binding data to degree exists. It carries two fields the cochain role does
not need, and it is bound to `SimplicialComplex` where the free function is generic.

**And the crate computes the cup product twice.** `Topology<R, G>::cup_product`
(`types/topology/ops/cup_product.rs:35`, **line numbers in this section are historical**: the file
was rewritten when the gap closed and now delegates in 84 lines) is a second, independent
implementation. It does not call
the free function. It extracts the Alexander-Whitney front and back faces by hand for the simplicial
case, which the generic function already covers, because `Simplex` implements `SplittableCell`
(`types/simplex/cell_splitting.rs:11`).

Measured before anything was decided: the two agree **bit-for-bit**, maximum difference exactly zero,
over the degree pairs the test exercises. `tests/types/cup_product/implementation_agreement_tests.rs`
pins that. It is the right artefact whichever way the duplication is resolved, and it is the safety
net for resolving it.

**The safety net is narrower than this register claimed.** The text said "every degree pair a
tetrahedron admits", and the test's own doc comment repeats the phrase, but the loop at
`implementation_agreement_tests.rs:31` is `[(0,0), (0,1), (1,0), (1,1), (0,2), (2,0)]`: six pairs,
all with `p+q ≤ 2`. The fixture is a tetrahedron with `max_dim() = 3`, so `(0,3)`, `(3,0)`, `(1,2)`
and `(2,1)` also exist and are untested. Six of ten. The uncovered pairs are untested rather than
divergent, and on simplices the two are provably identical: `Simplex::split` returns one split with
sign `+1` and `verts[p..]` is the same slice as the method's `verts[p..=r]`, while both index the
same sorted skeleton vector.

**Delegation is not behaviour-preserving, and this register did not say so.** Three deltas as
assessed, of which one turned out not to exist and a fourth was missing; see the closure below for
what the tree actually held. The line numbers are those of the retired implementation.

1. Out-of-range grade. `Topology::cup_product` returns `Ok` with a zero-filled cochain when
   `r > max_simplex_dimension()` (`ops/cup_product.rs:48-63`); the free function returns
   `Err(TopologyErrorEnum::InvalidGradeOperation)` (`cup_product/mod.rs:117-125`). A naive
   delegation turns a success into an error.
2. The method panics on a data/skeleton length mismatch, `.expect("Data/Skeleton mismatch")` at
   `ops/cup_product.rs:105,111`, where the free function's `check_len` (`cup_product/mod.rs:63`)
   returns a typed `DimensionMismatch`.
3. The method's `Arc::ptr_eq` complex-identity check (`ops/cup_product.rs:43-45`) has no counterpart
   in the free function.

Delegation also narrows a public impl. The block is `impl<R, G> Topology<R, G>` with no bound on
`R`; `SimplicialComplex<T>` implements `CellularComplex` only for `T: RealField`, so delegating
forces `R: RealField` onto it. The coefficient side is fine, `G: Field + Copy` already satisfies the
free function's `R: CommutativeRing + Copy`.

**A stale requirement forbade this, and it has been superseded rather than archived.**
`openspec/specs/cup-product/spec.md`'s first requirement said the cup product *"MUST NOT introduce a
dedicated `Cochain` type."* The author ruled it stale on 2026-08-31. Archiving the file would have
been wrong: it carries eight requirements and only the first was stale, the other seven covering
degree-generality, associativity, the torus pairing and graded commutativity, all of which this
change preserves. So the one requirement was rewritten in place, with the supersession and its
reason recorded there.

The evidence agrees with the ruling. Decision 2 of the archived change priced the prohibition on
conversion cost: every physics and CFD call site would need converting. That cost is zero. Neither
cup product has a production caller anywhere in the workspace; a search finds only the module
wiring, the re-export and the crate's own tests. Nothing in quantum, physics
or CFD calls either one.

**Chosen closure, in this order:** make `Topology::cup_product` delegate to the free function,
handling the deltas above, then add `Cochain<R>` and thread it through. Unifying first means the new
type is threaded through one implementation rather than two.

**Closure as built.** Both steps landed. `Cochain<R>` is `{ values: Vec<R>, degree: usize }` and
nothing more: `from_values` takes the established flat slice and `into_values` gives it back
unchanged, so the representation the superseded requirement was written to protect is untouched.
`cup_product` went from five parameters to three and `cup_product_n` from a slice of
`(values, degree)` tuples to a slice of one type. `Topology::cup_product` is 84 lines that delegate,
down from 128 that hand-rolled Alexander-Whitney; what it keeps is the pair of things a free
function taking one complex cannot express, the `Arc::ptr_eq` check that both operands live on the
same complex and the rewrap into a `Topology`.

**The deltas, corrected.** This register listed three. One does not exist and a fourth was missing.

- **The grade tightening is real.** Out of range now returns `InvalidGradeOperation` where the
  retired implementation returned `Ok` with a zero-filled cochain.
- **The panic delta does not exist.** The register said the method panics through
  `.expect("Data/Skeleton mismatch")` where the free function returns a typed error. The panic is in
  the retired code, but `Topology::new` validates `data.len()` against the skeleton size, so it was
  unreachable through the public constructor. A test written to pin the change could not construct
  its own precondition.
- **A fourth delta, unrecorded, and it is a loosening.** The retired implementation raised
  `SimplexNotFound` when a split's partner was absent from the complex. The shared body skips such a
  split instead, and that skip is load-bearing: on a lattice complex with a non-periodic axis the
  boundary cells genuinely have no partner and must contribute zero. So a *malformed* simplicial
  complex now yields zeros where it used to error. Deliberate, and recorded at the test that used to
  assert the old behaviour: nothing in the crate builds such a complex, since `SimplicialComplex`
  derives its skeletons from its facets.
- The bound narrowing happened as predicted: the impl block gained `R: RealField`.

**The safety net was widened on the way past.** The agreement test covered six of the ten degree
pairs a tetrahedron admits while it was pinning two implementations against each other. It now
covers all ten, and it is no longer an agreement test: with one implementation left it checks the
wrapping the method adds.

**Four mutations, four failures:** dropping the same-complex check fails 2 tests, swapping the
method's operands fails 2, having `cup_product_n` ignore all but the first factor fails 5, and
reporting the wrong result degree fails 6.

**Breaking.** `deep_causality_topology` goes 0.8.0 to **0.9.0**, and the four crates pinning `"0.8"`
were repinned: physics, cfd, algorithms and discovery. The example crates pin by path and needed
nothing.

**Closure.** ~~Fold into G-05: one type carrying data and degree together, used by both the cup
product and the gate layer.~~ **Void.** G-05 closed as `Gf2Chain<W>`, which is 𝔽₂-only and cannot
carry the ring-generic `R` the cup product needs. The two gaps needed two types, and that is what
shipped.

**Owner:** `deep_causality_topology`, where both cup products live.

---

## 3. Corrections to the design note

These are not code gaps. They are claims in `qcl-design-note.md` that this assessment showed to be
wrong, recorded so the spec does not inherit them.

| # | Claim | Status |
|---|---|---|
| C-1 | "Nesting quantum subgraphs would require preservation, which is false" | **Wrong.** F9 blocks *abstraction*, not composition. A node whose factor spans the union of its children's legs needs no marginalisation, and `FactorSupports::declare` expresses it today |
| C-2 | "The textbook Chiribella–D'Ariano–Perinotti form carries a partial transpose because it assumes a different index convention" | **Refuted numerically.** The CDP form agrees with the plain contraction to 1.097e-15. The partial transpose is the price of writing the contraction as a matrix product on the joint space, not a convention artifact |
| C-3 | `GaugeField::from_cochain` is underdetermined, partly a research question | **Wrong.** Reading the paper shows the requirement is a bitset and a circuit. It is a wrong-carrier bug (G-07), not research |
| C-4 | Flatness is a property of `FactorSupports` | **Wrong.** `declare` takes any leg list. The flat convention lives in `FactorSupports::from_graph` alone, and the causal layer is not the constraint: `Causaloid::from_causal_graph` builds a `CausaloidType::Graph` node routed to `evaluate_subgraph_from_cause_stateful` |

The pattern across all four: reasoning from a crate's type signatures instead of the source it
implements.

---

## 4. Dependency order for closure

```
CLOSED
  G-01  F₂ linear algebra                    deep_causality_linear
  G-02  betti via F₂ rank                    topology + linear
  G-03  LatticeComplex betti                 topology — closed form kept, now checked
  G-05  Gf2Chain                             homology — not topology as proposed
  G-06  the Table 1 gate alphabet            quantum — one diagonal kernel, plus a validation fix
  G-07  the Haruna layer on Gf2Chain         quantum — plus the homology edge, 0.2.0
  G-08  Poincaré-dual representative         homology — a scan, not a solve
  G-09  logical equivalence                  quantum — B.1 on bitsets, toric-code tested
  G-10  Stirling numbers                     num — both kinds, first one unsigned
  G-11  choi_compose                         quantum
  G-12  choi_identity                        quantum
  G-13  the scalar bound table               design note — premise did not reproduce
  G-14  the set-cover exponent               design note — linear in k, exponential in n
  G-15  the verdict guard                    design note — rule 2 narrowed
  G-16  the tolerance ruling                 quantum — a √(d_B) factor, not a yes or no
  G-17  partial_trace non-preservation       quantum — documented
  G-18  Cochain and one cup product          topology — duplicate retired, 0.9.0

```

**Nothing is open.** The chain that R1 through R9 asked for is complete end to end: `Gf2Chain`
(G-05) carries the vectors, `homology_representatives` (G-04) produces the classes,
`dual_representative` (G-08) pairs them, the Table 1 alphabet (G-06) and the Haruna layer (G-07)
emit the circuits, `LogicalBasis` (G-09) decides when two of them act alike, `Cochain` (G-18)
carries the ring-valued dual over a single cup product, and the boundary ruling (G-16) says what a
measured commutator is worth after marginalisation.


### What to tackle next

**Nothing in this register.** Every gap it opened is closed. What comes next is not gap work: the
geometric-QEC example, which §4 used to list as blocked on G-04 through G-09 and which is now
blocked on nothing but itself.

**What the three designed examples need.** The calibration and crosstalk examples need none of G-01
through G-09: two feasibility lenses reproduced their headline results on shipped APIs in scratch
crates. **The geometric-QEC example is no longer gap-blocked.** G-04 through G-09 have all closed
and the dependency edge with them, so what remains for it is the example itself.

---

## 5. Sources

- Haruna, J. (2025). *Note on Logical Gates by Gauge Field Formalism of Quantum Error Correction.*
  arXiv:2511.15224, in `deep_causality_quantum/papers/`. Read end to end; §1 defines the requirement
  table above, Table 1 gives the decompositions, Appendix A the integer identities, Appendix B the
  (co)homology-invariance proofs.
- `deep_causality_quantum/LEAN_QUANTUM.md` for the modality split, the refuted
  `partial_trace_preservation` and the conditional boundary theorem.
- `deep_causality_quantum/src/` (3980 lines), `deep_causality_unified_math/deep_causality_topology/src/`,
  `deep_causality_unified_math/deep_causality_multivector/README.md`, `deep_causality_unified_math/deep_causality_num/src/integer/`.
- [`qcl-design-note.md`](qcl-design-note.md) for the design these gaps are measured against.
