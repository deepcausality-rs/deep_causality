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

Verified against the tree on **2026-08-25**. **Eight gaps are closed, ten are open.**

`add-linear-algebra-crate` closed G-01 and G-02. The category-A sweep then closed G-03, G-10, G-11,
G-12, G-15 and G-17: everything that was unblocked, self-contained and small.

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

**Ten open, verified today.** `homology_representatives` and `dual_representative` do not exist
anywhere in `deep_causality_topology/src`; the only `Chain` is the weighted one and there is no
`Cochain`; `logical_z` is still typed on `CausalMultiVector` (`gates_haruna.rs:137-139`); `GateOp`
carries four of Table 1's seven gates; and `deep_causality_quantum` still depends on neither
`deep_causality_topology` nor `deep_causality_linear`.

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
(`deep_causality_num/src/integer/natural.rs`) is blanket-implemented for every unsigned width and
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

**G-05 has a name collision that was not visible before.** `deep_causality_topology` already exports
a `Chain<T>` (`src/types/chain/mod.rs:18`) — a *weighted* chain, holding an `Arc<SimplicialComplex<T>>`,
a `grade`, and a `CsrMatrix<T>` of weights. G-05 asks for something different: a bit-packed
`Chain { bits, degree }` with `supp`, `inner` and `intersect`. Two distinct objects, one name, one
crate. The spec must pick a different name or subsume the existing type deliberately; it cannot
assume the name is free.

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

Moving it removed no dependency edge either way: `qcl-gaps` records G-07 and G-09 as needing G-04
and G-05, both owned by `deep_causality_topology`, so quantum takes a topology dependency for the
𝔽₂ work regardless.

**What was built.** `deep_causality_linear/src/types/packed_gf2/` — the representation, over
`W: NaturalNumber` rather than a fixed `u64`, which is what the aside below asked for.
`deep_causality_linear/src/algorithms/gf2.rs` — rank, kernel basis, image basis. `Gf2` itself is a
`deep_causality_num` scalar reaching `Field` through the tower's blanket, and is confirmed by
compile probe not to reach `RealField`, `NormedScalar` or `ConjugateScalar`.

**On the algebra tower.** `deep_causality_num` has an `Integer` trait supplying exactly the needed
primitives (`count_ones`, `trailing_zeros`, `checked_*`, `wrapping_*`,
`deep_causality_num/src/integer/mod.rs:38`). 𝔽₂ as a tower scalar would store one bit per element and
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
// deep_causality_topology/src/types/simplicial_complex/topology/chain_complex_impl.rs:94
fn rank_of_csr(matrix: &CsrMatrix<i8>) -> usize {
    // build an f64 dense tensor, then
    let (_, s, _) = tensor.svd().expect("SVD failed");
    s_vec.iter().filter(|&x| x.abs() > 1e-5).count()
}
```

Used at `chain_complex_impl.rs:85-86` and mirrored by `CellComplex::rank_of_matrix`
(`cell_complex/mod.rs:158-160`).

Rank over ℝ is not rank over 𝔽₂. The two agree for the toric code, which is why the geometric-QEC
example's `[[32,2,4]]` comes out right, but that is a property of that code family. A qLDPC code with
even-weight dependencies has a smaller 𝔽₂ rank, so the reported `k` would be wrong and no error
would be raised.

**Closure as built.** Neither helper survives. `HomologyField`
(`deep_causality_topology/src/types/homology_field/mod.rs`) is an enum with one method,
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
// deep_causality_topology/src/types/lattice_complex/mod.rs:522
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

### G-04 — No homology representatives

**Severity S2.** Blocks R4, and therefore R6 and the whole gate layer.

`ChainComplex` exposes `betti_number(k) -> usize` and `boundary_matrix(k) -> Cow<CsrMatrix<i8>>`
(`traits/chain_complex.rs:53,64`). A count, not a basis. Searched for cycle, representative,
generator and basis accessors; none returns homology classes.

The paper needs actual `γ ∈ ker ∂₁ \ im ∂₂` to build any logical gate.

**Closure.** `fn homology_representatives(&self, k: usize) -> Vec<Chain>` off the G-01 kernel and
image bases.

**State on 2026-08-25: the bases exist, the return type does not.** `kernel_basis_gf2` and
`image_basis_gf2` ship, and `csr_to_packed_gf2_mod2` converts a `CsrMatrix<i8>` boundary matrix into
what they take, so the mathematics is available. The signature above returns `Vec<Chain>`, and the
`Chain` it means is G-05's bit-packed one, which does not exist. **G-05 is a prerequisite for G-04
as specified**, not an independent root as §4's diagram used to show. Either land G-05 first, or
return `PackedGf2` rows and accept that the gate layer then has to wrap them.

**Owner:** `deep_causality_topology`.

### G-05 — No `Chain` type

**Severity S2.** Blocks R2, R3, R5.

A cochain today is a bare `&[R]` with the degree passed as a separate argument
(`cup_product/mod.rs:42,65`), and there is no 1-chain type at all. Nothing carries the degree with
the data, and nothing supplies `supp`, the inner product or the intersection.

**Closure.** A bit-packed `Chain { bits, degree }` with `supp()`, `inner(&self, &Chain) -> u8`,
`intersect(&self, &Chain) -> Chain`, and pair and triple iterators over the support.

**Owner:** `deep_causality_topology` for the type, since it is a chain-complex object.

**Name collision.** The crate already exports a `Chain<T>` (`src/types/chain/mod.rs:18`): a *weighted*
chain carrying an `Arc<SimplicialComplex<T>>`, a `grade`, and a `CsrMatrix<T>` of weights. It is a
different object from the bit-packed one this gap asks for. Pick a distinct name, or subsume the
existing type deliberately, and do not assume `Chain` is available.

**State on 2026-08-25: unblocked, and it moved to the front.** Nothing blocked it before and nothing
does now. What changed is its position: G-04, G-08, G-07 and G-09 all name this type in their
signatures, so it is the prerequisite for the entire remaining chain rather than an independent
root. It is also the only remaining item with no dependency of its own that the geometric-QEC
example cannot proceed without.

### G-06 — No circuit type

**Severity S2.** Blocks R7.

The gate kernels exist (`gates_haruna.rs`, `logical_z/x/s/hadamard/cz/t`) but there is no
representation of a physical-gate sequence, which is what every Table 1 decomposition produces.

**Closure.** `Circuit` as an ordered list of gate applications over named qubit indices, covering
`S, CZ, H, T, CS†, CCZ, C^{m-1}Z`.

**State on 2026-08-25: it needs extending, and the shortfall is measured.** `GateOp`
(`qpu/circuit.rs:19-36`) carries `H, X, Y, Z, S, T, Cnot, Cz`. Against Table 1 that supplies four of
the seven. Missing: `CS†`, `CCZ`, and `C^{m-1}Z` for general `m`. No adjoint form exists for any
gate, so `S†` and `T†` are absent as well, and the multi-controlled case needs a control *list*
rather than the fixed `{ control, target }` shape the two-qubit variants use. `QuantumCircuit`
itself (`qpu/circuit.rs:59`) is a register width plus an ordered program and needs no change.

**The work is not only in the enum.** `GateOp` has an execution path: `sim.rs:162-223` dispatches
every variant into a state-vector simulator through `apply_single`, `apply_cnot` and `apply_cz`. So
each new gate needs a variant, a `qubits()` arm at `circuit.rs:41`, **and** a simulator
implementation. `C^{m-1}Z` needs a generic multi-control apply rather than a copy of `apply_cz`.

**Owner:** `deep_causality_quantum`.

### G-07 — The Haruna gate layer is typed on the wrong carrier

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

**A dependency edge does not exist yet.** `deep_causality_quantum/Cargo.toml` depends on
`deep_causality`, `algebra`, `core`, `haft`, `metric`, `multivector`, `num`, `num_complex`, `tensor`
and `uncertain`. It does not depend on `deep_causality_topology`, and it does not depend on
`deep_causality_linear`. Taking `Chain` as a parameter adds the topology edge. The register assumed
this edge throughout and it was never checked; it is recorded here so a spec budgets for it.

**Owner:** `deep_causality_quantum`.

### G-08 — No Poincaré-dual representative

**Severity S2.** Blocks R6, which the logical Hadamard needs (`H̄` requires a `γ̃` with `⟨γ,γ̃⟩ = 1`).

**Closure.** `fn dual_representative(&self, gamma: &Chain) -> Option<Chain>` over the cohomology
basis from G-04.

**Owner:** `deep_causality_topology`.

### G-09 — No logical-equivalence predicate

**Severity S2.** Blocks R8, and therefore `check_class_invariance`.

`Projection::commutes_with` (`verdict/projection.rs:192`) and `matrix_commutator` exist, but nothing
decides `O₁ ~ O₂`.

**Closure.** Per B.1 and B.3, `O ~ I` iff `O` commutes with every logical `Z̄(γ)` for `γ ∈ H₁` and
every `X̄(γ̃)` for `γ̃ ∈ H¹`. With the representatives from G-04 this is a bitset computation, not a
matrix comparison. Theorem A.1 supplies the underlying justification.

**Owner:** `deep_causality_quantum`.

### G-10 — Stirling numbers absent — **CLOSED**

**Severity S3.** Needed only for R9, the compact `a(γ)^m` form of A.12 to A.14.

Not required for any gate in Table 1. Record it so a later spec that wants the general
`O_k(γ₁…γₘ)` family of §3.5 knows the dependency.

**Closure as built.** `deep_causality_num/src/combinatorics/` with `stirling_second` and
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

### G-13 — The scalar bound table is over-tight in five of eight rows

**Severity S3.**

A feasibility lens checked `qcl-design-note.md` §6.4 against what the shipped functions actually
require and found five rows wider than the operations need, all in the same direction. It also
confirmed that **one `QclConfig<R>` parameter suffices**; the bounds do not conflict across stages.

This is the same failure mode as the cup product shipping on `RealField` when `CommutativeRing +
Copy` sufficed.

**Closure.** Correct the five rows against the actual signatures. Not yet re-derived here, so the
spec should re-check rather than copy.

**Owner:** the design note, then the implementation.

### G-14 — The `design` complexity claim is wrong

**Severity S3.**

`qcl-design-note.md` calls minimum-cost set cover "exponential in k". A feasibility lens found that
framing to be an artifact of choosing exhaustive enumeration, and that the problem has a much cheaper
formulation at the scale QCL operates on.

**Closure.** Re-derive the algorithm and correct §7.5 and §10.3. The benchmark plan's "sweep k from 4
to 20 to find the cliff" may be measuring a cliff that need not exist.

**Owner:** the design note, then the implementation.

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

### G-16 — `Boundary`'s second hypothesis is not decidable at the stated tolerance (UNRESOLVED)

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

**Owner:** `deep_causality_quantum`, with a note in `LEAN_QUANTUM.md`.

### G-17 — `partial_trace` does not document its non-preservation — **CLOSED**

**Severity S1.** This is friction F9.

`deep_causality_quantum/src/types/qgates/operator_linalg.rs:152` documents shape errors carefully and says nothing about commutation. The
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

### G-18 — No `Cochain` type

**Severity S3.** The dual of G-05.

`cup_product` takes two cochains and two degrees as four separate arguments
(`cup_product/mod.rs:95-101`): `(complex, alpha, alpha_degree, beta, beta_degree)`. `cup_product_n`
(line 180) takes `&[(&[R], usize)]`, pairing them by convention only. Nothing binds data to degree,
so a mismatch is caught by a runtime check rather than by the type.

The register previously cited `cup_product/mod.rs:62-70` and a single `cochain: &[R]`. Both are
stale. The real signature strengthens the case: a `Cochain` turns five parameters into three, and a
slice of tuples into a slice of one type.

**Closure.** Fold into G-05: one type carrying data and degree together, used by both the cup product
and the gate layer.

**Owner:** `deep_causality_topology`.

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
  G-10  Stirling numbers                     num — both kinds, first one unsigned
  G-11  choi_compose                         quantum
  G-12  choi_identity                        quantum
  G-15  the verdict guard                    design note — rule 2 narrowed
  G-17  partial_trace non-preservation       quantum — documented

OPEN
G-05 + G-18  Chain / Cochain               (topology, no deps)   <- head of the chain
  ├── G-04  homology representatives       (topology)       <- needs G-05 for its return type
  │     └── G-08  Poincaré-dual representative (topology)
  ├── G-07  retype the Haruna layer        (quantum; needs G-04, G-05, G-06, + a topology dep)
  └── G-09  logical equivalence            (quantum; needs G-04, G-05, + a topology dep)

G-06  Circuit                              (quantum, no deps; 3 variants AND 3 simulator arms)

G-16  resolve the tolerance question       (quantum; blocks predict when predict is built)
G-13, G-14  correct the note               (design note)
```

### What the remaining ten cost

Sized against the tree, not estimated.

**Mid effort, unblocked.** G-05 with G-18 heads the chain: four gaps name its type, and its cost is
the two decisions rather than the code — the name against the weighted `Chain<T>`, and whether it
wraps `PackedGf2` or carries its own words. G-06 needs three enum variants, three `qubits()` arms and
three simulator implementations. G-13 needs eight rows re-derived against real signatures, and the
lens's finding is explicitly not to be copied. G-14 needs the set-cover algorithm re-derived at
QCL's scale; the edit that follows is small. G-16 is a soundness decision with a written rationale:
whether a Q-TOL-satisfied commutator is warrant to invoke an exact-hypothesis Lean theorem.

**Mid effort, blocked.** G-04 needs G-05 for its return type; the 𝔽₂ bases and the
`CsrMatrix<i8>` bridge already ship, so the work is quotient-basis extraction for homology and
cohomology.

**Serious.** G-08 is a linear solve over 𝔽₂ against the cohomology basis. G-09 implements B.1 and
B.3. G-07 rewrites 272 lines and six public functions off `CausalMultiVector` onto `Chain` and
`Circuit`. All three additionally need the `deep_causality_quantum` → `deep_causality_topology`
dependency edge, which does not exist.

### What to tackle next

**G-05 with G-18.** It heads the remaining chain and has no dependency of its own. Nothing else on
the critical path can be specified until the shape of `Chain` is fixed, and specifying it wrongly
propagates into G-04's return type and both quantum gaps.

The cup product supplies a second argument the register did not record. `cup_product` takes
`(complex, alpha, alpha_degree, beta, beta_degree)` and `cup_product_n` takes `&[(&[R], usize)]`; a
`Cochain` turns five parameters into three and a slice of tuples into a slice of one type.

**What the three designed examples need.** The calibration and crosstalk examples need none of G-01
through G-09: two feasibility lenses reproduced their headline results on shipped APIs in scratch
crates. The geometric-QEC example needs G-05, G-04, G-08, G-06, G-07 and G-09, in roughly that
order, plus the dependency edge.

---

## 5. Sources

- Haruna, J. (2025). *Note on Logical Gates by Gauge Field Formalism of Quantum Error Correction.*
  arXiv:2511.15224, in `deep_causality_quantum/papers/`. Read end to end; §1 defines the requirement
  table above, Table 1 gives the decompositions, Appendix A the integer identities, Appendix B the
  (co)homology-invariance proofs.
- `deep_causality_quantum/LEAN_QUANTUM.md` for the modality split, the refuted
  `partial_trace_preservation` and the conditional boundary theorem.
- `deep_causality_quantum/src/` (3980 lines), `deep_causality_topology/src/`,
  `deep_causality_multivector/README.md`, `deep_causality_num/src/integer/`.
- [`qcl-design-note.md`](qcl-design-note.md) for the design these gaps are measured against.
