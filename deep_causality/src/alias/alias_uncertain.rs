/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalityError, Context, FloatType, ProposedAction};

/// The framework's uncertain carriers, at the scalar the framework works in.
///
/// `deep_causality_uncertain` is generic in its scalar and ships no aliases of its own: once
/// precision is a parameter, an alias says nothing the instantiation does not, and the one name a
/// consumer wants bare — `UncertainBool` — is taken there by the struct itself. An alias belongs to
/// whoever picks the scalar, and this crate picks [`FloatType`] for its whole surface.
///
/// Both names are the ones the call sites already spell, so the 45 of them across this crate needed
/// no edit when the carrier split landed. What did change is where they come from: `UncertainBool`
/// is now the Boolean *carrier* rather than `Uncertain<bool>`, which no longer exists — a Boolean
/// node reads a truth value off a graph of reals, so it keeps the graph's scalar.
pub type UncertainBool = deep_causality_uncertain::UncertainBool<FloatType>;

/// The real carrier at [`FloatType`]. Named for the scalar it is fixed at, which is `f64`.
pub type UncertainF64 = deep_causality_uncertain::Uncertain<FloatType>;

// Type alias for the uncertain activation predicate function pointer.
#[allow(clippy::type_complexity)]
pub type UncertainActivationPredicate<D, S, T, ST, SYM, VS, VT> =
    fn(
        &Context<D, S, T, ST, SYM, VS, VT>,
        &ProposedAction,
    ) -> Result<UncertainBool, CausalityError>;
