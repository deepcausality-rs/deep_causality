/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `StudyError` wraps the two causes a campaign meets and carries the verb it failed in, so a
//! verdict names where the study broke.

use deep_causality_cfd::StudyError;
use deep_causality_file::{DataLoadingError, NumericTable};
use deep_causality_physics::PhysicsError;
use std::error::Error;

#[test]
fn an_untagged_physics_cause_renders_without_a_stage() {
    let e: StudyError = PhysicsError::CalculationError("diverged".into()).into();
    assert_eq!(e.stage(), "");
    let msg = format!("{e}");
    assert!(msg.contains("diverged"), "{msg}");
    assert!(!msg.contains("in '"), "no stage clause when untagged: {msg}");
    assert!(e.source().is_none(), "physics cause is not chained");
}

#[test]
fn a_tagged_data_cause_names_the_verb_and_chains_its_source() {
    // Provoke a real DataLoadingError: a missing column.
    let table = NumericTable::from_columns([("mach", "-")], vec![vec![1.2_f64]]).unwrap();
    let data_err: DataLoadingError = table.column("airspeed").unwrap_err();

    let e = StudyError::in_stage("read", data_err);
    assert_eq!(e.stage(), "read");
    let msg = format!("{e}");
    assert!(msg.contains("in 'read'"), "names the verb: {msg}");
    assert!(msg.contains("airspeed"), "carries the cause: {msg}");
    assert!(e.source().is_some(), "data cause is chained as the source");
}

#[test]
fn in_stage_retags_an_already_converted_error() {
    let e = StudyError::in_stage("reduce", PhysicsError::CalculationError("nan".into()));
    assert_eq!(e.stage(), "reduce");
    assert!(format!("{e}").contains("in 'reduce'"));
}
