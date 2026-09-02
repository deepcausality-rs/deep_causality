/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec::Vec;
use deep_causality_algebra::RealField;

/// What one [`Check`] examined: the identifier the decision form is generic over.
///
/// A pair of graph nodes, an eigenvalue index, and a single whole-operator residual all fit one
/// record, which is what lets a pipeline fold reports from different checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CheckItem {
    /// One quantity over the whole operator, such as a Hermiticity defect or a trace defect.
    Whole,
    /// An indexed item, such as an eigenvalue or a generator.
    Index(usize),
    /// A pair, such as two graph nodes or two hypotheses.
    Pair(usize, usize),
}

/// One decision: a measured quantity against a threshold, with the margin and the verdict.
///
/// The shape is `CommutatorCheck`'s. A margin at or below one accepts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Check<R> {
    /// What was examined.
    pub item: CheckItem,
    /// The quantity measured.
    pub measured: R,
    /// The threshold it was compared against.
    pub threshold: R,
    /// `measured / threshold`, under the zero-threshold convention below.
    pub margin: R,
    /// Whether the item passed.
    pub accepted: bool,
}

impl<R: RealField> Check<R> {
    /// A check from its measurement and threshold, deciding the verdict and the margin.
    ///
    /// The zero-threshold convention follows `quantum_markov_check`: with a zero threshold the
    /// margin is the measured quantity itself when that is positive, and zero when both are zero,
    /// so a positive measurement against a zero threshold rejects and a zero one accepts.
    pub fn new(item: CheckItem, measured: R, threshold: R) -> Self {
        let accepted = measured <= threshold;
        let margin = if threshold > R::zero() {
            measured / threshold
        } else if measured > R::zero() {
            measured
        } else {
            R::zero()
        };
        Self {
            item,
            measured,
            threshold,
            margin,
            accepted,
        }
    }
}

/// Where the factors a Markov report certifies came from.
///
/// A re-check after composition or marginalisation runs on factors inherited from the parts. A
/// failure there is a failure of the certificate, not of the model: the composite may be Markov
/// under the induced factor assignment, which v1 does not construct. The provenance is what lets
/// the error variant say which of the two happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Factorization {
    /// The factors under test are the model's own.
    #[default]
    Rederived,
    /// The factors were carried forward from the parts of a composite.
    Inherited,
}

/// The three states a report can be in. `Vacuous` is an acceptance that examined nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CheckVerdict {
    /// Every examined item passed, and at least one was examined.
    Accepted,
    /// Nothing was examined, so nothing was certified.
    Vacuous,
    /// At least one item failed.
    Rejected,
}

/// The records of one decision, with the count of what it examined.
///
/// `examined` is carried beside the records rather than derived from them, because the two can
/// differ: a trace-preservation check examines every entry of a `d × d` residual and records one
/// defect. Where they agree, as for the Markov pairs, [`from_checks`](Self::from_checks) sets the
/// count from the records.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckReport<R> {
    checks: Vec<Check<R>>,
    examined: usize,
    factorization: Factorization,
}

impl<R: RealField> CheckReport<R> {
    /// A report from its records and the count of items examined.
    pub fn new(checks: Vec<Check<R>>, examined: usize) -> Self {
        Self {
            checks,
            examined,
            factorization: Factorization::Rederived,
        }
    }

    /// A report whose examined count is its record count.
    pub fn from_checks(checks: Vec<Check<R>>) -> Self {
        let examined = checks.len();
        Self::new(checks, examined)
    }

    /// The report of a check that examined nothing.
    pub fn vacuous() -> Self {
        Self::new(Vec::new(), 0)
    }

    /// The same report, marked with where its factors came from.
    pub fn with_factorization(mut self, factorization: Factorization) -> Self {
        self.factorization = factorization;
        self
    }

    /// The records, in the order they were examined.
    pub fn checks(&self) -> &[Check<R>] {
        &self.checks
    }

    /// How many items were examined.
    pub fn examined(&self) -> usize {
        self.examined
    }

    /// Where the factors this report certifies came from.
    pub fn factorization(&self) -> Factorization {
        self.factorization
    }

    /// Whether nothing was examined.
    pub fn is_vacuous(&self) -> bool {
        self.examined == 0
    }

    /// The record with the largest margin: the item closest to rejecting, or the one that did.
    pub fn worst(&self) -> Option<&Check<R>> {
        self.checks
            .iter()
            .fold(None, |acc: Option<&Check<R>>, c| match acc {
                Some(a) if a.margin >= c.margin => Some(a),
                _ => Some(c),
            })
    }

    /// The largest margin, or `None` if nothing was recorded.
    pub fn worst_margin(&self) -> Option<R> {
        self.worst().map(|c| c.margin)
    }

    /// The first record that rejected, if any.
    pub fn first_rejection(&self) -> Option<&Check<R>> {
        self.checks.iter().find(|c| !c.accepted)
    }

    /// Whether every recorded item passed. True of a vacuous report, which is why
    /// [`verdict`](Self::verdict) exists: this is the boolean derivable from the report, and the
    /// derivation runs one way.
    pub fn accepted(&self) -> bool {
        self.checks.iter().all(|c| c.accepted)
    }

    /// The verdict, with vacuity visible as its own state.
    pub fn verdict(&self) -> CheckVerdict {
        if !self.accepted() {
            CheckVerdict::Rejected
        } else if self.is_vacuous() {
            CheckVerdict::Vacuous
        } else {
            CheckVerdict::Accepted
        }
    }

    /// Two reports as one: the counts add, the records concatenate, and a vacuous member
    /// contributes no margin because it contributed no record. An inherited provenance on either
    /// side is inherited on the whole.
    pub fn fold(mut self, other: Self) -> Self {
        self.checks.extend(other.checks);
        self.examined += other.examined;
        if other.factorization == Factorization::Inherited {
            self.factorization = Factorization::Inherited;
        }
        self
    }
}
