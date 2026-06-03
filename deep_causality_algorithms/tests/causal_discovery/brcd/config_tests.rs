/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::brcd_config::{BrcdConfig, FamilyKind};
use deep_causality_algorithms::brcd::brcd_gaussian::Transform;

#[test]
fn continuous_sets_the_continuous_family_defaults() {
    let c = BrcdConfig::<f64>::continuous(7);
    assert_eq!(c.seed, 7);
    assert_eq!(c.family, FamilyKind::Continuous);
    assert_eq!(c.node_transform, Transform::None);
    assert!(!c.transform_parents);
    assert_eq!(c.num_root_causes, 1);
    assert!(c.ridge > 0.0);
    assert!(c.alpha_star > 0.0);
}

#[test]
fn discrete_switches_only_the_family() {
    let c = BrcdConfig::<f64>::discrete(5);
    assert_eq!(c.seed, 5);
    assert_eq!(c.family, FamilyKind::Discrete);
    // Everything else is inherited from the continuous defaults.
    assert_eq!(c.num_root_causes, 1);
    assert_eq!(c.node_transform, Transform::None);
}

#[test]
fn default_is_continuous_with_seed_zero() {
    let c = BrcdConfig::<f64>::default();
    assert_eq!(c.seed, 0);
    assert_eq!(c.family, FamilyKind::Continuous);
}

#[test]
fn family_kind_is_copy_and_comparable() {
    let f = FamilyKind::Discrete;
    let g = f; // Copy, not move.
    assert_eq!(f, g);
    assert_ne!(FamilyKind::Continuous, FamilyKind::Discrete);
    assert!(!format!("{f:?}").is_empty());
}

#[test]
fn config_is_cloneable() {
    let c = BrcdConfig::<f64>::discrete(3);
    let d = c.clone();
    assert_eq!(c.seed, d.seed);
    assert_eq!(c.family, d.family);
    assert!(!format!("{d:?}").is_empty());
}
