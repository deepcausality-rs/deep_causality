/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::CausalEffectLog;
use deep_causality_haft::LogAppend;
use std::thread;
use std::time::Duration;

#[test]
fn test_new_and_default_are_empty() {
    let log_new = CausalEffectLog::new();
    assert!(log_new.is_empty());

    let log_default = CausalEffectLog::default();
    assert!(log_default.is_empty());

    assert_eq!(log_new, log_default);
}

#[test]
fn test_with_capacity() {
    let log = CausalEffectLog::with_capacity(10);
    assert!(log.is_empty());
}

#[test]
fn test_add_entry() {
    let mut log = CausalEffectLog::new();
    assert!(log.is_empty());

    let msg = "This is a test message";
    log.add_entry(msg);

    assert!(!log.is_empty());

    // To verify the content, we can use the Display trait implementation
    let log_str = log.to_string();
    assert!(log_str.contains(msg));
    assert!(log_str.contains("ts_micros:"));
}

#[test]
fn test_append() {
    let mut log1 = CausalEffectLog::new();
    log1.add_entry("Message 1");

    let mut log2 = CausalEffectLog::new();
    log2.add_entry("Message 2");
    log2.add_entry("Message 3");

    log1.append(&mut log2);

    assert!(!log1.is_empty());
    assert!(log2.is_empty());

    let log_str = log1.to_string();
    assert!(log_str.contains("Message 1"));
    assert!(log_str.contains("Message 2"));
    assert!(log_str.contains("Message 3"));
    assert!(log_str.contains("3 entries"));
}

#[test]
fn test_append_to_empty() {
    let mut log1 = CausalEffectLog::new();
    let mut log2 = CausalEffectLog::new();
    log2.add_entry("Message 1");

    log1.append(&mut log2);

    assert!(!log1.is_empty());
    assert!(log2.is_empty());
    assert!(log1.to_string().contains("Message 1"));
}

#[test]
fn test_append_empty_to_non_empty() {
    let mut log1 = CausalEffectLog::new();
    log1.add_entry("Message 1");

    let mut log2 = CausalEffectLog::new();

    log1.append(&mut log2);

    assert!(!log1.is_empty());
    assert!(log2.is_empty());
    assert!(log1.to_string().contains("Message 1"));
    assert!(log1.to_string().contains("1 entries"));
}

#[test]
fn test_log_append_trait() {
    let mut log1 = CausalEffectLog::from("Log 1 entry");
    let mut log2 = CausalEffectLog::from("Log 2 entry");

    // Use the trait method
    LogAppend::append(&mut log1, &mut log2);

    assert!(log2.is_empty());
    let log_str = log1.to_string();
    assert!(log_str.contains("Log 1 entry"));
    assert!(log_str.contains("Log 2 entry"));
    assert!(log_str.contains("2 entries"));
}

#[test]
fn test_from_string() {
    let msg = "Create from String".to_string();
    let log: CausalEffectLog = msg.clone().into();

    assert!(!log.is_empty());
    assert!(log.to_string().contains(&msg));
    assert!(log.to_string().contains("1 entries"));
}

#[test]
fn test_from_str() {
    let msg = "Create from &str";
    let log: CausalEffectLog = msg.into();

    assert!(!log.is_empty());
    assert!(log.to_string().contains(msg));
    assert!(log.to_string().contains("1 entries"));
}

#[test]
fn test_display_empty() {
    let log = CausalEffectLog::new();
    let expected = "CausalEffectLog: (empty)";
    assert_eq!(log.to_string(), expected);
}

#[test]
fn test_display_non_empty() {
    let mut log = CausalEffectLog::new();
    log.add_entry("First entry");
    thread::sleep(Duration::from_millis(2)); // Ensure different timestamps
    log.add_entry("Second entry");

    let log_str = log.to_string();

    assert!(log_str.starts_with("CausalEffectLog (2 entries):"));
    assert!(log_str.contains("First entry"));
    assert!(log_str.contains("Second entry"));

    // Check that there are two timestamp lines
    let lines_with_ts: Vec<_> = log_str
        .lines()
        .filter(|line| line.contains("ts_micros:"))
        .collect();
    assert_eq!(lines_with_ts.len(), 2);
}
