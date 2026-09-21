/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::FloatType;
use deep_causality_context::*;

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
