/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Drawing an ensemble into a container the caller names.
//!
//! # Why there is no ensemble type here
//!
//! An ensemble is `n` draws of one quantity, and the obvious move is to give it a struct. That
//! struct would then need a `Functor` to map over the draws, a `Foldable` to reduce them, a
//! `Semigroupal` to pair two ensembles, an iterator, indexing, and a conversion to whatever
//! container the caller actually wanted — every one of which already exists on containers that are
//! not this one.
//!
//! So the carrier is a **parameter**. `materialize::<W>` takes a witness and hands back
//! `W::Type<R>`: a `DenseVector<R>`, a rank-1 `CausalTensor<R>`, a `Vec<R>`, or whatever else
//! implements [`Collectable`]. The ensemble arrives already carrying every categorical structure
//! that witness provides, and this crate writes none of it.
//!
//! The witnesses for the two container crates live in those crates, which is what keeps
//! `deep_causality_uncertain` naming neither of them.
//!
//! # Composing two ensembles
//!
//! Two quantities materialised from **one session at the same indices** are correlated by index:
//! draw `i` of one belongs with draw `i` of the other, because both were drawn at sample `i`.
//! Combining them is therefore a *positional* zip — `Semigroupal::zip_with` on
//! `ZipDenseVectorWitness` or `ZipTensorWitness`, and `DiagonalTraversable::sequence_zip` to turn a
//! structure of ensembles inside out.
//!
//! **`Traversable::sequence` is the hazard, not the tool.** It forms the *cartesian product* across
//! quantities, which for sampling is almost never what is meant: four quantities at 50 draws each
//! give 50⁴ = 6 250 000 combinations rather than 50 correlated tuples, and the count is the only
//! symptom — every combination is individually well-formed. Reach for the diagonal.
//!
//! Neither zip witness has `Pure`, because the unit of a positional zip is the infinite repeat, so
//! `sequence_zip` takes its accumulator as a parameter. For sampling that is the right shape: the
//! ensemble size is declared rather than inferred.
//!
//! # Not a field — and the scalar that moves the line
//!
//! Materialising an ensemble per cell of a simulation field is bounded by *cells × draws × width*,
//! and that product reaches the ceiling before any one factor looks large. A 256³ field at 1000
//! draws is 134 GB at `f64`, 67 GB at `f32` and 34 GB at `BFloat16`; a 512³ field at the same
//! ensemble size is 1.07 TB, 537 GB and 268 GB.
//!
//! The width is the factor the caller picks, and `BFloat16` is the one worth knowing about: it
//! keeps f32's 8 exponent bits and bias, so its normal range *is* f32's, and it trades significand
//! alone. For an ensemble that is a good trade — a half-ulp is 0.39% where the Monte-Carlo error of
//! an n-draw mean is 1/√n, and the two meet only at n = 65 536. See the crate documentation for the
//! full table and the two caveats (reduction is a separate question from storage, and a spread
//! loses more than a mean).
//!
//! **The shape that stays bounded** inverts the loop regardless: materialise the uncertain
//! *inputs* — a handful of quantities, not a field of them — traverse them diagonally so draw `i`
//! of each belongs together, and evaluate the field once per draw, reducing as you go. Memory is
//! then one field plus one accumulator, independent of the ensemble size, and the ensemble never
//! exists all at once.

use crate::{SampleSession, Uncertain, UncertainBool, UncertainError};
use deep_causality_haft::{Collectable, HKT};
use deep_causality_rand::RandScalar;

impl<R: RandScalar> Uncertain<R> {
    /// Draws `n` samples into the container `W` witnesses, advancing the session by `n` indices.
    ///
    /// One signature serves every witnessed rank-1 carrier; the caller picks which at the call
    /// site. The draws are the same values, in the same order, whichever carrier is chosen — the
    /// witness decides where they land and nothing else.
    ///
    /// # Arguments
    ///
    /// * `session` — advanced by `n`, so two quantities materialised from one session get
    ///   *different* indices. To correlate them use [`Uncertain::materialize_at`], which holds the
    ///   indices fixed.
    /// * `n` — the ensemble size, declared rather than inferred.
    ///
    /// # Errors
    ///
    /// The first draw that fails, carried through. A partially drawn ensemble is not returned,
    /// because half an ensemble is not a smaller ensemble — it is an ensemble of the wrong size,
    /// and a reduction over it would be silently wrong rather than visibly absent.
    ///
    /// # Cost
    ///
    /// One traversal per draw, plus one intermediate `Vec` before the collect: a draw can fail, and
    /// `Collectable::collect` has nowhere to put a failure, so the `Result`s are resolved before
    /// the container is built.
    pub fn materialize<W>(
        &self,
        session: &mut SampleSession,
        n: usize,
    ) -> Result<W::Type<R>, UncertainError>
    where
        W: Collectable<W> + HKT,
    {
        Ok(W::collect(self.take_samples(session, n)?))
    }

    /// Draws `n` samples at indices `0..n` of `session`, without advancing it.
    ///
    /// The form to use for **correlated** ensembles: two quantities materialised this way from one
    /// session are drawn at the same indices, so draw `i` of each belongs with draw `i` of the
    /// other and a positional zip pairs them correctly.
    pub fn materialize_at<W>(
        &self,
        session: &SampleSession,
        n: usize,
    ) -> Result<W::Type<R>, UncertainError>
    where
        W: Collectable<W> + HKT,
    {
        Ok(W::collect(self.samples_from(session, n)?))
    }
}

impl<R: RandScalar> UncertainBool<R> {
    /// Draws `n` truth values into the container `W` witnesses, advancing the session by `n`.
    ///
    /// The Boolean carrier's ensemble, mirroring [`Uncertain::materialize`]. An ensemble of
    /// verdicts reduces differently from one of reals — a count rather than a mean — which is a
    /// property of what the caller does with it, not of how it is carried.
    pub fn materialize<W>(
        &self,
        session: &mut SampleSession,
        n: usize,
    ) -> Result<W::Type<bool>, UncertainError>
    where
        W: Collectable<W> + HKT,
    {
        Ok(W::collect(self.take_samples(session, n)?))
    }

    /// Draws `n` truth values at indices `0..n` of `session`, without advancing it.
    ///
    /// See [`Uncertain::materialize_at`]: this is the form that correlates with another quantity
    /// drawn from the same session.
    pub fn materialize_at<W>(
        &self,
        session: &SampleSession,
        n: usize,
    ) -> Result<W::Type<bool>, UncertainError>
    where
        W: Collectable<W> + HKT,
    {
        Ok(W::collect(self.samples_from(session, n)?))
    }
}
