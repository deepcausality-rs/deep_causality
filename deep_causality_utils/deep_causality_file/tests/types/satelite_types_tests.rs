/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_file::SatId;
use std::collections::{BTreeSet, HashSet};

/// Every variant, paired with its `as_str` code and `as_num` number. This table drives the
/// round-trip checks so each match arm in `try_from`, `as_str`, and `as_num` is exercised.
const ALL: &[(SatId, &str, u32)] = &[
    (SatId::E01, "E01", 1),
    (SatId::E02, "E02", 2),
    (SatId::E03, "E03", 3),
    (SatId::E04, "E04", 4),
    (SatId::E05, "E05", 5),
    (SatId::E07, "E07", 7),
    (SatId::E08, "E08", 8),
    (SatId::E09, "E09", 9),
    (SatId::E10, "E10", 10),
    (SatId::E11, "E11", 11),
    (SatId::E12, "E12", 12),
    (SatId::E13, "E13", 13),
    (SatId::E14, "E14", 14),
    (SatId::E15, "E15", 15),
    (SatId::E18, "E18", 18),
    (SatId::E19, "E19", 19),
    (SatId::E21, "E21", 21),
    (SatId::E22, "E22", 22),
    (SatId::E24, "E24", 24),
    (SatId::E25, "E25", 25),
    (SatId::E26, "E26", 26),
    (SatId::E27, "E27", 27),
    (SatId::E30, "E30", 30),
    (SatId::E31, "E31", 31),
    (SatId::E33, "E33", 33),
    (SatId::E34, "E34", 34),
    (SatId::E36, "E36", 36),
];

#[test]
fn test_as_str_and_as_num_for_all_variants() {
    for (sat, code, num) in ALL {
        assert_eq!(sat.as_str(), *code);
        assert_eq!(sat.as_num(), *num);
    }
}

#[test]
fn test_try_from_plain_code() {
    for (sat, code, _) in ALL {
        assert_eq!(SatId::try_from(*code).unwrap(), *sat);
    }
}

#[test]
fn test_try_from_compact_p_prefixed_code() {
    // The SP3 compact form "PExx" must resolve to the same satellite as "Exx".
    for (sat, code, _) in ALL {
        let compact = format!("P{code}");
        assert_eq!(SatId::try_from(compact.as_str()).unwrap(), *sat);
    }
}

#[test]
fn test_try_from_unknown_is_err() {
    let err = SatId::try_from("Z99").unwrap_err();
    assert!(err.0.contains("Unknown Satellite"));
    assert!(err.0.contains("Z99"));
}

#[test]
fn test_try_from_empty_is_err() {
    assert!(SatId::try_from("").is_err());
}

#[test]
fn test_display() {
    assert_eq!(format!("{}", SatId::E14), "Galileo/E14");
    assert_eq!(format!("{}", SatId::E01), "Galileo/E01");
}

#[test]
fn test_debug_clone_copy_eq() {
    let a = SatId::E18;
    let b = a; // Copy
    let c = Clone::clone(&a); // exercises the derived Clone on a Copy type
    assert_eq!(a, b);
    assert_eq!(a, c);
    assert_ne!(SatId::E18, SatId::E19);
    assert_eq!(format!("{a:?}"), "E18");
}

#[test]
fn test_ord_and_hash() {
    // Ord: ascending declaration order.
    assert!(SatId::E01 < SatId::E14);
    assert!(SatId::E14 < SatId::E36);

    let mut bset = BTreeSet::new();
    bset.insert(SatId::E36);
    bset.insert(SatId::E01);
    assert_eq!(bset.iter().next(), Some(&SatId::E01));

    let mut hset = HashSet::new();
    assert!(hset.insert(SatId::E14));
    assert!(!hset.insert(SatId::E14));
}
