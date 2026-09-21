/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_metric::{Metric, MetricError};
use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};

// =============================================================================
// dimension() tests
// =============================================================================

#[test]
fn test_euclidean_dimension() {
    assert_eq!(Metric::Euclidean(3).dimension(), 3);
    assert_eq!(Metric::Euclidean(10).dimension(), 10);
}

#[test]
fn test_non_euclidean_dimension() {
    assert_eq!(Metric::NonEuclidean(4).dimension(), 4);
}

#[test]
fn test_minkowski_dimension() {
    assert_eq!(Metric::Minkowski(4).dimension(), 4);
    assert_eq!(Metric::Minkowski(11).dimension(), 11);
}

#[test]
fn test_pga_dimension() {
    assert_eq!(Metric::PGA(4).dimension(), 4);
}

#[test]
fn test_generic_dimension() {
    let m = Metric::Generic { p: 3, q: 1, r: 0 };
    assert_eq!(m.dimension(), 4);
}

#[test]
fn test_custom_dimension() {
    let m = Metric::Custom {
        dim: 5,
        neg_mask: 0b11,
        zero_mask: 0,
    };
    assert_eq!(m.dimension(), 5);
}

// =============================================================================
// sign_of_sq() tests
// =============================================================================

#[test]
fn test_euclidean_signs() {
    let m = Metric::Euclidean(4);
    for i in 0..4 {
        assert_eq!(m.sign_of_sq(i), 1);
    }
}

#[test]
fn test_non_euclidean_signs() {
    let m = Metric::NonEuclidean(4);
    for i in 0..4 {
        assert_eq!(m.sign_of_sq(i), -1);
    }
}

#[test]
fn test_minkowski_signs() {
    // West Coast: (+---)
    let m = Metric::Minkowski(4);
    assert_eq!(m.sign_of_sq(0), 1); // time
    assert_eq!(m.sign_of_sq(1), -1); // space
    assert_eq!(m.sign_of_sq(2), -1);
    assert_eq!(m.sign_of_sq(3), -1);
}

#[test]
fn test_pga_signs() {
    // e0 = 0, others = +1
    let m = Metric::PGA(4);
    assert_eq!(m.sign_of_sq(0), 0);
    assert_eq!(m.sign_of_sq(1), 1);
    assert_eq!(m.sign_of_sq(2), 1);
    assert_eq!(m.sign_of_sq(3), 1);
}

#[test]
fn test_generic_signs() {
    // Cl(2, 1, 1): first 2 are +1, next 1 is -1, last 1 is 0
    let m = Metric::Generic { p: 2, q: 1, r: 1 };
    assert_eq!(m.sign_of_sq(0), 1);
    assert_eq!(m.sign_of_sq(1), 1);
    assert_eq!(m.sign_of_sq(2), -1);
    assert_eq!(m.sign_of_sq(3), 0);
}

#[test]
fn test_custom_signs() {
    // Custom: dim=4, neg_mask=0b0001 (-+++), zero_mask=0
    let m = Metric::Custom {
        dim: 4,
        neg_mask: 0b0001,
        zero_mask: 0,
    };
    assert_eq!(m.sign_of_sq(0), -1);
    assert_eq!(m.sign_of_sq(1), 1);
    assert_eq!(m.sign_of_sq(2), 1);
    assert_eq!(m.sign_of_sq(3), 1);
}

#[test]
fn test_custom_with_zero() {
    // Custom with degenerate dimension
    let m = Metric::Custom {
        dim: 4,
        neg_mask: 0b0010,
        zero_mask: 0b0001,
    };
    assert_eq!(m.sign_of_sq(0), 0); // zero_mask bit 0
    assert_eq!(m.sign_of_sq(1), -1); // neg_mask bit 1
    assert_eq!(m.sign_of_sq(2), 1);
    assert_eq!(m.sign_of_sq(3), 1);
}

// =============================================================================
// signature() tests
// =============================================================================

#[test]
fn test_euclidean_signature() {
    assert_eq!(Metric::Euclidean(4).signature(), (4, 0, 0));
}

#[test]
fn test_non_euclidean_signature() {
    assert_eq!(Metric::NonEuclidean(3).signature(), (0, 3, 0));
}

#[test]
fn test_minkowski_signature() {
    // Minkowski(4) is (+---) = (1, 3, 0)
    assert_eq!(Metric::Minkowski(4).signature(), (1, 3, 0));
}

#[test]
fn test_pga_signature() {
    // PGA(4) = (3, 0, 1)
    assert_eq!(Metric::PGA(4).signature(), (3, 0, 1));
}

#[test]
fn test_generic_signature() {
    let m = Metric::Generic { p: 2, q: 3, r: 1 };
    assert_eq!(m.signature(), (2, 3, 1));
}

#[test]
fn test_custom_signature() {
    // Custom: (-+++) = (3, 1, 0)
    let m = Metric::Custom {
        dim: 4,
        neg_mask: 0b0001,
        zero_mask: 0,
    };
    assert_eq!(m.signature(), (3, 1, 0));
}

// =============================================================================
// flip_time_space() tests
// =============================================================================

#[test]
fn test_flip_minkowski() {
    // Minkowski (+---) -> (-+++)
    let m = Metric::Minkowski(4);
    let flipped = m.flip_time_space();

    assert_eq!(flipped.sign_of_sq(0), -1);
    assert_eq!(flipped.sign_of_sq(1), 1);
    assert_eq!(flipped.sign_of_sq(2), 1);
    assert_eq!(flipped.sign_of_sq(3), 1);
}

#[test]
fn test_flip_roundtrip() {
    // flip(flip(m)) should have same signature
    let m = Metric::Minkowski(4);
    let double_flipped = m.flip_time_space().flip_time_space();

    assert_eq!(m.signature(), double_flipped.signature());
}

#[test]
fn test_flip_euclidean() {
    // Euclidean flipped becomes all negative
    let m = Metric::Euclidean(3);
    let flipped = m.flip_time_space();

    for i in 0..3 {
        assert_eq!(flipped.sign_of_sq(i), -1);
    }
}

// =============================================================================
// tensor_product() tests
// =============================================================================

#[test]
fn test_tensor_product_euclidean() {
    let m1 = Metric::Euclidean(2);
    let m2 = Metric::Euclidean(3);
    let product = m1.tensor_product(&m2);

    assert_eq!(product, Metric::Euclidean(5));
}

#[test]
fn test_tensor_product_non_euclidean() {
    let m1 = Metric::NonEuclidean(2);
    let m2 = Metric::NonEuclidean(2);
    let product = m1.tensor_product(&m2);

    assert_eq!(product, Metric::NonEuclidean(4));
}

#[test]
fn test_tensor_product_mixed() {
    let m1 = Metric::Euclidean(2);
    let m2 = Metric::Minkowski(2);
    let product = m1.tensor_product(&m2);

    // Euclidean(2) = (2, 0, 0) + Minkowski(2) = (1, 1, 0) = (3, 1, 0)
    assert_eq!(product.signature(), (3, 1, 0));
}

// =============================================================================
// is_compatible() tests
// =============================================================================

#[test]
fn test_compatible_same() {
    let m1 = Metric::Minkowski(4);
    let m2 = Metric::Minkowski(4);
    assert!(m1.is_compatible(&m2));
}

#[test]
fn test_compatible_different_repr() {
    // Same signature, different representation
    let m1 = Metric::Minkowski(4);
    let m2 = Metric::Generic { p: 1, q: 3, r: 0 };
    assert!(m1.is_compatible(&m2));
}

#[test]
fn test_not_compatible_different_dim() {
    let m1 = Metric::Euclidean(3);
    let m2 = Metric::Euclidean(4);
    assert!(!m1.is_compatible(&m2));
}

#[test]
fn test_not_compatible_different_signature() {
    let m1 = Metric::Euclidean(4);
    let m2 = Metric::Minkowski(4);
    assert!(!m1.is_compatible(&m2));
}

// =============================================================================
// to_generic() tests
// =============================================================================

#[test]
fn test_to_generic_euclidean() {
    let m = Metric::Euclidean(3).to_generic();
    assert_eq!(m, Metric::Generic { p: 3, q: 0, r: 0 });
}

#[test]
fn test_to_generic_minkowski() {
    let m = Metric::Minkowski(4).to_generic();
    assert_eq!(m, Metric::Generic { p: 1, q: 3, r: 0 });
}

// =============================================================================
// from_signature() tests
// =============================================================================

#[test]
fn test_from_signature_euclidean() {
    let m = Metric::from_signature(4, 0, 0);
    assert_eq!(m, Metric::Euclidean(4));
}

#[test]
fn test_from_signature_non_euclidean() {
    let m = Metric::from_signature(0, 3, 0);
    assert_eq!(m, Metric::NonEuclidean(3));
}

#[test]
fn test_from_signature_minkowski() {
    let m = Metric::from_signature(1, 3, 0);
    assert_eq!(m, Metric::Minkowski(4));
}

#[test]
fn test_from_signature_pga() {
    let m = Metric::from_signature(3, 0, 1);
    assert_eq!(m, Metric::PGA(4));
}

#[test]
fn test_from_signature_generic() {
    let m = Metric::from_signature(2, 2, 1);
    assert_eq!(m, Metric::Generic { p: 2, q: 2, r: 1 });
}

// =============================================================================
// from_signs() and to_signs() tests
// =============================================================================

#[test]
fn test_from_signs_basic() {
    let signs = [1, -1, -1, -1];
    let m = Metric::from_signs(&signs).unwrap();

    assert_eq!(m.dimension(), 4);
    assert_eq!(m.sign_of_sq(0), 1);
    assert_eq!(m.sign_of_sq(1), -1);
}

#[test]
fn test_from_signs_with_zero() {
    let signs = [0, 1, 1, 1];
    let m = Metric::from_signs(&signs).unwrap();

    assert_eq!(m.sign_of_sq(0), 0);
    assert_eq!(m.sign_of_sq(1), 1);
}

#[test]
fn test_from_signs_empty() {
    let signs: [i32; 0] = [];
    let result = Metric::from_signs(&signs);
    assert!(result.is_err());
}

#[test]
fn test_from_signs_invalid() {
    let signs = [1, 2, -1]; // 2 is invalid
    let result = Metric::from_signs(&signs);
    assert!(result.is_err());
}

#[test]
fn test_to_signs_roundtrip() {
    let original = Metric::Minkowski(4);
    let signs = original.to_signs();
    let reconstructed = Metric::from_signs(&signs).unwrap();

    assert_eq!(original.signature(), reconstructed.signature());
}

// =============================================================================
// Display tests
// =============================================================================

#[test]
fn test_display_euclidean() {
    let m = Metric::Euclidean(3);
    assert_eq!(format!("{}", m), "Euclidean(3)");
}

#[test]
fn test_display_minkowski() {
    let m = Metric::Minkowski(4);
    assert_eq!(format!("{}", m), "Minkowski(4)");
}

#[test]
fn test_display_generic() {
    let m = Metric::Generic { p: 2, q: 1, r: 1 };
    assert_eq!(format!("{}", m), "Cl(2, 1, 1)");
}

// =============================================================================
// Hash and Eq tests
// =============================================================================

#[test]
fn test_metric_hash() {
    let mut set = HashSet::new();
    set.insert(Metric::Euclidean(3));
    set.insert(Metric::Euclidean(3)); // duplicate
    set.insert(Metric::Minkowski(4));

    assert_eq!(set.len(), 2);
}

#[test]
fn test_metric_eq() {
    assert_eq!(Metric::Euclidean(3), Metric::Euclidean(3));
    assert_ne!(Metric::Euclidean(3), Metric::Euclidean(4));
    assert_ne!(Metric::Euclidean(4), Metric::Minkowski(4));
}

// =============================================================================
// Lorentzian tests
// =============================================================================

#[test]
fn test_lorentzian_dimension() {
    assert_eq!(Metric::Lorentzian(4).dimension(), 4);
}

#[test]
fn test_lorentzian_signs() {
    // East Coast: (-+++)
    let m = Metric::Lorentzian(4);
    assert_eq!(m.sign_of_sq(0), -1); // time
    assert_eq!(m.sign_of_sq(1), 1); // space
    assert_eq!(m.sign_of_sq(2), 1);
    assert_eq!(m.sign_of_sq(3), 1);
}

#[test]
fn test_lorentzian_signature() {
    // Lorentzian(4) is (-+++) = (3, 1, 0)
    // p=3 (+1s), q=1 (-1s), r=0
    assert_eq!(Metric::Lorentzian(4).signature(), (3, 1, 0));
}

#[test]
fn test_from_signature_lorentzian() {
    // p=3, q=1, r=0 -> Lorentzian(4)
    let m = Metric::from_signature(3, 1, 0);
    assert_eq!(m, Metric::Lorentzian(4));
}

#[test]
fn test_display_lorentzian() {
    let m = Metric::Lorentzian(4);
    assert_eq!(format!("{}", m), "Lorentzian(4)");
}

#[test]
fn test_flip_minkowski_to_lorentzian() {
    // Minkowski (+---) -> Lorentzian (-+++)
    let m = Metric::Minkowski(4);
    let flipped = m.flip_time_space();

    // Note: flip_time_space currently generic Custom,
    // but we verify signs match Lorentzian
    let lorentzian = Metric::Lorentzian(4);

    assert_eq!(flipped.dimension(), lorentzian.dimension());
    assert_eq!(flipped.signature(), lorentzian.signature());

    for i in 0..4 {
        assert_eq!(flipped.sign_of_sq(i), lorentzian.sign_of_sq(i));
    }
}

// =============================================================================
// Display — every variant, and the variants stay apart
// =============================================================================

/// Every `Metric` variant and the string `Display` owes it.
///
/// The oracle is the documented rendering of each variant, written out here rather than derived
/// from the impl: `Cl(p, q, r)` for `Generic`, the variant name with its dimension otherwise, and
/// `Custom(dim)` with the masks suppressed.
const DISPLAY_TABLE: [(Metric, &str); 7] = [
    (Metric::Euclidean(3), "Euclidean(3)"),
    (Metric::NonEuclidean(3), "NonEuclidean(3)"),
    (Metric::Minkowski(4), "Minkowski(4)"),
    (Metric::Lorentzian(4), "Lorentzian(4)"),
    (Metric::PGA(3), "PGA(3)"),
    (Metric::Generic { p: 2, q: 1, r: 1 }, "Cl(2, 1, 1)"),
    (
        Metric::Custom {
            dim: 5,
            neg_mask: 0b0110,
            zero_mask: 0b1000,
        },
        "Custom(5)",
    ),
];

#[test]
fn test_display_every_variant() {
    for (metric, expected) in DISPLAY_TABLE {
        assert_eq!(format!("{}", metric), expected, "Display of {:?}", metric);
    }
}

#[test]
fn test_display_never_prints_one_variant_as_another() {
    // Same dimension across five variants: only the variant name separates them, so an arm
    // written against the wrong variant produces a duplicate here.
    let same_dimension = [
        Metric::Euclidean(4),
        Metric::NonEuclidean(4),
        Metric::Minkowski(4),
        Metric::Lorentzian(4),
        Metric::PGA(4),
    ];

    let rendered: HashSet<String> = same_dimension.iter().map(|m| format!("{}", m)).collect();
    assert_eq!(
        rendered.len(),
        same_dimension.len(),
        "two variants of dimension 4 print the same string: {:?}",
        rendered
    );
}

// =============================================================================
// Hash — the contract, and every field reaching the hasher
// =============================================================================

fn hash_of(metric: &Metric) -> u64 {
    let mut hasher = DefaultHasher::new();
    metric.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn test_hash_agrees_with_equality_for_every_variant() {
    // The `Hash` contract: `a == b` implies `hash(a) == hash(b)`. Checked on an independently
    // built copy of each variant so a hash reading uninitialised or identity state would differ.
    for (metric, _) in DISPLAY_TABLE {
        let copy = metric;
        assert_eq!(metric, copy);
        assert_eq!(hash_of(&metric), hash_of(&copy), "hash of {:?}", metric);
    }
}

#[test]
fn test_hash_separates_variants_carrying_the_same_payload() {
    // Five variants, one payload. Only the discriminant tells them apart, so dropping the
    // `discriminant(self).hash(state)` line collapses all five to one hash — which a `HashSet`
    // test cannot see, because equality still separates them.
    let same_payload = [
        Metric::Euclidean(3),
        Metric::NonEuclidean(3),
        Metric::Minkowski(3),
        Metric::Lorentzian(3),
        Metric::PGA(3),
    ];

    let hashes: HashSet<u64> = same_payload.iter().map(hash_of).collect();
    assert_eq!(
        hashes.len(),
        same_payload.len(),
        "variants of payload 3 collide: {:?}",
        hashes
    );
}

#[test]
fn test_hash_reads_every_generic_field() {
    // Each neighbour differs from the base in exactly one field, so a field never handed to the
    // hasher shows up as a collision here.
    let base = Metric::Generic { p: 2, q: 1, r: 0 };
    let neighbours = [
        Metric::Generic { p: 5, q: 1, r: 0 },
        Metric::Generic { p: 2, q: 7, r: 0 },
        Metric::Generic { p: 2, q: 1, r: 9 },
    ];

    for neighbour in neighbours {
        assert_ne!(
            hash_of(&base),
            hash_of(&neighbour),
            "{:?} hashes as {:?}",
            neighbour,
            base
        );
    }
}

#[test]
fn test_hash_reads_every_custom_field() {
    let base = Metric::Custom {
        dim: 4,
        neg_mask: 0b0010,
        zero_mask: 0b0001,
    };
    let neighbours = [
        Metric::Custom {
            dim: 6,
            neg_mask: 0b0010,
            zero_mask: 0b0001,
        },
        Metric::Custom {
            dim: 4,
            neg_mask: 0b0100,
            zero_mask: 0b0001,
        },
        Metric::Custom {
            dim: 4,
            neg_mask: 0b0010,
            zero_mask: 0b1000,
        },
    ];

    for neighbour in neighbours {
        assert_ne!(
            hash_of(&base),
            hash_of(&neighbour),
            "{:?} hashes as {:?}",
            neighbour,
            base
        );
    }
}

#[test]
fn test_hash_set_holds_one_entry_per_distinct_metric() {
    let mut set = HashSet::new();
    for (metric, _) in DISPLAY_TABLE {
        set.insert(metric);
        set.insert(metric); // inserted twice; equality must fold it to one
    }
    assert_eq!(set.len(), DISPLAY_TABLE.len());
}

// =============================================================================
// signature() — the degenerate count, and the identity p + q + r = N
// =============================================================================

/// Every variant, paired with the (p, q, r) counted by hand from its definition.
const SIGNATURE_TABLE: [(Metric, (usize, usize, usize)); 9] = [
    (Metric::Euclidean(3), (3, 0, 0)),
    (Metric::NonEuclidean(3), (0, 3, 0)),
    (Metric::Minkowski(4), (1, 3, 0)),
    (Metric::Lorentzian(4), (3, 1, 0)),
    (Metric::PGA(3), (2, 0, 1)),
    (Metric::Generic { p: 2, q: 1, r: 1 }, (2, 1, 1)),
    // signs, low bit first: +1, -1, 0, +1 -> p = 2, q = 1, r = 1
    (
        Metric::Custom {
            dim: 4,
            neg_mask: 0b0010,
            zero_mask: 0b0100,
        },
        (2, 1, 1),
    ),
    // Every generator degenerate.
    (
        Metric::Custom {
            dim: 3,
            neg_mask: 0,
            zero_mask: 0b111,
        },
        (0, 0, 3),
    ),
    // A mask bit above `dim` names no generator and must not be counted.
    (
        Metric::Custom {
            dim: 2,
            neg_mask: 0b1000,
            zero_mask: 0b0100,
        },
        (2, 0, 0),
    ),
];

#[test]
fn test_signature_counts_each_sign_class() {
    for (metric, expected) in SIGNATURE_TABLE {
        assert_eq!(metric.signature(), expected, "signature of {:?}", metric);
    }
}

#[test]
fn test_signature_components_sum_to_dimension() {
    // N = p + q + r is the definition of the dimension, so it holds for every variant whatever
    // the representation. A miscounted sign class breaks it without any expected tuple written.
    for (metric, _) in SIGNATURE_TABLE {
        let (p, q, r) = metric.signature();
        assert_eq!(p + q + r, metric.dimension(), "for {:?}", metric);
    }
}

// =============================================================================
// flip_time_space() — the swap identity, degeneracy, and the 64-bit boundary
// =============================================================================

#[test]
fn test_flip_time_space_swaps_p_and_q_and_keeps_r() {
    // Flipping every +1 to -1 and back exchanges the two definite counts and leaves the
    // degenerate count alone. Stated as an identity, so no expectation repeats the loop.
    for (metric, _) in SIGNATURE_TABLE {
        let (p, q, r) = metric.signature();
        assert_eq!(
            metric.flip_time_space().signature(),
            (q, p, r),
            "flip of {:?}",
            metric
        );
    }
}

#[test]
fn test_flip_pga_keeps_the_degenerate_generator_degenerate() {
    // PGA(4) is (0 + + +). Flipping leaves e0 null and turns the three spatial generators
    // negative, so bits 1..3 are set in the negative mask and bit 0 in the zero mask.
    let flipped = Metric::PGA(4).flip_time_space();
    assert_eq!(
        flipped,
        Metric::Custom {
            dim: 4,
            neg_mask: 0b1110,
            zero_mask: 0b0001,
        }
    );
    assert_eq!(flipped.sign_of_sq(0), 0);
    for i in 1..4 {
        assert_eq!(flipped.sign_of_sq(i), -1, "generator {}", i);
    }
}

#[test]
fn test_flip_time_space_at_and_above_bitmask_capacity() {
    // 64 generators is the last dimension a u64 mask can hold, so the result is still `Custom`.
    let at_capacity = Metric::Euclidean(64).flip_time_space();
    assert_eq!(
        at_capacity,
        Metric::Custom {
            dim: 64,
            neg_mask: u64::MAX,
            zero_mask: 0,
        },
        "the top generator must set bit 63 rather than overflow"
    );
    assert_eq!(at_capacity.sign_of_sq(63), -1);

    // One generator more and the mask cannot hold it, so the result falls back to `Generic`.
    let above_capacity = Metric::Euclidean(65).flip_time_space();
    assert_eq!(above_capacity, Metric::Generic { p: 0, q: 65, r: 0 });
    assert_eq!(above_capacity.signature(), (0, 65, 0));
}

// =============================================================================
// tensor_product() — the additivity identity and a mixed hand count
// =============================================================================

const TENSOR_PAIRS: [(Metric, Metric); 8] = [
    (Metric::Euclidean(2), Metric::Euclidean(3)),
    (Metric::NonEuclidean(2), Metric::NonEuclidean(3)),
    (Metric::Minkowski(4), Metric::Euclidean(2)),
    (Metric::Euclidean(2), Metric::Minkowski(4)),
    (Metric::PGA(3), Metric::Euclidean(2)),
    (Metric::Euclidean(2), Metric::PGA(3)),
    (Metric::Lorentzian(4), Metric::PGA(3)),
    (
        Metric::Generic { p: 1, q: 2, r: 3 },
        Metric::Generic { p: 3, q: 2, r: 1 },
    ),
];

#[test]
fn test_tensor_product_adds_the_signatures() {
    // A tensor product concatenates the generator lists, so each sign class of the result is the
    // sum of the two inputs' counts. The identity holds for every pairing, including the two
    // same-variant shortcuts, and no expectation restates the counting loop.
    for (a, b) in TENSOR_PAIRS {
        let (pa, qa, ra) = a.signature();
        let (pb, qb, rb) = b.signature();
        let product = a.tensor_product(&b);

        assert_eq!(
            product.signature(),
            (pa + pb, qa + qb, ra + rb),
            "{:?} (x) {:?}",
            a,
            b
        );
        assert_eq!(
            product.dimension(),
            a.dimension() + b.dimension(),
            "{:?} (x) {:?}",
            a,
            b
        );
    }
}

#[test]
fn test_tensor_product_of_lorentzian_and_pga() {
    // Lorentzian(4) is (- + + +) and PGA(3) is (0 + +). Counting the seven generators by hand:
    // five square to +1, one to -1, one to 0.
    let product = Metric::Lorentzian(4).tensor_product(&Metric::PGA(3));
    assert_eq!(product, Metric::Generic { p: 5, q: 1, r: 1 });
}

// =============================================================================
// from_signs() — the 64-bit boundary and the top generator
// =============================================================================

#[test]
fn test_from_signs_at_and_above_bitmask_capacity() {
    let at_capacity = Metric::from_signs(&[1i32; 64]).expect("64 generators fit a u64 mask");
    assert_eq!(at_capacity.dimension(), 64);
    assert_eq!(at_capacity.signature(), (64, 0, 0));

    let above_capacity = Metric::from_signs(&[1i32; 65]);
    assert_eq!(
        above_capacity,
        Err(MetricError::invalid_dimension(
            "dimension exceeds bitmask capacity (max 64)"
        ))
    );
}

#[test]
fn test_from_signs_sets_the_top_mask_bit() {
    // Index 63 is the highest bit a u64 mask has. A shift performed at a narrower width loses it.
    let mut signs = [1i32; 64];
    signs[63] = -1;

    let metric = Metric::from_signs(&signs).expect("64 generators fit a u64 mask");
    assert_eq!(
        metric,
        Metric::Custom {
            dim: 64,
            neg_mask: 1u64 << 63,
            zero_mask: 0,
        }
    );
    assert_eq!(metric.sign_of_sq(63), -1);
    assert_eq!(metric.sign_of_sq(62), 1);
    assert_eq!(metric.signature(), (63, 1, 0));
}
