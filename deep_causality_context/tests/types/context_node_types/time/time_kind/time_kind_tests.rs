/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::FloatType;
use deep_causality_context::*;

fn setup_newtonian() -> TimeKind<FloatType> {
    TimeKind::Newtonian(NewtonianTime::new(1, TimeScale::Second, 1.23))
}

fn setup_entropic() -> TimeKind<FloatType> {
    EntropicTime::new(2, 42).into()
}

fn setup_discrete() -> TimeKind<FloatType> {
    DiscreteTime::new(3, TimeScale::Second, 7).into()
}

fn setup_minkowski() -> TimeKind<FloatType> {
    MinkowskiTime::new(4, TimeScale::Second, 99.99).into()
}

#[test]
fn test_id_resolution() {
    assert_eq!(setup_newtonian().id(), 1);
    assert_eq!(setup_entropic().id(), 2);
    assert_eq!(setup_discrete().id(), 3);
    assert_eq!(setup_minkowski().id(), 4);
}

#[test]
fn test_time_scale() {
    assert_eq!(setup_newtonian().time_scale(), TimeScale::Second);
    assert_eq!(setup_entropic().time_scale(), TimeScale::NoScale);
    assert_eq!(setup_discrete().time_scale(), TimeScale::Second);
    assert_eq!(setup_minkowski().time_scale(), TimeScale::Second);
}

#[test]
fn test_time_unit_and_project() {
    let n = setup_newtonian();
    let r = setup_entropic();
    let d = setup_discrete();
    let m = setup_minkowski();

    assert!((n.time_unit() - 1.23).abs() < f64::EPSILON);
    assert_eq!(r.time_unit(), 42.0);
    assert_eq!(d.time_unit(), 7.0);
    assert!((m.time_unit() - 99.99).abs() < f64::EPSILON);

    assert_eq!(n.project(), 1.23);
    assert_eq!(r.project(), 42.0);
    assert_eq!(d.project(), 7.0);
    assert_eq!(m.project(), 99.99);
}

#[test]
fn test_display_trait() {
    let n = setup_newtonian();
    let r = setup_entropic();
    let d = setup_discrete();
    let m = setup_minkowski();

    let s1 = format!("{n}");
    let s2 = format!("{r}");
    let s3 = format!("{d}");
    let s4 = format!("{m}");

    assert!(s1.contains("NewtonianTime"));
    assert!(s2.contains("EntropicTime"));
    assert!(s3.contains("DiscreteTime"));
    assert!(s4.contains("MinkowskiTime"));

    assert!(s1.contains("id: 1"));
    assert!(s2.contains("id: 2"));
    assert!(s3.contains("id: 3"));
    assert!(s4.contains("id: 4"));

    // Newtonian and Minkowski time both name their coordinate `t`.
    assert_eq!(s1, "NewtonianTime(id: 1, t: 1.23)");
    assert_eq!(s4, "MinkowskiTime(id: 4, t: 99.99)");
}

#[test]
fn test_partial_eq() {
    let t1 = setup_newtonian();
    let t2 = TimeKind::Newtonian(NewtonianTime::new(1, TimeScale::Second, 1.23));
    let t3 = setup_discrete();

    assert_eq!(t1, t2);
    assert_ne!(t1, t3);
}

#[test]
fn test_clone_and_debug() {
    let t = setup_entropic();
    let c = t;
    assert_eq!(t, c);
    let dbg = format!("{t:?}");
    assert!(dbg.contains("Entropic"));
}
