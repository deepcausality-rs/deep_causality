/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg_attr(not(feature = "std"), no_std)]

//! Shared parallelism primitives for the DeepCausality workspace.
//!
//! Two items live here:
//!
//! * [`MaybeParallel`], the feature-conditional thread-safety marker that
//!   the `parallel` features of `deep_causality_topology`,
//!   `deep_causality_fft`, and their consumers share. Hosting it in one
//!   Tier-0 crate guarantees a single definition; Cargo feature
//!   unification on `deep_causality_par/parallel` keeps every crate in a
//!   build agreeing on whether the bound means `Send + Sync` or nothing.
//! * [`scoped_map`], the minimal in-house fork-join surface for few,
//!   long, data-independent tasks such as counterfactual branch fan-outs:
//!   an order-preserving parallel map over a slice on
//!   [`std::thread::scope`] threads under the `parallel` feature, a plain
//!   inline map without it. No thread pool, no external dependency.
//!
//! # Feature levels
//!
//! `default = ["std"]`, `std = ["alloc"]`, `alloc = []`, and
//! `no-std = ["alloc"]` follow the workspace three-level convention. The
//! crate's own needs are small: `core` for the marker trait, `alloc` for
//! the [`Vec`](alloc::vec::Vec) that [`scoped_map`] returns. There is no
//! external dependency to forward a level to, so `std` and `no-std` differ
//! only in whether the crate declares `no_std`. Bare-metal builds use
//! `--no-default-features --features no-std` and get the serial inline
//! map.
//!
//! # Why `parallel = ["std"]` stays
//!
//! The parallel arm of [`scoped_map`] calls [`std::thread::scope`] and
//! [`std::thread::available_parallelism`], and neither exists in `core` or
//! `alloc`. `parallel` therefore declares a dependency on `std`, and that
//! declaration earns its place: it is what makes a bare `--features
//! parallel` resolve to a working configuration, and what lets a
//! downstream crate forward `deep_causality_par/parallel` on its own
//! (`deep_causality_algorithms`, `deep_causality_topology`,
//! `deep_causality_cfd`, and `deep_causality_physics` all do) without also
//! having to remember `deep_causality_par/std`. Dropping it would turn
//! every one of those into a build error on the host, which is the common
//! case. `deep_causality_fft` states the same requirement the same way.
//!
//! What the declaration does **not** do is make `no-std` plus `parallel`
//! unrepresentable. Cargo features are additive and cannot express mutual
//! exclusion: `--features no-std,parallel` does not disable `parallel`, it
//! re-enables `std` underneath the `no-std` request. On a host that is
//! harmless, because the resulting `std` build is a valid one and only the
//! `no-std` request is dropped; `--all-features` relies on exactly that.
//! On a target that has no `std` to re-enable, it used to surface as
//! `can't find crate for `std`` plus a cascade of prelude errors that name
//! neither the feature that caused them nor the way out. The
//! `compile_error!` below states both instead.

// Rejects `parallel` on a target that has no `std` to fall back on, which
// is the one place where the additive `parallel = ["std"]` edge cannot
// deliver what it promises. Keyed on the target rather than on
// `feature = "no-std"` so that host builds carrying both flags — every
// `--all-features` build in CI does — stay unaffected.
#[cfg(all(feature = "parallel", target_os = "none"))]
compile_error!(
    "deep_causality_par: the `parallel` feature requires `std`, because `scoped_map` fans out \
     over `std::thread::scope`, but the selected target has no `std`. Cargo features are \
     additive, so `--features no-std,parallel` does not turn `parallel` off; it turns `std` back \
     on through `parallel = [\"std\"]`, and the build then fails to find the `std` crate. Build \
     bare-metal targets without `parallel` — `--no-default-features --features no-std` — and \
     `scoped_map` runs the serial inline map."
);

extern crate alloc;

mod functions;
pub mod traits;

pub use crate::functions::scoped_map::scoped_map;
pub use crate::traits::maybe_parallel::MaybeParallel;
