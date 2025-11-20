/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{ModificationLog, ModificationLogEntry, OpStatus};
use std::collections::HashMap;

#[test]
fn test_op_status_success() {
    let status = OpStatus::Success;
    assert_eq!(status, OpStatus::Success);
}

#[test]
fn test_op_status_failure() {
    let status = OpStatus::Failure;
    assert_eq!(status, OpStatus::Failure);
}

#[test]
fn test_op_status_clone() {
    let status = OpStatus::Success;
    let cloned = status.clone();
    assert_eq!(status, cloned);
}

#[test]
fn test_modification_log_entry_new() {
    let entry =
        ModificationLogEntry::new("TestOperation", "123", OpStatus::Success, "Test message");

    assert_eq!(entry.operation_name, "TestOperation");
    assert_eq!(entry.target_id, "123");
    assert_eq!(entry.status, OpStatus::Success);
    assert_eq!(entry.message, "Test message");
    assert_eq!(entry.timestamp, 0); // Not yet timestamped
    assert!(entry.metadata.is_empty());
}

#[test]
fn test_modification_log_entry_new_with_format() {
    let entry = ModificationLogEntry::new(
        "TestOperation",
        format!("id_{}", 456),
        OpStatus::Failure,
        format!("Error: {}", "test error"),
    );

    assert_eq!(entry.operation_name, "TestOperation");
    assert_eq!(entry.target_id, "id_456");
    assert_eq!(entry.status, OpStatus::Failure);
    assert_eq!(entry.message, "Error: test error");
}

#[test]
fn test_modification_log_entry_with_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("key1".to_string(), "value1".to_string());
    metadata.insert("key2".to_string(), "value2".to_string());

    let entry = ModificationLogEntry::with_metadata(
        "TestOperation",
        "789",
        OpStatus::Success,
        "Test with metadata",
        metadata.clone(),
    );

    assert_eq!(entry.operation_name, "TestOperation");
    assert_eq!(entry.target_id, "789");
    assert_eq!(entry.status, OpStatus::Success);
    assert_eq!(entry.message, "Test with metadata");
    assert_eq!(entry.metadata.len(), 2);
    assert_eq!(entry.metadata.get("key1"), Some(&"value1".to_string()));
    assert_eq!(entry.metadata.get("key2"), Some(&"value2".to_string()));
}

#[test]
fn test_modification_log_new() {
    let log = ModificationLog::new();
    assert_eq!(log.len(), 0);
    assert!(log.is_empty());
}

#[test]
fn test_modification_log_default() {
    let log = ModificationLog::default();
    assert_eq!(log.len(), 0);
    assert!(log.is_empty());
}

#[test]
fn test_modification_log_add_entry_timestamps() {
    let mut log = ModificationLog::new();

    let entry1 = ModificationLogEntry::new("Op1", "1", OpStatus::Success, "First operation");

    log.add_entry(entry1);

    // Timestamp should be automatically set and non-zero
    assert_eq!(log.len(), 1);
    assert!(log.entries[0].timestamp > 0);

    // Add another entry with a small delay
    std::thread::sleep(std::time::Duration::from_micros(10));

    let entry2 = ModificationLogEntry::new("Op2", "2", OpStatus::Success, "Second operation");

    log.add_entry(entry2);

    assert_eq!(log.len(), 2);
    assert!(log.entries[1].timestamp > 0);
    // Second timestamp should be >= first (monotonic)
    assert!(log.entries[1].timestamp >= log.entries[0].timestamp);
}

#[test]
fn test_modification_log_add_entry_overrides_timestamp() {
    let mut log = ModificationLog::new();

    let mut entry = ModificationLogEntry::new("TestOp", "1", OpStatus::Success, "Test");

    // Manually set a timestamp
    entry.timestamp = 12345;

    log.add_entry(entry);

    // Timestamp should be overridden by add_entry
    assert_ne!(log.entries[0].timestamp, 12345);
    assert!(log.entries[0].timestamp > 12345);
}

#[test]
fn test_modification_log_len() {
    let mut log = ModificationLog::new();
    assert_eq!(log.len(), 0);

    log.add_entry(ModificationLogEntry::new(
        "Op1",
        "1",
        OpStatus::Success,
        "msg1",
    ));
    assert_eq!(log.len(), 1);

    log.add_entry(ModificationLogEntry::new(
        "Op2",
        "2",
        OpStatus::Success,
        "msg2",
    ));
    assert_eq!(log.len(), 2);
}

#[test]
fn test_modification_log_is_empty() {
    let mut log = ModificationLog::new();
    assert!(log.is_empty());

    log.add_entry(ModificationLogEntry::new(
        "Op",
        "1",
        OpStatus::Success,
        "msg",
    ));
    assert!(!log.is_empty());
}

#[test]
fn test_modification_log_iter() {
    let mut log = ModificationLog::new();

    log.add_entry(ModificationLogEntry::new(
        "Op1",
        "1",
        OpStatus::Success,
        "msg1",
    ));
    log.add_entry(ModificationLogEntry::new(
        "Op2",
        "2",
        OpStatus::Failure,
        "msg2",
    ));
    log.add_entry(ModificationLogEntry::new(
        "Op3",
        "3",
        OpStatus::Success,
        "msg3",
    ));

    let entries: Vec<_> = log.iter().collect();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].operation_name, "Op1");
    assert_eq!(entries[1].operation_name, "Op2");
    assert_eq!(entries[2].operation_name, "Op3");
}

#[test]
fn test_modification_log_append() {
    use deep_causality::LogAppend;

    let mut log1 = ModificationLog::new();
    log1.add_entry(ModificationLogEntry::new(
        "Op1",
        "1",
        OpStatus::Success,
        "msg1",
    ));
    log1.add_entry(ModificationLogEntry::new(
        "Op2",
        "2",
        OpStatus::Success,
        "msg2",
    ));

    let mut log2 = ModificationLog::new();
    log2.add_entry(ModificationLogEntry::new(
        "Op3",
        "3",
        OpStatus::Failure,
        "msg3",
    ));
    log2.add_entry(ModificationLogEntry::new(
        "Op4",
        "4",
        OpStatus::Success,
        "msg4",
    ));

    log1.append(&mut log2);

    assert_eq!(log1.len(), 4);
    assert_eq!(log2.len(), 0); // log2 should be empty after append

    let entries: Vec<_> = log1.iter().collect();
    assert_eq!(entries[0].operation_name, "Op1");
    assert_eq!(entries[1].operation_name, "Op2");
    assert_eq!(entries[2].operation_name, "Op3");
    assert_eq!(entries[3].operation_name, "Op4");
}

#[test]
fn test_modification_log_clone() {
    let mut log = ModificationLog::new();
    log.add_entry(ModificationLogEntry::new(
        "Op1",
        "1",
        OpStatus::Success,
        "msg1",
    ));
    log.add_entry(ModificationLogEntry::new(
        "Op2",
        "2",
        OpStatus::Failure,
        "msg2",
    ));

    let cloned = log.clone();

    assert_eq!(log.len(), cloned.len());
    assert_eq!(
        log.entries[0].operation_name,
        cloned.entries[0].operation_name
    );
    assert_eq!(
        log.entries[1].operation_name,
        cloned.entries[1].operation_name
    );
}

#[test]
fn test_modification_log_entry_clone() {
    let entry = ModificationLogEntry::new("TestOp", "123", OpStatus::Success, "Test message");

    let cloned = entry.clone();

    assert_eq!(entry.operation_name, cloned.operation_name);
    assert_eq!(entry.target_id, cloned.target_id);
    assert_eq!(entry.status, cloned.status);
    assert_eq!(entry.message, cloned.message);
    assert_eq!(entry.timestamp, cloned.timestamp);
}
