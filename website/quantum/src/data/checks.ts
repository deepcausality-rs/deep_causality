/**
 * Every check the library runs, as data. `/checks/` renders this.
 *
 * Each row was read against the crate source. `rejection` names what a
 * failure carries, `vacuous` says when the check certifies nothing, and
 * `backing` says whether Lean proves the check's claim, tests witness it, or the
 * behaviour is a decision. Paths are relative to `deep_causality_quantum/`.
 *
 * A computed rejection has three outcomes, by stage and subject:
 *   - `check_markov` and `check_decomposable` on a plant drop a failing
 *     candidate from the admitted set.
 *   - On a model or a circuit, a rejection of those two checks is the stage's
 *     failure and `finalize()` returns the error. The circuit's `check_naturality`
 *     also fails the stage, and `check_alignment_structure` fails it only when the
 *     partition is not simple.
 *   - The three code checks record a rejecting entry in the report and
 *     `finalize()` still returns `Ok`.
 * A structural error, such as a shape mismatch or an image outside the
 * normalizer, fails the stage in every case.
 */

export type Backing = 'proved' | 'partial' | 'tested' | 'decision';

export type Check = {
  /** Function or type name as it appears in code. */
  name: string;
  /** Subjects that offer it. */
  on: string;
  question: string;
  rejection: string;
  vacuous: string;
  backing: Backing;
  /** The theorem id, or the test that witnesses it. */
  evidence: string;
  source: string;
};

export type CheckGroup = {
  id: string;
  title: string;
  lede: string;
  checks: Check[];
};

export const groups: CheckGroup[] = [
  {
    id: 'model',
    title: 'Three checks decide whether a structure is a quantum causal model.',
    lede: 'The factors that share a Hilbert leg must commute (check_markov). The structure must contain no C₃ sub-relation if a unitary circuit is to realise it with the same causal structure (check_decomposable). The structure must be acyclic (build()). A factor is one node’s operator, and its support is the set of Hilbert legs it acts on; no check simulates a device.',
    checks: [
      {
        name: 'check_markov',
        on: 'model, plant, circuit',
        question:
          'The check decides whether factors whose Hilbert supports overlap commute pairwise, which is the quantum Markov condition (Lorenz 2022, Definition 3.3). For each overlapping pair it embeds both factors on the union of their supports and takes the Frobenius norm of their commutator. The threshold is a forward-error budget built from the dimension of that space, the two factor norms and a per-node error budget. The check is sound, meaning it never accepts a pair whose commutator exceeds the threshold; the crate does not claim it is complete.',
        rejection:
          'On a model or a circuit, the stage fails with CommutatorNonZero, which carries the node indices of the first non-commuting pair, and finalize() returns it. On a plant, the candidate leaves the admitted set and its report entry records the pair. On the inherited factors of a composite, checked through check_markov_as and markov_certificate and not through validate, the error is CertificateNotInherited and carries the same pair.',
        vacuous: 'It certifies nothing when every pair of supports is disjoint, or when the model has a single factor. A disjoint pair imposes no obligation, so the report examines zero pairs.',
        backing: 'tested',
        evidence: 'Witnessed by markov_freeze_tests::test_check_noncommuting_names_the_pair. The Lean target quantum.markov_commutativity is deferred.',
        source: 'src/types/qcm/markov_freeze.rs',
      },
      {
        name: 'check_decomposable',
        on: 'model, plant, circuit',
        question:
          'The check decides whether the causal structure avoids the C₃ sub-relation, the influence pattern of two commuting CNOT gates, so that a unitary circuit can realise the structure with the same causal structure (Lorenz–Barrett’s G_U = G_C). The search visits every 3×3 induced sub-relation of inputs and outputs. A rejection concerns the structure, and no unitary is involved.',
        rejection:
          'It rejects with NotFaithfullyRepresentable, which carries the triple of inputs and the triple of outputs that form the C₃. On a plant the candidate leaves the admitted set; on a model or a circuit the stage fails.',
        vacuous: 'It certifies nothing when the structure has fewer than three inputs or fewer than three outputs, because no 3×3 sub-relation exists. The report then shows zero blocks examined.',
        backing: 'tested',
        evidence: 'Witnessed by faithfulness_tests::test_the_two_cnot_structure_is_c3_and_is_rejected. The check has no Lean statement.',
        source: 'src/types/qcm/faithfulness.rs',
      },
      {
        name: 'build()',
        on: 'plant (structural), model, circuit',
        question:
          'The builder decides whether the structure contains a directed cycle, and it decides before any check runs. It examines the structural candidates of a plant, the graph of a model and the grouping of a circuit. The library applies no criterion that rejects a cycle, so the refusal is a scope decision, and the error says so.',
        rejection: 'It rejects with CyclicStructureUnsupported. On a plant the error carries the candidate; on a model or a circuit it describes the graph or the grouping.',
        vacuous: 'It does not run on mechanism candidates, which make no structural claim.',
        backing: 'decision',
        evidence: 'Witnessed by pipeline_tests::test_a_cyclic_structural_candidate_is_rejected_at_build_by_name.',
        source: 'src/types/pipeline/config.rs',
      },
    ],
  },
  {
    id: 'channel',
    title: 'Two checks decide whether a channel is physical.',
    lede: 'A channel is physical when it is completely positive and trace preserving. Both checks read the channel’s Choi operator, the matrix that represents the channel, and each takes a tolerance.',
    checks: [
      {
        name: 'check_completely_positive',
        on: 'operator layer',
        question:
          'The check decides whether the Choi operator is positive semidefinite, which is complete positivity. It first measures the Hermiticity defect, then compares each eigenvalue with the tolerance; an eigenvalue below the negative tolerance rejects.',
        rejection:
          'It rejects with NonPositiveOperator, which carries the Hermiticity defect, when the operator is not Hermitian. Otherwise it rejects with NonCptpChannel, which carries the first eigenvalue below the negative tolerance. The report form returns the defect and one entry per eigenvalue.',
        vacuous: 'It certifies nothing about the spectrum when the input is not Hermitian. The report then records the Hermiticity defect alone and examines no eigenvalue.',
        backing: 'tested',
        evidence: 'Witnessed by channel_tests::test_non_cp_operator_rejected.',
        source: 'src/types/qgates/channel.rs',
      },
      {
        name: 'check_trace_preserving',
        on: 'operator layer',
        question:
          'The check decides whether the partial trace of the Choi operator over the output equals the identity on the input, which is trace preservation. One comparison settles it: the largest entry of the difference, against the tolerance.',
        rejection: 'It rejects with NonCptpChannel, which carries the largest deviation from the identity.',
        vacuous: 'It has no vacuous case; it examines all d_in² entries of the difference between the traced operator and the identity.',
        backing: 'tested',
        evidence: 'Witnessed by channel_tests::test_non_tp_family_rejected.',
        source: 'src/types/qgates/channel.rs',
      },
    ],
  },
  {
    id: 'code',
    title: 'Code checks are exact: no state vector, no tolerance.',
    lede: 'Every decision is over finite supports in F₂, where a code is a chain complex read as a CSS code. The library derives n and k from the complex and checks three properties: the weights of the check matrices, the class invariance of the diagonal logical gates, and the Clifford action of the logical Hadamard.',
    checks: [
      {
        name: 'derive_code',
        on: 'code',
        question:
          'The function derives the CSS code that the chain complex defines. Its n is the number of 1-cells, its k is the first Betti number over F₂, and its Z and X checks are the columns of ∂₂ and δ₀. It decides nothing, so it has no rejection state.',
        rejection: 'It cannot reject. It returns an error only when the F₂ rank computation fails (CalculationError) or a column cannot be read as a chain (DimensionMismatch).',
        vacuous: 'It has no vacuous case, because it decides nothing.',
        backing: 'tested',
        evidence: 'Witnessed by css_code_tests::test_the_simplicial_torus_derives_its_own_numbers.',
        source: 'src/types/qcode/css_code.rs',
      },
      {
        name: 'check_ldpc_weights',
        on: 'code',
        question:
          'The check decides whether every row weight and every column weight of both check matrices stays within one declared bound. A column is one check, and its weight is the number of qubits it acts on; a row is one qubit, and its weight is the number of checks that act on it.',
        rejection:
          'It rejects at the first item over the bound, in the order Z columns, Z rows, X columns, X rows. The report entry records the matrix (Z or X) and the column or row, and the check stops there. The stage still succeeds, and finalize() returns Ok.',
        vacuous: 'It certifies nothing about an empty family. A code with no Z checks records no Z columns, which shows as an empty half of the report.',
        backing: 'tested',
        evidence: 'Witnessed by css_code_tests::test_a_bound_the_x_checks_exceed_names_the_offender_and_stops_there.',
        source: 'src/types/qcode/css_code.rs',
      },
      {
        name: 'check_class_invariance',
        on: 'code',
        question:
          'The check decides whether a diagonal logical gate (Z̄, S̄ or T̄) acts on the homology class and not on the chosen representative. On the code space, U(γ) ~ U(γ ⊕ b) must hold for each boundary b supplied; the pipeline supplies the Z-check generators as the boundaries. The test is exact and uses rational phases.',
        rejection:
          'The standalone check returns a witness: the index of the failing boundary and three counts for the offending basis state, namely its qubits in both the gate’s chain and the boundary, in the boundary only, and in the gate’s chain only. A failure is an obstruction and has no margin. The pipeline records one entry per class and gate, and finalize() still returns Ok.',
        vacuous: 'It certifies nothing when no boundary is supplied. The standalone check then reports zero shifts tested, and the pipeline still records the entry as Accepted.',
        backing: 'tested',
        evidence: 'Witnessed by class_invariance_tests::test_the_witness_reproduces_the_failure_it_reports.',
        source: 'src/types/qcode/logical_equivalence.rs',
      },
      {
        name: 'check_clifford_action',
        on: 'code',
        question:
          'The check decides whether a Clifford gate program acts as the logical Hadamard on one logical qubit: it must swap Z̄(γ) and X̄(γ̃) up to phase and stabilizers, and it must leave the other logical qubits fixed. The pipeline builds H̄ for each logical qubit from the code’s homology and a symplectic dual basis, then checks that program. The check is exact.',
        rejection:
          'It rejects in one of two ways. A program that moves an operator out of the code space fails the stage with NotInNormalizer, which carries the first stabilizer generator the image anticommutes with. A program that stays in the code space but does not swap Z̄ and X̄, or moves another logical qubit, is recorded as a rejecting entry, and finalize() still returns Ok; the standalone form reports which swap failed.',
        vacuous: 'A code with no logical qubits records no entry. On a code with one logical qubit, the check pushes no other logical operator through the program.',
        backing: 'tested',
        evidence: 'Witnessed by clifford_action_tests::test_a_program_that_does_nothing_fails_the_check.',
        source: 'src/types/qcode/clifford_action.rs',
      },
    ],
  },
  {
    id: 'abstraction',
    title: 'An abstraction holds when its four checks pass.',
    lede: 'Its partition must pass the structural precheck, its squares must commute within tolerance, it must tolerate its faults, and stacked links must obey the composition bound. An abstraction maps a detailed model (physical qubits with noise) to a coarser one (logical qubits). The checks build on Lorenz and Tull, arXiv:2602.16612, which gives the exact case; the ε forms and the composition bound are this crate’s own. The validate stage runs check_alignment_structure and check_naturality on a circuit subject; check_fault_tolerance and compose are methods of the abstraction itself.',
    checks: [
      {
        name: 'check_alignment_structure',
        on: 'circuit',
        question:
          'The check decides whether a partition of the low-level model into one block per high-level vertex passes the structural precheck of Lorenz and Tull’s Definition 49. Let α(X) be the low-level vertices that reach block X without passing through the blocks of X’s parents. The partition is simple when α(X) never meets another block, and extra-simple when no two α sets meet. It is full when every vertex in the block of a non-input parent of X reaches block X. The scope of the result reads Equivalent when both models are classical, where Theorem 51 makes these conditions exact, and Necessary otherwise.',
        rejection:
          'A partition that is not simple fails the stage with CalculationError, which carries the pair (X, Y) where α(X) meets block Y. A partition that is not extra-simple or not full does not fail the stage; the report records a rejecting entry that holds the offending pair.',
        vacuous: 'It certifies nothing when the high-level model has one vertex or none. No ordered pair exists, so the simple and extra-simple conditions hold with nothing examined.',
        backing: 'tested',
        evidence: 'Witnessed by alignment_structure_tests::test_example_55_is_not_simple.',
        source: 'src/types/abstraction/alignment_structure.rs',
      },
      {
        name: 'check_naturality',
        on: 'circuit',
        question:
          'The check decides, for every query in the abstraction’s signature, whether running the low-level query and then applying the map τ gives the same channel as applying τ and then running the high-level query. The residual is the Frobenius norm of the difference between the two Choi operators, compared with the state tolerance at the square’s dimension.',
        rejection: 'The stage fails with CalculationError, which carries the query, the residual and the tolerance it exceeded. The standalone form returns the report with a rejecting entry for that query.',
        vacuous: 'It certifies nothing when the signature has no queries; the report then reads Vacuous.',
        backing: 'tested',
        evidence: 'Witnessed by abstraction_tests::test_an_entangling_gate_breaks_the_square_by_root_two.',
        source: 'src/types/abstraction/naturality.rs',
      },
      {
        name: 'check_fault_tolerance',
        on: 'abstraction',
        question:
          'The check decides, for each fault in a fault set, whether the abstraction tolerates it. On the exact path, a code abstraction pushes the fault through a logical gate; the fault is tolerated when the remainder is constant and the propagated Pauli weighs no more than the fault. The remainder is the logical phase left after the fault’s own Pauli is recovered. On the numeric path, a circuit abstraction compares each fault as a naturality square. The check claims no threshold, distance or asymptotic suppression.',
        rejection:
          'It records the fault, the propagated weight and a witness. On the exact path the witness is the flipped parity pattern with the remainder’s phase table, or the propagated Pauli and the logical operator it carries. On the numeric path the witness gives the residual against the tolerance.',
        vacuous: 'An empty fault set passes with nothing examined.',
        backing: 'tested',
        evidence: 'Witnessed by fault_tolerance_tests::test_t_bar_rejects_x_and_y_on_its_support_with_a_two_term_remainder_at_both_weights.',
        source: 'src/types/abstraction/fault_tolerance.rs',
      },
      {
        name: 'compose',
        on: 'abstraction',
        question:
          'When two abstractions stack, the check decides whether the composite’s measured residual stays under ‖τ₂‖_post · ε₁ + ‖τ₁‖_pre · ε₂, where ε₁ and ε₂ are the residuals of the first and second link. The two norms belong to the first link’s input-side τ and the second link’s output-side τ, and the check computes them for each query. When both residuals are zero the composite is exact, and Lean proves that case.',
        rejection:
          'It rejects a query when the measured residual exceeds the bound plus the state tolerance. The law’s holds() flag then turns false, and compose still returns the composite. Each row carries both residuals, both norms, the bound and the measured value.',
        vacuous: 'With an empty signature the law has no rows and holds vacuously.',
        backing: 'partial',
        evidence: 'Lean proves the exact case as quantum.abstraction.compose_exact and quantum.abstraction.compose_exact.defect. The bound follows from the triangle inequality and has no Lean statement.',
        source: 'src/types/abstraction/composition.rs',
      },
    ],
  },
  {
    id: 'marginal',
    title: 'A marginal can be trusted only within a bound.',
    lede: 'A marginal is the operator left after a partial trace, and tracing out part of a model can destroy the commutation the model relies on; Lean proves a counterexample (quantum.partial_trace_nonpreservation), two commuting operators whose partial traces do not commute. The boundary check below measures how nearly a boundary operator commutes with the joint operator, and it returns the bound that residual gives on the traced commutator.',
    checks: [
      {
        name: 'partial_trace_preservation_boundary',
        on: 'operator layer',
        question:
          'Given an operator Z on the kept system and an operator M on the whole space, the function measures how nearly Z ⊗ 1 commutes with M, and it returns a bound on [Z, Tr_B M]. The bound is √(d_B) times the residual, where d_B is the dimension of the traced system. A holds flag says whether the residual is at or under the tolerance you set. A commutator within tolerance warrants the conclusion only at √(d_B) times that tolerance, and it never warrants the exact conclusion.',
        rejection: 'The function itself returns the warrant and does not reject. Hypothesis::marginalise (src/types/qcm/hypothesis.rs) rejects with BoundaryNotHeld, which carries the residual, the tolerance and the amplification, and it traces nothing.',
        vacuous: 'It passes trivially when Z is a multiple of the identity, because Z ⊗ 1 then commutes with every M and the residual is zero up to rounding.',
        backing: 'partial',
        evidence: 'Lean proves the exact-equality theorem and the commutator identity as quantum.partial_trace_preservation_boundary and quantum.partial_trace.commutator_transport. The √(d_B) bound has no Lean statement.',
        source: 'src/types/qgates/operator_linalg.rs',
      },
    ],
  },
];
