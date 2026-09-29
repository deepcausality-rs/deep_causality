/**
 * Validation records — the adoption document's data.
 *
 * Every number here is copied from a committed artifact in
 * `deep_causality_cfd/verification/<target>/` — usually `baseline.txt`, but for
 * `dec_cylinder_verification` the recorded run output `re100_16_resolved.txt` —
 * or that target's README. Nothing is rounded for presentation and nothing is
 * estimated.
 *
 * All figures measured at `f64`, release build, on the machine named by
 * `MACHINE` in `src/consts.ts`. Quote that constant rather than restating the
 * spec, so a single edit moves every attribution on the site.
 *
 * `status` is deliberately narrow:
 *   'quantitative' — checked against a published reference to a stated tolerance
 *   'anchored'     — checked against flight data at order-of-magnitude
 *   'invariant'    — checked against a property the discretization must preserve
 *   'structural'   — gates rank or cost, NOT physical accuracy
 */

export type ValidationStatus = 'quantitative' | 'anchored' | 'invariant' | 'structural';

export interface ValidationRecord {
  id: string;
  target: string;
  family: 'DEC' | 'QTT' | 'Analytic';
  status: ValidationStatus;
  /** One plain sentence: the result, with its key number against the reference. */
  headline: string;
  /** One plain sentence: where this record falls short of a full accuracy claim. Derived from `caveat`. */
  shortfall: string;
  /** One line: what physical case runs. */
  problem: string;
  /** Published reference, or the invariant when there is no paper. */
  reference: string;
  /** Rows of measured-vs-reference. Kept as strings so units stay attached. */
  measured: { quantity: string; computed: string; expected: string; delta: string }[];
  command: string;
  /** The caveat a chief engineer would ask about. Never omitted. */
  caveat: string;
  /** True when the repo has a committed run artifact (baseline.txt, or an
   * equivalently committed recorded run output) carrying these figures. */
  hasArtifact: boolean;
}

export const validation: ValidationRecord[] = [
  {
    id: 'qtt-sod',
    target: 'qtt_sod',
    family: 'QTT',
    status: 'quantitative',
    headline: 'Density error 0.0175 against a 0.03 tolerance, and pressure between the waves 0.3032 against the exact 0.3031.',
    shortfall: 'The first-order scheme smears the contact discontinuity, so the tolerance bounds the average error, not how sharp each wave is.',
    problem: 'Sod shock tube: high-pressure gas meets low-pressure gas at t = 0. γ = 1.4, marched to t = 0.2 on 512 cells.',
    reference: 'The exact Riemann solution, with the canonical pressure p* = 0.3031 between the waves.',
    measured: [
      { quantity: 'Density, L1 over |x| ≤ 0.5', computed: '0.0175', expected: '0 (exact)', delta: 'tol 0.03' },
      { quantity: 'Velocity, L1 over |x| ≤ 0.5', computed: '0.0274', expected: '0 (exact)', delta: 'tol 0.03' },
      { quantity: 'Pressure, L1 over |x| ≤ 0.5', computed: '0.0151', expected: '0 (exact)', delta: 'tol 0.03' },
      { quantity: 'Star-region pressure (x = 0.10, 0.30)', computed: '0.3032, 0.3033', expected: '0.3031 (canonical p*)', delta: '≤ +0.0002' },
    ],
    command: 'cargo run --release -p deep_causality_cfd --example qtt_sod',
    caveat:
      'The first-order Rusanov scheme smears the contact, so the bound is on mean accuracy, not on peak resolution. The nonlinear flux and the equation of state are evaluated point by point: the field is expanded to full grid values, computed, then compressed again (dequantize, compute, requantize). A rank-preserving form (TT-cross) is the named upgrade for larger grids.',
    hasArtifact: true,
  },
  {
    id: 'qtt-ramc',
    target: 'qtt_ramc_stagline',
    family: 'QTT',
    status: 'anchored',
    headline: 'Peak electron density 2.251e19 m⁻³ against about 1e19 m⁻³ in flight: about a factor of 2.3 high (0.35 decades; one decade is a factor of 10). The run accepts anything within a factor of 5.',
    shortfall: 'Order of magnitude only: the chemistry is uncalibrated, and the ±0.70-decade band was pinned from this measurement.',
    problem: 'The plasma layer along the stagnation line (the streamline that ends at the nose) of the RAM-C II reentry vehicle at about 71 km and Mach 25. The shock is a fitted interface with the exact Rankine–Hugoniot jump across it.',
    reference:
      'RAM-C II flight experiment, NASA Langley (1970). Park, Nonequilibrium Hypersonic Aerothermodynamics (1990). Gupta–Yos–Thompson–Lee, NASA RP-1232 (1990).',
    measured: [
      { quantity: 'Peak n_e (uncalibrated finite-rate network)', computed: '2.251e19 m⁻³', expected: '~1e19 m⁻³', delta: '+0.35 dec (band ±0.70)' },
      { quantity: 'Peak n_e (closed-form Park-2T controller)', computed: '5.31e17 m⁻³', expected: '~1e19 m⁻³', delta: '−1.27 dec (reported, not re-admitted)' },
      { quantity: 'Post-shock temperature T₂', computed: '8044 K', expected: '~10⁴ K band', delta: 'in band' },
      { quantity: 'Plasma frequency ω_p', computed: '4.111e10 rad/s', expected: '> 9.40e9 comms band', delta: 'blackout true' },
      { quantity: 'Relaxation-profile bond', computed: '2', expected: 'O(1)', delta: 'cap 4' },
    ],
    command: 'cargo run --release -p deep_causality_cfd --example qtt_ramc_stagline',
    caveat:
      'The uncalibrated finite-rate network lands 0.35 decades above the flight anchor, inside the ±0.70-decade chemistry-spread band. That band was pinned from this measurement, and the program labels every gate in this record a tripwire, so a pass shows the result has not regressed and does not derive from the flight data\'s stated uncertainty. The closed-form Park-2T controller lands 1.27 decades below the anchor after the N₂–N₂ reduced-mass correction (μ = 14.007). Its former near-anchor landing came from an invalid μ = 7.0 (the N–N atomic pair, which has no vibrational mode), and the offset is reported rather than re-admitted. The controller is still a two-temperature Saha surrogate. Setting the electron temperature equal to the vibrational one (T_e = T_ve) is worth roughly 2×. The landing is also sensitive to the Millikan–White vibrational relaxation model τ_vt, within the documented 2–5× chemistry-model spread. γ = 1.1 is an effective-γ closure for strongly dissociated air, not a perfect gas.',
    hasArtifact: true,
  },
  {
    id: 'dec-cavity',
    target: 'dec_lid_cavity_re1000_verification',
    family: 'DEC',
    status: 'quantitative',
    headline: 'Centerline velocity RMSE 0.0617 against Ghia\'s tables on a 65² grid, with the primary vortex within 1e-4 of Ghia\'s position.',
    shortfall: 'The 65² run reports its error without a pass or fail bound, and its grid has a quarter of the cells of Ghia\'s own 129² grid.',
    problem: 'Lid-driven square cavity at Re = 1000: a box of fluid with three still walls and a lid sliding at U = 1.',
    reference:
      'Ghia, U., Ghia, K. N., Shin, C. T. (1982). High-Re solutions for incompressible flow using the Navier–Stokes equations and a multigrid method. J. Comput. Phys. 48, 387–411.',
    measured: [
      { quantity: 'Centerline RMSE vs Ghia (65², t_end = 100)', computed: '0.0617', expected: '0', delta: '—' },
      { quantity: 'Primary vortex position', computed: '(0.5312, 0.5625)', expected: '(0.5313, 0.5625)', delta: 'on Ghia\'s node' },
      { quantity: 'Corner eddies resolved', computed: 'both', expected: 'both', delta: 'bottom-left, bottom-right' },
      { quantity: 'Grid-trend gate (17² → 33²)', computed: '0.2369 → 0.1309', expected: 'decreasing', delta: 'gates 0.32 / 0.20' },
    ],
    command: 'cargo run --release -p deep_causality_cfd --example dec_lid_cavity_re1000_verification',
    caveat:
      'The default 65² run reports the centerline error and the vortex positions and has no gate. Its grid has a quarter of the cells of Ghia\'s own 129², and t_end = 100 rather than the ≥ 150 a settled reporting run would use. The bottom-left eddy centre sits one cell off in x (0.0781 against 0.0859). The gated mode is a separate, coarser sweep invoked as `… dec_lid_cavity_re1000_verification trend`: it runs 17² and 33² grids to t_end = 60 and checks that the error falls under refinement. Its bounds are pinned from this harness\'s own measurements, so a pass shows non-regression only, and the monthly CI run uses the default mode, so `trend` runs by hand.',
    hasArtifact: true,
  },
  {
    id: 'dec-cylinder',
    target: 'dec_cylinder_verification',
    family: 'DEC',
    status: 'quantitative',
    headline: 'Shedding frequency, as a Strouhal number, 0.1714 against 0.164–0.165 (+4.3%), and mean drag coefficient 1.246 against 1.32–1.36 (−6%).',
    shortfall: 'Acceptable but not reference quality at 16 cells per diameter, and the committed capture predates the harness move and was not regenerated.',
    problem: 'Flow past an isolated circular cylinder, 2-D and laminar, Re_D = 100, at 16 cells per diameter (D). The wall condition is applied at the true cylinder surface inside the grid cells it cuts (aperture-resolved cut cells).',
    reference:
      'Williamson (1996); Dröge & Verstappen (2005); Lehmkuhl, Rodríguez, Borrell & Oliva (2013). Window compiled in arXiv:2303.09262.',
    measured: [
      { quantity: 'Strouhal St', computed: '0.1714', expected: '0.164–0.165', delta: '+4.3% on the band' },
      { quantity: 'Mean drag C_d', computed: '1.246', expected: '1.32–1.36', delta: '−6%' },
      { quantity: 'C_d split (pressure + friction)', computed: '1.078 + 0.167', expected: 'friction ≈ 25%', delta: 'friction 13%' },
      { quantity: 'Lift C_l, drag swing', computed: '0.010, [1.238, 1.254]', expected: 'sustained limit cycle', delta: 'amplitude ≈ 0.41' },
    ],
    command:
      'CELLS_PER_D=16 LX_D=16 LY_D=16 STEPS=4000 CFL=0.4 CG_TOL=1e-6 cargo run --release -p deep_causality_cfd --example dec_cylinder_verification',
    caveat:
      'The committed capture predates the harness moving into deep_causality_cfd. It was taken under the former crate and example names at an earlier revision and has not been regenerated, so it is not the output of the code in the tree today. The figures in this record come from that 16 cells/D output, re100_16_resolved.txt. The default run (8 cells/D, 1500 steps) has its own committed baseline.txt, with St 0.1710 and C_d 1.345. All four pass or fail bounds in the harness are tripwires pinned from its own output, and the published values print beside each measurement. Acceptable but not reference quality at this grid. The integrated drag is close for the wrong reason: the pressure/friction split is off, with friction at 13% against the ~25% reference, so at 8 cells/D the total lands inside the reference band by cancellation. Most of the +4.3% Strouhal excess is LY_D = 16 blockage (the cross-stream domain height, ≈6.25%), leaving ~1–2% method error. A defensible accuracy claim needs a grid-convergence study (16→24→32/D, Richardson-extrapolated) plus C_L,rms, θ_sep and C_pb. Its staircase companion at the same 16 cells/D (a stair-stepped wall in place of cut cells) does not shed at all: the wake decays to a steady residual. That run\'s printed St 0.2444 is therefore the crossing detector firing on 7th-decimal noise, and its C_d 1.356 is a steady-flow value rather than a cycle mean. The aperture-resolved cut cells are what produce a sustained street here.',
    hasArtifact: true,
  },
  {
    id: 'dec-graded-mms',
    target: 'dec_graded_mms_verification',
    family: 'DEC',
    status: 'invariant',
    headline: 'Measured convergence order 1.98–2.01 against the design order 2.00, with divergence-freeness exact by construction.',
    shortfall: 'At strong grading the order on the coarser mesh pairs dips to about 1.7 and recovers to 2.0 as the mesh refines.',
    problem: 'Method of manufactured solutions (pick an exact field and test that the solver\'s operators reproduce it) on a torus with graded cell sizes, refined from 8² to 64², at grading amplitudes 0.0–0.3.',
    reference: 'Design order of accuracy 2.00. Discrete exterior calculus (DEC): Hirani (2003); Desbrun, Hirani, Leok & Marsden (2005).',
    measured: [
      { quantity: 'Convective order (finest pair)', computed: '1.98–1.99', expected: '2.00', delta: '≤ 0.02' },
      { quantity: 'Viscous order (finest pair)', computed: '2.00–2.01', expected: '2.00', delta: '≤ 0.01' },
      { quantity: 'Max error at 64² (convective)', computed: '5.13e-3 … 7.69e-3', expected: '—', delta: 'by grading' },
      { quantity: 'Divergence-freeness', computed: 'exact', expected: 'exact', delta: 'combinatorial' },
    ],
    command: 'cargo run --release -p deep_causality_cfd --example dec_graded_mms_verification',
    caveat:
      'At strong grading the order on the coarser mesh pairs dips to ~1.7 and recovers to ~2.0 as the mesh refines. The discrete fields must hold edge integrals, not point values: point values make the convective order look like it collapses on graded meshes.',
    hasArtifact: true,
  },
  {
    id: 'mms-tgv',
    target: 'mms_taylor_green_verification',
    family: 'Analytic',
    status: 'invariant',
    headline: 'The solver\'s right-hand-side kernel reproduces the exact Taylor–Green rate of change to 1.11e-16, at machine round-off.',
    shortfall: 'It checks one kernel, not a full solver run, and past a few thousand steps the two low-precision columns drift upward.',
    problem: 'Taylor–Green vortex (an exact decaying flow) through the incompressible Navier–Stokes right-hand-side kernel, with exact automatic-differentiation derivatives, Rk4 time stepping and 200 steps.',
    reference:
      'Taylor & Green (1937), Proc. R. Soc. Lond. A 158, 499–521. MMS methodology: Roache (2002); Salari & Knupp (2000).',
    measured: [
      { quantity: 'RHS kernel vs exact, max abs error', computed: '1.11e-16', expected: '0 (analytic)', delta: '≈ machine ε' },
      { quantity: 'Rk4 amplitude a(t) at t = 1', computed: '0.90483742', expected: '0.90483742', delta: '6.66e-16' },
      { quantity: 'Precision ladder (f32 / f64 / Float106)', computed: '3e-8 / 1e-16 / 8e-33', expected: '0', delta: 'by type' },
    ],
    command: 'cargo run --release -p deep_causality_cfd --example mms_taylor_green_verification',
    caveat:
      'Off-grid step counts introduce a phantom floor: dt = 0.005 is not a binary fraction, so steps·dt misses 1 by ~2e-17, which becomes a fixed ~1.9e-18 amplitude error. The check removes it by evaluating the reference at t_final = dt·steps. Past a few thousand steps the two low-precision columns drift upward.',
    hasArtifact: true,
  },
  {
    id: 'qtt-tgv',
    target: 'qtt_taylor_green_verification',
    family: 'QTT',
    status: 'quantitative',
    headline: 'Error falls at order 2.18 against a design order of 2.00 as the grid refines, reaching 5.316e-5 at 32² with the field 32× compressed.',
    shortfall: 'Smooth, single-mode, low-Reynolds flow only: immersed bodies and turbulence are not tested.',
    problem: 'A 2-D Taylor–Green vortex on a periodic box, evolved entirely in compressed form as a tensor train (a chain of small matrices in place of the full grid of values), on grids refined from 8² to 32².',
    reference: 'Taylor & Green (1937). Method: Peddinti et al. (2024), Commun. Phys. 7, 135; Gourianov et al. (2022), Nat. Comput. Sci. 2, 30–37.',
    measured: [
      { quantity: 'Observed order', computed: '2.18', expected: '2.00', delta: '+9%' },
      { quantity: 'Max error at 32²', computed: '5.316e-5', expected: '0 (analytic)', delta: 'bound 2.0e-4' },
      { quantity: 'Convection u·∇u vs closed form', computed: '3.207e-3', expected: '0', delta: '0.6% of 0.5 signal' },
      { quantity: 'Compression at 32²', computed: 'bond 32 vs 1024 dense', expected: '—', delta: '32×' },
    ],
    command: 'cargo run --release -p deep_causality_cfd --example qtt_taylor_green_verification',
    caveat:
      'Periodic, smooth, low-Reynolds and single-mode. It does not test immersed-body boundary conditions, turbulent rank growth, or a multi-mode cascade. The convection gate exists because single-mode Taylor–Green\'s convective term is a pure gradient the projection removes; a solver with a broken or zero u·∇u would still pass the error gate. The grid ladder stops at 32²: beyond it the explicit-Euler time error, of opposite sign, cancels the spatial error near N = 64–128 and the measured order stops meaning anything.',
    hasArtifact: true,
  },
  {
    id: 'dec-tgv-re1600',
    target: 'dec_taylor_green_re1600_verification',
    family: 'DEC',
    status: 'invariant',
    headline: 'Kinetic energy decays monotonically (E/E0 = 0.8929), and the dissipation peak reads 80% below the DNS value because 16³ is under-resolved.',
    shortfall: 'Only energy monotonicity is gated, and 16³ is far too coarse for the dissipation peak.',
    problem: 'A 3-D Taylor–Green vortex at Re = 1600, on the default 16³ grid marched to t* = 10, compared with published direct numerical simulation (DNS) data.',
    reference:
      'van Rees, Leonard, Pullin & Koumoutsakos (2011); Brachet et al. (1983); 1st Int. Workshop on High-Order CFD Methods (2012), case C3.5.',
    measured: [
      { quantity: 'Energy ratio E*/E0', computed: '0.8929', expected: 'monotone decay', delta: 'gate PASS' },
      { quantity: 'Peak dissipation (16³)', computed: '0.002468', expected: '≈0.0124 (DNS)', delta: '−80%' },
    ],
    command: 'cargo run --release -p deep_causality_cfd --example dec_taylor_green_re1600_verification',
    caveat:
      'Only the energy-monotonicity invariant is gated; the DNS comparison is informational. 16³ is grossly under-resolved and cannot represent the small-scale dissipation peak, hence the −80%. Strictly, the curve never peaks at this grid. The reported maximum falls at the final sample, t* = 10.05, so it is the end of a rising tail rather than a resolved peak. The DNS peak sits near t* ≈ 9. Reporting resolutions of 64³–128³ close this gap. Do not read the −80% as a solver error.',
    hasArtifact: true,
  },
  {
    id: 'qtt-cylinder',
    target: 'qtt_cylinder_verification',
    family: 'QTT',
    status: 'invariant',
    headline: 'The drag changes by 1.89e-11 between tensor-train bond 16 and 24, but the two checks that test whether it is a physical drag fail.',
    shortfall: 'The committed run fails two of its five gates, it ran at 32² where the harness now runs 256² by hand, and the absolute drag is not an isolated-cylinder value.',
    problem: 'Cylinder in a periodic free stream at 32². Brinkman volume penalization (a resistance term that brings the flow to rest inside the body) holds the body, and the drag is computed as a tensor-train contraction.',
    reference:
      'Angot, Bruneau & Fabrie (1999), Numer. Math. 81, 497–520. Cross-reference: the DEC cylinder target at C_d ≈ 1.345.',
    measured: [
      { quantity: 'C_d convergence |ΔC_d|, bond 16 → 24', computed: '1.89e-11', expected: '0 (converged)', delta: 'gate: relative ≤ 0.10' },
      { quantity: 'Interior max |u| (no-slip)', computed: '4.22e-2', expected: '0', delta: '4% of free stream' },
      { quantity: 'Divergence at bond 24', computed: '5.47e-14', expected: '0', delta: '≈ machine ε' },
      { quantity: 'Absolute C_d', computed: '23.7577', expected: 'not the isolated value', delta: 'see caveat' },
    ],
    command: 'cargo run --release -p deep_causality_cfd --example qtt_cylinder_verification',
    caveat:
      'The committed run is the last completed one, and it fails. Two of its five gates, the Brinkman-parameter (η) ladder and the mask-smoothing ladder, report NOT CONVERGING, so the program exits 1. The drag rises, peaks and falls as η shrinks (C_d 17.3875, 24.0175, 26.2464, 23.7577, 21.4031), and it moves 6.1× as the mask width varies. The bond gate passes, but it shows only that the compression has converged, not that the compressed quantity is a drag. Only the two ladders constrain the drag value. The figures on this page come from that 32² capture. The harness now runs at 256² (L = 8) with η chosen from a wall-error target, which takes about 4–9 hours, runs by hand and not in CI, and has its acceptance run pending. Whether these readings carry over to 256² is untested. The bond gate now bounds the relative change at 1.0e-6, not the ≤ 0.10 in the table. The absolute C_d ≈ 23.8 is NOT an isolated-cylinder drag coefficient: roughly 30% blockage, a penalization-integral force over a smoothed 2-cell skirt, and a fixed-horizon read rather than a steady state. The case also runs at Re_D = 37.7, not the Re = 100 of the DEC cross-reference. Reproducing an isolated C_d needs an inflow/outflow domain, out of scope for the periodic QTT solver.',
    hasArtifact: true,
  },
  {
    id: 'dec-cylinder-wake',
    target: 'dec_cylinder_wake_verification',
    family: 'DEC',
    status: 'invariant',
    headline: 'Velocity divergence stays at 3.334e-15, and the event log holds all 80 entries expected for 40 sensor dropouts.',
    shortfall: 'An internal-consistency check with no quantitative reference, in a confined channel.',
    problem: 'Cylinder in a confined channel, periodic in x. A noisy sensor stream sets the speed of the top wall and drops out every 50 steps, and the run logs each dropout.',
    reference: 'None quantitative; an internal-consistency exercise.',
    measured: [
      { quantity: 'Max divergence residual', computed: '3.334e-15', expected: '0', delta: 'tol 1e-6' },
      { quantity: 'EffectLog entries under dropout', computed: '80', expected: '80 (2 × 40)', delta: 'exact' },
    ],
    command: 'cargo run --release -p deep_causality_cfd --example dec_cylinder_wake_verification',
    caveat:
      'The DEC solver has no inflow/outflow surface here; the sensor drives a prescribed moving wall in a confined periodic-x channel. At 25% blockage the run reports no clear shedding in the developed signal, so the printed Strouhal is a qualitative check for that confined case, never gated, and no isolated-cylinder Reynolds ladder is claimed. The committed baseline.txt holds the 200-row probe stream, the summary lines and both gate results; the divergence figure is the maximum over its residual column.',
    hasArtifact: true,
  },
  {
    id: 'qtt-blunt-2d',
    target: 'qtt_blunt_body_2d',
    family: 'QTT',
    status: 'structural',
    headline: 'The tensor-train bond stays at 3 → 5 on a body-fitted grid and grows 16 → 61 when the shock is captured on a Cartesian grid; the check covers rank, not accuracy.',
    shortfall: 'Checks how compressed the field stays, not how accurate the physics is.',
    problem: 'A bow shock in front of a blunt body at constant standoff radius, stored two ways: on a body-fitted polar grid and on a plain Cartesian grid that captures the shock. Grids run from 2⁵ to 2⁷ cells per side. The bond is the size of the small matrices in the tensor train, so a small bond means the field compresses well.',
    reference: 'Structural claim only: fitted bond bounded, capture growing. No published value.',
    measured: [
      { quantity: 'Fitted bond χ', computed: '3 → 5', expected: 'bounded, ≤ 12', delta: 'gate BB-A' },
      { quantity: 'Cartesian capture bond χ', computed: '16 → 61', expected: '≥ 2× fitted', delta: 'gate BB-B' },
    ],
    command: 'cargo run --release -p deep_causality_cfd --example qtt_blunt_body_2d',
    caveat:
      'This gates rank, not physical accuracy; the quantitative accuracy gate for the compressible solver is qtt_sod, against the exact Riemann solution. The marched peak bond is reported and explicitly not asserted: a plain flux-through-front marcher injects angular structure and grows the bond to 64 over 6 steps even in the fitted coordinate. Bounding that is the open remainder (design item D9, studied in qtt_repin_marcher).',
    hasArtifact: true,
  },
  {
    id: 'qtt-reentry-3d',
    target: 'qtt_reentry_3d',
    family: 'QTT',
    status: 'structural',
    headline: 'The body-fitted forebody bond stays at 2 → 4 while Cartesian sampling grows 10 → 59; the check covers rank, not accuracy.',
    shortfall: 'Forebody only, and it checks compression, not accuracy: the wake is out of scope.',
    problem: 'The plasma sheath over a 3-D reentry forebody, stored on a body-fitted spherical grid and on a Cartesian grid, at 2³ to 2⁵ cells per side.',
    reference: 'Structural: the qtt_rank_3d study bound, not a paper.',
    measured: [
      { quantity: 'Fitted forebody bond χ', computed: '2 → 4', expected: 'bounded, ≤ 8', delta: 'gate RE-A' },
      { quantity: 'Cartesian bond χ', computed: '10 → 59', expected: '≥ 2× fitted', delta: 'gate RE-B' },
      { quantity: 'Wake bond', computed: '41', expected: 'out of scope', delta: 'reported only' },
    ],
    command: 'cargo run --release -p deep_causality_cfd --example qtt_reentry_3d',
    caveat:
      'Forebody only, and structural: it bounds rank, not physical accuracy. The wake is explicitly out of scope: a separated unsteady wake needs turbulence and is a multi-feature structure no single fitted coordinate aligns; its bond is reported, never gated. The dynamic marched forebody rank is likewise reported, not gated: there is no 3-D body-fit metric yet, so the marcher runs Cartesian and grows the bond to 16 over 6 steps. A 3-D body-fit metric plus re-pinning (moving the fitted coordinate with the live shock) is the open remainder.',
    hasArtifact: true,
  },
  {
    id: 'qtt-park2t',
    target: 'qtt_park2t_blackout',
    family: 'QTT',
    status: 'structural',
    headline: 'Peak electron density comes out three decades (a factor of 1,000) above the flight anchor, and qtt_ramc_stagline supersedes it.',
    shortfall: 'Superseded, and its six gates test the coupling machinery, not agreement with flight data.',
    problem: 'The first-version (Tier A) blackout closure on an incompressible flow: recovery temperature, then ionization, then electron density.',
    reference: 'Cross-references only: RAM-C II, Park two-temperature tables, the Saha limit, the duration of Apollo blackouts.',
    measured: [
      { quantity: 'Peak electron density n_e', computed: '1.000e22 m⁻³', expected: '~1e19 m⁻³ (RAM-C II)', delta: '+3 decades' },
      { quantity: 'Six LER acceptance gates', computed: 'all PASS', expected: 'closure behaviour', delta: 'not accuracy' },
    ],
    command: 'cargo run --release -p deep_causality_cfd --example qtt_park2t_blackout',
    caveat:
      'Superseded. The Tier-A closure over-predicts by three decades: it rides an incompressible rollout with a recovery-temperature reconstruction rather than a true post-shock thermodynamic path, and Saha equilibrium at the frozen Rankine–Hugoniot temperature drives full ionization (α = 1). Its six gates test the coupling machinery, and none compares a marched quantity against flight data. No absolute coupled-CFD match is claimed. Retired by the Tier-B compressible marcher; use qtt_ramc_stagline instead.',
    hasArtifact: true,
  },
];

export const statusLabel: Record<ValidationStatus, string> = {
  quantitative: 'Quantitative',
  anchored: 'Flight-anchored',
  invariant: 'Invariant',
  structural: 'Structural',
};

export const statusClass: Record<ValidationStatus, string> = {
  quantitative: 'status-validated',
  anchored: 'status-measured',
  invariant: 'status-measured',
  structural: 'status-partial',
};
