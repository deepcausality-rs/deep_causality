/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Composition of abstractions and the law its residuals obey.
//!
//! Two abstractions `L → M` and `M → H` compose to `L → H` with `π = π₁ ∘ π₂` on the types and
//! `τ = τ₂ ∘ τ₁` on the channels (Lorenz & Tull, arXiv:2602.16612, Proposition 17 for the exact
//! case). With the residuals measured in Frobenius norm on the Choi operators, the composite
//! defect of one query splits along the middle level,
//!
//! ```text
//! τ₂ ∘ ⟦π₁π₂ Q⟧_L − ⟦Q⟧_H ∘ τ₂ ∘ τ₁
//!   = τ₂,out ∘ (τ₁,out ∘ ⟦π₁ Q_M⟧_L − ⟦Q_M⟧_M ∘ τ₁,in)
//!   + (τ₂,out ∘ ⟦Q_M⟧_M − ⟦Q⟧_H ∘ τ₂,in) ∘ τ₁,in ,
//! ```
//!
//! so by the triangle inequality `ε ≤ ‖τ₂‖_post · ε₁ + ‖τ₁‖_pre · ε₂`. Post-composition by `τ₂`
//! acts on a Choi operator as `id ⊗ τ₂` on the output factor and pre-composition by `τ₁` as the
//! transpose of `τ₁`'s natural representation on the input factor; the Frobenius norm of a Choi
//! operator is the 2-norm of its vectorisation, on which these act as `I ⊗ N₂` and `N₁ᵀ ⊗ I`, whose
//! spectral norms are those of `N₂` and `N₁`. Both constants are therefore the Frobenius-induced
//! norms of the channels themselves, [`QcMorphism::frobenius_induced_norm`], and neither is one in
//! general: a partial trace over a factor of dimension `d` has norm `√d`. In the diamond norm both
//! would be one. The law is this crate's theorem; the exact case `ε₁ = ε₂ = 0 ⇒ ε = 0` is the Lean
//! statement in `lean/DeepCausalityFormal/Quantum/Abstraction.lean`.
//!
//! The report carries, per query, `ε₁`, `ε₂`, both constants, the bound and the composite's
//! measured residual, so a reader can recompute the bound and see it hold.
//!
//! [`QcMorphism::frobenius_induced_norm`]: crate::types::circuit_model::QcMorphism::frobenius_induced_norm

use crate::QuantumError;
use crate::types::abstraction::abstraction::Abstraction;
use crate::types::abstraction::qc_model::QcModel;
use crate::types::abstraction::query::Query;
use crate::types::abstraction::type_alignment::{AlignmentSide, AlignmentSpec, TypeAlignment};
use crate::types::circuit_model::NumericCaps;
use crate::types::decision::Tolerance;
use alloc::format;
use alloc::vec::Vec;
use core::fmt;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// The norm the composition law is stated in.
pub const COMPOSITION_NORM: &str = "frobenius-on-choi";

/// One query's row of the composition law.
#[derive(Debug, Clone, PartialEq)]
pub struct LawRow<R> {
    /// The high-level query.
    pub query: Query,
    /// The first link's residual on the query's image, `ε₁`.
    pub epsilon_first: R,
    /// The second link's residual on the query, `ε₂`.
    pub epsilon_second: R,
    /// `‖τ₁‖_pre`, the Frobenius-induced norm of the first link's input-side `τ`.
    pub pre: R,
    /// `‖τ₂‖_post`, the Frobenius-induced norm of the second link's output-side `τ`.
    pub post: R,
    /// `post · ε₁ + pre · ε₂`.
    pub bound: R,
    /// The composite's residual on the query.
    pub measured: R,
    /// The state tolerance at the composite square's dimension, the slack `holds` allows.
    pub tolerance: R,
}

impl<R: RealField> LawRow<R> {
    /// Whether the measured residual is within the bound, up to the state tolerance.
    pub fn holds(&self) -> bool {
        self.measured <= self.bound + self.tolerance
    }
}

/// The composition law over a composed signature.
#[derive(Debug, Clone, PartialEq)]
pub struct CompositionLaw<R> {
    /// One row per query of the composite signature, in order.
    pub rows: Vec<LawRow<R>>,
    /// The norm the residuals and the constants are in.
    pub norm: &'static str,
}

impl<R: RealField> CompositionLaw<R> {
    /// Whether every row holds; an empty signature holds vacuously.
    pub fn holds(&self) -> bool {
        self.rows.iter().all(LawRow::holds)
    }

    /// The largest bound over the rows, or zero.
    pub fn bound(&self) -> R {
        self.rows
            .iter()
            .map(|r| r.bound)
            .fold(R::zero(), |a, b| if b > a { b } else { a })
    }

    /// The largest measured residual over the rows, or zero.
    pub fn measured(&self) -> R {
        self.rows
            .iter()
            .map(|r| r.measured)
            .fold(R::zero(), |a, b| if b > a { b } else { a })
    }

    /// The row with the least slack, `bound − measured`, the first among equals.
    pub fn tightest(&self) -> Option<&LawRow<R>> {
        self.rows
            .iter()
            .fold(None, |acc: Option<&LawRow<R>>, r| match acc {
                Some(a) if a.bound - a.measured <= r.bound - r.measured => Some(a),
                _ => Some(r),
            })
    }
}

impl<R: RealField + fmt::Debug> fmt::Display for CompositionLaw<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "composition law in {}: ε ≤ ‖τ₂‖_post · ε₁ + ‖τ₁‖_pre · ε₂ over {} queries, {}",
            self.norm,
            self.rows.len(),
            if self.holds() { "holds" } else { "violated" }
        )?;
        for r in &self.rows {
            writeln!(
                f,
                "  {}: ε₁ = {:?}, ε₂ = {:?}, ‖τ₁‖_pre = {:?}, ‖τ₂‖_post = {:?}, bound = {:?}, measured = {:?}",
                r.query.kind(),
                r.epsilon_first,
                r.epsilon_second,
                r.pre,
                r.post,
                r.bound,
                r.measured
            )?;
        }
        Ok(())
    }
}

/// A composite abstraction with the law its links obey.
#[derive(Debug, Clone)]
pub struct Composed<R: RealField, L, H> {
    /// `L → H`, with `π = π₁ ∘ π₂` and `τ = τ₂ ∘ τ₁`.
    pub abstraction: Abstraction<R, L, H>,
    /// The law, one row per query.
    pub law: CompositionLaw<R>,
}

impl<R, L, M> Abstraction<R, L, M>
where
    R: RealField + FromPrimitive + Default + fmt::Debug,
    L: QcModel<R>,
    M: QcModel<R>,
{
    /// The composite of this abstraction `L → M` with `next : M → H`.
    ///
    /// The composite alignment sends each high-level type through `next`'s entry to its middle
    /// wires and through this abstraction's entries covering those wires to the low-level ones,
    /// with `τ` the composite channel and `E` the composite section; a two-sided entry of `next`
    /// over a sided alignment here splits into an input-side and an output-side entry. The
    /// composite query map sends a high-level query to the image under this abstraction of its
    /// image under `next`. Every link residual, both constants and the composite residual are
    /// computed on the numeric path and recorded in the law.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] when a query of `next`'s map has no image in this
    /// abstraction's signature, or a middle type of `next`'s alignment is not covered here; the
    /// alignment's, the semantics' and the eigensolver's errors.
    pub fn compose<H>(
        self,
        next: Abstraction<R, M, H>,
        caps: &NumericCaps,
    ) -> Result<Composed<R, L, H>, QuantumError>
    where
        H: QcModel<R>,
    {
        // The composite query map and the link data per query, before anything is moved.
        let mut map: Vec<(Query, Query)> = Vec::with_capacity(next.query_map().len());
        let mut partial: Vec<(Query, R, R, R, R)> = Vec::with_capacity(next.query_map().len());
        for (q_h, q_m) in next.query_map() {
            let q_l = self.image(q_m).cloned().ok_or_else(|| {
                QuantumError::CalculationError(format!(
                    "the middle-level query {q_m:?} has no image in the first link's signature"
                ))
            })?;
            let (l1, r1) = self.square_with(q_m, &q_l, caps)?;
            let (epsilon_first, _) = l1.frobenius_distance(&r1, caps)?;
            let (l2, r2) = next.square_with(q_h, q_m, caps)?;
            let (epsilon_second, _) = l2.frobenius_distance(&r2, caps)?;
            let pre = self.tau_in(q_m, &q_l, caps)?.frobenius_induced_norm(caps)?;
            let post = next.tau_out(q_h, q_m, caps)?.frobenius_induced_norm(caps)?;
            map.push((q_h.clone(), q_l));
            partial.push((q_h.clone(), epsilon_first, epsilon_second, pre, post));
        }

        // The composite alignment.
        let sided_here = self
            .alignment()
            .entries()
            .iter()
            .any(|e| e.side() != AlignmentSide::Any);
        let mut entries: Vec<(AlignmentSide, AlignmentSpec<R>)> = Vec::new();
        for e in next.alignment().entries() {
            let sides: Vec<AlignmentSide> = if e.side() == AlignmentSide::Any && sided_here {
                alloc::vec![AlignmentSide::Input, AlignmentSide::Output]
            } else {
                alloc::vec![e.side()]
            };
            for side in sides {
                let low = self.alignment().low_for_side(e.low(), side)?;
                let tau = self
                    .alignment()
                    .tau_for_side(e.low(), side, caps)?
                    .then(e.tau(), caps)?;
                let section = e.section().then(
                    &self.alignment().section_for_side(e.low(), side, caps)?,
                    caps,
                )?;
                entries.push((side, (e.high().to_vec(), low, tau, section)));
            }
        }
        let alignment = TypeAlignment::new_sided(entries)?;

        let (low, _, _, _) = self.into_parts();
        let (_, high, _, _) = next.into_parts();
        let abstraction = Abstraction::new(low, high, alignment, map)?;

        let mut rows = Vec::with_capacity(partial.len());
        for (query, epsilon_first, epsilon_second, pre, post) in partial {
            let (left, right) = abstraction.square(&query, caps)?;
            let (measured, _) = left.frobenius_distance(&right, caps)?;
            let tolerance = Tolerance::<R>::state()
                .threshold(left.d_in() * left.d_out(), R::one())
                .unwrap_or_else(|| R::epsilon().sqrt());
            rows.push(LawRow {
                query,
                epsilon_first,
                epsilon_second,
                pre,
                post,
                bound: post * epsilon_first + pre * epsilon_second,
                measured,
                tolerance,
            });
        }
        Ok(Composed {
            abstraction,
            law: CompositionLaw {
                rows,
                norm: COMPOSITION_NORM,
            },
        })
    }
}
