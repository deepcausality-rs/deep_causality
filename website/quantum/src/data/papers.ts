/**
 * The published works the crate cites, with the PDF under `deep_causality_quantum/papers/` where
 * the repository holds one.
 *
 * Titles and author lists are read off each PDF's own first page, not from the
 * filename. `citedFrom` lists the crate modules whose doc comments cite the
 * paper, established by grep over `src/`. `usedBy` lists the examples and the
 * verifications that cite it, established by grep over
 * `examples/quantum_examples/` and `deep_causality_quantum/verification/`. Only
 * papers that code cites are listed.
 *
 * The sensing papers support the sensing examples and the verification; no
 * module in `src/` cites them. Hu et al.'s PDF prints no identifier; its arXiv
 * number comes from the listing the file was downloaded from.
 */

export interface Paper {
  /** Title as printed on the paper. */
  title: string;
  authors: string;
  /** Journal reference or arXiv identifier as printed. */
  ref: string;
  year: string;
  /** Filename under papers/, when the repository holds a copy. */
  file?: string;
  /** Modules under src/ whose doc comments cite it. */
  citedFrom: string[];
  /** Examples and verifications that cite it. */
  usedBy?: string[];
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
    title: 'The influence of transverse motion within an atomic gravimeter',
    authors: 'Anne Louchet-Chauvet, Tristan Farah, Quentin Bodart, André Clairon, Arnaud Landragin, Sébastien Merlet, Franck Pereira Dos Santos',
    ref: 'New J. Phys. 13 065025',
    year: '2011',
    file: 'The influence of transverse motion within an atomic gravimeter-NJP13-065025-2011.pdf',
    citedFrom: [],
    usedBy: ['qcl_gravimeter_systematics', 'verification_v3_transverse_motion'],
    usedFor: 'The gravimeter example’s instrument, and the budget, protocol and temperature scan V3 reruns.',
  },
  {
    title: 'Effective velocity distribution in an atom gravimeter: effect of the convolution with the response of the detection',
    authors: 'T. Farah, P. Gillot, B. Cheng, A. Landragin, S. Merlet, F. Pereira dos Santos',
    ref: 'arXiv:1406.5998v1 [physics.atom-ph]',
    year: '2014',
    file: 'Effective velocity distribution in an atom gravimeter-1406.5998.pdf',
    citedFrom: [],
    usedBy: ['qcl_gravimeter_systematics', 'verification_v2_clipping'],
    usedFor: 'The clipping candidate and its 14.2 µGal per mm, and the wrong-sign Coriolis result V2 reruns.',
  },
  {
    title: 'Off-resonant Raman transitions impact in an atom interferometer',
    authors: 'A. Gauguet, T. E. Mehlstäubler, T. Lévèque, J. Le Gouët, W. Chaibi, B. Canuel, A. Clairon, F. Pereira Dos Santos, A. Landragin',
    ref: 'arXiv:0809.0149v1 [physics.atom-ph]',
    year: '2008',
    file: 'Off-resonant Raman transitions impact in an atom interferometer-0809.0149.pdf',
    citedFrom: [],
    usedBy: ['qcl_gravimeter_systematics', 'verification_v5_light_shift'],
    usedFor: 'The two-photon light shift’s response to the Rabi frequency and the field, and the light-shift measurement V5 reruns.',
  },
  {
    title: 'Improving the accuracy of atom interferometers with ultracold sources',
    authors: 'R. Karcher, A. Imanaliev, S. Merlet, F. Pereira Dos Santos',
    ref: 'arXiv:1804.04909v1 [physics.atom-ph]',
    year: '2018',
    file: 'Improving the accuracy of atom interferometers with ultracold sources-1804.04909.pdf',
    citedFrom: [],
    usedBy: ['qcl_gravimeter_systematics', 'verification_v1_wavefront'],
    usedFor: 'The wavefront’s response to atom temperature and the cost of the temperature scan, and the wavefront attribution V1 reruns.',
  },
  {
    title: 'Mapping the absolute magnetic field and evaluating the quadratic Zeeman effect induced systematic error in an atom interferometer gravimeter',
    authors: 'Qing-Qing Hu, Christian Freier, Bastian Leykauf, Vladimir Schkolnik, Jun Yang, Markus Krutzik, Achim Peters',
    ref: 'arXiv:1805.05159',
    year: '2017',
    file: 'Mapping the absolute magnetic field and evaluating the quadratic Zeeman effect in an atom interferometer gravimeter-1805.05159.pdf',
    citedFrom: [],
    usedBy: ['qcl_gravimeter_systematics', 'verification_v10_zeeman_field_map'],
    usedFor: 'The quadratic Zeeman response, a s² + b s + c, from the digitised field maps that V10 checks.',
  },
  {
    title: 'Gravity measurements below 10⁻⁹ g with a transportable absolute quantum gravimeter',
    authors: 'Vincent Ménoret, Pierre Vermeulen, Nicolas Le Moigne, Sylvain Bonvalot, Philippe Bouyer, Arnaud Landragin, Bruno Desruelle',
    ref: 'arXiv:1809.04908v1 [physics.atom-ph]',
    year: '2018',
    file: 'Gravity measurements below 1e-9 g with a transportable absolute quantum gravimeter-1809.04908.pdf',
    citedFrom: [],
    usedBy: ['qcl_gravimeter_systematics', 'verification_v6_time_model'],
    usedFor: 'The contrast and the tilt step of the gravimeter example, and the long-term data V6 checks the time model against.',
  },
  {
    title: 'Sensitivity limits of a Raman atom interferometer as a gravity gradiometer',
    authors: 'F. Sorrentino, Q. Bodart, L. Cacciapuoti, Y.-H. Lien, M. Prevedelli, G. Rosi, L. Salvi, G. M. Tino',
    ref: 'arXiv:1312.3741v1 [quant-ph]',
    year: '2014',
    file: 'Sensitivity limits of a Raman atom interferometer as a gravity gradiometer-1312.3741.pdf',
    citedFrom: [],
    usedBy: ['qcl_gradiometer_crosstalk', 'verification_v7_gradiometer_sweep'],
    usedFor: 'The gradiometer example’s instrument, and the parameter sweep V7 reruns.',
  },
  {
    title: 'Integration of a high-fidelity model of quantum sensors with a map-matching filter for quantum-enhanced navigation',
    authors: 'Samuel Lellouch, Michael Holynski',
    ref: 'arXiv:2504.11119v2 [quant-ph]',
    year: '2025',
    file: 'Integration of a high-fidelity model of quantum sensors with a map-matching filter for quantum-enhanced navigation-2504.11119.pdf',
    citedFrom: [],
    usedBy: ['qcl_gradiometer_crosstalk'],
    usedFor: 'The rotation candidate’s centrifugal phase.',
  },
  {
    title: 'Detecting crosstalk errors in quantum information processors',
    authors: 'Mohan Sarovar, Timothy Proctor, Kenneth Rudinger, Kevin Young, Erik Nielsen, Robin Blume-Kohout',
    ref: 'arXiv:1908.09855v3 [quant-ph]',
    year: '2020',
    file: 'Detecting crosstalk errors in quantum information processors-1908.09855.pdf',
    citedFrom: [],
    usedBy: ['verification_v4_crosstalk'],
    usedFor: 'The four crosstalk models V4 regenerates and attributes.',
  },
  {
    title: 'Completely positive linear maps on complex matrices',
    authors: 'Man-Duen Choi',
    ref: 'Linear Algebra Appl. 10, 285–290',
    year: '1975',
    citedFrom: ['types/qgates/channel'],
    usedFor: 'A channel is completely positive exactly when its Choi operator is positive semidefinite, which check_completely_positive tests.',
  },
  {
    title: 'The logic of quantum mechanics',
    authors: 'Garrett Birkhoff, John von Neumann',
    ref: 'Ann. of Math. 37, 823–843',
    year: '1936',
    citedFrom: ['types/verdict/projection'],
    usedFor: 'The orthomodular lattice of projections behind the projection verdict.',
  },
  {
    title: 'Quantum causal models',
    authors: 'Jonathan Barrett, Robin Lorenz, Ognyan Oreshkov',
    ref: 'arXiv:1906.10726v2 [quant-ph]',
    year: '2019',
    file: 'Quantum Causal Models-1906.10726v2.pdf',
    citedFrom: ['types/qcm/dilation'],
    usedFor: 'The factorization a unitary circuit with broken wires induces, which the dilation builds.',
  },
  {
    title: 'Improved simulation of stabilizer circuits',
    authors: 'Scott Aaronson, Daniel Gottesman',
    ref: 'Phys. Rev. A 70, 052328; arXiv:quant-ph/0406196v5',
    year: '2004',
    file: 'Improved simulation of stabilizer circuits-quant-ph-0406196v5.pdf',
    citedFrom: ['types/qcode/clifford_action'],
    usedFor: 'The stabilizer-tableau update rule (§III) that the Clifford-action check applies.',
  },
];

