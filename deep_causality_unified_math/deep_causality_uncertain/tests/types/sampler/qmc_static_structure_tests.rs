/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The QMC pre-pass's guard against data-dependent stochastic structure.

use deep_causality_ast::ConstTree;
use deep_causality_uncertain::utils_tests::raw_graph::qmc_sampler_from_root;
use deep_causality_uncertain::{
    DistributionEnum, Node, NormalDistributionParams, QmcSampler, Sample, SampledBindFn, Uncertain,
    UncertainBool, UncertainError,
};
use std::sync::Arc;

/// A `BindOp` chooses the graph to draw from by a value it has already drawn, so which
/// distributions the sample touches is not known before the sample. QMC assigns a fixed dimension
/// per drawing leaf ahead of time, so it cannot admit one.
///
/// No carrier method builds a `BindOp`, which is why the tree here is assembled by hand.
#[test]
fn the_pre_pass_rejects_a_bind_op() {
    let operand = ConstTree::new(Node::Distribution(DistributionEnum::Normal(
        NormalDistributionParams::new(0.0f64, 1.0),
    )));
    let func: Arc<dyn SampledBindFn<f64>> =
        Arc::new(|_v: Sample<f64>| ConstTree::new(Node::Value(Sample::Real(0.0))));
    let root = ConstTree::new(Node::BindOp { func, operand });

    let err = qmc_sampler_from_root(&root, None).expect_err("a BindOp must be refused");
    assert!(matches!(err, UncertainError::SamplingError(_)));
}

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
