/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// --- Domain quantities (moved from tests/kernels/<domain>/) ---
#[cfg(test)]
mod chronometric_quantities;
#[cfg(test)]
mod condensed_quantities;
#[cfg(test)]
mod dynamics_quantities;
#[cfg(test)]
mod em_quantities;
#[cfg(test)]
mod fluids_quantities;
#[cfg(test)]
mod hypersonic_quantities;
#[cfg(test)]
mod materials_quantities;
#[cfg(test)]
mod mhd_quantities;
#[cfg(test)]
mod nuclear_quantities;
#[cfg(test)]
mod photonics_quantities;
#[cfg(test)]
mod propulsion_quantities;
#[cfg(test)]
mod quantum_quantities;
#[cfg(test)]
mod relativity_quantities;
#[cfg(test)]
mod thermodynamics_quantities;

// --- Physical-unit types (moved from tests/units/) ---
#[cfg(test)]
mod body_force_one_form;
#[cfg(test)]
mod energy;
#[cfg(test)]
mod index_of_refraction;
#[cfg(test)]
mod pressure_zero_form;
#[cfg(test)]
mod ratio;
#[cfg(test)]
mod solenoidal_field;
#[cfg(test)]
mod temperature;
#[cfg(test)]
mod time;
#[cfg(test)]
mod velocity_one_form;
#[cfg(test)]
mod vorticity_two_form;
