/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::FloatType;
use deep_causality_context::*;
use std::marker::PhantomData;

pub type StdCtx = ContextoidType<
    Data<i32>,
    EuclideanSpace<FloatType>,
    EuclideanTime<FloatType>,
    EuclideanSpacetime<FloatType>,
>;

#[test]
fn test_contextoid_kind_and_accessors() {
    let id = 1;
    let data = 42;

    let d = Data::new(id, data);
    let s = EuclideanSpace::new(1, 1.0, 2.0, 3.0);
    let t = EuclideanTime::new(1, TimeScale::Second, 4.0);
    let st = EuclideanSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    let r = Root::new(0);

    let cd: StdCtx = ContextoidType::Datoid(d.clone());
    let ct: StdCtx = ContextoidType::Tempoid(t);
    let cr: StdCtx = ContextoidType::Root(r);
    let cs: StdCtx = ContextoidType::Spaceoid(s.clone());
    let cst: StdCtx = ContextoidType::SpaceTempoid(st);

    // Validate enum kind matching
    assert_eq!(cd.kind(), ContextKind::Datoid);
    assert_eq!(ct.kind(), ContextKind::Tempoid);
    assert_eq!(cr.kind(), ContextKind::Root);
    assert_eq!(cs.kind(), ContextKind::Spaceoid);
    assert_eq!(cst.kind(), ContextKind::SpaceTempoid);

    // Validate type-specific accessors
    assert_eq!(cd.dataoid(), Some(&d));
    assert_eq!(ct.tempoid(), Some(&t));
    assert_eq!(cr.root(), Some(&r));
    assert_eq!(cs.spaceoid(), Some(&s));
    assert_eq!(cst.space_tempoid(), Some(&st));

    // Validate negative cases
    assert!(cd.root().is_none());
    assert!(cs.root().is_none());

    // Validate Display impl
    assert!(cd.to_string().contains("Datoid"));
    assert!(ct.to_string().contains("Tempoid"));
    assert!(cr.to_string().contains("Root"));
    assert!(cs.to_string().contains("Spaceoid"));
    assert!(cst.to_string().contains("SpaceTempoid"));
}

#[test]
#[should_panic(expected = "internal error: entered unreachable code")]
fn test_kind_of_the_marker_variant_is_refused() {
    // `_Marker` is `#[doc(hidden)]` but public, so an outside caller can build one. `kind()` has
    // no `ContextKind` to answer with and refuses. A catch-all arm returning some existing kind
    // instead would compile and silently file the marker as a real node.
    let marker: StdCtx = ContextoidType::_Marker(PhantomData);
    let _ = marker.kind();
}

#[test]
#[should_panic(expected = "_Marker variant should never be accessed directly")]
fn test_display_of_the_marker_variant_is_refused() {
    let marker: StdCtx = ContextoidType::_Marker(PhantomData);
    let _ = marker.to_string();
}

#[test]
fn test_accessors_reject_the_marker_variant() {
    // The five accessors match one variant each and fall through for anything else, so the marker
    // must answer `None` to all of them rather than panic or match.
    let marker: StdCtx = ContextoidType::_Marker(PhantomData);

    assert!(marker.root().is_none());
    assert!(marker.dataoid().is_none());
    assert!(marker.tempoid().is_none());
    assert!(marker.spaceoid().is_none());
    assert!(marker.space_tempoid().is_none());
}

#[test]
fn test_every_variant_reports_its_own_kind() {
    // One kind per variant, and no two share. A `kind()` arm written against the wrong variant
    // collapses two of these onto one another.
    let variants: [(StdCtx, ContextKind); 5] = [
        (
            ContextoidType::Datoid(Data::new(1, 42)),
            ContextKind::Datoid,
        ),
        (
            ContextoidType::Tempoid(EuclideanTime::new(2, TimeScale::Second, 4.0)),
            ContextKind::Tempoid,
        ),
        (ContextoidType::Root(Root::new(3)), ContextKind::Root),
        (
            ContextoidType::Spaceoid(EuclideanSpace::new(4, 1.0, 2.0, 3.0)),
            ContextKind::Spaceoid,
        ),
        (
            ContextoidType::SpaceTempoid(EuclideanSpacetime::new(
                5,
                1.0,
                2.0,
                3.0,
                4.0,
                TimeScale::Second,
            )),
            ContextKind::SpaceTempoid,
        ),
    ];

    let mut seen = std::collections::HashSet::new();
    for (variant, expected) in variants {
        assert_eq!(variant.kind(), expected, "kind of {variant:?}");
        assert!(seen.insert(expected), "{expected:?} reported twice");
    }
    assert_eq!(seen.len(), 5);
}
