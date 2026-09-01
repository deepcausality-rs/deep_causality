/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cochain::Cochain;
use crate::types::cup_product::cup_product;
use crate::{Topology, TopologyError, TopologyErrorEnum};
use core::fmt::Debug;
use core::ops::Mul;
use deep_causality_algebra::{Field, RealField};
use deep_causality_num::Zero;
use deep_causality_tensor::CausalTensor;
use std::sync::Arc;

impl<R, G> Topology<R, G>
where
    R: RealField,
    G: Field + Copy + Clone + Zero + Mul<Output = G> + Debug,
{
    /// Computes the cup product `α ⌣ β` of a `p`-cochain and a `q`-cochain,
    /// yielding a `(p+q)`-cochain.
    ///
    /// # This delegates
    ///
    /// The body is [`crate::cup_product`], which is generic over
    /// [`CellularComplex`](crate::CellularComplex) and covers the cubical case
    /// and the `n`-fold form as well. This method used to carry a second,
    /// simplicial-only Alexander-Whitney implementation, extracting the front
    /// and back faces by hand; the two were measured to agree bit-for-bit
    /// before it was retired, and `tests/types/cup_product/implementation_agreement_tests.rs`
    /// is the artefact of that measurement.
    ///
    /// What this method adds over the free function is the pair of things a
    /// free function taking one complex cannot express: it checks that both
    /// operands live on the *same* complex, and it rewraps the result as a
    /// [`Topology`] carrying that complex.
    ///
    /// # Arguments
    ///
    /// * `other`: the `q`-cochain `β`. `self` is the `p`-cochain `α`.
    ///
    /// # Errors
    ///
    /// [`TopologyErrorEnum::GenericError`] when the two operands are held
    /// against different complexes.
    ///
    /// Two error paths changed when this began delegating, and both are
    /// tightenings a caller should know about:
    ///
    /// * A grade sum past the complex's maximum dimension now returns
    ///   [`TopologyErrorEnum::InvalidGradeOperation`]. It used to return `Ok`
    ///   with a zero-filled cochain, which reported a cochain in a degree the
    ///   complex does not have as a successful computation.
    /// * A cochain whose length does not match its skeleton now returns
    ///   [`TopologyErrorEnum::DimensionMismatch`]. It used to panic through
    ///   `.expect("Data/Skeleton mismatch")`.
    pub fn cup_product(&self, other: &Topology<R, G>) -> Result<Topology<R, G>, TopologyError> {
        // The one precondition the free function cannot state, since it takes a
        // single complex and these two carry their own.
        if !Arc::ptr_eq(&self.complex, &other.complex) {
            return Err(TopologyError(TopologyErrorEnum::GenericError(
                "Complex Mismatch".to_string(),
            )));
        }

        let alpha = Cochain::from_values(self.data.as_slice(), self.grade);
        let beta = Cochain::from_values(other.data.as_slice(), other.grade);
        let product = cup_product(self.complex.as_ref(), &alpha, &beta)?;

        let grade = product.degree();
        let values = product.into_values();
        let len = values.len();
        Ok(Topology {
            complex: self.complex.clone(),
            grade,
            // `CausalTensor::new` fails only on a shape/length disagreement, and
            // the shape is this vector's own length.
            data: CausalTensor::new(values, vec![len])
                .expect("a rank-one tensor over its own length"),
            cursor: 0,
        })
    }
}
