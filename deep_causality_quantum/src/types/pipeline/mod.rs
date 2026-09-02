/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The pipeline: one origin for configuration, three subjects, and the stages in order.
//!
//! ```text
//! config(subject, candidates)  →  validate(checks)  →  Screened  →  control(evidence)  →  report
//! ```
//!
//! Every configuration comes from `QclBuilder::config::<FloatType, NumberType>()`, the one site
//! naming the two working types: the real one buys accuracy and every tolerance derives from it,
//! the count one buys headroom and moves no threshold. The config branches on the subject, a
//! plant, a model or a code, and `build()` refuses what would answer unsoundly before any stage
//! runs. `validate` terminates in a [`Screened`], and `control` takes either a plant config whose
//! candidates are mechanisms or a `Screened`, so a structural candidate has no path into `control`
//! that skips validation, and the compiler is what says so.
//!
//! The pipeline adds ordering and naming, not a monad. The categorical structure it threads is the
//! causal monad's `PropagatingEffect`, with [`Ledger`] as its state.
//!
//! The whole module is behind the `qcm` feature, because candidates are [`Hypothesis`] values and
//! the model subject reaches `deep_causality` for its graph. Bare-metal reach for the plant path
//! is a follow-up that splits the graph-dependent half of `Hypothesis` from the rest.

pub(crate) mod config;
pub(crate) mod control;
pub(crate) mod ledger;
pub(crate) mod spec;
pub(crate) mod validate;

pub use config::*;
pub use control::*;
pub use ledger::*;
pub use spec::*;
pub use validate::*;

#[allow(unused_imports)]
use crate::types::qcm::hypothesis::Hypothesis;
