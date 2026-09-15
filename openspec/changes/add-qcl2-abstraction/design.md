<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## Context

`deep_causality_quantum` 0.2.5 ships QCL v1: the decision form (`Check<R>`, `CheckReport<R>`,
`Tolerance<R>`), the carriers (`Channel` CPTP-checked once), the hypothesis layer
(`ProcessFactors`, `FactorSupports`, `Hypothesis` with `intervene_mechanism`, `compose` and the
`Inherited | Rederived` provenance), the exact code checks (`LogicalBasis` with both stabilizer
families, `check_class_invariance` over the code space, `clifford_conjugate` and
`check_clifford_action`), the Table 1 emitters, the reified `QuantumCircuit`, and the builder
`QclBuilder` with three subjects and the `validate` → `Screened<R>` → `control` hand-off. Three
consumers run at three precisions. Every prerequisite the road map lists under Phase 0 is in code
(verification V-10).

Two things bound this design that the road map did not weigh. First, no dense operator on more
than a handful of qubits exists anywhere in the crate, and none can: the fixtures the code path
runs on are 18 and 32 qubits, and every check that reaches them is an 𝔽₂ or rational computation
over supports. Second, the theory the road map imports is stated for classical causal models in
one place it matters (Theorem 51) and stated exactly, with no approximate notion, throughout. The
register [`qcl2-roadmap-verification.md`](../../notes/quantum/qcl2-roadmap-verification.md) records
fifteen corrections; the decisions below apply them. The road map's own decisions D2-1 to D2-5 are
kept where the register does not touch them and cited by their numbers.

The paper is [`CausalandCompositionalAbstraction-2602.16612v1.pdf`](../../notes/quantum/CausalandCompositionalAbstraction-2602.16612v1.pdf).
Definitions and results are cited by number as they appear there.

## Goals / Non-Goals

**Goals:**

- Represent the relation between a physical process and its logical description as a value the
  pipeline holds, composes and perturbs: `Abstraction<L, H>`.
- Decide naturality on every register the code path already reaches, exactly, and on small
  registers numerically, with the report saying which path decided and to what residual.
- Make fault tolerance a predicate over an enlarged query signature rather than a new kind of
  check, and produce the Haruna filter as the first result the substrate alone could not.
- Replace certificate inheritance with abstraction composition under a stated, computed law.
- Validate a decoder's classical picture against the physical circuit as a causal statement.

**Non-Goals:**

- Implementing a decoder. `τ` is validated, never built (D2-5).
- Diamond-norm evaluation. There is no SDP in the workspace; the report states the Frobenius
  proxy and its amplification (D2-2).
- Claiming thresholds, distances or asymptotic suppression. The fault-tolerance claim is "holds
  under fault set `F` to residual `ε`" (D2-4).
- Making a general Barrett–Lorenz–Oreshkov process operator a compositional model. The circuit
  is carried and BLO's dilation theorem is cited (road map §9 rule 5; paper §8).
- Cyclic structures, device models, graph traversal and topology ownership, as in v1 §8.
- Editing the road map. The register is its errata.

## Decisions

### D1. Two semantics paths, and the report names which one decided

**Measured.** The Choi operator of the composite `τ ∘ U`, a channel `2^n → 2^k`, has `2^(2n+2k)`
entries: `1.0e6` on `[[8,2,2]]`, `1.1e12` (17 TB) on `[[18,2,3]]`, `2.9e20` on `[[32,2,4]]`. The Choi
of the low-level unitary alone is `2^(4n)`. The road map's `check_naturality` cannot be formed on
the fixtures its exit criteria name, on any machine.

`CircuitModel<R>` therefore carries two semantics functors and `check_naturality` runs whichever
the query admits:

- **Exact.** A program of Pauli and Clifford gates is carried as its symplectic action on
  `(x, z)`, through `clifford_conjugate`; a diagonal Table 1 gate is carried as a `DiagonalPhase`
  with its `Rational<i64>` phase polynomial. A square commutes exactly when the images agree up
  to stabilizers through `LogicalBasis::are_logically_equivalent` and the phase ratio is integral
  through `DiagonalPhase::phase_at`. No width limit; the residual is zero or it is not.
- **Numeric.** A general program is carried at the Kraus level, never as a Choi operator of its
  own: each gate's unitary is embedded on the register through `embed_on_legs` and multiplied into
  the running Kraus family, and a noise box multiplies the family out by its own Kraus operators.
  The Choi operator is formed once, for the composite `2^n → 2^k` channel `τ ∘ U`, from the
  family `{K_j U}` through the shipped `choi_from_kraus`, and the two sides of the square are
  compared in Frobenius norm against `Tolerance::state()`. The distinction matters at the fixture:
  the composite Choi on `[[8,2,2]]` is `2^20` entries, while the Choi of the 8-qubit program alone
  would be `2^32`, and evolving `2^16` basis inputs through the program to build the composite
  column by column would cost about `5·10^13` flops. The path carries two caps, both counted on
  `NumberType` with checked products and both refused before allocating: the composite entry count,
  default `2^24` (so `n + k ≤ 12`), as `NaturalityDimensionExceeded { n, k, entries, cap }`, and
  the Kraus family size `∏ kᵢ` over the noise boxes, default `2^12`, as
  `KrausFamilyExceeded { operators, cap }`. A fault set inserts one error channel with at most four
  Kraus operators, so the fault path never approaches the second cap. This is the discipline D1 and
  D7 of `add-qcl` apply to the code-space enumeration and the design cover.

The report carries `SemanticsPath::{Exact, Numeric}` beside the norm and the amplification, and a
scenario asserts that a query decidable by both paths gets the same verdict on a fixture the
numeric path reaches.

*Alternative rejected.* State-vector simulation of the low-level side only. `SimQpu` caps at 24
qubits and produces samples, not channels; a pure-state path cannot carry noise boxes and cannot
form the composite channel the square compares.

### D2. The generation regression is a reduction, and it is written first

On the exact path the strict `CodeAbstraction`'s naturality check *is* v1's two predicates:
`Z̄(γ) ↦ X̄(γ̃)` up to stabilizers is `check_clifford_action`, and the phase ratio integral on the
code space is `check_class_invariance`. The regression therefore does not compare two independent
computations. It pins the reduction: for a noiseless physical model and every Table 1 gate on every
fixture, `check_naturality` on the strict code abstraction and the v1 stage return the same verdict
and, where the v1 stage names a witness, the same witness. It is the first test Phase 3 writes, and
nothing about a noisy or composite abstraction is claimed until it passes. The numeric path's
agreement with the exact path on the small fixture (D1) is the second half of the same bridge.

### D3. The dilation fixes a leg convention the flat store does not have

`FactorSupports` declares one leg per node under `support(A) = {A} ∪ Pa(A)` with every leg
defaulting to a qubit, and `structure_from_supports` reads the DAG off that. Barrett, Lorenz and
Oreshkov's factor `ρ_{A|Pa(A)}` acts on `A^in ⊗ ⨂_{P∈Pa(A)} P^out`. One leg per node cannot name
both spaces.

`Dilation` fixes the convention: leg `A` has dimension `d_A^in · d_A^out`, declared through
`set_leg_dim`, with the input index outer and the output index inner in the row-major layout;
`ρ_{A|Pa(A)}` is embedded as the identity on `A^out` and on every `P^in`. The commutation check then
runs on the operators BLO's proposition says commute, and the DAG the supports encode is the
circuit's induced DAG (paper Example 61: a vertex per encoder input, per unitary box and per
measurement output, an edge per wire). A fixture asserts that the dilation of a two-node unitary
circuit is Markov for its induced DAG at Q-TOL.

*Consequence for v1's open question.* A composite of two circuits is a circuit, and its dilation is
the induced factorization D9 and X-2 could not construct. `CircuitModel::glue` followed by
`Dilation` supplies it; `CertificateNotInherited` remains the v1 behaviour for a caller who holds
only marginals.

### D4. `τ` carries a section, and surjectivity is its witness

Definition 14 asks for an epic `τ_X`; §7 does not restate the condition for QC; Proposition 18,
which derives the upward abstraction, needs every high-level sharp state to be `τ ∘ s` for a
low-level `s`. `TypeAlignment` therefore carries, beside each `τ_X : π(X) → X`, a section
`E_X : X → π(X)` and checks `τ_X ∘ E_X = id_X` at construction against `Tolerance::state()`,
refusing with `SectionNotInverse` otherwise. For a code `E` is the code-space isometry and `τ` the
ideal decoder built from the stabilizer generators; the composite is the identity on `2^k`. On the
exact path the section is the statement that `τ` inverts the encoding on the code space, which
`LogicalBasis` decides. Upward abstractions are derived from the downward one by composing with the
low-level sharp states per Proposition 18 and are not stored (road map §3).

*Alternative rejected.* A rank test on `τ`'s Choi operator. Rank is a numeric-path quantity, has no
exact-path form, and does not exhibit the state Proposition 18 needs.

*Amended during implementation (2026-09-09).* `τ_X` and `E_X` are `QcMorphism` values, not
`Channel`s: a `Channel` carries a dense Choi operator of `(d_in d_out)²` entries, which is
`2^32` for an eight-qubit unitary, and the alignment never reads it. The alignment entries carry a
side. A code's numeric abstraction is posed in the shape of Example 58, `E ; U ; N`: the low-level
model is the encoder unitary `W` on all `n` physical wires followed by the physical program, with
the first `k` wires as inputs, so its input type is the logical space and aligns by the identity,
and its output type is the physical space and aligns through the ideal decoder. Posing the square
on the full `2^n` input space instead fails for every non-Pauli gate, since the decoder's syndrome
corrections do not commute with `S̄` or `T̄` off the code space: the residual on `[[8,2,2]]` was
`30.5`. The encoder unitary is `|x⟩|0…0⟩ ↦ |x̄⟩`, completed by the coset states
`|c, χ⟩ = |S_X|^{-1/2} Σ_s χ(s) |c + s⟩` of the `X`-stabilizer group, one per coset and character,
which is an orthonormal basis containing the code words; Gram–Schmidt over the standard basis was
tried first and costs `O(d³)` in a debug build. It is held in a `CircuitBox::Kraus`, a box given by
Kraus operators alone, added for the same reason.

### D5. `check_alignment_structure` states what Theorem 51 licenses

Theorem 51 is stated for causal models in a Markov, cd or Cartesian structure category, each of
which has copy maps as syntax, and its proof runs through Lemma 66 on normalised network diagrams.
Quantum compositional models of DAGs (Definition 59) have no copy maps, and the paper proves no
characterisation of component-level abstraction for them (§8 names it future work).

The stage computes Definition 49's `α(X)`, the low-level vertices with a directed path to `π(X)`
avoiding `π(Pa(X))`, and the three predicates *simple*, *extra-simple* and *full*, as a graph
computation on the two DAGs, and reports which held with the offending pair as witness. Its
documentation states the scope in one sentence each way: for two classical models it is Theorem 51
and decides mechanism-level abstraction; for a quantum low-level model it is a necessary condition,
on Remark 56's argument that no network diagram for the opened model with inputs `π(Pa(X))` exists
when `α(X)` meets the low-level inputs, and the report reads `Necessary`, not `Equivalent`. The
paper's Examples 54 and 55 are the classical fixtures. It runs in `validate` before any operator is
formed, as the road map has it.

### D6. The ε-law has two constants, and both are computed

With `τ = τ₂ ∘ τ₁` and `π = π₁ ∘ π₂`, the composite defect splits as
`τ₂ (τ₁ ⟦π₁ Q_M⟧_L − ⟦Q_M⟧_M τ₁) + (τ₂ ⟦Q_M⟧_M − ⟦Q⟧_H τ₂) τ₁`, so
`ε ≤ ‖τ₂‖_post · ε₁ + ‖τ₁‖_pre · ε₂`, with the constants the induced norms of post-composition by
`τ₂` and pre-composition by `τ₁` in the norm the residuals are measured in. In the diamond norm both
are one. In Frobenius norm on Choi operators neither is one in general, and the road map's law
dropped the second factor.

In Frobenius norm the two constants are the Frobenius-induced norms `‖τ₂‖_{F→F}` and
`‖τ₁‖_{F→F}` of the channels themselves: post-composition acts on a Choi operator as `id ⊗ τ₂`,
pre-composition as the transpose of `τ₁` on the input factor, and neither the tensor with an
identity nor a transposition changes the induced norm on a Hilbert-space norm. So the constants do
not need the composition superoperators on Choi space, whose dimension `(d_in d_out)²` reaches
`10^6` on the fixture. Each is the largest singular value of the channel's natural representation,
a `d_out² × d_in²` matrix (`16 × 65 536` for the `[[8,2,2]]` decoder), obtained as the square root
of the largest eigenvalue of its `d_out² × d_out²` Gram matrix through the shipped
`eigen_hermitian`. `Abstraction::compose` computes both, records them with the norm in the
composite report's provenance, and the exact case `ε₁ = ε₂ = 0 ⇒ ε = 0` is Proposition 17
and is the Lean statement. The tightness test is on a constructed pair where both constants exceed
one, so a law with either constant assumed to be one fails it.

### D7. Faults propagate through Table 1 in the algebra of the logical `Z̄`s, with no cap

**What the road map assumed, and a first draft of this design repeated.** A Pauli fault carried
through `T̄`'s program of `T`, `CS†` and `CCZ` branches into a Pauli superposition bounded only by
`4^w` for the representative weight `w`, so the propagator needed a term cap and the qLDPC family,
with `w` in the tens to hundreds, was out of reach.

**What Haruna's construction says.** Every diagonal gate of Table 1 is defined as a function of the
logical `Z̄(γᵢ)` operators and its physical decomposition is derived from that by expanding modulo 2:
`S̄(γ) = exp(iπ/2 · (I − Z̄(γ))/2)` (Eq. 3.14), `CZ̄(γ₁, γ₂) = exp(iπ · p₁p₂)` (3.37),
`C^{m−1}Z̄ = exp(iπ · p₁⋯p_m)` (3.49), `T̄(γ) = exp(iπ/4 · (I − Z̄(γ))/2)` (3.56), and in general
`O_k(γ₁, …, γ_m) = exp(iπ/2^{k−1} · p₁⋯p_m)` with `pᵢ = (I − Z̄(γᵢ))/2` (3.63). Such a gate `G`
lies in the commutative algebra generated by the `m` Paulis `Z̄(γᵢ)`, of dimension `2^m`.
Conjugating a Pauli fault `P = X^a Z^b` gives `G P G† = P · (P† G P) G†`, and `P† G P` is `G` with
each `Z̄(γᵢ)` replaced by `(−1)^{⟨a, γᵢ⟩} Z̄(γᵢ)`. The remainder `(P† G P) G†` is therefore a
function of the `m` logical parities with at most `2^m` Pauli terms, each `P · Z̄(γ_S)` for a subset
`S` of the gate's logical qubits, and the representative weight `w` never enters. For a single-block
gate `O_k(γ)` and a fault with `⟨a, γ⟩ = 1` the remainder is `exp(iπ/2^{k−1}) · O_{k−1}(γ)†`, one
level down the Clifford hierarchy: `Z̄` for `S̄`, `exp(±iπ/4 Z̄)` for `T̄`; with `⟨a, γ⟩ = 0` it is
the identity. Z-type faults commute with every diagonal gate and never spread.

**Amended during implementation (2026-09-09).** The decision has a second clause for the Clifford
path: `H̄` and `X̄` propagate a fault through the tableau with the identity remainder, and `H̄`
spreads a weight-one fault to a Pauli of larger weight that carries a logical operator. A fault is
therefore tolerated when the remainder is constant *and* the propagated Pauli has no more weight
than the fault, so a recovery built for the set's weight still corrects it; the witness names the
weight and, through `LogicalBasis::is_logically_trivial`, whether the carried operator is a
non-trivial logical. The remainder tables come out with their global phase: `[1/4, 3/4]` for `S̄`,
which is `i · Z̄(γ)`, and `[1/8, 7/8]` for `T̄`. Fault sets are `Query::Fault` values, a Pauli box
inserted after a node of the circuit; `from_dem` takes the mechanisms' Pauli supports, which
`DemModel` (D10) will produce.

**Checked.** On `w = 3, 4, 5` with the crate's polynomial `(2n³ − 3n² + 2n)/8`, which equals the
paper's product entry for entry, `T̄ Z₀ T̄† = Z₀` and `T̄ X₀ T̄†` has exactly two Pauli terms, `X₀`
and `X₀ Z̄(γ)`, of modulus `1/√2` each. A first algebraic attempt predicted `2^{w−1}` terms and was
wrong; the numerics caught it and Eq. (3.63) explains it. `C³Z` on four qubits sends `X₀` to
`X₀ · CCZ₁₂₃`, a genuine non-Clifford remainder with eight terms, which is `2^m` for `m = 4`.

**The carrier.** `GaugeFieldGate`: the blocks `γ₁, …, γ_m` as `Gf2Chain`s and the phase function
on `{0,1}^m` as `2^m` values of `Turns`, which is Table 1's third column and `DiagonalPhase`
generalised from one block to `m`. Conjugation by a Pauli costs `m` inner products through
`Gf2Chain::inner` and yields another `GaugeFieldGate` on the same blocks. Clifford layers (`H̄`'s
transversal `H`, `S̄`, `CZ̄`) go through the tableau as before. The propagator therefore needs no
Pauli-basis expansion and no term cap for any Table 1 gate or any product of them; the emitter's
tuple cap on `logical_t` remains, because the physical program is a real cost, but the fault
analysis never materialises the program.

**The decision.** After the fault's own Pauli is recovered, the remainder acts on the code space as
`exp(2πi · Δg(p̂))` with `p̂` the logical parity operators. It is a non-trivial logical unitary
exactly when `Δg` is not constant on `{0,1}^m`, because the `γᵢ` are independent homology classes
and every `Z̄(γ_S)` with `S ≠ ∅` is a non-trivial logical operator. So the fault is tolerated iff the
`2^m` values of `Δg` agree, a comparison of `Turns`, and the witness is the flipped parity pattern
together with the remainder's phase table. No exact cyclotomic arithmetic is needed for the
decision; the Pauli coefficients of the remainder, sums of eighth roots of unity, are reported on
the numeric path only. The Haruna filter's answer under `pauli_weight(1)` then follows from the
algebra for every CSS code and every `w`: `Z̄` and `X̄` do not spread, and every other Table 1 gate
fails a single X-type fault on its support, which confirms and extends the road map's oracle. The
implementation computes the verdict; the algebra is the provenance of the test's expected values.

**Where a cap remains.** A user program that interleaves two or more non-Clifford layers with
non-diagonal Cliffords has no normal form of polynomial size that this design knows; there the
propagator refuses with `NoPropagationNormalForm` naming the layers, rather than expanding to a cap.
The per-gate label is `Exact` throughout Table 1, and `PauliBasisToCap` is retired.

*Naming.* Table 1 has no logical `CS̄†` or `CC̄Z` rows; `CS†` and `CCZ` are physical gates inside
`T̄`'s decomposition (3.59). The logical controlled gates are `CZ̄`, `C^{m−1}Z̄` and, in general,
`O_k` with `m ≥ 2`. Text elsewhere in this change that names `CS̄†` and `CC̄Z` as Table 1 gates
means `T̄` and `C^{m−1}Z̄`.

### D8. The Frobenius-to-diamond factor is `√(d_in d_out)`, and the bound is two-sided

For any linear map `Φ : L(X) → L(Y)` with the crate's unnormalised Choi operator
`J(Φ) = Σ_{ij} |i⟩⟨j| ⊗ Φ(|i⟩⟨j|)`:

1. `‖Φ‖_⋄ ≤ ‖J(Φ)‖_1`. Every unit vector on `X ⊗ X` is `(I ⊗ A)|Ω̃⟩` with `|Ω̃⟩ = Σᵢ |i⟩|i⟩` and
   `‖A‖_F = 1`, so `(Φ ⊗ id)(|ψ⟩⟨ψ|) = (I ⊗ A) J(Φ) (I ⊗ A)†` has trace norm at most
   `‖A‖_∞² ‖J(Φ)‖_1 ≤ ‖J(Φ)‖_1`. No Hermiticity assumption is used.
2. `‖J‖_1 ≤ √(rank J) · ‖J‖_F ≤ √(d_in d_out) · ‖J‖_F`, Cauchy–Schwarz on the singular values.
3. `J(Φ)/d_in = (Φ ⊗ id)(ω)` for the maximally entangled state `ω`, so `‖J‖_F ≤ ‖J‖_1 ≤ d_in ‖Φ‖_⋄`.

For a Frobenius residual `r` between two channels, `r / d_in ≤ ‖E − F‖_⋄ ≤ √(d_in d_out) · r`.
The register's first candidate carried an extra factor `d_in` on the upper bound; it was valid and
loose. The naturality report carries both ends, so a Frobenius residual of zero certifies a
diamond distance of zero.

**Checked.** On `id` against `R_z(θ)` on one qubit, whose diamond distance is `2 sin(θ/2)`:
`‖J‖_F = 2√2 sin(θ/2)` and `‖J‖_1 = 4 sin(θ/2)` at every `θ` in `(0, π]`, so the upper bound
exceeds the distance by the constant `2√2` and the lower bound `r/2 = √2 sin(θ/2)` sits below it.
Over 300 random pairs of two-Kraus channels on `d = 2` and `d = 3`, a diamond lower bound from 61
pure inputs never exceeded `‖J‖_1`, and `‖J‖_1` never exceeded `√(d_in d_out) · ‖J‖_F`, the worst
ratio being 1.98 against the factor 2. The θ-sweep is the requirement's test.

### D9. Interchange queries are validated at construction

`Inc(S₁, …, Sₙ)` is defined only on pairwise disjoint, each parallelisable, subsets of non-input
vertices (paper §7.2: no directed path between two members of one set). `QuerySignature` checks
that on the low-level DAG when the query is built and refuses with `NotParallelisable` naming the
path, so no semantics is ever asked for a query the paper does not define. `Open(S)` is the
mechanism-level intervention v1 names `intervene_mechanism`; the query wrapper adds a name and
changes nothing (road map §3).

### D10. The decoder is a black box, and the detector model is a `CausaloidGraph`

`DecoderAbstraction` is `Abstraction<CircuitModel, DemModel>` with `τ` the decoder's channel from
syndromes to logical outcomes, supplied by the caller as a `Channel` or a classical stochastic
matrix. `DemModel` is a `CausaloidGraph` over detector and observable variables with error
mechanisms as latent parents, and the Stim detector-error-model text format is one constructor
behind the `dem` feature. Nothing in the crate decodes, and no `Decoder` trait appears; if one does,
D2-5 has been violated. The logical attribution query enumerates the fault-set queries whose squares
fail and ranks them by residual; because the low-level model is a circuit, entanglement-mediated
correlations are represented as such rather than as a classical common cause.

*Amended during implementation (2026-09-15).* `τ` enters as a stochastic matrix over outcome
strings and is lifted into QC as a morphism on the trivial quantum system with one scalar block per
non-zero entry, the FStoch embedding; a `Channel` with a dense Choi operator would carry nothing
the blocks do not. The type alignment carries it as a classical output map beside its quantum
entries, where classical wires otherwise align by the identity. The `DemModel` answers `Io` and
`Fault` queries: a fault `X` on mechanism `k` is the distribution with `k`'s flip pattern applied
once more, which is what a Pauli injected at the mechanism's circuit location does to a classical
record. The decoder abstraction's signature is `Io` plus one such pair per circuit location the
caller names; a location the model does not represent receives a phantom mechanism, probability
zero and no flips, so an omitted correlated error fails at its own location with a residual of the
order of `√2` while every other square carries the nominal mismatch of the order of the omitted
probability. Attribution is the numeric fault check of the abstraction, every fault against the
model's nominal prediction, sorted by residual. The memory experiment's record opens `2^5`
measurement branches on top of the noise branches, so the numeric semantics now drops branches and
traced operators that are exactly zero, an exact pruning, and the fixture raises the operator cap.

### D11. Feature placement

`CircuitModel`, `TypeAlignment`, `QuerySignature`, `Abstraction`, `check_naturality`,
`check_alignment_structure`, `CodeAbstraction` and `FaultSet` need `alloc`, the tensor stack, the
homology stack and the crate's carriers; they compile in the default and `no-std` builds and are
not gated. `Dilation`, `.over_circuit`, `CircuitModel::glue`'s factorization and `DemModel` are
`qcm`-gated, because `ProcessFactors` and `CausaloidGraph` are, and `qcm` implies `std`. The Stim
parser is `dem`, which implies `qcm`. `BUILD.bazel` enables `dem` beside `qpu`.

### D12. The seven live specifications are restored before implementation

The live `openspec/specs/qcl-*/` directories are empty; the 57 requirements live only in the
archived `add-qcl` deltas. Task 0 restores them from the archive as `ADDED` requirements, exactly
as archived, and validates. Every QCL-2 requirement against an existing capability is written as
`ADDED` so the change validates with or without task 0 having run.

### D13. The crosstalk exit criterion is the decision, not the numbers

The crosstalk consumer's two-leg factor `diag(0.85, 0.05, 0.05, 0.05)` traces over the child to
`diag(0.9, 0.1)`, not the identity; it is not the Choi operator of a trace-preserving channel and
no circuit's dilation produces it. The Phase 1 exit criterion is re-stated: with each admitted
candidate re-expressed as a `CircuitModel` whose dilation yields normalised factors with the same
parental structure, the pipeline admits the same three by Markov and C₃, refuses the cyclic fourth at
`build()`, plans `{do(Q1), do(Q2)}` at cost 2 against tomography at 200, and names H₁ the
survivor. The factor values change; the screen and the plan do not.

### D14. Fixtures

The exact path runs on `[[18,2,3]]` (`square_torus(3)`) and `[[32,2,4]]` (`square_torus(4)`) as the
code path does today. The numeric path runs on `[[8,2,2]]` from `square_torus(2)`, which a probe against the working
tree confirmed is a valid complex: 4 vertices, 8 edges, 4 faces, Euler characteristic 0, `∂₁∂₂ = 0`
over ℤ, every face boundary four distinct edges with coefficients ±1, every edge in exactly two
faces, Betti numbers 1, 2, 1 over both ℤ and 𝔽₂; `derive_code` reads `[[8, 2]]` with weight-4
checks, both logical representatives have weight 2, and `check_class_invariance` for `Z̄, S̄, T̄` and
`check_clifford_action_on_qubit` for `H̄` hold on both qubits. Its composite Choi is `2^20` entries,
inside the numeric cap. The hand-built `[[4,2,2]]` chain complex (one 2-cell with `∂₂` the all-ones
column, four 1-cells, one 0-cell with `δ₀` the all-ones row, `∂₁∂₂ = 4 ≡ 0`, `k = 2`) stays as a
second small fixture, not as a fallback. Distance 2 means the numeric fault path on `[[8,2,2]]`
detects a weight-one fault but has no unique recovery for it; fault tolerance is decided on the
exact path, and the numeric path's role on faults is agreement on the remainder's Pauli terms. The classical fixtures for
D5 are the paper's Examples 54 and 55. Every published wall-clock figure carries the machine
(M3 Max, 16 cores, 128 GB).

### D15. Counts and caps

Every cap and every count is ℕ on `NumberType`, every dimension product is `checked_mul`, and every
real quantity follows `FloatType`, as `add-qcl` D6 has it. Configuration literals enter through
`lift`; `f64` appears at the display boundary and nowhere else.

### D16. Fresh inputs carry the type of what they replace, and the logical measurement query is deferred

An `Open` query replaces a mechanism's outputs by fresh inputs, so a fresh wire has the type of the
output it replaces. `TypeAlignment::extended` therefore takes the query: for `Open` it copies
output-side entries to the fresh wires as entries for both sides and never copies input-side ones;
for `Inc`, whose fresh wires are copies of the model's inputs and carry the input type, it copies
input-side entries with their side and never output-side ones. For a code in the Example 58
shape this makes `Open(S̄) ↦ Open(π(S̄))` typed: the opened low-level query has the logical inputs
and `n` fresh physical inputs, the latter aligned through the recovery, and both sides of the
square read `Tr_k ⊗ τ`. The opened square on `[[8,2,2]]` is a ten-qubit register and the default
cap refuses it; `check_naturality_on` checks a subset of the signature so the `Io` square is still
decided there, and the opened square is decided on `[[4,2,2]]`.

`Observe(Ō)` to the measurement of the logical operator `Z̄(γ)` is not in this change. The
alignment carries classical wires as the identity and requires equal outcome counts across the
square; a computational measurement of the physical support has `2^|γ|` outcomes where the logical
measurement has two, so the low-level side needs either an operator-valued observe query or a
classical coarse-graining `τ` (the parity of the support). Either is a spec extension and goes to a
follow-up change; the fault-tolerance predicate (D7) does not depend on it.

### D17. The three chains are built in the crate and shown in the examples

The consumers of the composition law live as constructors in `abstraction/chains.rs`, so the tests
and the three example binaries share one construction each and the examples only print.

*Concatenation.* The inner code's `n` physical qubits form `n / k_out` blocks, each encoded by the
outer code. The low-level model carries the inner encoder on the wires that stand for the middle
qubits (middle qubit `j` is logical qubit `j mod k_out` of block `j / k_out`), one outer encoder per
block, and the inner program with each gate replaced by the outer code's emitted program for it on
the block's wires; `Y` is `X` then `Z` up to a global phase. A two-qubit gate across blocks would
need a transversal gadget between code blocks and is refused by name, which is why the inner
`CZ̄` of `[[4,2,2]]`, whose representative pairs a qubit of each block, is the refusal the tests and
the example show; `Z̄` and `X̄` compose exactly. The first link aligns each block's logical wires by
the identity on the input side and the block through the outer recovery on the output side.

*Code switching.* The gadget is decode-A-then-encode-B, with an optional Kraus family on one
logical wire between the two, and it is the low-level query of the first link, aligned with code
B's model by the physical identity on the wider register; the second link is code B's abstraction.
The section check `τ ∘ E = id` of an alignment on eight qubits has a Choi operator of `2^32`
entries, so `TypeAlignment` now checks it through the Gram identity
`‖J(A) − J(B)‖²_F = Σ|Tr A_a†A_a'|² + Σ|Tr B_b†B_b'|² − 2Σ|Tr A_a†B_b|²`, quadratic in the operator
count and linear in `d_in · d_out`, without forming the Choi operator.

*Distillation round.* The low-level model encodes, depolarises every physical qubit with
probability `p`, and runs the encoded `T̄ H̄` on every logical qubit; the middle model is the same
without the noise; the high level is `T H`. The first link measures the noise, the second is the
code, and the composite's residual is the noise the ideal recovery does not remove. The paper
defers this case (§7.1); the example claims the residual it measures.

*The constants.* `‖τ‖_{F→F}` is computed from the natural representation `N = Σ_k K ⊗ conj(K)`
(`d_out² × d_in²`) as the square root of the largest eigenvalue of the smaller Gram matrix through
the shipped Hermitian eigensolver, per classical block, taking the largest. The tightness fixture
of the tests is a three-wire chain through two partial traces with the traced wires fully
depolarised at the level below, which is what makes post-composition by a trace attain its constant
`√2`; pre-composition by a trace always does.

*Unified math advance (2026-09-15).* `deep_causality_stats` and `deep_causality_rand` became
generic in the scalar. The quantum crate reaches `stats` at two call sites,
`bernoulli_proportion` and `bernoulli_standard_error`, both already generic in `T: RealField +
FromPrimitive`, and reaches `rand` nowhere; every QCL-2 kernel is Kraus-level linear algebra with
no sampling, and the consumers run at `f32`, `f64` and `Float106` as planned. No adjustment to
the plan follows from the advance.

## Risks / Trade-offs

**[The numeric path reaches almost nothing physical]** → It is the bridge to the exact path, not the
workhorse. Its job is to agree with the exact path on the small fixture and to carry noise boxes and
general channels where the exact path cannot. The cap makes the limit visible before it is a hang.

**[The fault analysis was expected to scale with the representative weight]** → It does not (D7):
the remainder lives in the `2^m`-dimensional algebra of the gate's logical `Z̄`s, and `m ≤ 3` for
every Table 1 gate but `C^{m−1}Z̄`. The qLDPC family is reached at no extra cost. Only the emitter's
tuple cap on `logical_t` remains, and the fault analysis does not call the emitter.

**[A general program outside Table 1's two layer types has no normal form]** → The propagator refuses
it by name (`NoPropagationNormalForm`) rather than expanding to a cap; the numeric path under its own
caps is the fallback where the register is small.

**[Theorem 51's quantum scope is a necessary condition only]** → Stated on the type, in the report
and in the docs (D5). A layout that fails the precheck fails for a reason the paper proves; one that
passes has not been shown to support a mechanism-level abstraction, and the report does not say it
has.

**[The ε-law's constants can be large in Frobenius]** → They are computed, not assumed, and the
report shows them. A caller who needs the tight bound needs the diamond norm, which is optional and
not on the critical path.

**[The dilation is bigger than the process operator]** → A leg of dimension `d_in · d_out` per node
squares the per-node dimension. The design-time positioning absorbs it; the cap reports it.

**[Restoring the live specifications from the archive re-opens a closed change]** → It copies text
that was reviewed and archived, adds nothing, and is one mechanical task with `openspec validate`
as its check.

**[The numeric path on `[[8,2,2]]` cannot decide fault tolerance]** → Distance 2 has no unique
recovery for a weight-one fault. The exact path decides fault tolerance on `[[18,2,3]]` and
`[[32,2,4]]`; the numeric path's fault scenarios on `[[8,2,2]]` compare the remainder's Pauli terms
against the exact path's, which is the agreement the bridge needs.

**[Stim's format changes]** → The parser handles the `error(p) D… L…` and `detector`/`logical_observable`
lines of the current text format, sits behind `dem`, and `DemModel` takes any `CausaloidGraph`.

## Migration Plan

Additive. No shipped signature changes. `Validate` on the model and plant subjects is unchanged;
the circuit subject is a fourth constructor. Callers holding only marginals keep v1's behaviour,
including `CertificateNotInherited`. release-plz derives the bump from the commit messages.

## Mathematical foundation

Nothing needs to be added to the unified math stack for this change. Every kernel the design names
is composed from primitives that ship:

| Kernel | Primitive | Crate |
|---|---|---|
| Kraus-level evolution, composite Choi | `embed_on_legs`, `apply_kraus`, `choi_from_kraus`, `CausalTensor::matmul` | `deep_causality_quantum`, `deep_causality_tensor` |
| ε-law constants (D6) | Gram matrix of the natural representation, `eigen_hermitian` | `deep_causality_linear` |
| code-space isometry `E` and decoder `τ` for the numeric path | code projector `∏ (I + Sᵢ)/2`, `eigen_hermitian` or `qr` for an orthonormal basis of its range | `deep_causality_linear` |
| `GaugeFieldGate` (D7) | `Gf2Chain::inner`, `Rational<i64>` as `Turns` | `deep_causality_homology`, `deep_causality_num_rational` |
| `α(X)`, parallelisable sets, induced DAG | reachability on the model's own DAG | in-crate |
| logical triviality, normalizer membership | `PackedGf2`, `rank_gf2`, `image_basis_gf2` through `LogicalBasis` | `deep_causality_linear`, `deep_causality_homology` |
| Frobenius residuals and the two-sided bound (D8) | `frobenius_norm`, `eigen_hermitian` for the trace norm where reported | in-crate, `deep_causality_linear` |
| classical channels in QC (`DemModel`) | Kraus family `{√p(y|x) |y⟩⟨x|}` through `Channel::from_kraus` | in-crate |

Three things were considered and are not needed. Exact cyclotomic arithmetic over `Q(ζ₈)` for the
remainder's Pauli coefficients: the fault decision compares `Turns` (D7), and the coefficients are
reported on the numeric path only. A complex SVD: the constants of D6 come from a Hermitian Gram
matrix, and `svd` exists in `deep_causality_linear` in any case. A diamond-norm SDP: out of scope by
D2-2, with the two-sided Frobenius bound of D8 in its place. The change can be implemented against
the stack as it stands.

## Open Questions

None. The three questions the first draft carried are closed above: `square_torus(2)` builds and is
`[[8,2,2]]` (D14); the Frobenius-to-diamond constant is `√(d_in d_out)` with a two-sided chain
(D8); the propagator's term cap is unnecessary for Table 1 and is replaced by the `GaugeFieldGate`
carrier, with a named refusal for programs outside its normal form (D7). The evidence for all three
is in [`notes/open-questions-resolved.md`](notes/open-questions-resolved.md).
