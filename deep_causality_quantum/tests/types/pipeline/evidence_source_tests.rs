/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A context records itself as its snapshot; the observation it lands in is tested through the
//! control stage.

use deep_causality_context::TimeScale;
use deep_causality_quantum::{
    EnvironmentReading, InterferometerConfiguration, InterferometerModel, ObservedContext,
    interferometer_context, record_environment,
};

#[test]
fn test_a_context_records_its_snapshot() {
    let model = InterferometerModel::new(1.6106e7, 0.1, 0.5, 2.4e-7, 0.5, 1000.0, 30.0).unwrap();
    let configuration = InterferometerConfiguration::new(1.0e-5, 2.0e-6, 1.5708e5).unwrap();
    let mut context = interferometer_context(1, "gravimeter", model, configuration).unwrap();
    record_environment(
        &mut context,
        60,
        TimeScale::Second,
        EnvironmentReading::new(1.2e-6, 296.15, 4.8e-5).unwrap(),
    )
    .unwrap();
    assert_eq!(
        context.context_snapshot().unwrap(),
        Some(context.snapshot().unwrap())
    );
}
