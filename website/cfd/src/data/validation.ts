/**
 * Validation records — the adoption document's data.
 *
 * Every number here is copied from a committed artifact in
 * `deep_causality_cfd/verification/<target>/` — usually `baseline.txt`, but for
 * `dec_cylinder_verification` the recorded run output `re100_16_resolved.txt` —
 * or that target's README. Nothing is rounded for presentation and nothing is
 * estimated.
 *
 * All figures measured at `f64` on an Apple M3 Max, release build.
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
    problem: 'Sod shock tube, γ = 1.4, marched to t = 0.2 on 512 cells.',
    reference: 'Exact Riemann solution (canonical star pressure p* = 0.3031).',
    measured: [
      { quantity: 'Density, L1 over |x| ≤ 0.5', computed: '0.0175', expected: '0 (exact)', delta: 'tol 0.03' },
      { quantity: 'Velocity, L1 over |x| ≤ 0.5', computed: '0.0274', expected: '0 (exact)', delta: 'tol 0.03' },
      { quantity: 'Pressure, L1 over |x| ≤ 0.5', computed: '0.0151', expected: '0 (exact)', delta: 'tol 0.03' },
      { quantity: 'Star pressure p*', computed: '0.3031', expected: '0.3031', delta: 'exact' },
    ],
    command: 'cargo run --release -p deep_causality_cfd --example qtt_sod',
    caveat:
      'First-order Rusanov smears the contact, so the bound is on mean accuracy, not on peak resolution. The nonlinear flux and EOS are evaluated pointwise (dequantize → compute → requantize); the rank-preserving TT-cross form is the large-L upgrade.',
    hasArtifact: true,
  },
  {
    id: 'qtt-ramc',
    target: 'qtt_ramc_stagline',
    family: 'QTT',
    status: 'anchored',
    problem: 'RAM-C II reentry stagnation streamline at ~71 km, M = 25, fitted shock interface with exact Rankine–Hugoniot jump.',
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
      'The uncalibrated finite-rate network lands within +0.35 decade of the flight anchor, inside the ±0.70 chemistry-spread band. The closed-form Park-2T controller lands 1.27 decades below the anchor after the N₂–N₂ reduced-mass correction (μ = 14.007); its former near-anchor landing was an artifact of an invalid μ = 7.0 (the N–N atomic pair, which has no vibrational mode), and the offset is reported rather than re-admitted. Still a two-temperature Saha surrogate; the T_e = T_ve lumping is worth roughly 2×, and the landing is sensitive to the Millikan–White τ_vt model within the documented 2–5× chemistry-model spread. γ = 1.1 is an effective-γ closure, not perfect gas.',
    hasArtifact: true,
  },
  {
    id: 'dec-cavity',
    target: 'dec_lid_cavity_re1000_verification',
    family: 'DEC',
    status: 'quantitative',
    problem: 'Lid-driven square cavity at Re = 1000, three no-slip walls, lid at U = 1.',
    reference:
      'Ghia, U., Ghia, K. N., Shin, C. T. (1982). High-Re solutions for incompressible flow using the Navier–Stokes equations and a multigrid method. J. Comput. Phys. 48, 387–411.',
    measured: [
      { quantity: 'Centerline RMSE vs Ghia (33², t = 40)', computed: '0.137', expected: '0', delta: '—' },
      { quantity: 'Primary vortex position', computed: '(0.563, 0.594)', expected: '(0.531, 0.563)', delta: '≈ 6% of span' },
      { quantity: 'Corner eddies resolved', computed: 'both', expected: 'both', delta: 'at 33²/t=40' },
      { quantity: 'Grid-trend gate (17² → 33²)', computed: '0.252 → 0.133', expected: 'decreasing', delta: 'gates 0.32 / 0.20' },
    ],
    command: 'cargo run --release -p deep_causality_cfd --example dec_lid_cavity_re1000_verification trend',
    caveat:
      'The 6%-of-span vortex offset is at a coarse 33² grid. Reporting resolution is 129² with t_end ≥ 150, Ghia\'s own grid, which takes hours. The committed baseline.txt for this target is a partial run log and records no RMSE; the figures above come from the target README.',
    hasArtifact: false,
  },
  {
    id: 'dec-cylinder',
    target: 'dec_cylinder_verification',
    family: 'DEC',
    status: 'quantitative',
    problem: 'Isolated circular cylinder, 2-D laminar, Re_D = 100, aperture-resolved cut cells at 16 cells/D.',
    reference:
      'Williamson (1996); Dröge & Verstappen (2005); Lehmkuhl, Rodríguez, Borrell & Oliva (2013). Window compiled in arXiv:2303.09262.',
    measured: [
      { quantity: 'Strouhal St', computed: '0.1714', expected: '0.164–0.165', delta: '+4.3%' },
      { quantity: 'Mean drag C_d', computed: '1.246', expected: '1.32–1.36', delta: '−6%' },
      { quantity: 'C_d split (pressure + friction)', computed: '1.078 + 0.167', expected: 'friction ≈ 25%', delta: 'friction 13%' },
      { quantity: 'Lift C_l, drag swing', computed: '0.010, [1.238, 1.254]', expected: 'sustained limit cycle', delta: 'amplitude ≈ 0.41' },
    ],
    command:
      'CELLS_PER_D=16 LX_D=16 LY_D=16 STEPS=4000 CFL=0.4 CG_TOL=1e-6 cargo run --release -p deep_causality_cfd --example dec_cylinder_verification',
    caveat:
      'Acceptable but not DNS-grade at this grid. The integrated drag is close for the wrong reason: the pressure/friction split is off, with friction at 13% against the ~25% reference. Most of the +4.3% Strouhal excess is LY_D = 16 blockage (≈6.25%), leaving ~1–2% method error. A defensible accuracy claim needs a grid-convergence study (16→24→32/D, Richardson-extrapolated) plus C_L,rms, θ_sep and C_pb. This target has no baseline.txt; the figures are the committed run output re100_16_resolved.txt. Its staircase companion at the same 16 cells/D does not shed at all: the wake decays to a steady residual. That run\'s printed St 0.2444 is therefore the crossing detector firing on 7th-decimal noise, and its C_d 1.356 is a steady-flow value rather than a cycle mean. The aperture-resolved cut cells are what produce a sustained street here.',
    hasArtifact: true,
  },
  {
    id: 'dec-graded-mms',
    target: 'dec_graded_mms_verification',
    family: 'DEC',
    status: 'invariant',
    problem: 'Method of manufactured solutions on a graded torus, 8²→64², grading amplitudes 0.0–0.3.',
    reference: 'Observed order of accuracy = 2.00. DEC: Hirani (2003); Desbrun, Hirani, Leok & Marsden (2005).',
    measured: [
      { quantity: 'Convective order (finest pair)', computed: '1.98–1.99', expected: '2.00', delta: '≤ 0.02' },
      { quantity: 'Viscous order (finest pair)', computed: '2.00–2.01', expected: '2.00', delta: '≤ 0.01' },
      { quantity: 'Max error at 64² (convective)', computed: '5.13e-3 … 7.69e-3', expected: '—', delta: 'by grading' },
      { quantity: 'Divergence-freeness', computed: 'exact', expected: 'exact', delta: 'combinatorial' },
    ],
    command: 'cargo run --release -p deep_causality_cfd --example dec_graded_mms_verification',
    caveat:
      'At strong grading the coarse-pair order dips to ~1.7 and recovers to ~2.0 as the mesh refines. An earlier revision of this study mis-measured a convective order collapse; the cause was a measurement bug: pointwise 1-form values instead of edge integrals. The cochain convention is load-bearing.',
    hasArtifact: true,
  },
  {
    id: 'mms-tgv',
    target: 'mms_taylor_green_verification',
    family: 'Analytic',
    status: 'invariant',
    problem: 'Taylor–Green vortex through the incompressible NS RHS kernel with exact autodiff derivatives, Rk4, 200 steps.',
    reference:
      'Taylor & Green (1937), Proc. R. Soc. Lond. A 158, 499–521. MMS methodology: Roache (2002); Salari & Knupp (2000).',
    measured: [
      { quantity: 'RHS kernel vs exact, max abs error', computed: '1.11e-16', expected: '0 (analytic)', delta: '≈ machine ε' },
      { quantity: 'Rk4 amplitude a(t) at t = 1', computed: '0.90483742', expected: '0.90483742', delta: '6.66e-16' },
      { quantity: 'Precision ladder (f32 / f64 / Float106)', computed: '3e-8 / 1e-16 / 8e-33', expected: '0', delta: 'by type' },
    ],
    command: 'cargo run --release -p deep_causality_cfd --example mms_taylor_green_verification',
    caveat:
      'Off-grid step counts introduce a phantom floor: dt = 0.005 is not a binary fraction, so steps·dt misses 1 by ~2e-17, which becomes a fixed ~1.9e-18 amplitude error. Guarded by evaluating the reference at t_final = dt·steps. Past a few thousand steps the two low-precision columns drift upward.',
    hasArtifact: true,
  },
  {
    id: 'qtt-tgv',
    target: 'qtt_taylor_green_verification',
    family: 'QTT',
    status: 'quantitative',
    problem: '2-D Taylor–Green vortex on a periodic box, evolved entirely as a tensor train, refinement ladder 8²→32².',
    reference: 'Taylor & Green (1937). Method: Peddinti et al. (2024), Commun. Phys. 7, 135; Gourianov et al. (2022), Nat. Comput. Sci. 2, 30–37.',
    measured: [
      { quantity: 'Observed order', computed: '2.18', expected: '2.00', delta: '+9%' },
      { quantity: 'Max error at 32²', computed: '5.316e-5', expected: '0 (analytic)', delta: 'bound 2.0e-4' },
      { quantity: 'Convection u·∇u vs closed form', computed: '3.207e-3', expected: '0', delta: '0.6% of 0.5 signal' },
      { quantity: 'Compression at 32²', computed: 'bond 32 vs 1024 dense', expected: '—', delta: '32×' },
    ],
    command: 'cargo run --release -p deep_causality_cfd --example qtt_taylor_green_verification',
    caveat:
      'Periodic, smooth, low-Reynolds and single-mode. It does not test immersed-body boundary conditions, turbulent rank growth, or multi-mode cascade. Gate 2 exists because single-mode Taylor–Green\'s convective term is a pure gradient the projection removes; a solver with a broken or zero u·∇u would still pass gate 1.',
    hasArtifact: true,
  },
  {
    id: 'dec-tgv-re1600',
    target: 'dec_taylor_green_re1600_verification',
    family: 'DEC',
    status: 'invariant',
    problem: '3-D Taylor–Green vortex at Re = 1600, default 16³ grid marched to t* = 10.',
    reference:
      'van Rees, Leonard, Pullin & Koumoutsakos (2011); Brachet et al. (1983); 1st Int. Workshop on High-Order CFD Methods (2012), case C3.5.',
    measured: [
      { quantity: 'Energy ratio E*/E0', computed: '0.8929', expected: 'monotone decay', delta: 'gate PASS' },
      { quantity: 'Peak dissipation (16³)', computed: '0.002468', expected: '≈0.0124 (DNS)', delta: '−80%' },
    ],
    command: 'cargo run --release -p deep_causality_cfd --example dec_taylor_green_re1600_verification',
    caveat:
      'Only the energy-monotonicity invariant is gated; the DNS comparison is informational. 16³ is grossly under-resolved and cannot represent the small-scale dissipation peak, hence the −80%. Strictly the curve never peaks at this grid: the reported maximum falls at the final sample, t* = 10.05, so it is a monotone-rising tail rather than a resolved peak, where the DNS peak sits near t* ≈ 9. Reporting resolutions of 64³–128³ close this gap. Do not read the −80% as a solver error.',
    hasArtifact: true,
  },
  {
    id: 'qtt-cylinder',
    target: 'qtt_cylinder_verification',
    family: 'QTT',
    status: 'invariant',
    problem: 'Cylinder in a periodic free-stream at 32², Brinkman volume penalization, drag as a tensor-train contraction.',
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
      'The absolute C_d ≈ 23.8 is NOT an isolated-cylinder drag coefficient: roughly 30% blockage, a penalization-integral force over a smoothed 2-cell skirt, and a fixed-horizon read rather than a steady state. The verification claim is the convergence trend plus no-slip and positivity, never the absolute number. Reproducing an isolated C_d needs an inflow/outflow domain, out of scope for the periodic QTT solver.',
    hasArtifact: true,
  },
  {
    id: 'dec-cylinder-wake',
    target: 'dec_cylinder_wake_verification',
    family: 'DEC',
    status: 'invariant',
    problem: 'Cylinder in a confined periodic-x channel driven by an uncertain sensor stream through the causal monad.',
    reference: 'None quantitative; an internal-consistency exercise.',
    measured: [
      { quantity: 'Max divergence residual', computed: '3.334e-15', expected: '0', delta: 'tol 1e-6' },
      { quantity: 'EffectLog entries under dropout', computed: '80', expected: '80 (2 × 40)', delta: 'exact' },
    ],
    command: 'cargo run --release -p deep_causality_cfd --example dec_cylinder_wake_verification',
    caveat:
      'The DEC solver has no inflow/outflow surface here; the sensor drives a prescribed moving wall in a confined periodic-x channel. At 25% blockage the run reports no clear shedding in the developed signal, so the printed Strouhal is a qualitative check for that confined case, never gated, and no isolated-cylinder Reynolds ladder is claimed. The committed baseline.txt is the 200-row probe stream; the divergence figure is the maximum over its residual column; the pass/fail summary lines are in cli_output.txt.',
    hasArtifact: true,
  },
  {
    id: 'qtt-blunt-2d',
    target: 'qtt_blunt_body_2d',
    family: 'QTT',
    status: 'structural',
    problem: 'Blunt-body bow shock at constant standoff radius, body-fitted polar fan versus Cartesian capture, ladder 2⁵–2⁷.',
    reference: 'Structural claim only: fitted bond bounded, capture growing. No published value.',
    measured: [
      { quantity: 'Fitted bond χ', computed: '3 → 5', expected: 'bounded, ≤ 12', delta: 'gate BB-A' },
      { quantity: 'Cartesian capture bond χ', computed: '16 → 61', expected: '≥ 2× fitted', delta: 'gate BB-B' },
    ],
    command: 'cargo run --release -p deep_causality_cfd --example qtt_blunt_body_2d',
    caveat:
      'This gates rank, not physical accuracy; the quantitative accuracy gate for the compressible solver is qtt_sod, against the exact Riemann solution. The marched peak bond is reported and explicitly not asserted: a plain flux-through-front marcher injects angular structure and grows the bond to 64 over 6 steps even in the fitted coordinate. Bounding that is design D9 and the qtt_repin_marcher study.',
    hasArtifact: true,
  },
  {
    id: 'qtt-reentry-3d',
    target: 'qtt_reentry_3d',
    family: 'QTT',
    status: 'structural',
    problem: '3-D reentry forebody sheath, body-fitted spherical versus Cartesian sampling, ladder 2³–2⁵.',
    reference: 'Structural: the qtt_rank_3d study bound, not a paper.',
    measured: [
      { quantity: 'Fitted forebody bond χ', computed: '2 → 4', expected: 'bounded, ≤ 8', delta: 'gate RE-A' },
      { quantity: 'Cartesian bond χ', computed: '10 → 59', expected: '≥ 2× fitted', delta: 'gate RE-B' },
      { quantity: 'Wake bond', computed: '41', expected: 'out of scope', delta: 'reported only' },
    ],
    command: 'cargo run --release -p deep_causality_cfd --example qtt_reentry_3d',
    caveat:
      'Forebody only, and structural: it bounds rank, not physical accuracy. The wake is explicitly out of scope: a separated unsteady wake needs turbulence and is a multi-feature structure no single fitted coordinate aligns; its bond is reported, never gated. The dynamic marched forebody rank is likewise reported, not gated: there is no 3-D body-fit metric yet, so the marcher runs Cartesian and grows the bond to 16 over 6 steps. A 3-D body-fit metric plus re-pinning is the open remainder.',
    hasArtifact: true,
  },
  {
    id: 'qtt-park2t',
    target: 'qtt_park2t_blackout',
    family: 'QTT',
    status: 'structural',
    problem: 'Tier-A blackout closure on an incompressible rollout: recovery temperature → ionization → electron density.',
    reference: 'Cross-references only: RAM-C II, Park two-temperature tables, the Saha limit, Apollo blackout dwell.',
    measured: [
      { quantity: 'Peak electron density n_e', computed: '1.000e22 m⁻³', expected: '~1e19 m⁻³ (RAM-C II)', delta: '+3 decades' },
      { quantity: 'Six LER acceptance gates', computed: 'all PASS', expected: 'closure behaviour', delta: 'not accuracy' },
    ],
    command: 'cargo run --release -p deep_causality_cfd --example qtt_park2t_blackout',
    caveat:
      'Superseded. The Tier-A closure over-predicts by three decades: it rides an incompressible rollout with a recovery-temperature reconstruction rather than a true post-shock thermodynamic path, and Saha equilibrium at the frozen RH temperature drives near-full ionization. No absolute coupled-CFD match is claimed. Retired by the Tier-B compressible marcher; use qtt_ramc_stagline instead.',
    hasArtifact: true,
  },
];

export const statusLabel: Record<ValidationStatus, string> = {
  quantitative: 'Quantitative',
  anchored: 'Flight-anchored',
  invariant: 'Invariant',
  structural: 'Structural (rank)',
};

export const statusClass: Record<ValidationStatus, string> = {
  quantitative: 'status-validated',
  anchored: 'status-measured',
  invariant: 'status-measured',
  structural: 'status-partial',
};
