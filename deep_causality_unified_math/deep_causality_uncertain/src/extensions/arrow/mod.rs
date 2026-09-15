/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The lazy graph as a value-level [`Arrow`].
//!
//! # Why a graph is an arrow
//!
//! haft's container traits — `Functor`, `Foldable`, `Semigroupal` — are for *data*, and they
//! deliberately carry no `'static`, because a container holds values that live as long as they
//! live. A lazy computation graph is not data; it is a **program**, and a program has to be
//! runnable later than it was built. `Arrow` is where haft puts that, and it is the surface the
//! four removed higher-kinded node arms were reaching for and could never have reached: they asked
//! a container to store a function.
//!
//! # Why this is well-typed
//!
//! [`Arrow::run`] takes `&self`. It can do that only because evaluating this graph at an address is
//! a **pure function** — every leaf draw is settled by the session seed, the sample index and the
//! leaf's ordinal, and by nothing else. A graph that drew from a thread-local seed or an ambient
//! generator could not satisfy the signature: two calls with the same input would differ, and there
//! would be nowhere to put the mutation. The session work is what makes the Arrow instance
//! possible, rather than the Arrow instance being a thing bolted on afterwards.
//!
//! `Out` is a `Result`, because a draw can fail — a leaf whose ordinal is missing, a distribution
//! whose parameters the constructor refuses. Composing downstream of that means composing with an
//! arrow whose `In` is the `Result`, which is honest: the failure is part of what this arrow
//! produces, and hiding it would move the decision somewhere the type stops mentioning it.

use crate::{SampleIndex, SampleSession, Uncertain, UncertainBool, UncertainError};
use deep_causality_haft::Arrow;
use deep_causality_rand::RandScalar;

impl<R: RandScalar> Arrow for Uncertain<R> {
    type In = SampleIndex;
    type Out = Result<R, UncertainError>;

    /// Draws this quantity at one address.
    ///
    /// Equal to [`Uncertain::sample_at`] at the same seed and index, and pure: running twice at one
    /// address gives one value, with nothing stored between the calls.
    #[inline]
    fn run(&self, at: SampleIndex) -> Self::Out {
        self.sample_at(&SampleSession::seeded(at.seed()), at.index())
    }
}

impl<R: RandScalar> Arrow for UncertainBool<R> {
    type In = SampleIndex;
    type Out = Result<bool, UncertainError>;

    /// Draws this verdict at one address. The Boolean carrier's arrow, mirroring
    /// [`Uncertain`]'s — one graph, two carriers, and the composition surface belongs to both.
    #[inline]
    fn run(&self, at: SampleIndex) -> Self::Out {
        self.sample_at(&SampleSession::seeded(at.seed()), at.index())
    }
}
