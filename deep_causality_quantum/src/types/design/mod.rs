/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Experiment design and adjudication: which experiments to run, and what the results decide.
//!
//! [`design`] answers "which experiments discriminate the surviving hypotheses at least cost" as
//! an exact minimum-cost set cover over the `C(n, 2)` hypothesis pairs, returning a
//! [`DesignPlan`] rather than one experiment because the crosstalk consumer's answer is a pair of
//! interventions. [`adjudicate`] folds the verdicts the forked worlds came back with under the
//! verdict law: projection-valued verdicts are tested for commutation first, because
//! `Projection<R, D>` is orthomodular and distributivity fails outside the commuting family, and
//! read-outs against a real-valued spec are not, because a threshold on a real quantity is a
//! classical proposition and the guard would reject sound folds. The outcome is one surviving
//! hypothesis on one side of `Either`, or the residual ambiguity on the other, because that is a
//! coproduct and `Either` is the coproduct.

pub(crate) mod adjudicate;
pub(crate) mod experiment_design;

pub use adjudicate::*;
pub use experiment_design::*;
