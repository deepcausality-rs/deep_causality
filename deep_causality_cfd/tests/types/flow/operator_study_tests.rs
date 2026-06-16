/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The operator-accuracy Flow solver kind: the viscous DEC operator `δd` swept over a resolution
//! ladder reports its (second-order) convergence without marching a field.

use deep_causality_cfd::{Flow, Operator};

#[test]
fn viscous_operator_converges_at_second_order() {
    let report = Flow::operator_study::<f64>("viscous-order")
        .operator(Operator::Viscous)
        .resolutions([16, 32, 64, 128])
        .run()
        .expect("operator study runs");

    let errors = report.series("operator_error").expect("operator_error series");
    let orders = report
        .series("convergence_order")
        .expect("convergence_order series");

    assert_eq!(errors.len(), 4, "one error per resolution");
    assert_eq!(orders.len(), 3, "one order between each consecutive pair");

    // The discrete δd reproduces the eigenvalue to O(h²): the error shrinks monotonically and the
    // observed order sits near 2.
    for w in errors.windows(2) {
        assert!(w[1] < w[0], "error decreases with resolution: {errors:?}");
    }
    for &order in orders {
        assert!(
            (order - 2.0).abs() < 0.25,
            "viscous operator order {order} is not near 2: {orders:?}"
        );
    }
}

#[test]
fn operator_study_requires_at_least_two_resolutions() {
    let result = Flow::operator_study::<f64>("too-few")
        .resolutions([16])
        .run();
    assert!(result.is_err(), "a single resolution has no defined order");
}
