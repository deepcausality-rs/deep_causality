/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Lund String Fragmentation Model
//!
//! This module implements the Lund string fragmentation model for QCD hadronization.
//! The model treats the color flux tube between a quark-antiquark pair as a
//! relativistic string with tension κ ≈ 1 GeV/fm.
//!
//! # Physical Model
//!
//! When a q-q̄ pair separates, the potential energy stored in the string
//! increases linearly with distance. When sufficient energy is available,
//! new q-q̄ pairs are created from the vacuum, breaking the string into
//! shorter segments that eventually form hadrons.
//!
//! # References
//!
//! - B. Andersson et al., "Parton Fragmentation and String Dynamics",
//!   Physics Reports 97 (1983) 31-145
//! - T. Sjöstrand et al., "PYTHIA 8.3", Comput. Phys. Commun. 265 (2021) 107810

#[cfg(feature = "os-random")]
mod flavor;
#[cfg(feature = "os-random")]
mod fragmentation;
#[cfg(feature = "os-random")]
mod kinematics;
#[cfg(feature = "os-random")]
mod string;

#[cfg(feature = "os-random")]
pub use fragmentation::lund_string_fragmentation_kernel;
