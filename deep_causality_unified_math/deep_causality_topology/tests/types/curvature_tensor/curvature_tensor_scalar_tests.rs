/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `CurvatureTensor` at every shipped real field.
//!
//! The other curvature tests are written at `f64` only, which cannot see an element bound that
//! excludes a scalar: `f32` is the one shipped scalar with no `From<f64>` impl, so a bound reading
//! `From<f64> + Into<f64>` would drop it silently. These cases run the same tensor at `f32`, `f64`
//! and `Float106`, so a bound admitting only some of them fails to compile here.
//!
//! The tensor is deliberately not flat. A flat tensor makes every quantity below zero, and a zero
//! is satisfied by a wrong constant as readily as by a right one.

use deep_causality_metric::Metric;
use deep_causality_num::{Float, Float106, lift};
use deep_causality_topology::{CurvatureSymmetry, CurvatureTensor};

macro_rules! curvature_tensor_scalar_tests {
    ($ty:ty, $name:ident) => {
        mod $name {
            use super::*;

            /// A 2D tensor with one independent non-zero component, `R^0_101 = 1`, which is the
            /// shape of a constant-curvature surface.
            fn curved() -> CurvatureTensor<$ty> {
                CurvatureTensor::<$ty>::from_generator(
                    2,
                    Metric::Euclidean(2),
                    CurvatureSymmetry::Riemann,
                    |d, a, b, c| {
                        if (d, a, b, c) == (0, 1, 0, 1) {
                            lift::<$ty>(1.0)
                        } else if (d, a, b, c) == (0, 1, 1, 0) {
                            lift::<$ty>(-1.0)
                        } else {
                            lift::<$ty>(0.0)
                        }
                    },
                )
            }

            #[test]
            fn flat_tensor_is_flat() {
                let flat = CurvatureTensor::<$ty>::flat(4);
                assert!(flat.is_flat());
                assert_eq!(flat.dim(), 4);
                assert_eq!(flat.ricci_scalar(), lift::<$ty>(0.0));
            }

            #[test]
            fn curved_tensor_is_not_flat() {
                // The negative control for `flat_tensor_is_flat`: `is_flat` compares against the
                // working type's epsilon, and a threshold that swallowed everything would pass
                // the flat case alone.
                assert!(!curved().is_flat());
                assert_eq!(curved().get(0, 1, 0, 1), lift::<$ty>(1.0));
            }

            #[test]
            fn ricci_contraction_is_reachable() {
                let ricci = curved().ricci_tensor();
                assert_eq!(ricci.len(), 4);
                // R_11 = R^0_101 = 1, so the contraction is pinned somewhere it does not vanish.
                assert_eq!(ricci[3], lift::<$ty>(1.0));
            }

            #[test]
            fn einstein_and_weyl_are_reachable() {
                let t = curved();
                assert_eq!(t.einstein_tensor().len(), 4);
                assert_eq!(t.weyl_tensor().len(), 16);
                assert_eq!(
                    t.contract(
                        &[lift::<$ty>(1.0), lift::<$ty>(0.0)],
                        &[lift::<$ty>(0.0), lift::<$ty>(1.0)],
                        &[lift::<$ty>(1.0), lift::<$ty>(0.0)],
                    )
                    .len(),
                    2
                );
            }

            #[test]
            fn kretschmann_and_bianchi_are_reachable() {
                let t = curved();
                assert!(Float::is_finite(t.kretschmann_scalar()));
                assert!(Float::is_finite(t.check_bianchi_identity()));
            }
        }
    };
}

curvature_tensor_scalar_tests!(f32, at_f32);
curvature_tensor_scalar_tests!(f64, at_f64);
curvature_tensor_scalar_tests!(Float106, at_float106);
