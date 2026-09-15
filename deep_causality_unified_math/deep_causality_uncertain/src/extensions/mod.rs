/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Implementations of **another crate's** traits on this crate's types.
//!
//! An extension is a bridge rather than a capability: the trait belongs elsewhere, the type belongs
//! here, and the impl is what lets the two meet. Keeping them together makes the crate's outward
//! surface readable in one place — what this crate promises to the rest of the workspace, as
//! opposed to what it does for itself.
//!
//! The same placement is used by `deep_causality_linear` and `deep_causality_tensor` for their
//! `Collectable` witnesses.

pub mod arrow;
