/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::{
    Data, EuclideanTime, GenerativeTrigger, NumberType, TimeKind, TimeScale,
};
use std::fmt::Write;
#[test]
fn test_time_elapsed_some() {
    let time_kind = TimeKind::Euclidean(EuclideanTime::new(1, TimeScale::Second, 42.0));
    let trigger: GenerativeTrigger<NumberType> = GenerativeTrigger::TimeElapsed(time_kind);
    assert_eq!(trigger.time_elapsed(), Some(&time_kind));
}

#[test]
fn test_time_elapsed_none_data_received() {
    let data = Data::new(1, 42u64);
    let trigger: GenerativeTrigger<NumberType> = GenerativeTrigger::DataReceived(data);
    assert_eq!(trigger.time_elapsed(), None);
}

#[test]
fn test_time_elapsed_none_manual_intervention() {
    let trigger: GenerativeTrigger<NumberType> =
        GenerativeTrigger::ManualIntervention("Test message".to_string());
    assert_eq!(trigger.time_elapsed(), None);
}

#[test]
fn test_data_received_some() {
    let data = Data::new(1, 42u64);
    let trigger: GenerativeTrigger<NumberType> = GenerativeTrigger::DataReceived(data);
    assert_eq!(trigger.data_received(), Some(&data));
}

#[test]
fn test_data_received_none_time_elapsed() {
    let time_kind = TimeKind::Euclidean(EuclideanTime::new(1, TimeScale::Second, 42.0));
    let trigger: GenerativeTrigger<NumberType> = GenerativeTrigger::TimeElapsed(time_kind);
    assert_eq!(trigger.data_received(), None);
}

#[test]
fn test_data_received_none_manual_intervention() {
    let trigger: GenerativeTrigger<NumberType> =
        GenerativeTrigger::ManualIntervention("Test message".to_string());
    assert_eq!(trigger.data_received(), None);
}

#[test]
fn test_manual_intervention_some() {
    let message = "System maintenance".to_string();
    let trigger: GenerativeTrigger<NumberType> =
        GenerativeTrigger::ManualIntervention(message.clone());
    assert_eq!(trigger.manual_intervention(), Some(&message));
}

#[test]
fn test_manual_intervention_none_time_elapsed() {
    let time_kind = TimeKind::Euclidean(EuclideanTime::new(1, TimeScale::Second, 42.0));
    let trigger: GenerativeTrigger<NumberType> = GenerativeTrigger::TimeElapsed(time_kind);
    assert_eq!(trigger.manual_intervention(), None);
}

#[test]
fn test_manual_intervention_none_data_received() {
    let data = Data::new(1, 42u64);
    let trigger: GenerativeTrigger<NumberType> = GenerativeTrigger::DataReceived(data);
    assert_eq!(trigger.manual_intervention(), None);
}

#[test]
fn test_generative_trigger_display_time_elapsed() {
    let time_kind = TimeKind::Euclidean(EuclideanTime::new(1, TimeScale::Second, 42.0));
    let trigger: GenerativeTrigger<NumberType> = GenerativeTrigger::TimeElapsed(time_kind);
    let mut s = String::new();
    write!(&mut s, "{trigger}").unwrap();
    // Assuming TimeKind::Euclidean(EuclideanTime::new(123)) formats as "EuclideanTime(123)"
    // or similar. Adjust expected string if TimeKind's Display impl differs.
    assert_eq!(s, "EuclideanTime(id: 1, τ: 42)");
}

#[test]
fn test_generative_trigger_display_data_received() {
    let data = Data::new(1, 99u64);
    let trigger: GenerativeTrigger<NumberType> = GenerativeTrigger::DataReceived(data);
    let mut s = String::new();
    write!(&mut s, "{trigger}").unwrap();
    // Assuming Data<u64> formats as "Data(id: 1, data: 99)" or similar.
    // Adjust expected string if Data's Display impl differs.
    assert_eq!(s, "Dataoid: id: 1 data: 99");
}

#[test]
fn test_generative_trigger_display_manual_intervention() {
    let message = "User initiated shutdown".to_string();
    let trigger: GenerativeTrigger<NumberType> =
        GenerativeTrigger::ManualIntervention(message.clone());
    let mut s = String::new();
    write!(&mut s, "{trigger}").unwrap();
    assert_eq!(s, message);
}

#[test]
fn test_generative_trigger_debug() {
    let time_kind = TimeKind::Euclidean(EuclideanTime::new(1, TimeScale::Second, 42.0));
    let trigger_time: GenerativeTrigger<NumberType> = GenerativeTrigger::TimeElapsed(time_kind);
    let debug_str = format!("{trigger_time:?}");
    // The exact debug string might vary slightly based on Rust version/formatting,
    // but it should contain the variant name and its content.
    assert!(debug_str.contains("TimeElapsed"));

    let data = Data::new(2, 123u64);
    let trigger_data: GenerativeTrigger<NumberType> = GenerativeTrigger::DataReceived(data);
    let debug_str = format!("{trigger_data:?}");
    assert!(debug_str.contains("DataReceived"));
    assert!(debug_str.contains("Data { id: 2, data: 123 }"));

    let message = "Debug test message".to_string();
    let trigger_manual: GenerativeTrigger<NumberType> =
        GenerativeTrigger::ManualIntervention(message);
    let debug_str = format!("{trigger_manual:?}");
    assert!(debug_str.contains("ManualIntervention"));
    assert!(debug_str.contains("\"Debug test message\""));
}
