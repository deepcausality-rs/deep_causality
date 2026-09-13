<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# QCL-2 road map: verification against the tree

**What this is.** Every claim in `qcl2-roadmap.md` that names a type, a function, a fixture, a
theorem or an exit criterion, read against `deep_causality_quantum` as it stands on 2026-09-09
(crate version 0.2.5, workspace pin `0.2`), against the archived `add-qcl` change, and against
Lorenz & Tull (arXiv:2602.16612v1, all 71 pages). Each entry names where the claim lives, what the
tree says, and what the specification derived from the road map does instead. Severity follows the
register convention of `qcl-corrections.md`: **S1** produces a wrong answer or an unreachable exit
criterion; **S2** misdescribes a designed capability or over-reaches a theorem; **S3** is wording,
placement or dependency detail.

**What holds.** The road map's reading of the paper is accurate on every citation checked:
Definitions 14 to 16 (pp. 17 to 18), Proposition 17 (p. 19), Proposition 18 (p. 19), Definition 49
and Theorem 51 (pp. 36 to 38), Lemma 52, Examples 54 and 55 (pp. 38 to 39), Example 57 (QC, p. 40),
Example 58 (p. 41), the opening and interchange queries of §7.2 (pp. 44 to 45), Example 62 on
BLO models (p. 44), and the deferrals in §7.1 and §8. Phase 0 is landed in code: the six
corrections the road map names as prerequisites are all in the tree (V-10). The substrate the road
map builds on exists under the names it uses (V-9 lists it). The corrections below are to the road
map's engineering claims, not to its reading of the theory, with one exception (V-4).

---

## 1. Corrections that change an exit criterion

### V-1 — The live `qcl-*` specifications are empty — **S1**

**Where.** Road map §1, Phase 0 exit criterion: "corrections landed".

**Tree.** `openspec/specs/qcl-carriers/`, `qcl-code-checks/`, `qcl-decision-form/`,
`qcl-evidence/`, `qcl-experiment-design/`, `qcl-hypothesis/` and `qcl-pipeline/` each exist and
each is empty. The requirements live only in the archived deltas under
`openspec/changes/archive/2026-09-03-add-qcl/specs/` (57 requirements across the seven
capabilities plus two under `quantum-crate-scaffold`). The archive step did not merge them.

**Consequence.** A QCL-2 delta cannot `MODIFY` a requirement that has no live text. The
`.over_circuit` addition the road map puts on `QclBuilder` would have to modify `qcl-pipeline`.

**Correction.** The QCL-2 change carries a task 0 that restores the seven live specifications from
the archived deltas before any other task, and every QCL-2 requirement against an existing
capability is written as `ADDED`, never `MODIFIED`, so the change validates whether or not task 0
has run when a reader opens it.

### V-2 — The Choi-operator naturality check cannot reach the named fixtures — **S1**

**Where.** Road map §3, `check_naturality` ("compute both sides ... as Choi operators"); Phase 2
exit criterion ("residual exactly zero on the `[[18,2,3]]` torus for every diagonal Table 1 gate");
Phase 4 exit criterion (`[[18,2,3]]` and `[[32,2,4]]`); D2-2, which discusses which norm to take on
the Choi operator and not whether the operator can be formed.

**Tree.** No dense operator on more than a few qubits exists anywhere in the crate. `SimQpu`
(`qpu/sim.rs:120`) is a pure-state sampler and refuses above 24 qubits. `Hypothesis::joint_operator`
builds the joint operator on the union of the supports by `embed_on_legs`, which is the same
exponential. The code checks that do reach 18 and 32 qubits (`check_class_invariance`,
`check_clifford_action`, `is_logically_trivial`) are exact 𝔽₂ and rational computations over
supports, and the geometric-QEC consumer says so in its own header: "Nothing here is simulated".

**Measured.** The storage the road map's formulation needs, for a low-level model on `n` qubits and
a high-level model on `k` logical qubits, at one complex `f64` per entry:

| Object | Entries | `[[8,2,2]]` | `[[18,2,3]]` | `[[32,2,4]]` |
|---|---|---|---|---|
| density matrix on `n` qubits | `4^n` | 65 536 | `6.9e10` | `1.8e19` |
| Choi of the composite `τ ∘ U`, `2^n → 2^k` | `2^(2n+2k)` | `1.0e6` | `1.1e12` | `2.9e20` |
| Choi of the low-level unitary, `2^n → 2^n` | `2^(4n)` | `4.3e9` | `4.7e21` | `3.4e38` |

At 16 bytes an entry the composite Choi on `[[18,2,3]]` is 17 TB. The exit criterion is not
reachable by the formulation the road map gives it, on any machine, and D2-2's norm discussion is
downstream of an operator that cannot be formed.

**Second point.** "Residual exactly zero" in floating point holds only where every entry on both
sides is computed through the same arithmetic on exactly representable values. Clifford gates and
the ideal decoder have entries in `{0, ±1, ±i}` scaled by powers of two, and a small sum of those is
exact in `f64`. `T̄` introduces `e^{iπ/4}`, which is not representable, so its numeric residual is
of order `ε` and not zero. The exact-zero claim belongs to the symbolic path only.

**Correction.** `check_naturality` has two semantics, both named in the report:

- **Exact.** For a query whose low-level program is Pauli or Clifford, and for a diagonal Table 1
  gate, the square is decided over 𝔽₂ and `Rational<i64>` by the predicates v1 already ships
  (`clifford_conjugate`, `are_logically_equivalent`, `DiagonalPhase::phase_at`). This path has no
  width limit and its residual is exactly zero or exactly not. On this path the Phase 3 "generation
  regression" is not a comparison between two independent computations; it is the statement that
  the strict `CodeAbstraction`'s naturality check *is* v1's two predicates, and the regression
  pins that reduction.
- **Numeric.** For a general query, both sides are formed as Choi operators of the composite
  channel `2^n → 2^k` and compared in Frobenius norm. The path carries a dimension cap on
  `2^(2n+2k)` (default `2^24` entries, so `n + k ≤ 12`), errors above it with
  `NaturalityDimensionExceeded { n, k, entries, cap }` before allocating, and reports the entries it
  formed. This is the discipline D1 and D7 already apply to the code-space enumeration and the
  design cover.

The Phase 2 exit criterion becomes: the numeric path and the exact path agree on every gate they
both decide, on a fixture the numeric path can reach. The candidate is the `[[8,2,2]]` code of
`LatticeComplex::<2, _>::square_torus(2)` (`n + k = 10`); whether a 2×2 periodic lattice builds as a
valid complex is unverified in the tree, which uses `square_torus(3)` and `(4)` only, so the
fallback is a hand-built `[[4,2,2]]` chain complex (one 2-cell with `∂₂` the all-ones column, one
0-cell with `δ₀` the all-ones row, `∂₁∂₂ = 4 ≡ 0`). The torus fixtures `[[18,2,3]]` and `[[32,2,4]]`
are reached by the exact path only, and the change says so wherever it names them.

*Amended 2026-09-09.* `square_torus(2)` was probed against the tree and is a valid complex with
Betti numbers 1, 2, 1 over ℤ and 𝔽₂, weight-2 representatives, and every v1 code check holding; it
is the numeric fixture, and the `[[4,2,2]]` complex is a second small fixture rather than a fallback
(`changes/add-qcl2-abstraction/notes/open-questions-resolved.md` §1).

### V-3 — The Haruna filter's non-Clifford gates are neither exact nor numerically reachable — **S1**

**Where.** Road map §5, "Run `check_fault_tolerance` on each Table 1 gate's emitted program under
`pauli_weight(1)`"; Phase 4 exit criterion on `[[18,2,3]]` and `[[32,2,4]]`.

**Tree.** `clifford_conjugate` (`qcode/clifford_action.rs:68`) refuses `T`, `Tdg`, `Csdg`, `Ccz`
and `Cmz` on three or more qubits with `NonCliffordGate`, by design: a Pauli conjugated through a
non-Clifford gate is not a Pauli. `logical_t` emits `T`, `CS†` and `CCZ` over the support, its pairs
and its triples (`gates_haruna.rs:126`). So a single Pauli fault inserted before a `CS†` or `CCZ` in
`T̄`'s program propagates to an operator the tableau cannot carry, and V-2 rules out simulating it.

**Correction.** Fault propagation is Pauli-basis (Heisenberg) propagation: a Pauli fault is carried
through the program as a linear combination of Paulis on the qubits it has touched, exact in the
coefficients `{±1, ±i, 1/√2}` as `Complex<R>`, with a cap on the number of Pauli terms (the spread
through `CS†` and `CCZ` is bounded by the support weight, `4^w` terms at worst, and the cap errors
with `PauliTermCountExceeded` before allocating). Correctability of the resulting error set is then
decided against the stabilizer generators `LogicalBasis` carries: a term in the normalizer that is
not a stabilizer is a logical fault, and the witness names it. On the Clifford subset (`Z̄`, `X̄`,
`S̄`, `CZ̄`, `H̄`) this reduces to one tableau pass per fault and is the exact path of V-2. The
filter's output therefore has two labels, *exact* for the Clifford subset and *Pauli-basis to cap*
for `T̄`, `CS̄†`, `CC̄Z`, and the report carries the label per gate. The external-oracle facts the road
map lists (`Z̄`, `X̄` transversal and weight-1 FT; `S̄` with CZ pairs not, on the toric code) are
kept as the oracle and derived by hand in the change's notes before the test that asserts them is
written, per the anti-circularity protocol.

*Amended 2026-09-09.* The Pauli-basis expansion and its cap are unnecessary for Table 1. Haruna
defines every diagonal gate as `O_k(γ₁, …, γ_m) = exp(iπ/2^{k−1} · p₁⋯p_m)` in the logical
`Z̄(γᵢ)` (Eq. 3.63), so a propagated fault has at most `2^m` Pauli terms for a gate on `m` logical
qubits, independent of the representative weight; `T̄` under a single `X` leaves exactly
`exp(±iπ/4 Z̄(γ))`, two terms, verified at `w = 3, 4, 5`. The change's D7 replaces the cap with a
`GaugeFieldGate` carrier and a `Turns` comparison, keeps a named refusal for programs outside the
normal form, and corrects the naming: Table 1 has no `CS̄†` or `CC̄Z` rows, those are physical gates
inside `T̄`'s decomposition (`open-questions-resolved.md` §3).

### V-4 — Theorem 51 is a theorem about classical causal models — **S2**

**Where.** Road map §4.1, `check_alignment_structure`: "Theorem 51 characterises when a
constructive abstraction extends to the mechanism level ... purely in terms of the partition π ...
For a code layout it answers: can this assignment of logical qubits to physical blocks support a
mechanism-level abstraction at all?"; §10 verification table, "Thm. 51 conditions are what
`check_alignment_structure` computes".

**Paper.** Theorem 51 (p. 37) is stated for "causal models `L`, `H` in `C`" with structure categories
`Cart(S)`, `Markov(S)` or `CD(S)`, each of which has copy maps as syntax; its proof (Appendix A.1)
runs through Lemma 66 on normalised *network diagrams*, which are built from single-output boxes,
copy maps and discards. The quantum compositional models of §7 are models of `D(G, V^out)`
(Definition 59, p. 43), and the paper says of them: "this is no longer the case for compositional
models of open DAGs ... due to no longer assuming the presence of copy maps, in the quantum case due
to the famous 'no-cloning' theorem" (p. 44). Component-level abstraction is defined for them
(Definition 46), but no characterisation of it in terms of `π` is proved for them anywhere in the
paper. Section 8 names "genuine quantum causal abstraction" as future work.

**Correction.** `check_alignment_structure` computes Definition 49's `α(X)` and the three
predicates *simple*, *extra-simple* and *full* exactly, as a graph computation on the two DAGs,
and reports which held. Its documentation states the theorem's scope: when both models are
classical causal models it is Theorem 51 and decides mechanism-level abstraction; when the
low-level model is quantum it is applied as a **necessary** condition, on Remark 56's argument that
a network diagram for the opened low-level model with inputs `π(Pa(X))` cannot exist when `α(X)`
meets the low-level inputs, and the report says *necessary*, not *equivalent*. The paper's
Examples 54 and 55 are the fixtures for the classical case, as the road map has them. The
verification table's row is corrected to "Definition 49 predicates; Theorem 51 in the classical
case; necessary condition otherwise, with the argument cited".

### V-5 — The ε-composition law is missing a factor — **S2**

**Where.** Road map §6, "the composite `L → H` is an ε-abstraction with `ε ≤ ε₁·‖τ₂‖ + ε₂`".

**Derivation.** With `τ = τ₂ ∘ τ₁` and `π = π₁ ∘ π₂`, the composite square's defect is
`τ₂ τ₁ ⟦π₁π₂Q⟧_L − ⟦Q⟧_H τ₂ τ₁ = τ₂ (τ₁ ⟦π₁ Q_M⟧_L − ⟦Q_M⟧_M τ₁) + (τ₂ ⟦Q_M⟧_M − ⟦Q⟧_H τ₂) τ₁`
where `Q_M = π₂ Q`. The first term is post-composition by `τ₂` of a defect of size `ε₁`, the second
is pre-composition by `τ₁` of a defect of size `ε₂`. So `ε ≤ ‖τ₂‖_post · ε₁ + ‖τ₁‖_pre · ε₂`, with
both norms the induced norms of the composition maps in whatever norm the residuals are measured
in. In the diamond norm both are one and the law is `ε ≤ ε₁ + ε₂`. In Frobenius norm on Choi
operators neither is one in general, and the road map's law drops the second factor.

**Correction.** `Abstraction::compose` computes both constants from the superoperators of `τ₁` and
`τ₂` (the largest singular value of pre- and post-composition on Choi space, which are finite
matrices), records them and the norm in the composite report's provenance, and the exact case
`ε₁ = ε₂ = 0 ⇒ ε = 0` is Proposition 17 and is the Lean statement. The tightness test is on a
constructed pair where both constants exceed one.

### V-6 — The crosstalk consumer's factors are not the dilation of any circuit — **S2**

**Where.** Road map §2, Phase 1 exit criterion: "The crosstalk consumer runs `.over_circuit` and
reproduces its v1 result."

**Tree.** `examples/quantum_examples/qcl_examples/qcl_crosstalk/model.rs` builds every structural
candidate from `diag(0.9, 0.1)` on a qubit and `diag(0.85, 0.05, 0.05, 0.05)` on a qubit and one
parent, chosen "diagonal so that every candidate is a legal QCM". The two-leg factor's partial
trace over the child leg is `diag(0.9, 0.1)`, not the identity, so it is not the Choi operator of a
trace-preserving channel from parent to child, and no circuit with a broken wire produces it. The
consumer's factors are legal for the commutation check and nothing else.

**Correction.** The exit criterion is re-stated on the decision, not the numbers: with each of the
three admitted candidates re-expressed as a `CircuitModel` whose `Dilation` yields normalised
factors with the same parental structure, the pipeline admits the same three by Markov and C₃,
refuses the cyclic fourth at `build()`, plans `{do(Q1), do(Q2)}` at cost 2 against tomography at
200, and names H₁ the survivor. The factor values change; the screen and the plan do not.

---

## 2. Corrections to design claims

### V-7 — The flat leg convention has no input/output split — **S2**

**Where.** Road map §2, `Dilation: CircuitModel → (ProcessFactors, FactorSupports)`, "the BLO
construction".

**Tree.** `FactorSupports` (`qcm/process_factors.rs:68`) declares one leg per node under the
convention `support(Aᵢ) = {Aᵢ} ∪ Pa(Aᵢ)` with every leg defaulting to dimension 2, and
`Hypothesis::structure_from_supports` reads the DAG off that convention. Barrett, Lorenz and
Oreshkov's factor `ρ_{A|Pa(A)}` acts on `A^in ⊗ ⨂_{P∈Pa(A)} P^out`: a node has an input space and an
output space, and the parents contribute their outputs while the node contributes its input. One
leg per node cannot name both.

**Correction.** `Dilation` fixes the convention: leg `A` has dimension `d_A^in · d_A^out` with the
input index outer and the output index inner, declared through `set_leg_dim`, and the factor
`ρ_{A|Pa(A)}` is embedded as the identity on `A^out` and on every `P^in`. The commutation check then
runs on the operators BLO's proposition says commute, and the DAG the supports encode is the
circuit's. A fixture asserts that the dilation of a two-node unitary circuit is Markov for its
induced DAG at Q-TOL, which is the road map's own verification obligation.

### V-8 — Surjectivity of `τ` is a witness, not a rank — **S2**

**Where.** Road map §11, "`τ` is required to be a CPTP channel that is surjective onto the logical
space; `TypeAlignment` checks the second by rank."

**Paper.** Definition 14 asks for an epic channel; §7 does not restate the condition for QC.
Proposition 18, which the road map uses to derive upward abstractions, composes with sharp states of
the low-level type and needs every high-level sharp state to be `τ ∘ s` for some low-level `s`.

**Correction.** `TypeAlignment` carries, beside each `τ_X`, a section `E_X : X → π(X)` with
`τ_X ∘ E_X = id_X` checked at construction to the state tolerance. For a code `E` is the code-space
isometry and `τ` the ideal decoder, and the composite is the identity on `2^k` exactly. The witness
is what Proposition 18 needs, it is constructive, and on the exact path it is the statement that
`τ` restricted to the code space inverts the encoding, which `LogicalBasis` can decide.

### V-9 — What the road map builds on, and what it does not have — **S3**

The substrate exists under the names the road map uses, and its dependencies into the unified math
stack are the ones below. Nothing new is needed from that stack; every kernel the road map adds is
composed from these parts.

| Road map name | Tree | Unified math it stands on |
|---|---|---|
| `QuantumCircuit`, `GateOp`, `LogicalProgram` | `qpu/circuit.rs`, always compiled | none |
| Choi operators, `choi_compose`, CP/TP checks | `qgates/channel.rs` | `deep_causality_tensor` (`CausalTensor`), `deep_causality_linear` (Hermitian eigendecomposition under the CP check) |
| `embed_on_legs`, `partial_trace`, `frobenius_norm` | `qgates/operator_linalg.rs` | `deep_causality_tensor` |
| `Channel` (CPTP once at construction) | `carriers/quantum_channel.rs` | as above, `Tolerance` from `deep_causality_algebra::RealField::epsilon` |
| `ProcessFactors`, `FactorSupports`, `Hypothesis::compose` | `qcm/`, behind `qcm` (implies `std`) | `deep_causality` for the graph; `deep_causality_tensor` |
| `CheckReport<R>`, `Factorization::{Inherited, Rederived}` | `decision/check.rs` | `deep_causality_algebra::RealField` |
| `LogicalBasis` with Z and X stabilizers, `clifford_conjugate`, `symplectic_dual_basis` | `qcode/` | `deep_causality_homology` (`Gf2Chain`, `ChainComplex`), `deep_causality_linear` (`PackedGf2`, `rank_gf2`, `image_basis_gf2`) |
| `DiagonalPhase`, `Turns = Rational<i64>` | `qcode/diagonal_phase.rs` | `deep_causality_num_rational` |
| Table 1 emitters | `qgates/gates_haruna.rs` | `deep_causality_homology` |
| `CausaloidGraph` for `DemModel` | `deep_causality` | outside the math stack; `qcm` feature |
| Precision as a parameter | every check generic in `R: RealField` | `deep_causality_num` lifts; the three consumers run at `f32`, `f64`, `Float106` |

What the tree does not have, and the change adds: a density-matrix evolution of a `GateOp` program
on `n` qubits with noise boxes (composed from `embed_on_legs` and `apply_kraus`; cost `O(gates · 8^n)`
naive, under the V-2 cap); a `GaugeFieldGate` propagator (V-3, as amended); the Choi of a
`2^n → 2^k` composite; a Stim detector-error-model text parser; and the Frobenius-induced channel
norms of V-5. There is no SDP anywhere
in the workspace, so the diamond norm stays optional as D2-2 has it.

### V-10 — Phase 0 is landed in code — **S3**

**Where.** Road map §1, the six prerequisite corrections.

**Tree.** X-1 and X-2: `Factorization` provenance in `decision/check.rs:101`,
`CertificateNotInherited` in `error/quantum_error.rs:67`, `Hypothesis::compose` with the
disjoint-legs fast path in `qcm/hypothesis.rs:605`. X-4: `Hypothesis::intervene_mechanism` at
`qcm/hypothesis.rs:365`, with the module documentation naming `intervene_instrument` as reserved.
X-5 and X-6: `check_clifford_action` and `check_clifford_action_on_qubit` in
`qcode/clifford_action.rs`, `NotInNormalizer` at `error/quantum_error.rs:97`. X-15: the Non-Goal
and the reserved name `qcl-abstraction` in the archived `proposal.md` and `design.md`. Three
consumers run under `examples/quantum_examples/qcl_examples/`, not the two the road map's Phase 0
exit criterion asks for. The specification side of Phase 0 is V-1.

### V-11 — The Frobenius-to-diamond factor is stated, not cited — **S3**

**Where.** Road map §3 D2-2, "Frobenius on the Choi operator upper-bounds the diamond distance up to
a dimension factor"; §10, "Frobenius-on-Choi bounds diamond distance with stated factor".

**Correction.** The factor is derived in the change's notes before it is written into a docstring:
for a Hermiticity-preserving map `Φ` with unnormalised Choi operator `J(Φ)` on `d_in · d_out`
dimensions, `‖Φ‖_⋄ ≤ d_in · ‖J(Φ)‖_1 ≤ d_in · √(d_in d_out) · ‖J(Φ)‖_F`. The report carries the
factor it used, and one test compares it against a pair whose diamond distance has a closed form
(two unitary channels differing by a rotation by `θ` on one qubit). If the derivation at
implementation time gives a different constant, the docstring follows the derivation.

*Amended 2026-09-09.* The constant is `√(d_in d_out)`; the `d_in` in front was valid and loose. The
chain is `‖Φ‖_⋄ ≤ ‖J‖_1` by the `(I ⊗ A)|Ω̃⟩` argument, `‖J‖_1 ≤ √(d_in d_out) ‖J‖_F`, and from
below `‖J‖_F ≤ ‖J‖_1 ≤ d_in ‖Φ‖_⋄`, so `r / d_in ≤ diamond ≤ √(d_in d_out) · r`. Checked on the
`R_z(θ)` pair at every `θ` and on 300 random channel pairs (`open-questions-resolved.md` §2).

### V-12 — Phases 1 to 3 carry a representability decision — **S3**

**Where.** Road map §12, "Phases 1–3 are construction against a published definition and carry no
research risk."

**Correction.** V-2 and V-4 are decisions the paper does not make for the implementer: which
semantics decides a square on a register the numeric path cannot reach, and what Theorem 51 licenses
when the low-level model is quantum. Neither is research, both are design, and both are recorded as
decisions in the change so the sentence is accurate again.

### V-13 — Lean placement — **S3**

The `lean/DeepCausalityFormal/Quantum/` tree holds `Choi.lean`, `PartialTrace.lean` and
`PartialTraceCounterexample.lean` and is exempt from the CI `sorry` gate while the foundation is
extended (`LEAN_QUANTUM.md`). Proposition 17 in the exact case is a statement about two commuting
squares pasting, elementary over matrices with the crate's pair-indexed model; it goes in a new
`Abstraction.lean` bound through `lean/THEOREM_MAP.md` to the Rust regression, and the ε-law of
V-5 is Rust-only with its derivation in the docstring, as G-16 did for `√(d_B)`.

### V-14 — Feature placement — **S3**

`CircuitModel`, `TypeAlignment`, `QuerySignature`, `Abstraction`, `check_naturality`,
`check_alignment_structure`, `CodeAbstraction` and `FaultSet` need `alloc`, the tensor stack, the
homology stack and the crate's own carriers, none of which needs `std`; they compile in the default
and `no-std` builds. `Dilation` targets `ProcessFactors`, which lives under `qcm` (`qcm` implies
`std` because `deep_causality` is std-only), so `Dilation` and `.over_circuit` are `qcm`-gated.
`DemModel` takes a `CausaloidGraph` and is `qcm`-gated; the Stim text import sits behind a further
`dem` feature, as the road map's §11 has it. `BUILD.bazel` enables every feature and needs the new
one added.

### V-15 — Versioning — **S3**

The workspace pins `deep_causality_quantum = "0.2"` and the crate is at 0.2.5. The change is
additive except where a task says otherwise; release-plz derives the bump from the commit
messages and generates the changelog, and nothing in the change edits `CHANGELOG.md` or the version
by hand.

---

## 3. Summary

| # | Sev | Road map | One line |
|---|---|---|---|
| V-1 | S1 | Phase 0 exit | Live `qcl-*` specs are empty; restore first, write QCL-2 deltas as `ADDED` |
| V-2 | S1 | Phase 2, D2-2 | Choi naturality is 17 TB on `[[18,2,3]]`; two semantics, exact and capped numeric; small fixture |
| V-3 | S1 | Phase 4 | Non-Clifford gates leave a logical remainder in the `Z̄(γᵢ)` algebra, `2^m` terms, no cap (amended) |
| V-4 | S2 | §4.1, §10 | Theorem 51 is classical; quantum use is a necessary condition, documented as such |
| V-5 | S2 | Phase 5 | `ε ≤ ‖τ₂‖·ε₁ + ‖τ₁‖·ε₂`; compute both constants; diamond gives `ε₁ + ε₂` |
| V-6 | S2 | Phase 1 exit | Crosstalk factors are not Choi operators; reproduce the decision, not the values |
| V-7 | S2 | Phase 1 | One leg per node has no in/out split; leg dimension `d_in · d_out` with declared order |
| V-8 | S2 | §11 | Surjectivity by a section `E` with `τ ∘ E = id`, not by rank |
| V-9 | S3 | all | Dependency map into unified math; the five kernels the tree lacks |
| V-10 | S3 | Phase 0 | X-1, X-2, X-4, X-5, X-6, X-15 landed; three consumers, not two |
| V-11 | S3 | D2-2, §10 | Frobenius-to-diamond constant is `√(d_in d_out)`, two-sided; tested on a closed-form pair (amended) |
| V-12 | S3 | §12 | Phases 1 to 3 carry two design decisions; say so |
| V-13 | S3 | §10 | Proposition 17 goes in `Abstraction.lean`; the ε-law stays Rust |
| V-14 | S3 | §11 | Abstraction layer is `alloc`; `Dilation`, `DemModel` are `qcm`; Stim is `dem` |
| V-15 | S3 | none | release-plz owns the bump and the changelog |

**Disposition.** Every entry is applied in `openspec/changes/add-qcl2-abstraction/`: V-1 as task 0
and the `ADDED`-only rule; V-2, V-3, V-4, V-5, V-7 and V-8 as design decisions with their own
requirements and scenarios; V-6 as the re-stated Phase 1 exit scenario; V-9, V-13, V-14 and V-15 in
the proposal's impact section; V-10, V-11 and V-12 as text. The road map itself is not edited;
this register is its errata, and the change cites both.
