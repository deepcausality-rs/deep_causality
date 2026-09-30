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
    title: 'Is this structure a legal quantum causal model?',
    lede: 'These checks run on a factorization and on the structure its supports encode. A structure that fails them cannot be realised the way the candidate claims. Both checks are computations, and neither simulates anything.',
    checks: [
      {
        name: 'check_markov',
        on: 'model, plant, circuit',
        question:
          'Do factors whose Hilbert supports overlap commute pairwise? This is the quantum Markov condition (Lorenz 2022, Definition 3.3). It compares the Frobenius norm of each commutator with a forward-error tolerance built from the dimension of the common support, the two norms and a per-node error budget. The check is sound, since it never accepts a non-commuting model, and it may be incomplete.',
        rejection:
          'CommutatorNonZero with the two node indices of the first offending pair. On the inherited factors of a composite, reached through check_markov_as and markov_certificate and not through validate, CertificateNotInherited names the pair instead.',
        vacuous: 'when every pair of supports is disjoint, since a disjoint pair imposes no obligation',
        backing: 'tested',
        evidence: 'markov_freeze_tests::test_check_noncommuting_names_the_pair; the Lean target quantum.markov_commutativity is deferred',
        source: 'src/types/qcm/markov_freeze.rs',
      },
      {
        name: 'check_decomposable',
        on: 'model, plant, circuit',
        question:
          'Does the structure avoid the C₃ sub-relation, so that a unitary circuit can realise it with the same causal structure (Lorenz–Barrett’s G_U = G_C)? The search examines every 3×3 induced sub-relation of inputs and outputs. A rejection concerns the structure, and no unitary is involved.',
        rejection: 'NotFaithfullyRepresentable, carrying the triple of inputs and the triple of outputs that form the C₃.',
        vacuous: 'when the structure has too few systems to contain a C₃; the report says nothing was examined',
        backing: 'tested',
        evidence: 'faithfulness_tests::test_the_two_cnot_structure_is_c3_and_is_rejected; no Lean statement',
        source: 'src/types/qcm/faithfulness.rs',
      },
      {
        name: 'build()',
        on: 'plant (structural), model, circuit',
        question:
          'Is any candidate structure a cycle? The builder refuses it before a check runs. The criterion the library applies does not reject a cycle, so the refusal is a scope decision and the error names it as one.',
        rejection: 'CyclicStructureUnsupported, naming the candidate on a plant',
        vacuous: 'not applicable; mechanism candidates make no structural claim and get no cycle check',
        backing: 'decision',
        evidence: 'pipeline_tests::test_a_cyclic_structural_candidate_is_rejected_at_build_by_name',
        source: 'src/types/pipeline/config.rs',
      },
    ],
  },
  {
    id: 'channel',
    title: 'Is this channel physical?',
    lede: 'A quantum channel is physical when it is completely positive and trace preserving. Both properties come straight from its Choi operator.',
    checks: [
      {
        name: 'check_completely_positive',
        on: 'operator layer',
        question: 'Is the Choi operator positive semidefinite? The check measures the most negative eigenvalue against a tolerance.',
        rejection:
          'NonPositiveOperator when the operator is not Hermitian; NonCptpChannel naming the negative eigenvalue otherwise. The report form returns one entry per eigenvalue.',
        vacuous: 'a non-Hermitian input records the Hermiticity defect alone and examines no eigenvalue',
        backing: 'tested',
        evidence: 'channel_tests::test_non_cp_operator_rejected',
        source: 'src/types/qgates/channel.rs',
      },
      {
        name: 'check_trace_preserving',
        on: 'operator layer',
        question: 'Does the partial trace of the Choi operator over the output equal the identity on the input? One comparison settles it.',
        rejection: 'NonCptpChannel with the largest deviation from the identity',
        vacuous: 'never; it examines d_in² entries',
        backing: 'tested',
        evidence: 'channel_tests::test_non_tp_family_rejected',
        source: 'src/types/qgates/channel.rs',
      },
    ],
  },
  {
    id: 'code',
    title: 'Does this code have the structure it claims?',
    lede: 'These checks run on a chain complex read as a CSS code. They are exact. Each one decides over finite supports in F₂, with no state vector and no tolerance.',
    checks: [
      {
        name: 'derive_code',
        on: 'code',
        question: 'What are n, k and the check matrices of the code the chain complex defines? It derives the code and decides nothing, so it has no rejection state.',
        rejection: 'not applicable; an error only for a rank failure or a shape mismatch',
        vacuous: 'never; it decides nothing',
        backing: 'tested',
        evidence: 'css_code_tests::test_the_simplicial_torus_derives_its_own_numbers',
        source: 'src/types/qcode/css_code.rs',
      },
      {
        name: 'check_ldpc_weights',
        on: 'code',
        question: 'Do all row and column weights of both check matrices stay within one declared bound?',
        rejection: 'A report entry naming the check matrix (Z or X) and the column or row, recorded at the first item over the bound',
        vacuous: 'a code with no Z checks examines no Z columns, visible as an empty half',
        backing: 'tested',
        evidence: 'css_code_tests::test_a_bound_the_x_checks_exceed_names_the_offender_and_stops_there',
        source: 'src/types/qcode/css_code.rs',
      },
      {
        name: 'check_class_invariance',
        on: 'code',
        question:
          'Does a diagonal logical gate (Z̄, S̄ or T̄) act on the homology class and not on the chosen representative, so that U(γ) ~ U(γ ⊕ b) holds for each boundary b supplied? The pipeline supplies the Z-check generators. The test is exact and uses rational phases.',
        rejection: 'A witness holding the boundary and the shared, shift-only and gate-only parts of the failure. A failure is an obstruction and has no margin.',
        vacuous: 'no boundary is supplied; the standalone check then reports tested as zero, and the pipeline still records the check as Accepted',
        backing: 'tested',
        evidence: 'class_invariance_tests::test_the_witness_reproduces_the_failure_it_reports',
        source: 'src/types/qcode/logical_equivalence.rs',
      },
      {
        name: 'check_clifford_action',
        on: 'code',
        question: 'Does a Clifford gate program act as the logical Hadamard, swapping Z̄(γ) and X̄(γ̃) up to phase and stabilizers? The check is exact.',
        rejection:
          'NotInNormalizer naming the generator whose image leaves the code space; otherwise a report entry that records which swap failed',
        vacuous: 'a code with no logical qubits records nothing',
        backing: 'tested',
        evidence: 'clifford_action_tests::test_a_program_that_does_nothing_fails_the_check',
        source: 'src/types/qcode/clifford_action.rs',
      },
    ],
  },
  {
    id: 'abstraction',
    title: 'Does the abstraction hold?',
    lede: 'An abstraction maps a detailed model (physical qubits with noise) to a coarser one (logical qubits). These checks build on Lorenz and Tull, arXiv:2602.16612, which gives the exact case. The ε forms and the composition bound are this crate’s own. The checks apply to a circuit subject.',
    checks: [
      {
        name: 'check_alignment_structure',
        on: 'circuit',
        question:
          'Does a partition of the low-level model into blocks satisfy the structural precheck (simple, extra-simple, full)? The scope of the result reads Equivalent when both models are classical and Necessary otherwise.',
        rejection:
          'CalculationError when the partition is not simple. Extra-simple and full failures are recorded as witnesses holding the offending pair.',
        vacuous: 'when the high level has one vertex or none, no pair exists to examine',
        backing: 'tested',
        evidence: 'alignment_structure_tests::test_example_55_is_not_simple',
        source: 'src/types/abstraction/alignment_structure.rs',
      },
      {
        name: 'check_naturality',
        on: 'circuit',
        question:
          'For every query in the signature, does mapping then querying equal querying then mapping, within a tolerance? The residual is measured in the Frobenius norm on Choi operators.',
        rejection: 'CalculationError naming the query and the residual against the tolerance',
        vacuous: 'an empty signature reads Vacuous',
        backing: 'tested',
        evidence: 'abstraction_tests::test_an_entangling_gate_breaks_the_square_by_root_two',
        source: 'src/types/abstraction/naturality.rs',
      },
      {
        name: 'check_fault_tolerance',
        on: 'abstraction',
        question:
          'For each fault in a list, is the remainder constant and does the propagated Pauli weigh no more than the fault? No threshold, distance or asymptotic suppression is claimed.',
        rejection: 'A record holding the fault, the propagated weight and a witness: the flipped parity pattern, or the propagated Pauli and the logical operator it carries',
        vacuous: 'an empty fault set holds vacuously',
        backing: 'tested',
        evidence: 'fault_tolerance_tests::test_t_bar_rejects_x_and_y_on_its_support_with_a_two_term_remainder_at_both_weights',
        source: 'src/types/abstraction/fault_tolerance.rs',
      },
      {
        name: 'compose',
        on: 'abstraction',
        question:
          'When two abstractions stack, does the composite’s measured residual stay under ‖τ₂‖_post · ε₁ + ‖τ₁‖_pre · ε₂? The two norms are computed for each query. When both residuals are zero the composite is exact, and Lean proves that case.',
        rejection: 'A law row per query with both residuals, both norms, the bound and the measured value; the law holds when every measured value sits at or under its bound plus the state tolerance',
        vacuous: 'an empty signature holds vacuously',
        backing: 'partial',
        evidence: 'The exact case is proved: quantum.abstraction.compose_exact and quantum.abstraction.compose_exact.defect. The bound follows from the triangle inequality and has no Lean statement.',
        source: 'src/types/abstraction/composition.rs',
      },
    ],
  },
  {
    id: 'marginal',
    title: 'Can a marginal be trusted?',
    lede: 'Taking a partial trace of a legal model does not always give a legal model. The library checks the condition that does hold, and it returns the residual with the bound that residual buys.',
    checks: [
      {
        name: 'partial_trace_preservation_boundary',
        on: 'operator layer',
        question:
          'Given an operator Z on the kept system and M on the whole space, how nearly does Z ⊗ 1 commute with M, and what bound does that give on [Z, Tr_B M]? It returns the bound with its √(d_B) amplification, and a holds flag against the tolerance you name. A commutator within tolerance is warrant for the conclusion at √(d_B) times that tolerance, and no warrant for the exact conclusion.',
        rejection: 'BoundaryNotHeld with the residual, the tolerance and the amplification, raised by Hypothesis::marginalise (src/types/qcm/hypothesis.rs) when it refuses a marginalisation',
        vacuous: 'not applicable',
        backing: 'partial',
        evidence: 'The exact-equality theorem and the commutator identity are proved: quantum.partial_trace_preservation_boundary and quantum.partial_trace.commutator_transport. The √(d_B) bound has no Lean statement.',
        source: 'src/types/qgates/operator_linalg.rs',
      },
    ],
  },
];
