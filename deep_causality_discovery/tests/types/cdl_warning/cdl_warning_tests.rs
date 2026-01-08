/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{CdlWarning, CdlWarningLog};
use deep_causality_haft::{LogAddEntry, LogAppend, LogSize};

#[test]
fn test_cdl_warning_variants() {
    let w1 = CdlWarning::DataIssue("missing val".into());
    let w2 = CdlWarning::FeatureIssue("low var".into());
    let w3 = CdlWarning::ModelIssue("diverged".into());
    let w4 = CdlWarning::Generic("foo".into());

    match w1 {
        CdlWarning::DataIssue(s) => assert_eq!(s, "missing val"),
        _ => panic!("Wrong variant"),
    }
    match w2 {
        CdlWarning::FeatureIssue(s) => assert_eq!(s, "low var"),
        _ => panic!("Wrong variant"),
    }
    match w3 {
        CdlWarning::ModelIssue(s) => assert_eq!(s, "diverged"),
        _ => panic!("Wrong variant"),
    }
    match w4 {
        CdlWarning::Generic(s) => assert_eq!(s, "foo"),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_cdl_warning_from_str() {
    let w: CdlWarning = "test".into();
    assert_eq!(w, CdlWarning::Generic("test".into()));
}

#[test]
fn test_cdl_warning_log_traits() {
    let mut log = CdlWarningLog::default();
    assert!(log.is_empty());
    assert_eq!(log.len(), 0);

    log.add_entry("entry 1");
    assert!(!log.is_empty());
    assert_eq!(log.len(), 1);
    assert_eq!(log.entries[0], CdlWarning::Generic("entry 1".into()));

    let mut other_log = CdlWarningLog::default();
    other_log
        .entries
        .push(CdlWarning::DataIssue("data error".into()));

    log.append(&mut other_log);
    assert_eq!(log.len(), 2);
    assert_eq!(log.entries[1], CdlWarning::DataIssue("data error".into()));
}
