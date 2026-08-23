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

## 0. Status: what the numeric-tower work changed

The algebra tower was completed after this register was written — ℕ and ℚ added, ℤ admitted, three
correctness bugs fixed. It is worth being exact about what that did and did not do here.

**It closed no gap in this register.** All eighteen remain open. Re-verified against the tree after
the tower work merged, rather than inferred: no 𝔽₂ or bit-matrix module exists in `topology`,
`sparse`, `algebra` or `num`; `rank_of_csr` still lifts to `f64` and runs an SVD
(`chain_complex_impl.rs:113`); `LatticeComplex::betti_number` still returns the binomial;
`homology_representatives` and `dual_representative` do not exist; there is no bit-packed chain and
no `Cochain`; `logical_z` is still typed on `CausalMultiVector`; `partial_trace` still says nothing
about commutation; and neither `choi_compose` nor `choi_identity` exists.

The merged branch touched `deep_causality_quantum` and `deep_causality_topology` in one line each —
the `deep_causality_algebra` requirement moving from `0.2` to `0.3`. No functional change reached
either crate, which is the expected shape: the tower work was orthogonal to this register.

Three things did change, and each affects how a spec should be written.

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

### G-01 — No 𝔽₂ linear algebra anywhere in the workspace

**Severity S1.** Blocks R1, R4, R6.

Searched `deep_causality_topology`, `deep_causality_sparse` and `deep_causality_algebra` for GF(2),
mod-2 elimination or binary-field arithmetic. There is none. The only `mod 2` reference is a comment
in `lattice_complex/cell_splitting.rs:48` about the cup-product sign rule.

**Closure.** A bit-packed 𝔽₂ matrix with Gaussian elimination returning rank, kernel basis and image
basis. Roughly 200 lines over `u64` words using XOR and `count_ones`.

**Owner:** `deep_causality_topology`, because that is where chain complexes live and topology must
not learn about codes.

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

### G-02 — Homology rank is computed by `f64` SVD, not over 𝔽₂

**Severity S1.**

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

**Closure.** Route `betti_number` through the G-01 𝔽₂ rank for complexes used as CSS codes. Keep the
real-valued path where the complex is genuinely a manifold discretisation.

**Owner:** `deep_causality_topology`. **Note:** this is a pre-existing correctness risk independent
of QCL.

### G-03 — `LatticeComplex::betti_number` never reads the boundary matrices

**Severity S2.**

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

**Owner:** `deep_causality_topology`.

### G-04 — No homology representatives

**Severity S2.** Blocks R4, and therefore R6 and the whole gate layer.

`ChainComplex` exposes `betti_number(k) -> usize` and `boundary_matrix(k) -> Cow<CsrMatrix<i8>>`
(`traits/chain_complex.rs:53,64`). A count, not a basis. Searched for cycle, representative,
generator and basis accessors; none returns homology classes.

The paper needs actual `γ ∈ ker ∂₁ \ im ∂₂` to build any logical gate.

**Closure.** `fn homology_representatives(&self, k: usize) -> Vec<Chain>` off the G-01 kernel and
image bases.

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
existing type deliberately — but do not assume `Chain` is available.

### G-06 — No circuit type

**Severity S2.** Blocks R7.

The gate kernels exist (`gates_haruna.rs`, `logical_z/x/s/hadamard/cz/t`) but there is no
representation of a physical-gate sequence, which is what every Table 1 decomposition produces.

**Closure.** `Circuit` as an ordered list of gate applications over named qubit indices, covering
`S, CZ, H, T, CS†, CCZ, C^{m-1}Z`. `deep_causality_quantum/src/types/qpu/circuit.rs` already has a
`QuantumCircuit` and a `GateOp` enum; determine whether it can carry these or needs extending.

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

### G-10 — Stirling numbers absent

**Severity S3.** Needed only for R9, the compact `a(γ)^m` form of A.12 to A.14.

Not required for any gate in Table 1. Record it so a later spec that wants the general
`O_k(γ₁…γₘ)` family of §3.5 knows the dependency.

**Owner:** `deep_causality_num` if ever wanted. The crate now has a natural home for it: ℕ is
represented by `NaturalNumber` and ℤ by `Integer`, both in the tower, so Stirling numbers would sit
beside them rather than needing a new abstraction.

### G-11 — No quantum channel composition

**Severity S2.** Designed and verified; not implemented.

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

**Closure.** `choi_compose` as a free function in `channel.rs`, bound on `R: RealField` only, with
dimensions validated through `square_dim`. Composition is pure linear-map composition; CP and TP ride
along as consequences, so no CPTP re-validation is required.

**Owner:** `deep_causality_quantum`.

### G-12 — No identity channel

**Severity S3.** Blocks the natural unit-law tests for G-11.

**Closure.** `choi_identity(d)` giving `J(id_d)_{(i,j),(k,l)} = δ_ij δ_kl`. With it, `channel.rs`
carries a category: objects are dimensions, morphisms are Choi operators.

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

### G-15 — The orthomodular guard may be misplaced (UNRESOLVED)

**Severity: unknown.**

`qcl-design-note.md` §4 requires `adjudicate` to check `Projection::commutes_with` before folding
verdicts from forked worlds. But the forked worlds in the calibration example carry
`Uncertain<FloatType>` from a Born read-out, not `Projection<R, D>`. If that is what flows, the guard
has nothing to apply to and §4's second rule over-reaches, however sound the underlying law is.

The lens assigned to settle this did not complete before the workflow was stopped.

**Closure.** Determine what `fork` actually produces. If `Uncertain`, either narrow §4's second rule
to the projection-valued case or remove it.

**Owner:** the design note.

### G-16 — `Boundary`'s second hypothesis is not decidable at the stated tolerance (UNRESOLVED)

**Severity S2 if confirmed.**

The carriers lens found that the `Z ⊗ 1_B` **form** of `partial_trace_preservation_boundary` is
decidable from `FactorSupports` leg data alone, soundly and cheaply. The theorem's other hypothesis,
`[Z⊗1_B, M] = 0`, is not: it needs a numeric commutator, and the crate supplies commutation only to
Q-TOL tolerance while the Lean theorem is stated on exact equality.

**Closure.** Decide whether a Q-TOL-satisfied commutator is sufficient warrant to invoke an
exact-hypothesis theorem, and record the answer. This is a soundness question, not an ergonomics one.

**Owner:** `deep_causality_quantum`, with a note in `LEAN_QUANTUM.md`.

### G-17 — `partial_trace` does not document its non-preservation

**Severity S1.** This is friction F9.

`deep_causality_quantum/src/types/qgates/operator_linalg.rs:152` documents shape errors carefully and says nothing about commutation. The
refutation of `quantum.partial_trace_preservation` lives only in `LEAN_QUANTUM.md` and the Lean tree,
so a caller marginalising a validated factorization would destroy the Markov property that validate
had certified, with nothing at the call site to say so.

**Closure.** Document the non-preservation at the function, name the conditional
`partial_trace_preservation_boundary` as the sound path, and gate `predict` behind the boundary check
of G-16.

**Owner:** `deep_causality_quantum`.

### G-18 — No `Cochain` type

**Severity S3.** The dual of G-05.

`cup_product` takes `cochain: &[R]` plus a separate `degree: usize` (`cup_product/mod.rs:62-70`).
Nothing binds them, so a degree/length mismatch is caught by a runtime check rather than by the type.

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
G-01  F₂ linear algebra                    (topology, no deps)
  ├── G-02  betti via F₂ rank              (topology)
  ├── G-03  LatticeComplex betti from matrices (topology)
  └── G-04  homology representatives       (topology)
        └── G-08  Poincaré-dual representative (topology)

G-05 + G-18  Chain / Cochain type          (topology, no deps)

G-06  Circuit                              (quantum, no deps)
G-07  retype the Haruna layer              (quantum; needs G-04, G-05, G-06)
G-09  logical equivalence                  (quantum; needs G-04, G-05)

G-11  choi_compose                         (quantum, no deps; formula settled)
G-12  choi_identity                        (quantum, no deps)

G-17  document non-preservation            (quantum, no deps)
G-16  resolve the tolerance question       (quantum; blocks predict)
G-15  resolve the verdict guard            (design note)
G-13, G-14  correct the note               (design note)

G-10  Stirling numbers                     (num; only for the §3.5 general family)
```

**Two independent starting points.** G-01 unblocks the topology chain and is the load-bearing item
for the geometric-QEC example. G-11 and G-12 are self-contained, already verified, and unblock
nothing else; they can land in any order.

**What the three designed examples need.** The calibration and crosstalk examples need none of
G-01 through G-09: two feasibility lenses reproduced their headline results on shipped APIs in
scratch crates. The geometric-QEC example needs the whole topology chain plus G-06, G-07 and G-09.

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
