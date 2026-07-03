/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The nozzle study's constants: the duct geometry, the reservoir state, the numerical
//! configuration, and the gate bands. Every band is measured or derived, and says so.

/// Inlet area of the converging-diverging profile, m². A 2:1:2 contraction-expansion is the
/// textbook demonstration nozzle: exit-to-throat ratio 2 puts the design exit Mach near 2.2
/// and gives every operating regime a comfortable back-pressure window.
pub const INLET_AREA_M2: f64 = 2.0;
/// Throat area, m² (the strict minimum; the sonic reference `A*` when choked).
pub const THROAT_AREA_M2: f64 = 1.0;
/// Exit area, m².
pub const EXIT_AREA_M2: f64 = 2.0;
/// Duct length, m. The throat sits at the midpoint of the parabolic profile.
pub const LENGTH_M: f64 = 1.0;

/// Reservoir stagnation pressure, Pa. A 3-bar demonstration reservoir; the map is reported as
/// `p_back / p0`, so the absolute level only sets dimensional outputs.
pub const P0_PA: f64 = 300_000.0;
/// Reservoir stagnation temperature, K.
pub const T0_K: f64 = 500.0;
/// Ratio of specific heats for air, perfect gas.
pub const GAMMA: f64 = 1.4;

/// Grid cells along the duct. The gate bands below were measured at this resolution; changing
/// it re-opens both bands.
pub const CELLS: usize = 128;
/// Step budget for the quasi-steady march (the verification cases converge far below this).
pub const MAX_STEPS: usize = 200_000;
/// Residual gate: maximum relative change of the conserved state per step at convergence.
pub const RESIDUAL_TOL: f64 = 1.0e-10;

/// Interior band for the shock-free Mach profile against the area-Mach relation, relative.
/// Measured by the duct-march verification at 128 cells (worst interior deviation, stations
/// within 0.1 of the throat and 0.05 of either end excluded): the first-order scheme lands
/// inside 5 percent there.
pub const AREA_MACH_BAND: f64 = 0.05;
/// Shock-position band in cell widths, against the closed-form position (isentropic to the
/// shock, Rankine-Hugoniot across it, subsonic recovery to the exit). Measured by the same
/// verification: the smeared first-order shock lands within a dozen cells.
pub const SHOCK_BAND_CELLS: f64 = 12.0;
/// How close to the throat the sonic crossing must sit on a choked row, in cell widths. The
/// parabolic throat is flat (A' = 0 at the throat), so the sonic point wanders a few cells at
/// first order; measured at 128 cells the crossing stays within this window.
pub const SONIC_AT_THROAT_BAND_CELLS: f64 = 8.0;

/// The sentinel written into the table's shock-position column for shock-free rows, meters.
/// The `Report` omits the series instead; the flat table needs a value, and negative one is
/// impossible as a position, documented in the README and the column unit.
pub const NO_SHOCK_SENTINEL_M: f64 = -1.0;
