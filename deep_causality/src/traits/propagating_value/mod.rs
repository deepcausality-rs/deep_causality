//! This module defines the `PropagatingValue` trait.

/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use std::fmt::Debug;

/// A marker trait to signify that a type can be used as a value within the causal system.
///
/// Types implementing `PropagatingValue` must also implement `Debug`, `Clone`, `Default`,
/// and have a `'static` lifetime. These bounds are necessary for practical use with
/// `EffectValue` and `Any` in the causaloid registry, enabling type-safe and flexible
/// handling of causal data.
pub trait PropagatingValue: Debug + Clone + 'static {}
