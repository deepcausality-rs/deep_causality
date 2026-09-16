//! Gravity-related types for GQCD experiments.
//!
//! These types are generic over `R: RealField` to support both `f64` and
//! `DoubleFloat` precision.

use deep_causality_algebra::RealField;

/// Structure to hold GM result and associated altitude for correlation analysis
#[derive(Debug, Clone, Copy)]
pub struct GmDataPoint<R: RealField> {
    pub altitude: R,
    pub gm: R,
    pub radial_velocity: R,
    pub latitude: R,
    pub longitude: R,
}

impl<R: RealField> GmDataPoint<R> {
    pub fn new(altitude: R, gm: R, radial_velocity: R, latitude: R, longitude: R) -> Self {
        Self {
            altitude,
            gm,
            radial_velocity,
            latitude,
            longitude,
        }
    }
}
