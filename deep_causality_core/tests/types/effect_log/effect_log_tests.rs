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

#[test]
fn test_eq_ignores_timestamps() {
    // Two logs with the same message sequence, built independently (so their entry timestamps
    // differ): value equality compares messages, not timestamps, so these must be equal.
    let mut a = EffectLog::new();
    a.add_entry("alpha");
    a.add_entry("beta");

    let mut b = EffectLog::new();
    b.add_entry("alpha");
    b.add_entry("beta");

    assert_eq!(
        a, b,
        "identical message sequences must be equal regardless of timestamps"
    );
}

#[test]
fn test_eq_distinguishes_content_order_and_length() {
    let mut a = EffectLog::new();
    a.add_entry("one");
    a.add_entry("two");

    let mut diff_content = EffectLog::new();
    diff_content.add_entry("one");
    diff_content.add_entry("THREE");
    assert_ne!(a, diff_content, "different message content must not be equal");

    let mut diff_order = EffectLog::new();
    diff_order.add_entry("two");
    diff_order.add_entry("one");
    assert_ne!(a, diff_order, "different message order must not be equal");

    let mut diff_len = EffectLog::new();
    diff_len.add_entry("one");
    assert_ne!(a, diff_len, "different length must not be equal");

    assert_eq!(EffectLog::new(), EffectLog::new(), "empty logs are equal");
}

#[test]
fn test_eq_with_timestamps_holds_for_clone() {
    let mut log = EffectLog::new();
    log.add_entry("x");
    let copy = log.clone(); // clone preserves timestamps
    assert!(log.eq_with_timestamps(&copy));
    assert_eq!(log, copy);
}

#[test]
fn test_eq_with_timestamps_requires_matching_timestamps() {
    let mut a = EffectLog::new();
    a.add_entry("same");

    // Force a distinct wall-clock timestamp for the second entry.
    std::thread::sleep(std::time::Duration::from_millis(5));

    let mut b = EffectLog::new();
    b.add_entry("same");

    assert_eq!(a, b, "same message → value-equal");
    assert!(
        !a.eq_with_timestamps(&b),
        "independently timestamped entries must not be timestamp-equal"
    );
}
