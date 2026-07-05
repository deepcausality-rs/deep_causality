/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The study warning channel: classification, message extraction, and the log's `LogAddEntry` /
//! `LogAppend` / `LogSize` behavior the effect substrate relies on.

use deep_causality_cfd::{StudyWarning, StudyWarningLog};
use deep_causality_haft::{LogAddEntry, LogAppend, LogSize};

#[test]
fn a_plain_str_classifies_as_generic_and_exposes_its_message() {
    let w = StudyWarning::from("something to note");
    assert_eq!(w, StudyWarning::Generic("something to note".to_string()));
    assert_eq!(w.message(), "something to note");
    assert_eq!(StudyWarning::Data("d".into()).message(), "d");
    assert_eq!(StudyWarning::Case("c".into()).message(), "c");
}

#[test]
fn the_log_adds_appends_and_reports_size() {
    let mut log = StudyWarningLog::default();
    assert!(log.is_empty());
    assert_eq!(log.len(), 0);

    log.add_entry("first"); // LogAddEntry -> Generic
    log.push(StudyWarning::Data("second".into()));
    assert!(!log.is_empty());
    assert_eq!(log.len(), 2);

    let mut more = StudyWarningLog::default();
    more.push(StudyWarning::Case("third".into()));
    log.append(&mut more);

    assert_eq!(log.len(), 3);
    assert!(more.is_empty(), "append drains the source");

    let msgs: Vec<&str> = log.entries().iter().map(StudyWarning::message).collect();
    assert_eq!(msgs, ["first", "second", "third"]);
}
