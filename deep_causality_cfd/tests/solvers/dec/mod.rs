/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#[cfg(test)]
pub mod boundary_construction_tests;
#[cfg(test)]
pub mod boundary_zone_tests;
#[cfg(test)]
pub mod cavity_tests;
#[cfg(test)]
pub mod cut_cell_wiring_tests;
#[cfg(test)]
pub mod dec_cross_validation_tests;
#[cfg(test)]
pub mod dec_ns_rate_tests;
#[cfg(test)]
pub mod dec_ns_solver;
#[cfg(test)]
pub mod diagnostics_tests;
#[cfg(test)]
pub mod energy_budget_tests;
#[cfg(test)]
pub mod inflow_outflow_tests;
#[cfg(test)]
pub mod inviscid_invariants_tests;
#[cfg(test)]
pub mod poiseuille_tests;
#[cfg(test)]
pub mod scalar_transport_tests;
#[cfg(test)]
pub mod shear_layer_tests;
#[cfg(test)]
pub mod slip_wall_tests;
pub mod spectral_diffusion_tests;
#[cfg(test)]
pub mod step_output_tests;
mod surface_force_tests;
#[cfg(test)]
pub mod taylor_green_2d_tests;
#[cfg(test)]
pub mod taylor_green_3d_tests;
#[cfg(all(test, feature = "std"))]
pub mod uncertain_boundary_source_tests;
#[cfg(all(test, feature = "std"))]
pub mod uncertain_inflow_tests;
#[cfg(test)]
pub mod wrappers_tests;
