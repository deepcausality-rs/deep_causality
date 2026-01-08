/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::EffectLog;
use deep_causality_haft::{LogAddEntry, LogAppend, LogSize};

#[test]
fn test_new_log() {
    let log = EffectLog::new();
    assert!(log.is_empty());
    assert_eq!(log.len(), 0);
}

#[test]
fn test_with_capacity() {
    let log = EffectLog::with_capacity(10);
    assert!(log.is_empty());
    assert_eq!(log.len(), 0);
}

#[test]
fn test_add_entry() {
    let mut log = EffectLog::new();
    log.add_entry("test message");
    assert!(!log.is_empty());
    assert_eq!(log.len(), 1);

    let display = format!("{}", log);
    assert!(display.contains("test message"));
}

#[test]
fn test_append() {
    let mut log1 = EffectLog::new();
    log1.add_entry("msg1");

    let mut log2 = EffectLog::new();
    log2.add_entry("msg2");

    log1.append(&mut log2);

    assert_eq!(log1.len(), 2);
    assert_eq!(log2.len(), 0); // log2 entries moved to log1

    let display = format!("{}", log1);
    assert!(display.contains("msg1"));
    assert!(display.contains("msg2"));
}

#[test]
fn test_from_string() {
    let log = EffectLog::from("hello".to_string());
    assert_eq!(log.len(), 1);
    assert!(format!("{}", log).contains("hello"));
}

#[test]
fn test_from_str() {
    let log = EffectLog::from("world");
    assert_eq!(log.len(), 1);
    assert!(format!("{}", log).contains("world"));
}
