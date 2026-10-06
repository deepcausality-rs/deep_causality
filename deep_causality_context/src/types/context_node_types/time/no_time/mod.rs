/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod display;
mod identifiable;
mod recordable;
mod temporable;

/// The absence of time, as a type: it fills the temporal slot of a `Context` whose graph holds no
/// time node.
///
/// A `Context` names a spatial, a temporal and a spacetime type whether or not its graph holds
/// nodes of each. `NoTime` fills the temporal slot, beside [`NoSpace`](crate::NoSpace) for the
/// spatial slot and [`NoSpaceTime`](crate::NoSpaceTime) for the spacetime slot:
///
/// ```
/// use deep_causality_context::{Context, Data, EuclideanSpace, NoSpaceTime, NoTime};
///
/// type Layout = Context<Data<f64>, EuclideanSpace<f64>, NoTime, NoSpaceTime<f64>>;
///
/// let context: Layout = Context::with_capacity(1, "positions only", 4);
/// assert_eq!(context.name(), "positions only");
/// ```
///
/// # What its trait impls say
///
/// It is zero-sized. Its time unit is `()` and its scale `TimeScale::NoScale`: there is no
/// instant to report. It has no record: writing one is `Unrecordable`, reading one is
/// `WrongVariant`.
///
/// # Example
/// ```
/// use deep_causality_context::{NoTime, Temporal, TimeScale};
///
/// let none = NoTime::new();
///
/// assert_eq!(size_of::<NoTime>(), 0);
/// assert_eq!(none.time_scale(), TimeScale::NoScale);
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct NoTime;

impl NoTime {
    pub fn new() -> Self {
        Self
    }
}
