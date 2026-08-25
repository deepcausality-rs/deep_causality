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
      'The definition the crate implements. The per-node Choi–Jamiołkowski factorization, and the quantum Markov condition of Def. 3.3 that becomes the freeze-time commutativity check.',
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
      'Theorem 3.2, the C₃-exclusion criterion. A causal structure containing a C₃ has no traditional-circuit causally faithful decomposition, and the freeze rejects it.',
  },
  {
    title:
      'Note on Logical Gates by Gauge Field Formalism of Quantum Error Correction',
    authors: 'Junichi Haruna',
    ref: 'arXiv:2511.15224v1 [hep-th]',
    year: '2025',
    file: 'logical_quantum_gates_by_gauge_field_formalism.pdf',
    citedFrom: ['types/qgates/gates_haruna'],
    usedFor:
      'The six logical gates the crate builds on gauge fields: S, Z, X, Hadamard, CZ and T, each as an exponential of polynomial functions of the gauge fields.',
  },
  {
    title: 'Causal and compositional structure of unitary transformations',
    authors: 'Robin Lorenz, Jonathan Barrett',
    ref: 'arXiv:2001.07774v2 [quant-ph]',
    year: '2021',
    file: 'Causal and compositional structure of unitary transformations-2001.07774v2.pdf',
    citedFrom: [],
  },
  {
    title: 'Cyclic Quantum Causal Models',
    authors: 'Jonathan Barrett, Robin Lorenz, Ognyan Oreshkov',
    ref: 'arXiv:2002.12157v3 [quant-ph]',
    year: '2021',
    file: 'Cyclic Quantum Causal Models-2002.12157v3.pdf',
    citedFrom: [],
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
];
