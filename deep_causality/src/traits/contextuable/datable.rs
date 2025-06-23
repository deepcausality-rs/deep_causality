// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::Identifiable;

/// Represents data-bearing entities in a causal context graph.
///
/// This trait marks nodes or values that carry domain-specific data
/// relevant to inference, observation, or explanation. It extends
/// [`Identifiable`] to ensure that each instance has a unique identity.
///
/// This trait is intentionally left minimal to allow full flexibility
/// in how data is modeled. You may wrap sensor input, encoded strings,
/// discrete values, or even external references.
///
/// # Example
/// ```
/// use deep_causality::prelude::{Datable, Identifiable};
///
/// struct SensorReading { id: u64, value: f64 }
/// impl Identifiable for SensorReading { fn id(&self) -> u64 { self.id } }
/// impl Datable for SensorReading {}
/// ```
pub trait Datable: Identifiable {}