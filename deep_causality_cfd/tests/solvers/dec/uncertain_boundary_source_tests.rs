/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Group U — the cross-domain `UncertainBoundarySource` (`add-boundary-zone-abstraction`).
//!
//! The source is exercised **standalone** — only `MaybeUncertain<R>` and an `EffectLog`, no
//! fluid-dynamics type — to pin its domain-agnostic contract: a present sample resolves to its
//! collapsed value and updates the last-good; a dropout returns the last-good and (at the default
//! verbosity) records a dropout entry. The fluid `UncertainInflowZone` is one application of it,
//! and its Group-C tests pass bit-for-bit through this source.

use deep_causality_cfd::{DropoutVerbosity, UncertainBoundarySource};
use deep_causality_core::EffectLog;
use deep_causality_haft::LogSize;
use deep_causality_uncertain::{MaybeUncertain, Uncertain};

fn fast_source(default_value: f64) -> UncertainBoundarySource<f64> {
    UncertainBoundarySource::new(default_value)
        .with_presence_gate(0.5, 0.9, 0.1, 64)
        .with_collapse_samples(16)
}

#[test]
fn present_sample_resolves_to_its_value_and_updates_last_good() {
    let source = fast_source(0.0);
    let mut last_good = 0.0;
    let sample = MaybeUncertain::<f64>::from_uncertain(Uncertain::normal(3.0, 0.01));

    let (value, dropout) = source.resolve(&sample, &mut last_good).unwrap();
    assert!(!dropout, "a present sample must not be a dropout");
    assert!(
        (value - 3.0).abs() < 0.1,
        "resolved value {value} far from 3.0"
    );
    assert!(
        (last_good - value).abs() < 1e-12,
        "last-good must update to the resolved value"
    );
}

#[test]
fn dropout_returns_last_good_and_records_at_default_verbosity() {
    let source = fast_source(0.0);
    let mut last_good = 2.5; // a prior present value.
    let absent = MaybeUncertain::<f64>::always_none();

    let (value, dropout) = source.resolve(&absent, &mut last_good).unwrap();
    assert!(dropout, "an absent sample must be a dropout");
    assert_eq!(value, 2.5, "a dropout returns the last-good value");
    assert_eq!(last_good, 2.5, "a dropout must not change the last-good");

    let mut logs = EffectLog::new();
    source.record(&mut logs, 7, dropout, false, last_good);
    assert_eq!(logs.len(), 1, "EachDropout records each dropout");
    assert!(
        format!("{logs}").contains("dropout"),
        "the dropout is recorded in the log"
    );
}

#[test]
fn transition_verbosity_records_only_onset_and_recovery() {
    let source = fast_source(0.0).with_verbosity(DropoutVerbosity::Transitions);
    let mut logs = EffectLog::new();

    // onset (dropout after present), continued dropout, recovery (present after dropout).
    source.record(&mut logs, 1, true, false, 1.0); // ONSET
    source.record(&mut logs, 2, true, true, 1.0); // continued — no record
    source.record(&mut logs, 3, false, true, 1.0); // RECOVERY
    assert_eq!(logs.len(), 2, "only the two transitions are recorded");
    let log = format!("{logs}");
    assert!(
        log.contains("ONSET") && log.contains("RECOVERY"),
        "log: {log}"
    );
}
