/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! **CFD configuration** (the "what") — owned, validated parameter bundles, separate from the
//! [`CfdFlow`](crate::CfdFlow) workflow-composition DSL (the "how").
//!
//! [`CfdConfigBuilder`] is the single entry: it builds the per-solver config and the marching-case
//! container ([`MarchConfig`]). The scenario value types ([`Mesh`], [`Body`], [`Seed`], [`Observe`])
//! are the configuration inputs. Mirrors the Discovery `CdlConfigBuilder` → `CdlBuilder` split.

mod body;
mod cfd_config_builder;
pub(crate) mod manufactured;
mod march_builder;
pub(crate) mod march_config;
mod mesh;
mod observe;
mod seed;
#[cfg(feature = "std")]
pub(crate) mod uncertain_march_config;

pub use body::Body;
pub use cfd_config_builder::CfdConfigBuilder;
pub use manufactured::{
    Manufactured, ManufacturedSample, TaylorGreen, VerifyConfig, VerifyConfigBuilder,
};
pub use march_builder::MarchConfigBuilder;
pub use march_config::{MarchConfig, MarchStop};
pub use mesh::{Grading, Mesh};
pub use observe::Observe;
pub use seed::Seed;
#[cfg(feature = "std")]
pub use uncertain_march_config::{UncertainMarchConfig, UncertainMarchConfigBuilder};
