# add-quantum-crate — per-phase commit messages

One prepared commit message per completed phase (tests + clippy green at each
boundary). Copy-paste ready; nothing here is committed automatically.

---

## Phase 0 — blockers gated

```
doc(openspec): add-quantum-crate Phase 0 — blockers gated, witness corrects the counterexample sign

- 0.1: numerically witnessed the partial-trace counterexample — [X,Y]=0 but
  [Tr₂X,Tr₂Y] = +4i·σy; the previously recorded −4i·σy was wrong (hand-check
  agrees: [σx+σz, σx−σz] = 2[σz,σx] = +4iσy). Sign corrected in proposal,
  design, spec, and tasks.
- 0.5: confirmed deep_causality_tensor carries the complex load (matmul bounds
  satisfied by Complex<R>; svd_truncated/qr are ConjugateScalar-generic).
  Dense Hermitian eigen exists but was private (sym_eig) — task 2.0 promotes it.
- 0.6: verified the true breakage set — physics lib.rs, the quantum kernel
  tests, ikkt_matrix_model/main.rs, multi_physics_pipeline/main.rs;
  quantum_counterfactual and multi_physics_pipeline/model.rs are NOT dependents.
- 0.9: fixed the exact Q-TOL bound — incremental first-order forward-error
  budget (Higham γ_n product term, d_B·u·‖M‖_F per partial trace), C_safety=8,
  per-check margin telemetry shared with B1(c).
- 0.3: deferred quantum.* ids recorded; remaining decisions confirmed in design.md.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
```

---

## Phase 1 — crate scaffold + kernel migration

```
feat(deep_causality_quantum): new crate — quantum-information kernels move out of physics (Phase 1)

BREAKING (deep_causality_physics): the quantum-information layer moves to the
new workspace crate deep_causality_quantum: QuantumGates / QuantumOps<R>, the
Haruna logical gates (arXiv:2511.15224), the born_probability /
expectation_value / apply_gate / commutator / fidelity kernels with their
PropagatingEffect wrappers, and the Operator/Gate aliases. klein_gordon (a
relativistic field-theory PDE kernel) STAYS in physics; kernels/quantum splits
accordingly.

- Errors: moved kernels return the crate-local QuantumError (outer newtype
  over QuantumErrorEnum, typed variants incl. CommutatorNonZero naming the
  offending node pair); From<QuantumError> for CausalityError bridges the monad.
- born_probability/fidelity wrappers now carry the clamped plain R on the
  value channel (physics' Probability<R> quantity type stays in physics).
- MaybeParallel bound dropped from the moved kernels (multivector needs no
  deep_causality_par; the marker was a physics-side convention).
- HilbertState stays in deep_causality_multivector; metric signatures come
  from deep_causality_metric (SSOT).
- Call sites re-pointed: ikkt_matrix_model, multi_physics_pipeline.

bazel test //...: 1108/1108 green; clippy -D warnings clean.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
```

---

## Phase 2 — the operator / channel layer

```
feat(deep_causality_quantum): the operator/channel layer on CausalTensor<Complex<R>> (Phase 2)

L0 (deep_causality_tensor): new generic ops — dagger (conjugate transpose),
kronecker (2-D ⊗), and eigen_hermitian (the in-tree cyclic-Jacobi sym_eig,
promoted from private DMRG internals to a public dense op; the solver now
imports the shared kernel). reshape/permute_axes already existed.

L1 (multivector + quantum): the ket ↔ matrix bridge — HilbertState::to_ket /
from_ket over the existing to_matrix/from_matrix isomorphism, KET_COLUMN = 0
ratified as a named constant, 1/√D gain so a Dirac-normalized ket gives a
unit-trace ρ. Two numerically established boundaries, both pinned by tests:
(1) on Cl(0,n) the reversion+conj bracket is degenerate on the minimal left
ideal — the Clifford conjugation is the metric-correct Dirac adjoint (new
dirac_bracket_kernel dispatches by signature; mixed signatures rejected);
(2) to_matrix is an algebra homomorphism ONLY for Euclidean metrics (the
gamma basis squares to +1 regardless of signature).

L1–L4 (quantum): DensityMatrix<R> with enforced invariants (Hermitian, PSD
via eigen, unit trace; constructors from ket/ensemble/Choi), the named-subset
partial trace with the Q-PTP identities as tested properties (linearity,
Tr_B(X⊗Y) = X·Tr(Y), the bimodule law), the Choi–Jamiołkowski layer
(choi_from_kraus / kraus_from_choi via the PSD-Choi eigendecomposition,
CP ⟺ Choi PSD, TP ⟺ Tr_out(J) = I, round-trip tested), the matrix
commutator + support intersection + Kronecker-with-identity leg embedding.

The B1 counterexample now has its permanent Rust witness ([X,Y] = 0 but
[Tr₂X,Tr₂Y] = +4i·σy, ‖·‖_F = √32) alongside the boundary-case witness, both
also at Complex<Float106>. Monad law 3 (encapsulation = flat) is exercised
over the arity-5 STATE channel carrying complex-MATRIX payloads at Float106.

bazel test //...: 1117/1117 green; workspace clippy -D warnings clean.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
```

---

## Phase 3 — the verifiable QCM slice (simulated CJ + freeze check)

```
feat(deep_causality_quantum): verifiable QCM slice + emergent QPU seam (Phase 3)

The verifiable path (always compiled):
- ProcessFactors<R> / FactorSupports (R3): the CJ factorization σ = ∏ ρ_{Aᵢ|Pa(Aᵢ)}
  as an external node-keyed store — the operator analogue of LambdaEdges,
  static freeze-time decoration, never on the runtime STATE channel. Supports
  build from the graph via the public contains_edge (support(Aᵢ) = {Aᵢ}∪Pa(Aᵢ)),
  so no direct ultragraph dep.
- The Layer-D freeze commutativity check: quantum_markov_check computes [ρ_j,ρ_k]
  only on intersecting supports (Kronecker-with-identity leg embedding), against
  the condition-driven forward-error tolerance (CommutatorTolerance, Q-TOL: the
  Higham γ_n bound, safety C=8, per-node budgets, instrumented per-check margins).
  freeze_quantum wires it as the freeze_verified_with_check hook; a failure rolls
  the graph back (hook unfreeze) and the structured QuantumError is recovered via
  a RefCell stash across the orphan-legal impl From<QuantumError> for
  CausalityGraphError. Sound: never accepts a non-commuting pair.
- The C₃-exclusion faithfulness check (van der Lugt & Lorenz, 2508.11762 Thm 3.2):
  C₃ is exactly the bipartite 6-cycle K_{3,3} minus a perfect matching, detected
  by the row/column degree-2 test; CausalStructure::check_c3_exclusion rejects a
  C₃-containing structure at freeze with NotFaithfullyRepresentable.
- EnvironmentalPrep<R>: an immutable read-only handle for the Bell-prep ρ_A.

The emergent seam (feature `qpu`, off by default — verifiable-only default build):
- Reified QuantumCircuit / GateOp (pure data, validating constructor), the generic
  QpuSampler / ShotHistogram traits (classical counts, never amplitudes; no dyn),
  the shots→Uncertain bridges, and the 5-channel qpu_effect wrapper.
- SimQpu: a deterministic dense state-vector simulator (splitmix64-seeded, no
  deep_causality_rand dep, no unsafe) — GHZ/Bell/X circuits verified.

The engine crate (deep_causality) joins the dependency set — the freeze hook and
CausalityGraphError live there (no cycle; the engine does not depend back). Bazel
enables `qpu` (per the sparse optional-feature precedent) so the seam is tested.

bazel test //...: 1124/1124 green; clippy -D warnings clean (default and qpu).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
```

---

## Phase 4 — the orthomodular Verdict carrier

```
feat(deep_causality_quantum): orthomodular projection-lattice Verdict carrier (Phase 4)

- Projection<R, const D: usize>: a validated Hermitian idempotent on a fixed
  D-dimensional Hilbert space (the const D is what lets the nullary
  Verdict::bottom/top know their space). impl Verdict: bottom = 0, top = I,
  complement = I − P, join = the projector onto range(P+Q) via the Hermitian
  eigendecomposition, meet = ¬(¬P ∨ ¬Q) (De Morgan). Constructors from a
  validated matrix or a ket; leq / commutes_with / rank predicates.
- Born extraction at the measurement boundary: born_projective_probability =
  Tr(Pρ) (clamped, imaginary-part guarded) and born_projective_prob → the Prob
  MV-algebra verdict. NO blanket Verdict impl over operators — verdicts are read
  out from a state + measurement, per B7.
- Tests pin the orthomodular laws (bounded lattice, orthocomplement, De Morgan,
  orthomodular law) and the DISTRIBUTIVITY FAILURE on the |0⟩,|1⟩,|+⟩ triple, and
  confirm distributivity is restored within a commuting family. The eigen-based
  join/meet were numerically pre-validated against numpy.

Also fixes the one adversarial-review finding from Phase 3: FactorSupports::from_graph
and CausalStructure::from_graph_reachability now require a FROZEN graph (dense node
ids) and reject a dynamic one — a dynamic CausaloidGraph keeps sparse ids after
remove_node, which would silently drop edges from the derived structure (an unsound
false negative in the C₃ faithfulness gate). freeze() compacts to 0..n-1.

bazel test //...: green; clippy -D warnings clean (default and qpu).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
```

---

## Phase 5 — Lean formalization (foundation + B1 headline)

```
feat(lean): quantum partial-trace/Choi foundation + the B1 non-preservation witness (Phase 5)

Mathlib v4.15.0 has neither partial trace nor a Choi–Jamiołkowski layer, so both are built from
first principles on the pair-indexed matrix model in DeepCausalityFormal/Quantum/ (the project's
first consumer of Mathlib's matrix linear algebra). All zero `sorry`.

- PartialTrace.lean: local `kron`, `partialTraceRight`, and its lemma library — additivity, scalar
  homogeneity, the product identity Tr_B(X⊗Y) = Tr(Y)•X, and the LEFT and RIGHT bimodule laws
  Tr_B((Z⊗1)·M) = Z·Tr_B(M) / Tr_B(M·(Z⊗1)) = Tr_B(M)·Z. From those, the Q-PTP conditional theorem
  `partial_trace_preservation_boundary`: a boundary operator Z⊗1_B commuting with M forces Z to
  commute with Tr_B(M).
- PartialTraceCounterexample.lean: the B1 headline `partial_trace_nonpreservation` — X = σx⊗|0><0| +
  σz⊗|1><1|, Y = σx⊗|0><0| − σz⊗|1><1| commute yet their partial traces do not. Every entry is an
  integer (the physics writeup's +4i·σy is the matrix [[0,4],[−4,0]]), so the whole witness is stated
  over ℤ and closed by `decide` — including the exact commutator value. This retires the "false
  theorem" that motivated the change.
- Choi.lean: the Choi matrix `choiOf` and reconstructed action `applyChoi`, with add/smul linearity.
  The CJ reconstruction isomorphism applyChoi(choiOf E) = E is deferred (Mathlib stdBasisMatrix
  plumbing); the Rust round-trip tests carry it.

10 THEOREM_MAP ids, each with a Rust witness (operator_linalg_tests / channel_tests) and a MAP row;
lake build green, traceability green. The remaining QCM theorems (no_influence, markov_commutativity,
unitary_factorization, classical_embedding, cyclic_support, verdict.orthomodular) are stated as
deferred targets — net-new Mathlib machinery — and the /Quantum/ tree is exempt from the CI sorry
gate while the foundation is extended.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
```

---

## Phase 6 — examples + docs

```
docs(deep_causality_quantum): QCM freeze-check example + LEAN_QUANTUM status note (Phase 6)

- examples/quantum_examples/qcm_freeze_check: a simulated-CJ QCM exercising the
  freeze-time Markov commutativity check — a commuting model (two diagonal factors
  on the shared leg) freezes and reports its checked pairs; a non-commuting model
  (σx and σz on the shared leg) aborts the freeze, names the offending node pair,
  and rolls the graph back to dynamic. Config/execution split with a per-example
  FloatType alias, per the house layout.
- deep_causality_quantum/LEAN_QUANTUM.md: the verification-status note — the
  verifiable/emergent modality split, the Lean theorem table (10 ids, zero sorry),
  the B1 headline (partial_trace_preservation is false), the deferred targets, and
  the C₃-exclusion faithfulness scope.
- The migrated quantum examples (ikkt_matrix_model, multi_physics_pipeline) build
  and run against the new crate; quantum_examples gains the deep_causality engine
  dep for the freeze example.

Final gate: bazel test //... green; workspace clippy -D warnings clean (default and
qpu); lean lake build + traceability green.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
```
