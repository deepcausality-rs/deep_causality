/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// Uniform on the real interval `[0, 1)`.
///
/// A real-valued distribution, which is why it lives here rather than in the entropy crate: the
/// machine words it consumes come from `deep_causality_rand`, but what it *means* is a statement
/// about a real field, and it belongs beside the densities that speak the same language.
///
/// Machine words are drawn through `deep_causality_rand::StandardWord` and Booleans through
/// `StandardBool`. Those are different objects and have their own samplers.
#[derive(Clone, Copy, Debug, Default)]
pub struct StandardUniform;
