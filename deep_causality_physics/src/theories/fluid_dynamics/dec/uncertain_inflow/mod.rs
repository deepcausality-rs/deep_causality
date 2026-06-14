/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Stage-4 uncertain-inflow zone (CFD Group C): the first `MaybeUncertain` data zone.
//!
//! A sensor-fed inflow boundary patch composed into a `CausalFlow` time march. The solver stays
//! stateless — the monad carries the state and the dropout intervention; the uncertain types
//! never reach the solver core (design D6/D10). See [`inflow_march`] for the march mechanics and
//! [`UncertainInflowZone`] for the configuration.

pub(crate) mod dropout_verbosity;
pub(crate) mod inflow_march;
pub(crate) mod uncertain_inflow_zone;

pub use dropout_verbosity::DropoutVerbosity;
pub use inflow_march::{
    InflowContext, InflowMarchState, InflowProcess, inflow_march_step, march_inflow,
};
pub use uncertain_inflow_zone::UncertainInflowZone;
