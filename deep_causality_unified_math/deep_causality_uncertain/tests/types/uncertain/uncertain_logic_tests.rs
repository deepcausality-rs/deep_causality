/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::Uncertain;

#[test]
fn test_uncertain_bool_bitand() {
    let t = Uncertain::<bool>::point(true);
    let f = Uncertain::<bool>::point(false);

    assert!((t.clone() & t.clone()).sample_from_entropy().unwrap());
    assert!(!(t.clone() & f.clone()).sample_from_entropy().unwrap());
    assert!(!(f.clone() & t.clone()).sample_from_entropy().unwrap());
    assert!(!(f.clone() & f.clone()).sample_from_entropy().unwrap());
}

#[test]
fn test_uncertain_bool_bitor() {
    let t = Uncertain::<bool>::point(true);
    let f = Uncertain::<bool>::point(false);

    assert!((t.clone() | t.clone()).sample_from_entropy().unwrap());
    assert!((t.clone() | f.clone()).sample_from_entropy().unwrap());
    assert!((f.clone() | t.clone()).sample_from_entropy().unwrap());
    assert!(!(f.clone() | f.clone()).sample_from_entropy().unwrap());
}

#[test]
fn test_uncertain_bool_not() {
    let t = Uncertain::<bool>::point(true);
    let f = Uncertain::<bool>::point(false);

    assert!(!(!t).sample_from_entropy().unwrap());
    assert!((!f).sample_from_entropy().unwrap());
}

#[test]
fn test_uncertain_bool_bitxor() {
    let t = Uncertain::<bool>::point(true);
    let f = Uncertain::<bool>::point(false);

    assert!(!(t.clone() ^ t.clone()).sample_from_entropy().unwrap());
    assert!((t.clone() ^ f.clone()).sample_from_entropy().unwrap());
    assert!((f.clone() ^ t.clone()).sample_from_entropy().unwrap());
    assert!(!(f.clone() ^ f.clone()).sample_from_entropy().unwrap());
}
