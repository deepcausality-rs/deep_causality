/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain-specific type aliases for common physics conventions.
//!
//! These aliases provide semantic meaning when working with different physics domains:
//! - **General Relativity** uses East Coast (-+++) convention
//! - **Particle Physics** uses West Coast (+---) convention
//!
//! Every name here says which convention it means. There is deliberately no unqualified default:
//! a name like `MINKOWSKI_4D` reads as "the Minkowski metric" while silently being one of the two,
//! so a caller who wanted the other gets no signal at the point of use. Both conventions are
//! correct in their own literature, and the crate exists to make the choice visible rather than to
//! settle it.

use crate::conventions::{EastCoastMetric, WestCoastMetric};

/// For General Relativity modules: East Coast (-+++)
///
/// This is the convention used by Misner-Thorne-Wheeler (MTW) and most
/// GR textbooks. Timelike vectors have negative norm: g(u,u) < 0.
pub type RelativityMetric = EastCoastMetric;

/// Standard 4D Minkowski spacetime for General Relativity.
pub const RELATIVITY_MINKOWSKI_4D: RelativityMetric = EastCoastMetric::MINKOWSKI_4D;

/// For Particle Physics modules: West Coast (+---)
///
/// This is the convention used by Weinberg and most particle physics literature.
/// Timelike vectors have positive norm: g(u,u) > 0.
pub type ParticleMetric = WestCoastMetric;

/// Standard 4D Minkowski spacetime for Particle Physics.
pub const PARTICLE_MINKOWSKI_4D: ParticleMetric = WestCoastMetric::MINKOWSKI_4D;
