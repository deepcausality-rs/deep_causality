/**
 * The papers committed under `deep_causality_quantum/papers/`.
 *
 * Titles and author lists are read off each PDF's own first page, not from the
 * filename. `citedFrom` lists the crate modules whose doc comments cite the
 * paper, established by grep over `src/`. A paper that is committed but not yet
 * cited from any module carries an empty list, and the page says so rather than
 * implying an implementation that does not exist.
 */

export interface Paper {
  /** Title as printed on the paper. */
  title: string;
  authors: string;
  /** Journal reference or arXiv identifier as printed. */
  ref: string;
  year: string;
  /** Filename under papers/. */
  file: string;
  /** Modules under src/ whose doc comments cite it. */
  citedFrom: string[];
  /** What the crate takes from it, when it takes something. */
  usedFor?: string;
}

export const papers: Paper[] = [
  {
    title:
      'Quantum causal models: the merits of the spirit of Reichenbach’s principle for understanding quantum causal structure',
    authors: 'Robin Lorenz',
    ref: 'Synthese (2022) 200:424',
    year: '2022',
    file: 'Quantum causal models-lorenz2022.pdf',
    citedFrom: ['types/density_matrix', 'types/qgates/channel', 'types/qcm/markov_freeze'],
    usedFor:
      'The model the crate implements: a Choi–Jamiołkowski factor per node, and Definition 3.3, the quantum Markov condition that the Markov check tests.',
  },
  {
    title:
      'Unitary causal decompositions: a combinatorial characterisation via lattice theory',
    authors: 'Tein van der Lugt, Robin Lorenz',
    ref: 'arXiv:2508.11762v1 [quant-ph]',
    year: '2025',
    file: 'Unitary causal decompositions-2508.11762v1.pdf',
    citedFrom: ['types/qcm/faithfulness', 'error/quantum_error'],
    usedFor:
      'Definition 3.1 and Theorem 3.2, the C₃-exclusion criterion. A causal structure containing a C₃ has no traditional-circuit causally faithful decomposition, and the decomposability check rejects such a structure.',
  },
  {
    title: 'Causal and Compositional Abstraction',
    authors: 'Robin Lorenz, Sean Tull',
    ref: 'arXiv:2602.16612v1',
    year: '2026',
    file: 'Causal and Compositional Abstraction-2602.16612v1.pdf',
    citedFrom: [
      'types/abstraction/composition',
      'types/abstraction/alignment_structure',
      'types/abstraction/naturality',
      'types/abstraction/abstraction',
      'types/abstraction/query',
      'types/abstraction/type_alignment',
      'types/circuit_model',
      'types/pipeline/config',
      'types/qcm/dem_model',
    ],
    usedFor:
      'The abstraction layer: the structural precheck (Definition 49, Theorem 51), the naturality square, and composition (Proposition 17, whose exact case Lean proves).',
  },
  {
    title:
      'Note on Logical Gates by Gauge Field Formalism of Quantum Error Correction',
    authors: 'Junichi Haruna',
    ref: 'arXiv:2511.15224v1 [hep-th]',
    year: '2025',
    file: 'logical_quantum_gates_by_gauge_field_formalism.pdf',
    citedFrom: [
      'types/qgates/gates_haruna',
      'types/qcode/gauge_field_gate',
      'types/qcode/diagonal_phase',
      'types/qcode/logical_equivalence',
      'types/qcode/clifford_action',
      'types/abstraction/fault_tolerance',
      'types/circuit_model/exact_semantics',
    ],
    usedFor:
      'The six logical gates built on gauge fields (S, Z, X, Hadamard, CZ and T), the class-invariance check of Equation 3.20, and the gate programs the fault-tolerance check propagates.',
  },
  {
    title: 'Causal and compositional structure of unitary transformations',
    authors: 'Robin Lorenz, Jonathan Barrett',
    ref: 'arXiv:2001.07774v2 [quant-ph]',
    year: '2021',
    file: 'Causal and compositional structure of unitary transformations-2001.07774v2.pdf',
    citedFrom: ['types/qcm/faithfulness'],
    usedFor:
      'Theorem 3, which sets the scope of the decomposability check: the check rejects a causal structure and makes no claim about a unitary, and it applies to traditional, non-routed circuits only.',
  },
  {
    title: 'Cyclic Quantum Causal Models',
    authors: 'Jonathan Barrett, Robin Lorenz, Ognyan Oreshkov',
    ref: 'arXiv:2002.12157v3 [quant-ph]',
    year: '2021',
    file: 'Cyclic Quantum Causal Models-2002.12157v3.pdf',
    citedFrom: ['error/quantum_error', 'types/qcm/markov_freeze', 'types/qcm/hypothesis', 'types/qcm/dilation'],
    usedFor:
      'The error that build() returns for a cyclic structure cites this paper as the setting the crate leaves out.',
  },
  {
    title:
      'Classifying Logical Gates in Quantum Codes via Cohomology Operations and Symmetry',
    authors: 'Po-Shen Hsin, Ryohei Kobayashi, Guanyu Zhu',
    ref: 'arXiv:2411.15848v3 [quant-ph]',
    year: '2025',
    file: 'Classifying logical gates via cohomology operations-2411.15848.pdf',
    citedFrom: [],
  },
];

/**
 * Works cited from the source that are not committed under papers/. Listed
 * separately so the papers page does not imply a PDF that is not there.
 */
export const citedElsewhere: { work: string; citedFrom: string }[] = [
  {
    work: 'M.-D. Choi, “Completely positive linear maps on complex matrices”, Linear Algebra Appl. 10 (1975) 285–290.',
    citedFrom: 'types/qgates/channel',
  },
  {
    work: 'Birkhoff–von Neumann quantum logic: the orthomodular lattice of projections.',
    citedFrom: 'types/verdict/projection',
  },
  {
    work: 'arXiv:1906.10726, the theorem on unitary circuits with broken wires, cited as the source of the dilation.',
    citedFrom: 'types/qcm/dilation',
  },
  {
    work: 'S. Aaronson and D. Gottesman, arXiv:quant-ph/0406196, §III, the stabilizer-tableau update rule.',
    citedFrom: 'types/qcode/clifford_action',
  },
];
