/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Gauge group implementations for Standard Model + Gravity.
//!
//! This module provides marker types for fundamental gauge groups:
//!
//! | Group         | Lie Dim | Abelian | Physics               |
//! |---------------|---------|---------|------------------------|
//! | U1            | 1       | Yes     | Electromagnetism (QED) |
//! | SU2           | 3       | No      | Weak isospin           |
//! | SU3           | 8       | No      | Strong force (QCD)     |
//! | Electroweak   | 4       | No      | SU(2)×U(1) unified     |
//! | StandardModel | 12      | No      | Full SM (except gravity)|
//! | Lorentz       | 6       | No      | General Relativity     |

mod electroweak;
mod lorentz;
mod standard_model;
mod su2;
mod su3;
mod u1;

pub use electroweak::Electroweak;
pub use lorentz::Lorentz;
pub use standard_model::StandardModel;
pub use su2::SU2;
pub use su3::SU3;
pub use u1::U1;
