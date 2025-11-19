/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// Defines the metric signature of the Clifford Algebra Cl(p, q, r).
///
/// The metric determines the squaring behavior of the basis vectors $e_i$:
/// * $e_i^2 = +1$
/// * $e_i^2 = -1$
/// * $e_i^2 = 0$ (degenerate)
///
/// The dimension $N = p + q + r$ is the total number of basis vectors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Metric {
    /// All basis vectors square to +1.
    /// Signature: (N, 0, 0)
    Euclidean(usize),

    /// All basis vectors square to -1.
    /// Signature: (0, N, 0)
    NonEuclidean(usize),

    /// Standard Relativistic Spacetime.
    /// Convention: e0^2 = +1, all others -1.
    /// Signature: (1, N-1, 0)
    Minkowski(usize),

    /// Projective Geometric Algebra (PGA).
    /// Convention: e0^2 = 0 (degenerate), others +1.
    /// Signature: (N-1, 0, 1) where the *first* vector is the zero vector.
    PGA(usize),

    /// Explicit generic signature Cl(p, q, r).
    /// Order of generators assumed:
    /// First p are (+), next q are (-), last r are (0).
    Generic { p: usize, q: usize, r: usize },

    /// Fully arbitrary signature defined by bitmasks.
    /// dim: Total dimension
    /// neg_mask: bit is 1 if e_i^2 = -1
    /// zero_mask: bit is 1 if e_i^2 = 0
    /// (If both bits are 0, default is +1)
    Custom {
        dim: usize,
        neg_mask: u64,
        zero_mask: u64,
    },
}

impl Metric {
    /// Returns the total dimension of the vector space (N = P + Q + R)
    pub fn dimension(&self) -> usize {
        match self {
            Metric::Euclidean(d)
            | Metric::NonEuclidean(d)
            | Metric::Minkowski(d)
            | Metric::PGA(d) => *d,
            Metric::Generic { p, q, r } => p + q + r,
            Metric::Custom { dim, .. } => *dim,
        }
    }

    /// Returns the value of the basis vector squared: $e_i^2$.
    /// Possible return values: 1, -1, or 0.
    ///
    /// # Arguments
    /// * `i` - The 0-indexed generator index (0..N-1).
    pub fn sign_of_sq(&self, i: usize) -> i32 {
        match self {
            Metric::Euclidean(_) => 1,

            Metric::NonEuclidean(_) => -1,

            // Minkowski: e0 is Time (+), e1..eN are Space (-)
            Metric::Minkowski(_) => {
                if i == 0 {
                    1
                } else {
                    -1
                }
            }

            // PGA: e0 is Origin/Horizon (0), e1..eN are Euclidean (+)
            Metric::PGA(_) => {
                if i == 0 {
                    0
                } else {
                    1
                }
            }

            Metric::Generic { p, q, r: _ } => {
                if i < *p {
                    1
                } else if i < p + *q {
                    -1
                } else {
                    0 // The last r dimensions are degenerate
                }
            }

            Metric::Custom {
                dim: _,
                neg_mask,
                zero_mask,
            } => {
                let is_zero = (zero_mask >> i) & 1 == 1;
                if is_zero {
                    return 0;
                }

                let is_neg = (neg_mask >> i) & 1 == 1;
                if is_neg {
                    return -1;
                }

                1 // Default to positive
            }
        }
    }

    /// Merges two metrics during a Tensor Product (Monad bind).
    /// e.g., Euclidean(2) + Euclidean(2) -> Euclidean(4)
    pub fn tensor_product(&self, other: &Self) -> Self {
        use Metric::*;
        let dim_a = self.dimension();
        let dim_b = other.dimension();

        match (self, other) {
            (Euclidean(a), Euclidean(b)) => Euclidean(a + b),
            (NonEuclidean(a), NonEuclidean(b)) => NonEuclidean(a + b),
            // Mixing signatures or using Minkowski defaults to a Generic construction
            // where we append the B dimensions after A.
            _ => {
                // Returning Euclidean sum as a safe default for "Generic" size growth:
                Euclidean(dim_a + dim_b)
            }
        }
    }
}
use std::fmt;

impl fmt::Display for Metric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Metric::Euclidean(d) => write!(f, "Euclidean({})", d),
            Metric::NonEuclidean(d) => write!(f, "NonEuclidean({})", d),
            Metric::Minkowski(d) => write!(f, "Minkowski({})", d),
            Metric::PGA(d) => write!(f, "PGA({})", d),
            Metric::Generic { p, q, r } => write!(f, "Generic({}, {}, {})", p, q, r),
            Metric::Custom { dim, .. } => write!(f, "Custom({})", dim),
        }
    }
}
