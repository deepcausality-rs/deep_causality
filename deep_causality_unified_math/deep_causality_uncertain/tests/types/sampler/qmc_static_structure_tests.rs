/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The QMC pre-pass's guard against data-dependent stochastic structure.
//!
//! QMC assigns each drawing leaf a fixed Sobol dimension before any sample is taken, so it can only
//! admit a graph whose set of drawing leaves is settled before the draw. The `BindOp` arm — which
//! chose the graph to draw from by a value it had already drawn — was the other way that could
//! fail; it is gone, and a branch-divergent conditional is the arm that now represents the same
//! hazard. Unlike `BindOp`, it is reachable from the public constructors, so this needs no
//! crate-internal reach-through.

use deep_causality_uncertain::{QmcSampler, Uncertain, UncertainBool, UncertainError};

/// A conditional whose branches draw different distributions is refused for the same reason: the
/// set of dimensions the sample touches would depend on the condition's draw.
#[test]
fn the_pre_pass_rejects_branch_divergent_conditionals() {
    let condition = UncertainBool::<f64>::bernoulli(0.5);
    let divergent = Uncertain::conditional(
        condition,
        Uncertain::normal(0.0, 1.0),
        Uncertain::normal(9.0, 1.0),
    );
    assert!(matches!(
        QmcSampler::new(&divergent, None),
        Err(UncertainError::SamplingError(_))
    ));
}

/// Sharing the branch makes the structure static again, and the sampler builds.
#[test]
fn the_pre_pass_accepts_a_shared_branch() {
    let condition = UncertainBool::<f64>::bernoulli(0.5);
    let shared = Uncertain::normal(0.0, 1.0);
    let convergent = Uncertain::conditional(condition, shared.clone(), shared);
    assert!(QmcSampler::new(&convergent, None).is_ok());
}
