/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Experiment design and adjudication: which experiments to run, and what the results decide.
//!
//! [`design`] answers "which experiments discriminate the surviving hypotheses at least cost" as
//! an exact minimum-cost set cover over the `C(n, 2)` hypothesis pairs, returning a
//! [`DesignPlan`] rather than one experiment because the crosstalk consumer's answer is a pair of
//! interventions. With an [`InstrumentTime`] it sizes each experiment's shots and prices them in
//! seconds; combining, it adds a pair's bits across experiments. [`adjudicate`] folds the verdicts the forked worlds came back with under the
//! verdict law: projection-valued verdicts are tested for commutation first, because
//! `Projection<R, D>` is orthomodular and distributivity fails outside the commuting family, and
//! read-outs against a real-valued spec are not, because a threshold on a real quantity is a
//! classical proposition and the guard would reject sound folds. The outcome is one surviving
//! hypothesis on one side of `Either`, or the residual ambiguity on the other, because that is a
//! coproduct and `Either` is the coproduct. [`adjudicate_campaign`] folds worlds that read several
//! experiments, summing each pair's separation over them.

pub(crate) mod adjudicate;
pub(crate) mod cover_mode;
pub(crate) mod experiment_design;
pub(crate) mod instrument_time;

pub use adjudicate::*;
pub use cover_mode::*;
pub use experiment_design::*;
pub use instrument_time::*;
