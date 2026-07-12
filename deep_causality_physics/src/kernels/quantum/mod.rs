/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// The quantum-information kernels (gates, Haruna gates, Dirac-notation ops and
// their PropagatingEffect wrappers) live in `deep_causality_quantum`. What
// remains here is the Klein-Gordon PDE kernel — a relativistic field-theory
// operator on a simplicial manifold, kept with the other physics kernels.

pub(crate) mod mechanics;
pub(crate) mod wrappers;

pub use mechanics::*;
pub use wrappers::*;
