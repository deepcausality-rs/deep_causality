// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::RelationKind;

#[test]
fn test_datial()
{
    let rk = RelationKind::Datial;
    assert_eq!(rk, RelationKind::Datial);
    assert_eq!(rk.to_string(), format!("Datial"));
}

#[test]
fn test_temporal()
{
    let rk = RelationKind::Temporal;
    assert_eq!(rk, RelationKind::Temporal);
    assert_eq!(rk.to_string(), format!("Temporal"));
}

#[test]
fn test_spatial()
{
    let rk = RelationKind::Spatial;
    assert_eq!(rk, RelationKind::Spatial);
    assert_eq!(rk.to_string(), format!("Spatial"));
}

#[test]
fn test_space_temporal()
{
    let rk = RelationKind::SpaceTemporal;
    assert_eq!(rk, RelationKind::SpaceTemporal);
    assert_eq!(rk.to_string(), format!("SpaceTemporal"));
}