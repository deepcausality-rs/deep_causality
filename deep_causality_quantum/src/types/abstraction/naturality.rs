/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The naturality check: for every query of the signature, `τ ∘ ⟦π(Q)⟧_L = ⟦Q⟧_H ∘ τ`
//! (Lorenz & Tull, arXiv:2602.16612, Eq. 15). An abstraction whose squares commute to residual `ε`
//! is an **ε-abstraction**. That notion is this crate's definition; the paper gives the exact case.

use crate::QuantumError;
use crate::types::abstraction::abstraction::Abstraction;
use crate::types::abstraction::diamond_bound::DiamondBound;
use crate::types::abstraction::qc_model::QcModel;
use crate::types::abstraction::query::Query;
use crate::types::circuit_model::{NumericCaps, SemanticsPath};
use crate::types::decision::{Check, CheckItem, CheckReport, Tolerance};
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// The norm a naturality report measured in.
pub const FROBENIUS_ON_CHOI: &str = "frobenius-on-choi";

/// What `check_naturality` examined and concluded.
#[derive(Debug, Clone, PartialEq)]
pub struct NaturalityReport<R> {
    /// One record per query, in signature order; the residual against the state tolerance.
    pub report: CheckReport<R>,
    /// The path that decided.
    pub path: SemanticsPath,
    /// The norm the residuals are in.
    pub norm: &'static str,
    /// The two-sided diamond bound of the worst residual.
    pub bound: DiamondBound<R>,
    /// The Choi entries formed over every square.
    pub entries: u64,
}

impl<R: RealField> NaturalityReport<R> {
    /// The largest residual, or zero on an empty signature.
    pub fn worst_residual(&self) -> R {
        self.report
            .worst()
            .map(|c| c.measured)
            .unwrap_or_else(R::zero)
    }
}

impl<R, L, H> Abstraction<R, L, H>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
    L: QcModel<R>,
    H: QcModel<R>,
{
    /// The numeric naturality check over the whole signature.
    ///
    /// Each square's two sides are formed as morphisms from the low-level input type to the
    /// high-level output type and compared in Frobenius norm on their Choi operators against
    /// `Tolerance::state()` at the square's dimension. The report's examined count is the number
    /// of queries; an empty signature reads `Vacuous`.
    ///
    /// # Errors
    ///
    /// The semantics' caps and the alignment's errors.
    pub fn check_naturality(
        &self,
        caps: &NumericCaps,
    ) -> Result<NaturalityReport<R>, QuantumError> {
        self.check_naturality_on(self.signature().queries(), caps)
    }

    /// The numeric naturality check over the listed queries of the signature only. Records keep
    /// their index in the signature. A fixture may reach the caps on one query and not another:
    /// the opening of an eight-qubit code doubles its register.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] for a query not in the signature; otherwise as
    /// [`check_naturality`](Self::check_naturality).
    pub fn check_naturality_on(
        &self,
        queries: &[Query],
        caps: &NumericCaps,
    ) -> Result<NaturalityReport<R>, QuantumError> {
        let all = self.signature().queries();
        if let Some(q) = queries.iter().find(|q| !all.contains(q)) {
            return Err(QuantumError::CalculationError(alloc::format!(
                "the query {q:?} is not in the signature"
            )));
        }
        let mut checks = Vec::with_capacity(queries.len());
        let mut entries: u64 = 0;
        let mut worst = (R::zero(), 1usize, 1usize, Vec::new(), Vec::new());
        for (i, high) in all.iter().enumerate() {
            if !queries.contains(high) {
                continue;
            }
            let (left, right) = self.square(high, caps)?;
            let (residual, formed) = left.frobenius_distance(&right, caps)?;
            entries = entries.saturating_add(formed);
            let dim = left.d_in() * left.d_out();
            let tolerance = Tolerance::<R>::state()
                .threshold(dim, R::one())
                .unwrap_or_else(|| R::epsilon().sqrt());
            if residual >= worst.0 {
                worst = (
                    residual,
                    left.d_in(),
                    left.d_out(),
                    left.classical_in().to_vec(),
                    left.classical_out().to_vec(),
                );
            }
            checks.push(Check::new(CheckItem::Index(i), residual, tolerance));
        }
        Ok(NaturalityReport {
            report: CheckReport::from_checks(checks),
            path: SemanticsPath::Numeric,
            norm: FROBENIUS_ON_CHOI,
            bound: DiamondBound::from_frobenius_blocks(
                worst.0, worst.1, worst.2, &worst.3, &worst.4,
            ),
            entries,
        })
    }
}
