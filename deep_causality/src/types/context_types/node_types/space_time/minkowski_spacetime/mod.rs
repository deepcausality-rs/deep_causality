// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality_macros::Constructor;
use crate::prelude::TimeScale;

mod coordinate;
mod identifiable;
mod space_temporal;
mod space_temporal_interval;
mod spatial;
mod temporal;
mod display;

/// A 4D spacetime context based on the Minkowski metric of special relativity.
///
/// `MinkowskiSpacetime` represents an event in flat spacetime using four real-valued coordinates:
/// `t` (time) and `x`, `y`, `z` (space). It assumes a **Minkowski metric signature** (−+++),
/// and enables interval calculations according to:
///
/// ```text
/// s² = −c²·Δt² + Δx² + Δy² + Δz²
/// ```
///
/// This allows precise modeling of:
/// - Time-like, space-like, and light-like (null) separations
/// - Proper time and relativistic intervals
/// - Special relativistic propagation constraints
///
/// # Fields
/// - `id`: Unique numeric identifier
/// - `t`: Time coordinate (in seconds)
/// - `x, y, z`: Spatial coordinates (in meters)
///
/// # Common Applications
/// - Relativistic simulation
/// - Causal propagation with light cones
/// - Quantum field theory spacetime diagrams
///
/// # Example
/// ```
/// use deep_causality::prelude::*;
///
/// let e1 = MinkowskiSpacetime::new(1, 1.0, 0.0, 0.0, 0.0, TimeScale::Second);
/// let e2 = MinkowskiSpacetime::new(2, 3.0, 3.0, 4.0, 0.0, TimeScale::Second);
///
/// let s2 = e1.interval_squared(&e2);
/// println!("s² = {}", s2);
/// assert!(s2 < 0.0); // time-like interval
/// ```
#[derive(Constructor, Debug, Copy, Clone, PartialEq)]
pub struct MinkowskiSpacetime {
    /// Unique numeric ID for this event
    id: u64,
    /// Spatial X coordinate in meters
    x: f64,
    /// Spatial Y coordinate in meters
    y: f64,
    /// Spatial Z coordinate in meters
    z: f64,
    // time in SI time unit, 
    t: f64,
    time_scale: TimeScale, // SI time unit
}