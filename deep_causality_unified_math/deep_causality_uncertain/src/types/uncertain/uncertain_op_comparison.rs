/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::uncertain::uncertain_op_arithmetic::binary;
use crate::{ArithmeticOperator, ComparisonOperator, Node, Uncertain, UncertainBool};
use deep_causality_rand::RandScalar;

// Note: We do not implement the standard `PartialOrd` and `PartialEq` traits because their
// signatures return `bool`, which is misleading for uncertain values. Instead, we provide methods
// that return an `UncertainBool<R>` — the Boolean carrier over the same graph and the same scalar.
//
// Every threshold is `R`. It has to be: the operand is drawn at `R`, and a threshold at some other
// precision could only be compared against it by converting one of the two, which is a narrowing
// the caller did not ask for.
impl<R: RandScalar> Uncertain<R> {
    /// Whether each draw exceeds `threshold`.
    pub fn greater_than(&self, threshold: R) -> UncertainBool<R> {
        self.compare(ComparisonOperator::GreaterThan, threshold)
    }

    /// Whether each draw falls below `threshold`.
    pub fn less_than(&self, threshold: R) -> UncertainBool<R> {
        self.compare(ComparisonOperator::LessThan, threshold)
    }

    /// Whether each draw equals `threshold`, to the scalar's own epsilon.
    pub fn equals(&self, threshold: R) -> UncertainBool<R> {
        self.compare(ComparisonOperator::EqualTo, threshold)
    }

    /// Whether this value's draw exceeds the other's, at the same index.
    pub fn gt_uncertain(&self, other: &Self) -> UncertainBool<R> {
        self.compare_to(ComparisonOperator::GreaterThan, other)
    }

    /// Whether this value's draw falls below the other's, at the same index.
    pub fn lt_uncertain(&self, other: &Self) -> UncertainBool<R> {
        self.compare_to(ComparisonOperator::LessThan, other)
    }

    /// Whether this value's draw equals the other's, at the same index.
    pub fn eq_uncertain(&self, other: &Self) -> UncertainBool<R> {
        self.compare_to(ComparisonOperator::EqualTo, other)
    }

    /// Whether each draw is within `tolerance` of `target`.
    ///
    /// The band `[target − tolerance, target + tolerance]`, so it inherits
    /// [`Self::within_range`]'s treatment of a `NaN` draw (excluded) and of a negative tolerance
    /// (an empty band, which admits nothing — not even the target).
    pub fn approx_eq(&self, target: R, tolerance: R) -> UncertainBool<R> {
        self.within_range(target - tolerance, target + tolerance)
    }

    /// Whether each draw lies in `[min, max]`.
    ///
    /// Built from two comparison nodes rather than a stored predicate, so the quasi-Monte-Carlo
    /// pre-pass can see through it and neither bound has to be captured in a closure. Both
    /// comparisons clone the same sub-graph, so the memo gives the operand one draw and the two
    /// bounds are tested against the same number.
    ///
    /// # Not the negation of the strict comparisons
    ///
    /// The two nodes are `>=` and `<=` **stated directly**, not `!(<)` and `!(>)`. Under IEEE a
    /// `NaN` draw makes every comparison false, so the negated form reports `true` for both halves
    /// and calls a `NaN` in range — which is the single worst answer available, since the caller
    /// asked precisely in order to exclude the values outside it. De Morgan holds in Boolean logic
    /// and not over a value unordered with respect to everything.
    ///
    /// A reversed range (`min > max`) contains nothing, which follows from the same two nodes with
    /// no special case.
    pub fn within_range(&self, min: R, max: R) -> UncertainBool<R> {
        self.compare(ComparisonOperator::GreaterThanOrEqual, min)
            & self.compare(ComparisonOperator::LessThanOrEqual, max)
    }

    /// One comparison node against a fixed threshold.
    fn compare(&self, op: ComparisonOperator, threshold: R) -> UncertainBool<R> {
        UncertainBool::from_root_node(Node::ComparisonOp {
            op,
            threshold,
            operand: self.root_node().clone(),
        })
    }

    /// Compares two values by the sign of their difference, so both are drawn at one index and the
    /// comparison is between two draws of the same sample rather than two independent ones.
    fn compare_to(&self, op: ComparisonOperator, other: &Self) -> UncertainBool<R> {
        let difference = binary(ArithmeticOperator::Sub, self.clone(), other.clone());
        UncertainBool::from_root_node(Node::ComparisonOp {
            op,
            threshold: R::zero(),
            operand: difference.into_tree(),
        })
    }
}
