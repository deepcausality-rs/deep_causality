/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::CpdagError;
use std::error::Error;

#[test]
fn test_display_variants() {
    assert!(
        CpdagError::FileNotFound("p.csv".into())
            .to_string()
            .contains("CPDAG file not found")
    );
    assert!(CpdagError::Io("x".into()).to_string().contains("IO error"));
    assert!(CpdagError::MissingHeader.to_string().contains("vertices=N"));
    assert!(
        CpdagError::Parse("bad".into())
            .to_string()
            .contains("parse error")
    );
    assert!(
        CpdagError::VertexOutOfRange {
            index: 5,
            num_vertices: 3
        }
        .to_string()
        .contains("out of range")
    );
    assert!(
        CpdagError::Graph("boom".into())
            .to_string()
            .contains("graph construction failed")
    );
}

#[test]
fn test_is_error() {
    let e = CpdagError::MissingHeader;
    let _: &dyn Error = &e;
    assert!(e.source().is_none());
}

#[test]
fn test_eq() {
    assert_eq!(CpdagError::MissingHeader, CpdagError::MissingHeader);
    assert_ne!(CpdagError::Parse("a".into()), CpdagError::Parse("b".into()));
}
