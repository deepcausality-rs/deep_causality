/**
 * The sensing section, as data: the two sensing examples and the verifications.
 *
 * Every figure on `/sensing/`, the landing page's sensing section and the hero's fourth row reads
 * from here, so they cannot disagree.
 *
 * Sources, all in this repository:
 *   - the gravimeter run:
 *       cargo run --release -p quantum_examples --example qcl_gravimeter_systematics
 *     instrument values in `qcl_gravimeter_systematics/constants.rs`, each citing its paper; the
 *     responses in `model.rs` (`offset`), which decide which experiment moves which systematic
 *   - the gradiometer run:
 *       cargo run --release -p quantum_examples --example qcl_gradiometer_crosstalk
 *   - the verification: the eight verifications under `deep_causality_quantum/verification/sensing/`,
 *     run as `cargo run --release -p deep_causality_quantum --features qcm --example <name>`
 *
 * Both examples are seeded and print the same output on every run. What is typed, computed and
 * sampled: the instrument values come from the cited papers; the values `constants.rs` marks as
 * placeholders (setup times, tide, leak, dark background) stand in for a lab's measurements; the
 * plans, draw counts, predictions and biases are computed; every observation is drawn from the
 * world the run names as the cause, so the costs a campaign spends and its verdicts follow from
 * simulated data.
 */

/** `x` with `digits` decimals and a typographic minus sign when negative. */
export const signed = (x: number, digits = 0): string =>
  `${x < 0 ? '−' : ''}${Math.abs(x).toFixed(digits)}`;

export type Systematic = {
  id: 'H1' | 'H2' | 'H3' | 'H4' | 'H5' | 'H6' | 'H7';
  name: string;
  /** What the instrument does to it, as `model.rs` writes the response. */
  response: string;
  /** The paper the response comes from, when one does. */
  source?: string;
};

export const systematics: Systematic[] = [
  { id: 'H1', name: 'Coriolis', response: 'flips under a 180° turn about vertical', source: 'Louchet-Chauvet et al. 2011' },
  { id: 'H2', name: 'quadratic Zeeman', response: 'flips with the wave vector; scales with the coil current as a s² + b s + c', source: 'Hu et al. 2017, eq. 9' },
  { id: 'H3', name: 'tilt', response: 'reads only low; a deliberate tilt adds to it' },
  { id: 'H4', name: 'mirror vibration', response: 'vanishes under the accelerometer correction' },
  { id: 'H5', name: 'wavefront', response: 'vanishes at zero atom temperature', source: 'Karcher et al. 2018' },
  { id: 'H6', name: 'two-photon light shift', response: 'scales with the Rabi frequency at constant pulse area; a third follows the field', source: 'Gauguet et al. 2008' },
  { id: 'H7', name: 'clipping', response: 'flips under a turn; moves 14.2 µGal per mm of cloud displacement', source: 'Farah et al. 2014' },
];

export type GravimeterExperiment = {
  id: 'E1' | 'E2' | 'E3' | 'E4' | 'E5' | 'E6' | 'E7' | 'E8';
  name: string;
  /** Setup time in s, from `constants.rs`. */
  setup: number;
  /** Whether `constants.rs` marks the setup time a placeholder. */
  placeholder: boolean;
  /** The systematics whose prediction this configuration moves away from the rest. */
  moves: Systematic['id'][];
  planned: boolean;
};

export const gravimeterExperiments: GravimeterExperiment[] = [
  { id: 'E1', name: 'reverse the wave vector', setup: 30, placeholder: true, moves: ['H2'], planned: true },
  { id: 'E2', name: 'turn the head 180°', setup: 600, placeholder: true, moves: ['H1', 'H7'], planned: true },
  { id: 'E3', name: 'apply the accelerometer correction', setup: 0, placeholder: true, moves: ['H4'], planned: true },
  { id: 'E4', name: 'tilt by 1.5 mrad', setup: 900, placeholder: true, moves: ['H3'], planned: true },
  { id: 'E5', name: 'halve the coil current', setup: 60, placeholder: true, moves: ['H2', 'H6'], planned: false },
  { id: 'E6', name: 'extrapolate to 0 K', setup: 20000, placeholder: false, moves: ['H5'], planned: false },
  { id: 'E7', name: 'halve the Rabi frequency', setup: 60, placeholder: true, moves: ['H6'], planned: true },
  { id: 'E8', name: 'displace the cloud 1 mm', setup: 300, placeholder: true, moves: ['H7'], planned: true },
];

export type CampaignRun = {
  cause: Systematic['id'];
  ran: GravimeterExperiment['id'][];
  seconds: number;
  replans: number;
};

/** One campaign per systematic as the cause of the −5 µGal offset, as the run prints them. */
export const gravimeterCampaigns: CampaignRun[] = [
  { cause: 'H1', ran: ['E1', 'E8', 'E3', 'E2'], seconds: 1815, replans: 3 },
  { cause: 'H2', ran: ['E1'], seconds: 166, replans: 0 },
  { cause: 'H3', ran: ['E1', 'E8', 'E3', 'E2', 'E4'], seconds: 2716, replans: 4 },
  { cause: 'H4', ran: ['E1', 'E8', 'E3'], seconds: 1079, replans: 2 },
  { cause: 'H5', ran: ['E1', 'E8', 'E3', 'E2', 'E4', 'E7'], seconds: 4953, replans: 5 },
  { cause: 'H6', ran: ['E1', 'E8', 'E3', 'E2', 'E4', 'E7'], seconds: 4953, replans: 5 },
  { cause: 'H7', ran: ['E1', 'E8'], seconds: 534, replans: 1 },
];

export const GRAVIMETER = {
  command: 'cargo run --release -p quantum_examples --example qcl_gravimeter_systematics',
  slug: 'qcl-gravimeter-systematics',
  seed: 20261008,
  offsetUgal: -5,
  /** The placeholder tide: the principal lunar semidiurnal term, amplitude in µGal. */
  tideUgal: 80,
  /** Louchet-Chauvet et al. 2011, Table 1: the mean tidal correction over twelve days, in µGal. */
  publishedTideUgal: -18.8,
  floorBits: 5,
  agreementSigmas: 3,
  /** Louchet-Chauvet et al. 2011, as the run's header prints the instrument. */
  interrogationTime: '70 ms',
  sensitivity: '22.1 µGal/√Hz',
  whiteNoise: '5000 s',
  planCost: 4956,
  /** The cheapest and the dearest campaign over the seven causes. */
  cheapest: Math.min(...gravimeterCampaigns.map((c) => c.seconds)),
  dearest: Math.max(...gravimeterCampaigns.map((c) => c.seconds)),
  /** The run with a +5 µGal offset: the baseline refuses tilt. */
  positive: { predictsUgal: 76.43, readsUgal: 82.54, planCost: 4056, campaign: 4054 },
  /** The run whose contexts record no tide: the baseline refuses all seven. */
  noTide: { predictsUgal: -5.0, readsUgal: 72.41 },
  /** The run with clipping as the cause and left off the list. */
  clippingOff: { planCost: 4588, ran: ['E1', 'E3', 'E2'], seconds: 1447, survivor: 'H1' },
  /** The hero's output line, copied from the run. */
  heroLine: 'campaign ran E1 k reversed for 166 s; … H2 quadratic Zeeman survives',
} as const;

export type GradiometerExperiment = {
  id: 'E0' | 'E1' | 'E2' | 'E3';
  name: string;
  reads: string;
  /** Predicted read-out under each candidate, in the world where the leak runs from A to B. */
  predicts: Record<GradiometerId, number>;
  /** Effective draws and cost the plan sizes, when planned. */
  draws?: number;
  seconds?: number;
};

export type GradiometerId = 'H1' | 'H2' | 'H3' | 'H4';

export const gradiometerCandidates = [
  { id: 'H1', name: 'leak A→B', mechanism: 'cloud A’s fluorescence reaches B’s detection signal', biased: true },
  { id: 'H2', name: 'leak B→A', mechanism: 'cloud B’s fluorescence reaches A’s detection signal', biased: true },
  { id: 'H3', name: 'shared phase', mechanism: 'the common laser and mirror phase alone', biased: false },
  { id: 'H4', name: 'rotation', mechanism: 'a centrifugal phase 2 b k_eff T² Ω² from platform rotation', biased: true },
] as const;

export const gradiometerExperiments: GradiometerExperiment[] = [
  { id: 'E0', name: 'passive', reads: 'both signals excited', predicts: { H1: 0.25982, H2: 0.25982, H3: 0.25982, H4: 0.25982 } },
  { id: 'E1', name: 'brighten A, darken B', reads: 'B’s signal excited', predicts: { H1: 0.0788, H2: 0.02, H3: 0.02, H4: 0.02 }, draws: 340, seconds: 60.0 },
  { id: 'E2', name: 'brighten B, darken A', reads: 'A’s signal excited', predicts: { H1: 0.02, H2: 0.0788, H3: 0.02, H4: 0.02 }, draws: 340, seconds: 60.0 },
  { id: 'E3', name: 'rotate at 1 mrad/s', reads: 'the two signals disagreeing', predicts: { H1: 0.5156, H2: 0.5156, H3: 0.5156, H4: 0.52632 }, draws: 60265, seconds: 601.8 },
];

export const GRADIOMETER = {
  command: 'cargo run --release -p quantum_examples --example qcl_gradiometer_crosstalk',
  slug: 'qcl-gradiometer-crosstalk',
  seed: 20261008,
  planCost: 721.8,
  /** Sorrentino et al. 2014, as the run's header prints the instrument. */
  interrogationTime: '160 ms',
  contrast: 0.45,
  whiteNoise: '8000 s',
  /** Placeholders for the lab's measurements. */
  leak: 0.06,
  darkBackground: 0.02,
  /** The gradient bias a 6 % leak implies, and rotation at Earth's horizontal rate. */
  leakBias: { rad: -0.044, eotvos: -356 },
  rotationBias: { rad: 6.9e-4, eotvos: 5.5 },
} as const;

/** The published values the method rests on, each with its source. */
export const FIELD = {
  vessel: {
    cite: 'Everitt et al., arXiv:2608.25563',
    vessel: '29 m',
    routeNmi: 45,
    hours: 6,
    aidedNmi: 2.2,
    unaidedNmi: 14,
  },
  model: { cite: 'Lellouch and Holynski, arXiv:2504.11119', tiltDeg: 3.3 },
  budget: { cite: 'Louchet-Chauvet et al., New J. Phys. 13, 065025', wavefrontUgal: '4.0', totalUgal: '5.2' },
} as const;

export type Verification = {
  id: string;
  source: string;
  ref: string;
  /** The question the paper answered, in plain words. */
  question: string;
  /** Whether QCL reaches the paper's answer, and the answer. */
  answer: string;
  /** What else the rerun found, in one plain line. */
  also?: string;
  checks: number;
  example: string;
};

/** The command that runs a verification, by its example name. */
export const verificationCommand = (example: string) =>
  `cargo run --release -p deep_causality_quantum --features qcm --example ${example}`;

/**
 * One row per paper, in plain words. The figures, tables and equations each line rests on are in
 * `deep_causality_quantum/verification/sensing/README.md` and the verifications' output.
 */
export const verifications: Verification[] = [
  {
    id: 'V1',
    source: 'Karcher et al. 2018',
    ref: 'arXiv:1804.04909',
    question: 'Which effect shifts gravity as the atoms cool?',
    answer: 'Yes: the wavefront',
    also: 'Two of the paper’s figures disagree with each other. Its bound on atom interactions holds at one standard error; at three, interactions remain possible.',
    checks: 9,
    example: 'verification_v1_wavefront',
  },
  {
    id: 'V2',
    source: 'Farah et al. 2014',
    ref: 'arXiv:1406.5998',
    question: 'Why did a Coriolis test show the wrong sign?',
    answer: 'Yes: detection clipping',
    also: 'The text uses two conversion factors and one wrong unit.',
    checks: 5,
    example: 'verification_v2_clipping',
  },
  {
    id: 'V3',
    source: 'Louchet-Chauvet et al. 2011',
    ref: 'New J. Phys. 13, 065025',
    question: 'Which change separates Coriolis from the wavefront?',
    answer: 'Yes: a 180° turn',
    also: 'One uncertainty total and one equation disagree with their own inputs.',
    checks: 8,
    example: 'verification_v3_transverse_motion',
  },
  {
    id: 'V4',
    source: 'Sarovar et al. 2020',
    ref: 'arXiv:1908.09855',
    question: 'Which qubits cross-talk, and how?',
    answer: 'Yes, in all four cases',
    also: 'The paper’s own detection method, rerun on the same data, misses the published answer in three.',
    checks: 8,
    example: 'verification_v4_crosstalk',
  },
  {
    id: 'V5',
    source: 'Gauguet et al. 2008',
    ref: 'arXiv:0809.0149',
    question: 'Is the shift a two-photon light shift?',
    answer: 'Yes: 32 against 33 mrad',
    also: 'Repeat measurements spread ±16 %, against a stated ±10 %.',
    checks: 8,
    example: 'verification_v5_light_shift',
  },
  {
    id: 'V6',
    source: 'Ménoret et al. 2018',
    ref: 'arXiv:1809.04908',
    question: 'How far does averaging reduce the noise?',
    answer: 'Yes, up to one hour',
    also: 'Day-long averages show noise beyond white noise.',
    checks: 5,
    example: 'verification_v6_time_model',
  },
  {
    id: 'V7',
    source: 'Sorrentino et al. 2014',
    ref: 'arXiv:1312.3741',
    question: 'What drives each drift of a gradiometer?',
    answer: 'Yes, three of three',
    also: 'The text’s list of main drifts leaves out the third largest.',
    checks: 4,
    example: 'verification_v7_gradiometer_sweep',
  },
  {
    id: 'V10',
    source: 'Hu et al. 2017',
    ref: 'arXiv:1805.05159',
    question: 'How large is the quadratic Zeeman bias?',
    answer: 'Yes: 2.01 against 2.04 µGal',
    also: 'One coefficient’s unit is printed wrong.',
    checks: 6,
    example: 'verification_v10_zeeman_field_map',
  },
];

export const VERIFICATION = {
  count: verifications.length,
  checks: verifications.reduce((n, v) => n + v.checks, 0),
  readme: 'deep_causality_quantum/verification/sensing/README.md',
} as const;

/** The setup time of the wavefront's own test, the temperature scan E6, in seconds. */
export const SCAN_SETUP_S = gravimeterExperiments.find((e) => e.id === 'E6')!.setup;

const SUPERSCRIPT: Record<string, string> = {
  '-': '⁻', '0': '⁰', '1': '¹', '2': '²', '3': '³', '4': '⁴', '5': '⁵', '6': '⁶', '7': '⁷', '8': '⁸', '9': '⁹',
};

/** `value` in scientific notation with Unicode superscripts, e.g. `−3.6 × 10⁻⁷`. */
export const scientific = (value: number, digits: number): string => {
  const [mantissa, exponent] = value.toExponential(digits).split('e');
  const sign = mantissa.startsWith('-') ? '−' : '';
  const power = String(Number(exponent)).split('').map((ch) => SUPERSCRIPT[ch]).join('');
  return `${sign}${mantissa.replace('-', '')} × 10${power}`;
};

/** One eötvös is 10⁻⁹ s⁻². */
export const EOTVOS_PER_S2 = 1e-9;
