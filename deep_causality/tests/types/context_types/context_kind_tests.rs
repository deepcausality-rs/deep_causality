// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::prelude::ContextKind;

#[test]
fn test_time() {
    let ct = ContextKind::Time;
    assert_eq!(ct, ContextKind::Time);
    assert_eq!(ct.to_string(), "Time");
}

#[test]
fn test_space() {
    let ct = ContextKind::Space;
    assert_eq!(ct, ContextKind::Space);
    assert_eq!(ct.to_string(), "Space");
}

#[test]
fn test_space_time() {
    let ct = ContextKind::SpaceTime;
    assert_eq!(ct, ContextKind::SpaceTime);
    assert_eq!(ct.to_string(), "SpaceTime");
}

#[test]
fn test_data() {
    let ct = ContextKind::Data;
    assert_eq!(ct, ContextKind::Data);
    assert_eq!(ct.to_string(), "Data");
}

#[test]
fn test_time_data() {
    let ct = ContextKind::TimeData;
    assert_eq!(ct, ContextKind::TimeData);
    assert_eq!(ct.to_string(), "TimeData");
}

#[test]
fn test_space_data() {
    let ct = ContextKind::SpaceData;
    assert_eq!(ct, ContextKind::SpaceData);
    assert_eq!(ct.to_string(), "SpaceData");
}

#[test]
fn test_space_time_data() {
    let ct = ContextKind::SpaceTimeData;
    assert_eq!(ct, ContextKind::SpaceTimeData);
    assert_eq!(ct.to_string(), "SpaceTimeData");
}