/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The resumable, forkable QTT march: the public names for the shared carrier machinery
//! ([`CarrierPause`]/[`CarrierFork`]) instantiated over the incompressible
//! [`QttCarrier`](super::qtt_march_run::QttCarrier).
//!
//! [`QttMarchRun::run_until`](super::QttMarchRun::run_until) pauses the coupled loop at a
//! predicate and yields a [`MarchPause`]: the borrowed world (config + coupling), the fluid state,
//! and the coupled field at the pause step, with the fluid and field behind `Arc`. A
//! [`MarchPause::fork`] is **O(1)** — it clones the `Arc`s, not the tensor data — so a
//! counterfactual study spawns one fork per candidate world from the *same* paused state. Each
//! fork alternates its world or state through the verbatim `deep_causality_core` vocabulary
//! (`alternate_context` / `alternate_state` / `alternate_value`, each appending its
//! `!!*Alternation!!` audit entry), then [`MarchFork::continue_march`] rebuilds the solver from
//! the (alternated) config and resumes from the branch state.
//!
//! **Copy-on-write.** The march never mutates fluid trains in place (each step *produces* the next
//! state), so a continued branch reads the shared paused state and replaces its own `Arc` — no
//! tensor data is ever copied. The coupled field *is* mutated (stages write scalars), so the first
//! write triggers exactly one `Arc::make_mut` clone; the pause's copy stays pristine and every
//! further fork sees the identical paused state. Read → share, write → CoW.
//!
//! **Error channel.** A step failure inside `run_until` is captured into the pause (with a
//! provenance entry), not thrown: the pause is a carrier. Per the `Alternatable` contract the
//! error channel is never alternated — alternation on an errored fork applies nothing and appends
//! only the audit entry — and `continue_march` on an errored fork returns the captured error.

use super::carrier::{CarrierFork, CarrierPause};
use super::qtt_march_run::QttCarrier;

/// A coupled QTT march paused mid-flight: the shared branch state every counterfactual fork
/// resumes from. Produced by [`QttMarchRun::run_until`](super::QttMarchRun::run_until).
pub type MarchPause<'c, R, S> = CarrierPause<'c, R, S, QttCarrier<R>, 2>;

/// One counterfactual branch forked from a [`MarchPause`]: alternate its world or state, then
/// `continue_march`. Alternation uses the verbatim core vocabulary; the error channel is never
/// alternated.
pub type MarchFork<'p, 'c, R, S> = CarrierFork<'p, 'c, R, S, QttCarrier<R>, 2>;
