/**
 * The error surface.
 *
 * Every row is one variant of `QuantumErrorEnum`, copied from
 * `deep_causality_quantum/src/error/quantum_error.rs`. `raisedIn` lists the
 * modules under `src/` that construct the variant, established by grep rather
 * than by reading the doc comments.
 *
 * `QuantumError` is a newtype over the enum. Each variant has a matching
 * constructor function on the newtype, so call sites write
 * `QuantumError::DimensionMismatch(msg)` rather than wrapping by hand.
 */

export interface ErrorVariant {
  /** Variant name, exactly as spelled in the enum. */
  name: string;
  /** Payload, exactly as declared. */
  payload: string;
  /** What the variant means, from its own doc comment. */
  meaning: string;
  /** Modules under `src/` that construct it. */
  raisedIn: string[];
}

export const errorVariants: ErrorVariant[] = [
  {
    name: 'DimensionMismatch',
    payload: 'String',
    meaning:
      'States or operators with incompatible dimensions or shapes were combined.',
    raisedIn: [
      'density_matrix',
      'qcm/process_factors',
      'qgates/operator_linalg',
      'qgates/channel',
      'verdict/born',
      'verdict/projection',
      'qpu/circuit',
      'qpu/sampler bridge',
      'qpu/sim',
    ],
  },
  {
    name: 'MetricMismatch',
    payload: 'String',
    meaning: 'The operands carry different Clifford metric signatures.',
    raisedIn: ['qgates/bridge', 'qgates/mechanics'],
  },
  {
    name: 'UnsupportedMetric',
    payload: 'String',
    meaning:
      'The Clifford metric is unsupported for the requested operation, such as an odd-dimensional metric for the ket-to-matrix bridge, or a metric convention error surfaced from deep_causality_metric.',
    raisedIn: ['qgates/bridge'],
  },
  {
    name: 'NonFiniteValue',
    payload: 'String',
    meaning: 'A NaN or an infinity was produced or encountered.',
    raisedIn: [
      'density_matrix',
      'qgates/mechanics',
      'qgates/channel',
      'qgates/gates_haruna',
      'verdict/born',
      'verdict/projection',
    ],
  },
  {
    name: 'NormalizationError',
    payload: 'String',
    meaning: 'Probability normalization failed: a value below 0, above 1, or a sum that is not 1.',
    raisedIn: ['density_matrix', 'qgates/mechanics', 'verdict/projection', 'qpu/bridge'],
  },
  {
    name: 'NonPositiveOperator',
    payload: 'String',
    meaning: 'An operator required to be positive semidefinite is not.',
    raisedIn: ['density_matrix', 'qgates/mechanics', 'qgates/channel', 'verdict/projection'],
  },
  {
    name: 'NonUnitTrace',
    payload: 'String',
    meaning:
      'A density or Choi–Jamiołkowski operator does not have the required trace.',
    raisedIn: ['density_matrix'],
  },
  {
    name: 'NonCptpChannel',
    payload: 'String',
    meaning: 'A channel is not completely positive and trace-preserving.',
    raisedIn: ['qgates/channel'],
  },
  {
    name: 'PartialTraceShape',
    payload: 'String',
    meaning: 'A partial trace was requested with an inconsistent subsystem shape.',
    raisedIn: ['qgates/operator_linalg'],
  },
  {
    name: 'CommutatorNonZero',
    payload: '{ node_j: usize, node_k: usize, detail: String }',
    meaning:
      'The freeze-time quantum Markov check found a non-commuting factor pair. The two fields name the offending operators by graph node index.',
    raisedIn: ['qcm/markov_freeze'],
  },
  {
    name: 'NotFaithfullyRepresentable',
    payload: 'String',
    meaning:
      'The declared causal structure contains a C₃ sub-relation, so it has no traditional-circuit causally faithful decomposition.',
    raisedIn: ['qcm/faithfulness', 'qcm/markov_freeze'],
  },
  {
    name: 'CalculationError',
    payload: 'String',
    meaning: 'Numerical conversion or general calculation failure.',
    raisedIn: [
      'density_matrix',
      'qcm/markov_freeze',
      'qcm/faithfulness',
      'qcm/process_factors',
      'qgates/channel',
      'qgates/mechanics',
      'qgates/operator_linalg',
      'qgates/gates_haruna',
      'verdict/projection',
    ],
  },
];
