// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

mod adjustable;
mod coordinate;
mod display;
mod identifiable;
mod metric;
mod space_temporal;
mod spatial;
mod temporal;

use crate::prelude::TimeScale;
use deep_causality_macros::Constructor;

/// A concrete 3D + time context based on classical (Euclidean) geometry.
///
/// `EuclideanSpacetime` models a spatial point in 3D Euclidean space with an
/// associated monotonic time unit (e.g., nanoseconds, milliseconds). This is
/// the default representation for causal modeling in low-latency engineering
/// systems such as sensor fusion, MagNav, or robotics.
///
/// The space is assumed to be flat and orthogonal (Cartesian), and time is
/// treated as an absolute scalar clock.
///
/// # Fields
/// - `id`: Unique numeric identifier
/// - `coords`: Spatial coordinates `[x, y, z]` in meters
/// - `time`: Time unit (e.g., nanoseconds since boot or epoch)
/// - `scale`: Time scale unit (e.g., microseconds, milliseconds)
///
/// # Common Applications
/// - Sensor placement and motion modeling
/// - Low-latency messaging with time-stamped spatial tags
/// - Aircraft, drone, or embedded system causal modeling
///
/// # Example
/// ```
/// use deep_causality::prelude::*;
///
/// let s1 = AdjustableEuclideanSpacetime::new(1, [0.0, 0.0, 0.0], 1_000_000.00f64, TimeScale::Microseconds);
/// let s2 = AdjustableEuclideanSpacetime::new(2, [3.0, 4.0, 0.0], 2_000_000.00f64, TimeScale::Microseconds);
///
/// let spatial_dist = s1.distance(&s2); // should be 5.0
/// println!("Distance: {:.2} meters", spatial_dist);
/// assert_eq!(s1.dimension(), 3);
/// assert_eq!(s2.coordinate(0), &3.0);
/// ```
#[derive(Constructor, Debug, Copy, Clone, PartialEq)]
pub struct AdjustableEuclideanSpacetime {
    /// Unique numeric ID for this context
    id: u64,
    /// Spatial coordinates in `[x, y, z]` (meters)
    coords: [f64; 3],
    /// Scalar time value (e.g., seconds since epoch)
    t: f64, // time in SI time unit
    ///  SI time unit
    time_scale: TimeScale,
}
