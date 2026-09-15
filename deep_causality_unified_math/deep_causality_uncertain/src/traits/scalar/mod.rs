/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The scalar a computation graph is carried at.

use deep_causality_rand::RandScalar;

/// Every scalar an uncertain value can be carried at.
///
/// Blanket-implemented, so a scalar joins by satisfying the algebra rather than by an entry in a
/// list here. There is no per-type fact in this crate to keep in step with a type added elsewhere.
///
/// # The algebra
///
/// [`RandScalar`] is `RealField + FromPrimitive`, and it is the whole of the requirement that a
/// numeric type can fail. It has to be a *field* rather than merely a
/// [`Real`](deep_causality_algebra::Real): a probability is a ratio of two counts, `Real` in
/// `deep_causality_algebra` is a commutative ring with an order and carries no `Div`, and division
/// arrives with `Field`. `RealField` is the weakest structure that can state a frequency, which is
/// why this crate's bound is not `Real`.
///
/// # The `'static`
///
/// `'static` is not the number's requirement. It is what the four unreachable higher-kinded node
/// arms — `PureOp`, `FmapOp`, `ApplyOp`, `BindOp` — impose by storing an `Arc<dyn …<R>>`, because a
/// trait object's default lifetime reaches the type it is parameterised by. No public constructor
/// builds one of those arms, and removing them removes the bound; the Arrow layer is where a stored
/// function belongs.
///
/// A mapped function does **not** contribute to this. `Uncertain::map` takes `fn(R) -> R` — a
/// plain pointer, statically dispatched — so `Send + Sync` came off this bound when the stored
/// closure did, and every scalar in the workspace is `'static` anyway, being a plain value type.
pub trait UncertainScalar: RandScalar + 'static {}

impl<T: RandScalar + 'static> UncertainScalar for T {}
