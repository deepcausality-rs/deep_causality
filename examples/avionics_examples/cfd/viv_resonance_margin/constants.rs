/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Every tuned constant of the vortex-shedding resonance-margin study, with its justification.
//! The wake case reuses the validated isolated-cylinder configuration from
//! `deep_causality_cfd/verification/dec_cylinder_verification` (inflow, outflow, far-field slip
//! walls, immersed cut-cell disk with the aperture-resolved no-slip), trimmed to a sweep-affordable
//! grid; the trimmed grid's Strouhal band is re-pinned from measurement below.

// ── The structure and the air (the physical dimensionalization)

/// Diameter of the circular member under assessment, m: a 2 mm guy wire on an instrumentation
/// mast. Chosen so the swept airspeeds land the Reynolds number inside the solver's validated
/// laminar-wake band (Re 100 to 200); a larger member at flight speeds would be far outside it.
pub const DIAMETER_M: f64 = 2.0e-3;
/// Kinematic viscosity of air, m^2/s, at roughly 15 C and sea level (ICAO standard atmosphere).
pub const NU_AIR_M2_S: f64 = 1.5e-5;
/// Stated structural natural frequency of the member, Hz. A chosen demonstration value for a
/// mast-like structure (a tensioned 2 mm wire's first mode sits in this range), not a measured
/// one; a real assessment would take it from a ground vibration test or a modal analysis.
pub const F_STRUCT_HZ: f64 = 150.0;

// ── The wake case (nondimensional: D = 1, U = 1, so nu = 1/Re and f_shed = St * V / D)

/// Grid resolution, cells across one diameter. The verification ladder validates the
/// aperture-resolved no-slip's shedding at 16 cells/D (St 0.171 at Re 100); 16/D costs about
/// 0.65 s/step here, which puts a swept run far past the minutes budget. 8/D sheds with this
/// configuration (measured below) and holds the sweep near two minutes of wall-clock.
pub const CELLS_PER_D: usize = 8;
/// Streamwise domain extent, diameters. Shorter than the verification harness's 12 to 16 D:
/// the probe needs only the near wake, and the outflow zone passes the street cleanly.
pub const LX_D: f64 = 9.0;
/// Cross-stream domain extent, diameters. 6 D means 16.7 percent blockage, which confines the
/// wake and raises the Strouhal number above the unconfined 0.164; the acceptance band below is
/// re-pinned from measurement on exactly this domain.
pub const LY_D: f64 = 6.0;
/// Cylinder center, streamwise, diameters from the inlet (the verification harness's quarter-span
/// placement, scaled to this domain).
pub const CENTER_X_D: f64 = 2.5;
/// Transverse offset of the cylinder center off the domain midline, diameters. The DSL's uniform
/// seed carries no perturbation, and a perfectly symmetric cut pattern sheds only from round-off,
/// which develops too slowly for a swept run. A third-of-a-cell offset makes the cut pattern
/// itself asymmetric and tips the wake off the symmetric branch, replacing the verification
/// harness's explicit transverse seed blob.
pub const CENTER_OFFSET_D: f64 = 0.04;
/// Cut-cell volume-fraction merge floor (sliver-cell stabilization), the verification value.
pub const MERGE_FLOOR: f64 = 0.25;
/// Advective CFL number, dt = CFL * h / U. The flow accelerates to about 1.9 U around the
/// cylinder, so the advective limit binds near 0.45 (measured in the verification README);
/// 0.4 is the validated setting.
pub const CFL: f64 = 0.4;
/// March steps. At dt = CFL/CELLS_PER_D = 0.05 this is t = 110: the street is developed by
/// about t = 55 from the geometric asymmetry (measured), and the second-half tail the Strouhal
/// is read from then carries about nine shedding cycles.
pub const STEPS: usize = 2200;
/// Projection CG tolerance. The verification README measures machine-epsilon tolerance as the
/// dominant cost on cut-cell systems; 1e-6 keeps the divergence residual far below the physics
/// while cutting iterations. Warm start is on, as in the verification harness.
pub const CG_TOL: f64 = 1e-6;

// ── Gates

/// Acceptance band for the extracted Strouhal number over the swept Re 100 to 160 range.
/// Measured on this grid: St 0.1818 to 0.1909 across the four scheduled airspeeds, sitting on
/// the unconfined laminar reference (Williamson: St rises from about 0.164 at Re 100 to about
/// 0.185 at Re 160), with the coarse grid and the domain blockage pushing the low-Re end
/// slightly high. The band brackets the measured values and the reference with room for
/// platform round-off, and still catches a dead wake (St = 0) or a numerically broken one.
pub const ST_BAND: (f64, f64) = (0.16, 0.21);
/// Minimum acceptable resonance margin, min over rows of |f_struct - f_shed| / f_struct.
/// 0.15 is a demonstration placard line: the swept schedule's worst row must keep the shedding
/// frequency at least 15 percent away from the stated structural mode.
pub const MARGIN_MIN: f64 = 0.15;
