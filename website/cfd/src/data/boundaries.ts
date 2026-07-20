/**
 * Capability boundaries — measured, not guessed.
 *
 * Every entry is a finding from a study in `deep_causality_cfd/studies/`, with
 * its numbers. Three of these refuted a hypothesis the project held; those are
 * the most useful entries on the site and are marked `refuted`.
 */

export interface Boundary {
  id: string;
  title: string;
  /** Where you hit it. Stated as the situation, not as a feature gap. */
  hitWhen: string;
  finding: string;
  numbers: string[];
  study: string;
  /** True when running the experiment overturned a project assumption. */
  refuted: boolean;

  /* --- Optional fields for a fully worked negative result. ---
   *
   * A negative finding is only useful if it says what is responsible and what
   * has been eliminated. Without those, a reader cannot tell a real physical
   * limit from a bug someone has not found yet. Entries that carry these four
   * fields render as a full record rather than a bullet list.
   */

  /** The hypothesis as it was posed, before the answer was known. */
  question?: string;
  /** What the measurement attributes the result to. */
  attribution?: string;
  /** Candidate causes the measurement eliminated. The load-bearing part. */
  ruledOut?: string[];
  /** What follows for someone using the crate today. */
  consequence?: string;
  /** Prior attempts and why they were superseded. */
  history?: string;
}

export const boundaries: Boundary[] = [
  {
    id: 'alignment',
    title: 'Compression is conditional on coordinate alignment',
    hitWhen:
      'You put a shock on a Cartesian QTT grid and it is not aligned with the codec axes.',
    finding:
      'The rank driver is coordinate alignment, not sharpness and not curvature. Capturing a misaligned shock makes QTT net-negative against a dense grid; aligning it makes QTT a ~290× win. A straight 45° oblique shock is worse than a curved one.',
    numbers: [
      'flat axis-aligned 2-D shock: χ ≈ 5',
      'curved bow shock: χ ≈ 151',
      'straight 45° oblique shock: χ ≈ 394 — worse than the curve',
      'body-fitted, both cases: χ ≈ 5',
      'captured curved shock costs 3.1× dense storage; captured oblique, 21.3×',
      'aligning the same curved shock is ~291× smaller than capturing it',
      'misalignment costs twice: across a captured 5× sound-speed jump the implicit acoustic solve degrades from ρ(A₀⁻¹A₁) = 0.590 to 0.872, toward the divergence threshold at 1 (qtt_acoustic_precond)',
    ],
    study: 'qtt_rank_study',
    refuted: false,
  },
  {
    id: 'sqrt-side',
    title: 'In 3-D, captured-shock rank grows as √side',
    hitWhen: 'You refine a 3-D grid with a curved shock captured on Cartesian axes.',
    finding:
      'Bond dimension scales as roughly the square root of the side length — bounded, but unbounded in resolution. The real cost is the solve, not the storage: tensor-train ops are O(χ²)–O(χ³) per core, so a flight-relevant grid implies χ in the thousands.',
    numbers: [
      'χ ≈ 45 / 56 / 89 / 135 at 16³ / 32³ / 64³ / 128³',
      'χ ~ side^0.53',
      'flat and body-fitted stay at χ ≈ 5–6 across the same ladder',
      'dense/QTT storage ratio — above 1.0 means QTT is the smaller of the two — crosses 1.0 near 64³ and reaches 2.74× at 128³; the break-even grid is a small-grid artifact, not the finding',
    ],
    study: 'qtt_rank_3d',
    refuted: false,
  },
  {
    id: 'thickening',
    title: 'Viscous thickening is not a rank lever',
    hitWhen: 'You try to buy rank back by adding artificial viscosity to smear a curved shock.',
    finding:
      'The hypothesis was refuted by running it. Curved-shock rank is set by misalignment, which viscosity cannot remove. Worse, naive over-thickening is diffusion-CFL-unstable, so you cannot simply crank it in an explicit scheme.',
    numbers: [
      '2-D forming curved shock at 64²: bond climbs 7 → 20, and grows with resolution',
      'at ν = 6 dx the diffusion number reaches 1.2 ≫ 0.25 and the run blows up to full rank (64)',
    ],
    study: 'qtt_rank_nonlinear',
    refuted: true,
  },
  {
    id: 'static-fit',
    title: 'A static body-fitted coordinate does not survive marching',
    hitWhen: 'You fit the coordinate once and then march.',
    finding:
      'Under Cartesian fluxes the marched front drifts off the fitted coordinate and the bond grows to no better than the plain capture. Feedback re-pinning is necessary, not optional.',
    numbers: [
      'axis-aligned front: bond 7 at both 64² and 128², flat in resolution',
      'misaligned curved shock: 20 → 25',
      'marched off a static fit: 25 → 35, no better than capture',
    ],
    study: 'qtt_rank_fitted_dynamic',
    refuted: true,
  },
  {
    id: 'repin',
    title: 'Re-pinning alone does not bound the marched rank either',
    hitWhen: 'You re-pin the coordinate to the live front and still march fluxes across it.',
    finding:
      'The obvious fix also fails. The driver is the angular structure that a flux-through-front march injects, not the front drifting off the coordinate. The lever that works is treating the front as an exact Rankine–Hugoniot interface and marching smooth regions either side of it.',
    numbers: [
      'marching Cartesian fluxes through the front: 25 → 35 with resolution',
      '18 re-pins at 128² does not curb it',
      'radial flux with the front as a tracked interface: bond 8, flat in resolution',
    ],
    study: 'qtt_repin_marcher',
    refuted: true,
  },
  {
    id: 'srp',
    title: 'Supersonic retropropulsion drag collapse is not reproducible on this harness',
    hitWhen:
      'You couple a retro-plume into the compressible layer and expect the measured Jarvinen–Adams drag collapse.',
    question:
      'Does a momentum-carrying jet — a formed plume through the same forcing seam — recover the drag collapse that the earlier pinned-envelope imprint could not?',
    finding:
      'No. The measured result is monotone drag augmentation where the wind-tunnel reference shows collapse, and the total-axial-force dip is absent entirely. The answer moved the attribution: this is not the model class failing, it is the harness.',
    numbers: [
      'annulus fraction rises 1.03 → 3.61 across C_T 0.25 → 8',
      '1.413 at C_T 1.00 against the Jarvinen–Adams reference 0.124',
      'sweep minimum 1.031 — no value below unity anywhere',
      'total-axial-force dip absent (monotone across the range)',
      'stagnation interface frozen at x = 0.469–0.531 across a 32× thrust range',
    ],
    attribution:
      'The dissipation floor (ν = ½·s_ref·Δx, jet-cell Péclet ≈ 1.3–1.8) pins the stagnation interface at the body face, so injected momentum reads as face pressure. That is the inverse of the blanketing reorganisation the real physics performs, which is why the sign comes out backwards rather than merely the magnitude.',
    ruledOut: [
      'Compression is innocent — raising the bond cap 24 → 32 (exact at 2⁵) leaves every observable unchanged at displayed precision.',
      'Model class is not the cause — both the pinned-envelope and momentum-jet couplings fail, which is what moved the attribution to the discretization.',
      'Convergence drift is not the cause — tail-averaged drift is ≤ 0.02% through C_T 2 and at worst 0.14% at the top of the sweep, orders too small to mask a collapse.',
    ],
    consequence:
      'In-flight drag authority stays with the cited A0 correlation rather than a decrement contracted from the field. Any retropulsion work on this crate should read drag from the correlation and treat the marched plume as flow structure only.',
    history:
      'The first harness pinned the entire plume envelope to a uniform ambient-pressure state and appeared to show monotone drag reduction (1.208 → 0.647). That was a measurement artifact: the force strip was largely reading the pin itself, which overlapped 20–72% of the strip height. Correcting the model class inverted the sign. The superseded harness is kept under reverted/ with its original output as provenance.',
    study: 'srp_momentum_jet',
    refuted: true,
  },
  {
    id: 'timing-3d',
    title: 'The 3-D fitted marcher is over its own wall-clock budget',
    hitWhen: 'You want the 3-D body-fitted shell that the rank studies say 3-D tractability requires.',
    finding:
      'Measured at the smallest candidate grid it is already more than 3× over the ten-minute corridor budget, so larger 3-D grids are a foregone conclusion. The shipped corridor runs the 2-D fallback for this reason.',
    numbers: [
      '16³ with bond cap 16: >3× over the 600 s budget',
      'the corridor therefore marches a 2-D layer, with 3-D reserved for stagnation-line validation',
    ],
    study: 'compressible_carrier_timing',
    refuted: false,
  },
  {
    id: 'turbulence',
    title: 'Turbulence is staged, not available',
    hitWhen: 'You need a flight-Reynolds wake, a separated unsteady region, or LES.',
    finding:
      'Wake rank is reported and never gated: a separated unsteady wake is a multi-feature structure no single fitted coordinate aligns. The validated incompressible cases sit at Re 100–1600. This is scheduled work on the DEC solver, not a permanent exclusion.',
    numbers: [
      '3-D wake bond 41, recorded as an out-of-scope datapoint',
      'the VIV example sweeps Re 100–160 and claims nothing turbulent',
    ],
    study: 'qtt_reentry_3d',
    refuted: false,
  },
];

/**
 * How to read a passing gate. This misreading is the one the study text
 * explicitly guards against, so it belongs on the site.
 */
export const gateSemantics =
  'A passing gate means the measured structure is reproducible, not that a physics target was met. In the retropulsion studies "GATES PASSED" sits directly above a recorded miss against the Jarvinen–Adams reference: the gate protects the finding from regressing, and the finding is a negative one.';
