/**
 * Capability boundaries — measured, not guessed.
 *
 * Every entry is a finding with its numbers, sourced to the code that produced
 * it. Four of these refuted a hypothesis the project held; those are the most
 * useful entries on the site and are marked `refuted`.
 */

export interface Boundary {
  id: string;
  title: string;
  /** Where you hit it. Stated as the situation, not as a feature gap. */
  hitWhen: string;
  finding: string;
  numbers: string[];
  /**
   * Repo-relative path to the code behind the finding, rendered verbatim.
   * Usually a study under `deep_causality_cfd/studies/`, but a boundary can
   * equally come from a shipped example or a verification target, so the path
   * carries its own location rather than assuming one.
   */
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
      'straight 45° oblique shock: χ ≈ 394, worse than the curve',
      'body-fitted, both cases: χ ≈ 5',
      'captured curved shock costs 3.1× dense storage; captured oblique, 21.3×',
      'aligning the same curved shock is ~291× smaller than capturing it',
      'misalignment costs twice: across a captured 5× sound-speed jump the implicit acoustic solve degrades from ρ(A₀⁻¹A₁) = 0.590 to 0.872, toward the divergence threshold at 1 (qtt_acoustic_precond)',
    ],
    study: 'deep_causality_cfd/studies/qtt_rank_study',
    refuted: false,
  },
  {
    id: 'sqrt-side',
    title: 'In 3-D, captured-shock rank grows as √side',
    hitWhen: 'You refine a 3-D grid with a curved shock captured on Cartesian axes.',
    finding:
      'Bond dimension scales as roughly the square root of the side length: bounded, but unbounded in resolution. The real cost is the solve, not the storage: tensor-train ops are O(χ²)–O(χ³) per core, so a flight-relevant grid implies χ in the thousands.',
    numbers: [
      'χ ≈ 45 / 56 / 89 / 135 at 16³ / 32³ / 64³ / 128³',
      'χ ~ side^0.53',
      'flat and body-fitted stay at χ ≈ 5–6 across the same ladder',
      'dense/QTT storage ratio, where above 1.0 means QTT is the smaller of the two, crosses 1.0 near 64³ and reaches 2.74× at 128³; the break-even grid is a small-grid artifact, not the finding',
    ],
    study: 'deep_causality_cfd/studies/qtt_rank_3d',
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
    study: 'deep_causality_cfd/studies/qtt_rank_nonlinear',
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
    study: 'deep_causality_cfd/studies/qtt_rank_fitted_dynamic',
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
    study: 'deep_causality_cfd/studies/qtt_repin_marcher',
    refuted: true,
  },
  {
    id: 'srp',
    title: 'Supersonic retropropulsion drag collapse is not reproducible on this harness',
    hitWhen:
      'You couple a retro-plume into the compressible layer and expect the measured Jarvinen–Adams drag collapse.',
    question:
      'Does a momentum-carrying jet, a formed plume through the same forcing seam, recover the drag collapse that the earlier pinned-envelope imprint could not?',
    finding:
      'No. The measured result is monotone drag augmentation where the wind-tunnel reference shows collapse, and the total-axial-force dip is absent entirely. The answer moved the attribution: this is not the model class failing, it is the harness.',
    numbers: [
      'annulus fraction rises 1.03 → 3.61 across C_T 0.25 → 8',
      '1.413 at C_T 1.00 against the Jarvinen–Adams reference 0.124',
      'sweep minimum 1.031, with no value below unity anywhere',
      'total-axial-force dip absent (monotone across the range)',
      'stagnation interface frozen at x = 0.469–0.531 across a 32× thrust range',
    ],
    attribution:
      'The dissipation floor (ν = ½·s_ref·Δx, jet-cell Péclet ≈ 1.3–1.8) pins the stagnation interface at the body face, so injected momentum reads as face pressure. That is the inverse of the blanketing reorganisation the real physics performs, which is why the sign comes out backwards rather than merely the magnitude.',
    ruledOut: [
      'Compression is innocent: raising the bond cap 24 → 32 (exact at 2⁵) leaves every observable unchanged at displayed precision.',
      'Model class is not the cause: both the pinned-envelope and momentum-jet couplings fail, which is what moved the attribution to the discretization.',
      'Convergence drift is not the cause: tail-averaged drift is ≤ 0.02% through C_T 2 and at worst 0.14% at the top of the sweep, orders too small to mask a collapse.',
    ],
    consequence:
      'In-flight drag authority stays with the cited A0 correlation rather than a decrement contracted from the field. The shipped retropulsion descent applies exactly this: it evaluates the correlation per branch and treats the marched plume as state realism only. A further consequence measured there is that the two SRP models barely overlap (Jarvinen-Adams covers Mach 0.4-2.0, Cordell-Braun Mach 2-4), so the plume geometry sits outside its own envelope for most of the burn.',
    history:
      'The first harness pinned the entire plume envelope to a uniform ambient-pressure state and appeared to show monotone drag reduction (1.208 → 0.647). That was a measurement artifact: the force strip was largely reading the pin itself, which overlapped 20–72% of the strip height. Correcting the model class inverted the sign. The superseded harness is kept under reverted/ with its original output as provenance.',
    study: 'deep_causality_cfd/studies/srp_momentum_jet',
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
    study: 'deep_causality_cfd/studies/compressible_carrier_timing',
    refuted: false,
  },
  {
    id: 'precision-alias-drift',
    title: 'Precision as a parameter does not currently reach the corridor',
    hitWhen:
      'You switch the FloatType alias to Float106 and expect the plasma-blackout corridor to rerun at 106-bit.',
    finding:
      'It does not compile. The scalar generic reaches most of the way, and the turbulence example still runs f32, f64 and Float106 from one rate field, but the corridor example has accumulated f64-specific code and a handful of spots in the crate have too. The corridor was flown at 106-bit once, on the superseded surrogate-era build, and that run is the source of the identical-gates result and the roughly 11x cost ratio. Neither is reproducible on the tree today.',
    numbers: [
      '44 compile errors on switching the alias: 29 × E0308, 6 × E0631, 5 × E0599, 4 × E0277',
      'example-side diagnostics: corridor/model.rs 37, corridor/main.rs 9, shared/utils.rs 7',
      'crate-side diagnostics: compressible_march_run.rs 4, corridor/branch.rs 2, tensor_bridge/codec.rs 2',
      'the turbulence example is unaffected and still reports horizons of t 21.5 / 44.5 / ~81 across the three precisions',
    ],
    consequence:
      'Treat the corridor round-off result and its cost ratio as a record of an earlier build rather than as a property you can re-measure. The precision ladder is demonstrable today on the turbulence example and on the manufactured-solution verification target, both of which are committed.',
    study: 'examples/avionics_examples/src/shared/mod.rs',
    refuted: false,
  },
  {
    id: 'regime-diagnostic',
    title: 'Flow-regime classification is a diagnostic, not a closure switch',
    hitWhen:
      'You expect the Knudsen band the classifier reports to change the equations the solver is integrating.',
    finding:
      'RegimeClassify bands the freestream Knudsen number into a governing model, one of continuum, slip-corrected continuum, transitional or free-molecular, and logs every transition. The crate does not switch closures on that classification, and no slip, transitional or free-molecular closure is implemented. What the classification does drive is real: march predicates read it, so a regime change is an event a run can stop at, and the link regime gates the navigation filter.',
    numbers: [
      'the corridor crosses one Knudsen band (slip to continuum) and logs both transitions',
      'four regime entries in the corridor log; eight regime transitions across the retropulsion descent',
      'closures implemented for the continuum band only',
    ],
    study: 'examples/avionics_examples/cfd/plasma_blackout/corridor',
    refuted: false,
  },
  {
    id: 'regime-switch-uncalled',
    title: 'The integrator switch is public API the shipped engine never calls',
    hitWhen:
      'You expect a trajectory to change integrators on its own when aerodynamic force overtakes gravity.',
    finding:
      'RegimeSwitch and aero_gravity_ratio express the criterion on the force ratio ε = a_aero/a_grav: while gravity dominates, a trajectory advances on the exact KS-conformal core with aero applied as a between-step kick; once aero dominates, direct Cowell integration is the accurate choice. Both are public API, and the shipped navigation engine does not call either. Applying the switch is the caller\'s job, and a caller who assumes otherwise gets the core integrator everywhere.',
    numbers: [
      'RegimeSwitch and aero_gravity_ratio: public, uncalled by the shipped engine',
      'the criterion is the orbit entry and exit boundary, where the integrator that is exact in orbit loses accuracy in atmosphere',
    ],
    study: 'examples/avionics_examples/cfd/plasma_blackout/corridor',
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
    study: 'deep_causality_cfd/studies/qtt_reentry_3d',
    refuted: false,
  },
];

/**
 * How to read a passing gate. This misreading is the one the study text
 * explicitly guards against, so it belongs on the site.
 */
export const gateSemantics =
  'A passing gate means the measured structure is reproducible, not that a physics target was met. In the retropulsion studies "GATES PASSED" sits directly above a recorded miss against the Jarvinen–Adams reference: the gate protects the finding from regressing, and the finding is a negative one.';
