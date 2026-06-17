/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the `Report` accessors (`name`, `series`, `final_field`). A `Report` is only produced
//! by a workflow run, so it is obtained here from the verify workflow (no final field) and a DEC
//! march (with a final edge cochain), then its accessors are exercised.

use deep_causality_cfd::{CfdConfigBuilder, CfdFlow, Mesh, Observe, Seed, TaylorGreen};

#[test]
fn verify_report_exposes_name_and_series_and_has_no_final_field() {
    let config = CfdConfigBuilder::verify::<f64, _>("named-run", TaylorGreen::new(0.1, 1.0))
        .sample_at([1.0, 0.5, 0.0], 0.0)
        .build()
        .unwrap();
    let report = CfdFlow::verify(&config).run().unwrap();

    assert_eq!(report.name(), "named-run");
    assert!(report.series("mms_error").is_some());
    // A label that was never recorded yields None.
    assert!(report.series("does-not-exist").is_none());
    // The pointwise verify workflow runs no DEC march, so there is no final field.
    assert!(report.final_field().is_none());
}

#[test]
fn march_report_carries_the_final_edge_cochain() {
    let config = CfdConfigBuilder::march::<2, f64>("march-report")
        .mesh(Mesh::box_domain([6, 6]))
        .solver(
            CfdConfigBuilder::dec_ns()
                .viscosity(0.05)
                .time_step(0.005)
                .build()
                .unwrap(),
        )
        .seed(Seed::Rest)
        .march_for(1)
        .observe(Observe::default().kinetic_energy())
        .build()
        .unwrap();
    let manifold = config.materialize().unwrap();
    let report = CfdFlow::march(&config).on(&manifold).run().unwrap();

    // A DEC march records the final velocity edge cochain.
    let field = report
        .final_field()
        .expect("a march records its final field");
    assert!(!field.is_empty());
}
