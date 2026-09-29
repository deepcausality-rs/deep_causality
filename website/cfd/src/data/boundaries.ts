/**
 * Capability boundaries — measured, not guessed.
 *
 * Every entry is a finding with its numbers, sourced to the code that produced
 * it. Three of these refuted a hypothesis the project held; those are the most
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
    title: 'A shock compresses well only when it lines up with the grid axes',
    hitWhen:
      'You capture a shock on a plain Cartesian grid, and the shock does not run along a grid axis.',
    finding:
      'A quantized tensor train (QTT) stores a field as a chain of small matrices, and their size (the bond dimension, χ) depends on how the shock sits against the grid axes, not on how sharp or curved it is. On a 512² grid, a captured misaligned shock makes QTT larger than a dense grid, and aligning the shock makes QTT about 290× smaller. A straight 45° oblique shock is worse than a curved one.',
    numbers: [
      'flat axis-aligned 2-D shock: χ ≈ 5',
      'curved bow shock: χ ≈ 151',
      'straight 45° oblique shock: χ ≈ 394, worse than the curve',
      'body-fitted, both cases: χ ≈ 5',
      'captured curved shock costs 3.1× dense storage; captured oblique, 21.3×',
      'aligning the same curved shock is ~291× smaller than a dense grid',
      'misalignment costs twice: across a captured 5× sound-speed jump the implicit acoustic solve degrades from ρ(A₀⁻¹A₁) = 0.590 to 0.872, toward the divergence threshold at 1 (qtt_acoustic_precond)',
    ],
    study: 'deep_causality_cfd/studies/qtt_rank_study',
    refuted: false,
  },
  {
    id: 'sqrt-side',
    title: 'In 3-D, compressed size grows as the square root of the grid side',
    hitWhen: 'You refine a 3-D grid with a curved shock captured on Cartesian axes.',
    finding:
      'The tensor-train bond dimension χ (the size of the small matrices that make up the compressed field) grows roughly as the square root of the grid side, so it has no bound as resolution rises. The real cost is the solve, not the storage: tensor-train operations cost O(χ²) to O(χ³) per core, so a flight-relevant grid implies χ in the thousands. These figures are lower bounds from a scalar Burgers march, and the square-root law is read from four grid sizes.',
    numbers: [
      'χ ≈ 45 / 56 / 89 / 135 at 16³ / 32³ / 64³ / 128³',
      'χ ~ side^0.53',
      'flat and body-fitted references: χ = 5 and 6, measured at 64³ only; their constancy across the ladder is inferred, not measured',
      'dense/QTT storage ratio, where above 1.0 means QTT is the smaller of the two, crosses 1.0 near 64³ and reaches 2.74× at 128³; the break-even grid is a small-grid artifact, not the finding',
    ],
    study: 'deep_causality_cfd/studies/qtt_rank_3d',
    refuted: false,
  },
  {
    id: 'thickening',
    title: 'Artificial viscosity does not buy back rank in an explicit scheme',
    hitWhen: 'You try to buy rank back by adding artificial viscosity to smear a curved shock.',
    finding:
      'At the stable viscosity, a forming 2-D curved shock still drives the compressed size (the bond) from 7 to 20. The explicit scheme has no room to smear the shock further: stable viscosity ends at 1.25 dx, and the one run beyond it blew up to full rank. So this study could not test thickening as a lever, and the candidates left are aligning the coordinate with the shock and an implicit step.',
    numbers: [
      '2-D forming curved shock at 64²: bond climbs 7 → 20, and grows with resolution',
      'at ν = 6 dx the diffusion number reaches 1.2 ≫ 0.25 and the run blows up to full rank (64)',
    ],
    study: 'deep_causality_cfd/studies/qtt_rank_nonlinear',
    refuted: false,
  },
  {
    id: 'static-fit',
    title: 'A body-fitted grid fitted once does not stay compressed while marching',
    hitWhen: 'You fit a body-fitted grid to the shock once, then march the flow.',
    finding:
      'Fluxes computed on the Cartesian axes push the marched shock off the fitted coordinate, and the bond grows to no better than plain capture. Feedback re-pinning (moving the coordinate with the live shock) is necessary, not optional. The test is a scalar viscous Burgers march at 64² and 128², which shows a direction, not a scaling law.',
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
    title: 'Re-pinning alone does not bound the compressed size either',
    hitWhen: 'You re-pin the coordinate to the live front and still march fluxes across it.',
    finding:
      'The obvious fix also fails: the driver is the angular structure that marching fluxes through the front injects, not the front drifting off the coordinate. What worked in the test is treating the front as an exact Rankine–Hugoniot interface and marching the smooth regions on either side of it. That case transports radially by construction, an idealization a real solver would have to earn, and the test is again scalar Burgers.',
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
    title: 'The simulated retropulsion plume does not reproduce the measured drag collapse',
    hitWhen:
      'You couple a retro-rocket plume into the compressible layer and expect the drag collapse that Jarvinen and Adams measured in wind tunnels.',
    question:
      'Does a jet that carries momentum, so the plume forms in the flow through the same forcing region, recover the drag collapse that the earlier fixed-pressure plume could not?',
    finding:
      'The retropulsion example takes its drag change from the published Jarvinen–Adams correlation because the simulated plume did not reproduce it. When the study simulates the plume as a jet that carries momentum, drag rises steadily with thrust where the wind-tunnel reference shows it collapse, and the dip in total axial force is absent. Both plume models fail on this harness, so the limit lies in the harness (its numerical dissipation and its domain) and not in the model class.',
    numbers: [
      'annulus fraction rises 1.03 → 3.61 across C_T 0.25 → 8',
      '1.413 at C_T 1.00 against the Jarvinen–Adams reference 0.124',
      'sweep minimum 1.031, with no value below unity anywhere',
      'total-axial-force dip absent (monotone across the range)',
      'stagnation interface frozen at x = 0.469–0.531 across a 32× thrust range',
    ],
    attribution:
      'Two effects. The dissipation floor (ν = ½·s_ref·Δx, jet-cell Péclet ≈ 1.3–1.8) pins the stagnation interface at the body face, so injected momentum reads as face pressure. That is the inverse of the blanketing reorganisation the real physics performs, which is why the sign comes out backwards rather than merely the magnitude. The domain is the second limit. The upstream probe leaves the freestream by +6.5% at C_T 0.25 and +285% at C_T 8, so the correlation\'s own transition variable, p_e/p∞ ≈ 7, is out of reach (the sweep tops out at 4.78). Compression is not excluded: every row ran at bond cap 24 and the peak bond reaches 24, and the cap-32 companion run is not committed. The attribution rests on the frozen interface and the domain, not on a two-cap sweep.',
    ruledOut: [
      'Model class is not the cause: both the pinned-envelope and momentum-jet couplings fail, which is what moved the attribution to the harness.',
      'Convergence drift is not the cause: tail-averaged drift is ≤ 0.02% through C_T 2 and at worst 0.14% at the top of the sweep, orders too small to mask a collapse.',
    ],
    consequence:
      'In-flight drag authority stays with the cited A0 correlation instead of a decrement contracted from the field. The shipped retropulsion descent does exactly this: it evaluates the correlation for each branch and treats the marched plume as state realism only. The two plume models also barely overlap (Jarvinen–Adams covers Mach 0.4–2.0, Cordell–Braun Mach 2–4), so for most of the burn the plume geometry sits outside its own model\'s envelope.',
    history:
      'The first harness pinned the entire plume envelope to a uniform ambient-pressure state and appeared to show steady drag reduction (1.208 → 0.647). That was a measurement artifact: the force strip was largely reading the pin itself, which overlapped 20–72% of the strip height. Correcting the model class inverted the sign. The superseded harness is kept under reverted/ with its original output as provenance.',
    study: 'deep_causality_cfd/studies/srp_momentum_jet',
    refuted: true,
  },
  {
    id: 'timing-3d',
    title: 'The 3-D body-fitted marcher runs more than three times over its time budget',
    hitWhen: 'You want the 3-D body-fitted grid that the rank studies say a 3-D run needs.',
    finding:
      'At the smallest candidate grid (16³) the marcher is already more than 3× over the ten-minute corridor budget: the committed run projects 3049 s for the 200-step corridor against 600 s. Larger 3-D grids are a foregone conclusion, so the shipped corridor marches a 2-D layer.',
    numbers: [
      '16³ with bond cap 16: >3× over the 600 s budget',
      'the corridor therefore marches a 2-D layer, with 3-D reserved for stagnation-line validation',
    ],
    study: 'deep_causality_cfd/studies/compressible_carrier_timing',
    refuted: false,
  },
  {
    id: 'precision-alias-drift',
    title: 'Switching the corridor example to 106-bit precision does not compile',
    hitWhen:
      'You switch the FloatType alias to Float106 and expect the plasma-blackout corridor to rerun at 106-bit.',
    finding:
      'The corridor example and a handful of spots in the crate have accumulated f64-specific code, so the switch fails to compile. The generic scalar type reaches most of the way, and the turbulence example still runs f32, f64 and Float106 from one rate field. The corridor was flown at 106-bit once, on an earlier build, and that run is the source of the identical-gates result and the roughly 11x cost ratio, neither of which is reproducible on the tree today.',
    numbers: [
      '44 compile errors on switching the alias: 29 × E0308, 6 × E0631, 5 × E0599, 4 × E0277',
      'example-side diagnostics: corridor/model.rs 37, corridor/main.rs 9, shared/utils.rs 7',
      'crate-side diagnostics: compressible_march_run.rs 4, corridor/branch.rs 2, tensor_bridge/codec.rs 2',
      'the turbulence example is unaffected and still reports horizons of t 21.5 / 44.5 / ~81 across the three precisions',
    ],
    consequence:
      'Treat the corridor round-off result and its cost ratio as a record of an earlier build, not a property you can re-measure. The precision ladder is demonstrable today on the turbulence example and on the manufactured-solution verification target, both committed.',
    study: 'examples/avionics_examples/src/shared/mod.rs',
    refuted: false,
  },
  {
    id: 'regime-diagnostic',
    title: 'The flow-regime classifier reports; it does not change the equations',
    hitWhen:
      'You expect the Knudsen band the classifier reports to change the equations the solver is integrating.',
    finding:
      'RegimeClassify sorts the freestream Knudsen number (a measure of how rarefied the flow is) into a governing model, one of continuum, slip-corrected continuum, transitional or free-molecular, and logs every transition. The crate does not switch closures on that result, and only the continuum closures exist: no slip, transitional or free-molecular closure is implemented. The classification still does real work: a march can stop at a regime change, and the link regime decides whether the navigation filter uses GNSS fixes.',
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
    title: 'The integrator switch exists, and the shipped navigation engine never calls it',
    hitWhen:
      'You expect a trajectory to change integrators on its own when aerodynamic force overtakes gravity.',
    finding:
      'RegimeSwitch and aero_gravity_ratio are public API that express the criterion on the force ratio ε = a_aero/a_grav, and the shipped navigation engine calls neither. While gravity dominates, a trajectory advances on the exact Kustaanheimo–Stiefel (KS) core with aero applied as a between-step kick; once aero dominates, direct Cowell integration is the accurate choice. Applying the switch is the caller\'s job, and a caller who assumes otherwise gets the core integrator everywhere.',
    numbers: [
      'RegimeSwitch and aero_gravity_ratio: public, uncalled by the shipped engine',
      'the criterion is the orbit entry and exit boundary, where the integrator that is exact in orbit loses accuracy in atmosphere',
    ],
    study: 'examples/avionics_examples/cfd/plasma_blackout/corridor',
    refuted: false,
  },
  {
    id: 'turbulence',
    title: 'No turbulence models yet',
    hitWhen: 'You need a wake at flight Reynolds number, a separated unsteady region, or large-eddy simulation (LES).',
    finding:
      'The validated incompressible cases sit at Re 100–1600. A separated unsteady wake has many features that no single body-fitted grid lines up, so the wake\'s compressed size (its bond, 41 in 3-D) is reported and never checked by a gate. Turbulence is scheduled work on the DEC solver, not a permanent exclusion.',
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
  'A gate is a pass or fail check on a result. A passing gate means the measured structure is reproducible, not that a physics target was met. In the retropulsion studies "GATES PASSED" sits directly above a recorded miss against the Jarvinen–Adams reference: the gate keeps the finding from regressing, and the finding is a negative one.';
