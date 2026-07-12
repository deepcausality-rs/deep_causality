/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The owned configuration for the quasi-one-dimensional duct march: an area
//! profile, the inlet stagnation state, the back pressure, the grid
//! resolution, and a stop condition. [`CfdFlow::march`](crate::CfdFlow::march)
//! lowers it onto the duct driver and returns the standard
//! [`Report`](crate::Report) (design D4).

use crate::CfdScalar;
use deep_causality_physics::PhysicsError;

/// The duct's cross-sectional area as a function of axial position.
#[derive(Debug, Clone)]
pub enum DuctAreaProfile<R> {
    /// A sampled `(x, A)` table, strictly ascending in `x` with positive
    /// areas; the driver interpolates linearly between points.
    Table(Vec<(R, R)>),
    /// An analytic converging–diverging profile on `x ∈ [0, length]` with the
    /// throat fixed at mid-duct: each half is a parabola in `x`, so the area
    /// slope vanishes at the throat (the smooth transonic passage needs
    /// `dA/dx = 0` where `M = 1`).
    ConvergingDiverging {
        /// Inlet area `A_in` at `x = 0`.
        inlet_area: R,
        /// Throat area `A_t` at `x = length/2` (the minimum).
        throat_area: R,
        /// Exit area `A_e` at `x = length`.
        exit_area: R,
        /// Duct length.
        length: R,
    },
}

impl<R: CfdScalar> DuctAreaProfile<R> {
    /// The axial start of the profile.
    pub(crate) fn x_start(&self) -> R {
        match self {
            Self::Table(points) => points[0].0,
            Self::ConvergingDiverging { .. } => R::zero(),
        }
    }

    /// The axial end of the profile.
    pub(crate) fn x_end(&self) -> R {
        match self {
            Self::Table(points) => points[points.len() - 1].0,
            Self::ConvergingDiverging { length, .. } => *length,
        }
    }

    /// The minimum area — the reference throat area `A*` of the thrust
    /// coefficient. For a table the minimum sits on a node (the profile is
    /// piecewise linear); for the analytic variant it is the throat.
    pub(crate) fn min_area(&self) -> R {
        match self {
            Self::Table(points) => {
                let mut min = points[0].1;
                for &(_, a) in points.iter().skip(1) {
                    if a < min {
                        min = a;
                    }
                }
                min
            }
            Self::ConvergingDiverging { throat_area, .. } => *throat_area,
        }
    }

    /// The area at `x`, clamped to the profile's ends.
    pub(crate) fn area_at(&self, x: R) -> R {
        match self {
            Self::Table(points) => {
                if x <= points[0].0 {
                    return points[0].1;
                }
                let last = points[points.len() - 1];
                if x >= last.0 {
                    return last.1;
                }
                for w in points.windows(2) {
                    if x <= w[1].0 {
                        let t = (x - w[0].0) / (w[1].0 - w[0].0);
                        return w[0].1 + t * (w[1].1 - w[0].1);
                    }
                }
                last.1
            }
            Self::ConvergingDiverging {
                inlet_area,
                throat_area,
                exit_area,
                length,
            } => {
                // 0.5 is exactly representable in every supported scalar (f32/f64/Float106); a
                // silent `R::one()` fallback would move the throat to `x = length` and disable the
                // diverging half, so a scalar that cannot represent it is a hard error rather than
                // a wrong geometry.
                let half = R::from_f64(0.5).expect("0.5 must be representable in the CFD scalar");
                let x_t = half * *length;
                let clamped = if x < R::zero() {
                    R::zero()
                } else if x > *length {
                    *length
                } else {
                    x
                };
                if clamped <= x_t {
                    // Converging parabola: A(0) = A_in, A(x_t) = A_t, A'(x_t) = 0.
                    let s = (x_t - clamped) / x_t;
                    *throat_area + (*inlet_area - *throat_area) * s * s
                } else {
                    // Diverging parabola: A(x_t) = A_t, A(L) = A_e, A'(x_t) = 0.
                    let s = (clamped - x_t) / (*length - x_t);
                    *throat_area + (*exit_area - *throat_area) * s * s
                }
            }
        }
    }
}

/// The inlet stagnation (reservoir) state of a duct case.
#[derive(Debug, Clone, Copy)]
pub struct DuctInlet<R> {
    /// Stagnation pressure `p₀`.
    pub p0: R,
    /// Stagnation temperature `T₀` (K).
    pub t0: R,
}

/// The stop condition of the quasi-steady duct march: a step budget and the
/// residual the march must settle below within it.
#[derive(Debug, Clone, Copy)]
pub struct DuctStop<R> {
    /// The step budget.
    pub max_steps: usize,
    /// The residual gate: maximum relative change of the conserved state per
    /// step.
    pub residual_tol: R,
}

/// The owned configuration for a quasi-one-dimensional duct march: geometry,
/// inlet stagnation state, back pressure, resolution, and the stop condition.
/// Holds only owned specs; the same config can be run repeatedly.
#[derive(Debug, Clone)]
pub struct DuctConfig<R: CfdScalar> {
    pub(crate) profile: DuctAreaProfile<R>,
    /// Inlet stagnation pressure `p₀`.
    pub(crate) p0: R,
    /// Inlet stagnation temperature `T₀` (K).
    pub(crate) t0: R,
    /// Ratio of specific heats.
    pub(crate) gamma: R,
    /// Static back pressure at the exit plane (same unit as `p₀`).
    pub(crate) back_pressure: R,
    /// Finite-volume cell count.
    pub(crate) cells: usize,
    /// Step budget of the quasi-steady march.
    pub(crate) max_steps: usize,
    /// The residual gate: the march stops when the maximum relative change of
    /// the conserved state per step drops below this tolerance.
    pub(crate) residual_tol: R,
}

impl<R: CfdScalar> DuctConfig<R> {
    /// A validated duct case: `profile` geometry, `inlet` stagnation state,
    /// ratio of specific heats `gamma`, exit-plane `back_pressure`, `cells`
    /// finite volumes, and the `stop` condition.
    ///
    /// # Errors
    /// [`PhysicsError::PhysicalInvariantBroken`] on a table with fewer than
    /// two points, non-ascending `x`, or a non-positive or non-finite area;
    /// on an analytic profile whose throat is not the strict minimum or whose
    /// length is not positive; on a `p0`, `t0`, or `back_pressure` that is
    /// not finite and positive; on `back_pressure >= p0` (nothing drives the
    /// flow); on `gamma` not finite and `> 1`; on `cells < 8` (the driver
    /// needs a resolvable throat); on a zero `max_steps`; or on a
    /// `residual_tol` that is not finite and positive.
    pub fn new(
        profile: DuctAreaProfile<R>,
        inlet: DuctInlet<R>,
        gamma: R,
        back_pressure: R,
        cells: usize,
        stop: DuctStop<R>,
    ) -> Result<Self, PhysicsError> {
        let DuctInlet { p0, t0 } = inlet;
        let DuctStop {
            max_steps,
            residual_tol,
        } = stop;
        let positive = |x: R| x.is_finite() && x > R::zero();
        match &profile {
            DuctAreaProfile::Table(points) => {
                if points.len() < 2 {
                    return Err(PhysicsError::PhysicalInvariantBroken(
                        "DuctConfig area table needs at least two (x, area) points".into(),
                    ));
                }
                if points.iter().any(|&(x, a)| !x.is_finite() || !positive(a)) {
                    return Err(PhysicsError::PhysicalInvariantBroken(
                        "DuctConfig area table needs finite x and finite, positive areas".into(),
                    ));
                }
                if points.windows(2).any(|w| w[1].0 <= w[0].0) {
                    return Err(PhysicsError::PhysicalInvariantBroken(
                        "DuctConfig area table must be strictly ascending in x".into(),
                    ));
                }
            }
            DuctAreaProfile::ConvergingDiverging {
                inlet_area,
                throat_area,
                exit_area,
                length,
            } => {
                if !positive(*inlet_area) || !positive(*throat_area) || !positive(*exit_area) {
                    return Err(PhysicsError::PhysicalInvariantBroken(
                        "DuctConfig areas must be finite and positive".into(),
                    ));
                }
                if !(*throat_area < *inlet_area && *throat_area < *exit_area) {
                    return Err(PhysicsError::PhysicalInvariantBroken(
                        "DuctConfig throat_area must be the strict minimum (below inlet and exit)"
                            .into(),
                    ));
                }
                if !positive(*length) {
                    return Err(PhysicsError::PhysicalInvariantBroken(
                        "DuctConfig length must be finite and positive".into(),
                    ));
                }
            }
        }
        if !positive(p0) || !positive(t0) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "DuctConfig stagnation state (p0, t0) must be finite and positive".into(),
            ));
        }
        if !(gamma.is_finite() && gamma > R::one()) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "DuctConfig gamma must be finite and > 1".into(),
            ));
        }
        if !positive(back_pressure) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "DuctConfig back_pressure must be finite and positive".into(),
            ));
        }
        if back_pressure >= p0 {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "DuctConfig back_pressure must be below the stagnation pressure p0".into(),
            ));
        }
        if cells < 8 {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "DuctConfig needs at least 8 cells".into(),
            ));
        }
        if max_steps == 0 {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "DuctConfig max_steps must be at least 1".into(),
            ));
        }
        if !positive(residual_tol) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "DuctConfig residual_tol must be finite and positive".into(),
            ));
        }
        Ok(Self {
            profile,
            p0,
            t0,
            gamma,
            back_pressure,
            cells,
            max_steps,
            residual_tol,
        })
    }

    /// The area profile.
    pub fn profile(&self) -> &DuctAreaProfile<R> {
        &self.profile
    }

    /// The inlet stagnation pressure `p₀`.
    pub fn p0(&self) -> R {
        self.p0
    }

    /// The inlet stagnation temperature `T₀`.
    pub fn t0(&self) -> R {
        self.t0
    }

    /// The ratio of specific heats.
    pub fn gamma(&self) -> R {
        self.gamma
    }

    /// The exit-plane static back pressure.
    pub fn back_pressure(&self) -> R {
        self.back_pressure
    }

    /// The finite-volume cell count.
    pub fn cells(&self) -> usize {
        self.cells
    }

    /// The step budget.
    pub fn max_steps(&self) -> usize {
        self.max_steps
    }

    /// The residual gate of the quasi-steady stop condition.
    pub fn residual_tol(&self) -> R {
        self.residual_tol
    }
}
