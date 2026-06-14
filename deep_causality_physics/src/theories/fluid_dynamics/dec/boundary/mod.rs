/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Composable boundary zones for the DEC Navier–Stokes solver (CFD Stage-4
//! `add-boundary-zone-abstraction`).
//!
//! A boundary condition is a [`BoundaryZone`] term with layered dispatch — one hook per solver
//! stage — composed **statically** (typed tuples, no `dyn`). The zone set carries the explicit
//! boundary actuators (a [`BodyForceZone`], a [`MovingWall`], and — added with the open-boundary
//! groups — inflow and outflow); structural boundaries (wall no-slip, immersed cut bodies) are
//! derived automatically at the topology/metric layer.

pub(crate) mod body_force_zone;
pub(crate) mod boundary_zone;
pub(crate) mod moving_wall;

pub use body_force_zone::BodyForceZone;
pub use boundary_zone::BoundaryZone;
pub use moving_wall::MovingWall;
