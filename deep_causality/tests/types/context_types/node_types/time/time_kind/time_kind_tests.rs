// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::*;

fn setup_euclidean() -> TimeKind {
    TimeKind::Euclidean(EuclideanTime::new(1, TimeScale::Second, 1.23))
}

fn setup_entropic() -> TimeKind {
    EntropicTime::new(2, 42).into()
}

fn setup_discrete() -> TimeKind {
    DiscreteTime::new(3, TimeScale::Second, 7).into()
}

fn setup_lorentzian() -> TimeKind {
    LorentzianTime::new(4, TimeScale::Second, 99.99).into()
}

#[test]
fn test_id_resolution() {
    assert_eq!(setup_euclidean().id(), 1);
    assert_eq!(setup_entropic().id(), 2);
    assert_eq!(setup_discrete().id(), 3);
    assert_eq!(setup_lorentzian().id(), 4);
}

#[test]
fn test_time_scale() {
    assert_eq!(setup_euclidean().time_scale(), TimeScale::Second);
    assert_eq!(setup_entropic().time_scale(), TimeScale::NoScale);
    assert_eq!(setup_discrete().time_scale(), TimeScale::Second);
    assert_eq!(setup_lorentzian().time_scale(), TimeScale::Second);
}

#[test]
fn test_time_unit_and_project() {
    let e = setup_euclidean();
    let r = setup_entropic();
    let d = setup_discrete();
    let l = setup_lorentzian();

    assert!((e.time_unit() - 1.23).abs() < f64::EPSILON);
    assert_eq!(r.time_unit(), 42.0);
    assert_eq!(d.time_unit(), 7.0);
    assert!((l.time_unit() - 99.99).abs() < f64::EPSILON);

    assert_eq!(e.project(), 1.23);
    assert_eq!(r.project(), 42.0);
    assert_eq!(d.project(), 7.0);
    assert_eq!(l.project(), 99.99);
}

#[test]
fn test_display_trait() {
    let e = setup_euclidean();
    let r = setup_entropic();
    let d = setup_discrete();
    let l = setup_lorentzian();

    let s1 = format!("{}", e);
    let s2 = format!("{}", r);
    let s3 = format!("{}", d);
    let s4 = format!("{}", l);

    assert!(s1.contains("EuclideanTime"));
    assert!(s2.contains("EntropicTime"));
    assert!(s3.contains("DiscreteTime"));
    assert!(s4.contains("LorentzianTime"));

    assert!(s1.contains("id: 1"));
    assert!(s2.contains("id: 2"));
    assert!(s3.contains("id: 3"));
    assert!(s4.contains("id: 4"));
}

#[test]
fn test_partial_eq() {
    let t1 = setup_euclidean();
    let t2 = TimeKind::Euclidean(EuclideanTime::new(1, TimeScale::Second, 1.23));
    let t3 = setup_discrete();

    assert_eq!(t1, t2);
    assert_ne!(t1, t3);
}

#[test]
fn test_clone_and_debug() {
    let t = setup_entropic();
    let c = t.clone();
    assert_eq!(t, c);
    let dbg = format!("{:?}", t);
    assert!(dbg.contains("Entropic"));
}
