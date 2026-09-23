/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context_store::VerticalDatum;
use std::collections::HashSet;

/// Every member, paired with the name `Display` owes it. The names are the identifiers the
/// geodetic literature uses, so a rendering that abbreviates or renumbers one names a different
/// reference surface than the value does.
const DATUMS: [(VerticalDatum, &str); 5] = [
    (VerticalDatum::WGS84, "WGS84"),
    (VerticalDatum::EGM96, "EGM96"),
    (VerticalDatum::EGM2008, "EGM2008"),
    (VerticalDatum::ISA, "ISA"),
    (VerticalDatum::Terrain, "Terrain"),
];

#[test]
fn test_display_names_every_member() {
    for (datum, expected) in DATUMS {
        assert_eq!(format!("{}", datum), expected, "Display of {datum:?}");
    }
}

#[test]
fn test_display_never_prints_one_datum_as_another() {
    // An altitude is a number and a reference, so two datums that print alike make a reading
    // unreadable. EGM96 and EGM2008 are the pair most easily collapsed.
    let rendered: HashSet<String> = DATUMS.iter().map(|(d, _)| format!("{}", d)).collect();
    assert_eq!(rendered.len(), DATUMS.len(), "collision among {rendered:?}");
}

#[test]
fn test_default_is_wgs84() {
    // WGS84 is what a GNSS receiver reports before any geoid model is applied, so it is the
    // reference a caller who names none is handed. Moving `#[default]` to another member would
    // silently reinterpret every altitude built through `Default`.
    assert_eq!(VerticalDatum::default(), VerticalDatum::WGS84);
    assert_eq!(VerticalDatum::default().to_string(), "WGS84");
}

#[test]
fn test_the_table_covers_every_member() {
    // An exhaustive match fails to compile when a member is added, which is what keeps DATUMS
    // from silently falling behind the enum and quietly shrinking every test below.
    for (datum, _) in DATUMS {
        match datum {
            VerticalDatum::WGS84
            | VerticalDatum::EGM96
            | VerticalDatum::EGM2008
            | VerticalDatum::ISA
            | VerticalDatum::Terrain => {}
        }
    }

    let distinct: HashSet<VerticalDatum> = DATUMS.iter().map(|(d, _)| *d).collect();
    assert_eq!(
        distinct.len(),
        5,
        "DATUMS lost a member or gained a duplicate"
    );
}

#[test]
fn test_members_compare_and_hash_as_distinct_values() {
    // The datum is stored beside the altitude and compared before two altitudes are combined, so
    // equality has to separate all five.
    let distinct: HashSet<VerticalDatum> = DATUMS.iter().map(|(d, _)| *d).collect();
    assert_eq!(distinct.len(), DATUMS.len());

    assert_ne!(VerticalDatum::EGM96, VerticalDatum::EGM2008);
    assert_ne!(VerticalDatum::WGS84, VerticalDatum::EGM96);
    assert_ne!(VerticalDatum::ISA, VerticalDatum::Terrain);
}

#[test]
fn test_debug_and_display_agree() {
    // `Display` is written as the debug rendering, so the two must not drift apart.
    for (datum, expected) in DATUMS {
        assert_eq!(format!("{datum:?}"), expected);
        assert_eq!(format!("{datum}"), format!("{datum:?}"));
    }
}
