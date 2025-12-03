/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalityError, EffectLog, PropagatingEffectWitness};
use deep_causality_haft::Effect3;

/// `CausalEffectSystem` is a marker struct that serves as a concrete instance of the
/// `Effect3` trait from the `deep_causality_haft` crate.
///
/// It explicitly defines the fixed types for the error and log components that will be
/// carried alongside the primary value within the monadic context. This allows the
/// `CausalMonad` to operate on a consistent structure for error propagation and logging.
///
/// By implementing `Effect3`, `CausalEffectSystem` declares:
/// - `Fixed1 = CausalityError`: The type used for representing errors in the causal system.
///   When an operation fails, a `CausalityError` is propagated.
/// - `Fixed2 = CausalEffectLog`: The type used for accumulating a history of operations.
///   Every step in a monadic chain can add entries to this log.
/// - `HktWitness = PropagatingEffectWitness<Self::Fixed1, Self::Fixed2>`: A phantom type
///   that links this system to the `CausalPropagatingEffect` structure, which is the
///   actual container for the value, error, and logs.
///
/// This setup is crucial for enabling the Higher-Kinded Type (HKT) pattern, allowing
/// generic monadic operations over `CausalPropagatingEffect` instances.
pub struct CausalEffectSystem;

impl Effect3 for CausalEffectSystem {
    type Fixed1 = CausalityError;
    type Fixed2 = EffectLog;
    type HktWitness = PropagatingEffectWitness<Self::Fixed1, Self::Fixed2>;
}
