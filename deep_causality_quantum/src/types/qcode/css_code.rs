/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::decision::{Check, CheckItem, CheckReport};
use alloc::format;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_homology::{ChainComplex, Gf2Chain, HomologyField};
use deep_causality_linear::{MatrixView, PackedGf2, csr_to_packed_gf2_mod2};
use deep_causality_num::{FromPrimitive, NaturalNumber};

/// A CSS code read off a chain complex: `n` qubits from the 1-cells, `k` logical qubits from `β₁`
/// over 𝔽₂, and the two check families as the raw columns of `∂₂` and `δ₀`.
///
/// No distance is carried. The minimum weight of a non-trivial class is a computation this type
/// does not make, and a field it does not have cannot be misread as one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssCode<W> {
    n: usize,
    k: usize,
    z_generators: Vec<Gf2Chain<W>>,
    x_generators: Vec<Gf2Chain<W>>,
}

impl<W: NaturalNumber> CssCode<W> {
    /// The number of physical qubits, the 1-cells.
    pub fn n(&self) -> usize {
        self.n
    }

    /// The number of logical qubits, `β₁` over 𝔽₂.
    pub fn k(&self) -> usize {
        self.k
    }

    /// The Z-check generators, one per 2-cell: the columns of `∂₂`.
    pub fn z_generators(&self) -> &[Gf2Chain<W>] {
        &self.z_generators
    }

    /// The X-check generators, one per 0-cell: the columns of `δ₀`.
    pub fn x_generators(&self) -> &[Gf2Chain<W>] {
        &self.x_generators
    }
}

/// The CSS code of a chain complex at grade 1, composed from the shipped counts and columns.
///
/// # Errors
///
/// [`QuantumError::CalculationError`] if the 𝔽₂ rank underneath `β₁` fails;
/// [`QuantumError::DimensionMismatch`] if a column cannot be read as a chain.
pub fn derive_code<W, K>(complex: &K) -> Result<CssCode<W>, QuantumError>
where
    W: NaturalNumber,
    K: ChainComplex + ?Sized,
{
    let n = complex.num_cells(1);
    let k = complex
        .betti_number_over(1, HomologyField::Gf2)
        .map_err(|e| QuantumError::CalculationError(format!("{e}")))?;
    let d2 = csr_to_packed_gf2_mod2::<W>(&complex.boundary_matrix(2));
    let delta0 = csr_to_packed_gf2_mod2::<W>(&complex.coboundary_matrix(0));
    Ok(CssCode {
        n,
        k,
        z_generators: columns(&d2)?,
        x_generators: columns(&delta0)?,
    })
}

fn columns<W: NaturalNumber>(m: &PackedGf2<W>) -> Result<Vec<Gf2Chain<W>>, QuantumError> {
    (0..m.cols())
        .map(|c| {
            Gf2Chain::from_column(m, c, 1)
                .map_err(|e| QuantumError::DimensionMismatch(format!("{e}")))
        })
        .collect()
}

/// Which check matrix an LDPC item came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CheckMatrix {
    /// `∂₂`, the Z checks.
    Z,
    /// `δ₀`, the X checks.
    X,
}

/// Whether an LDPC item is a column, one check, or a row, one qubit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LdpcItem {
    /// A check: its weight is the number of qubits it acts on.
    Column(usize),
    /// A qubit: its weight is the number of checks acting on it.
    Row(usize),
}

/// What `check_ldpc_weights` examined and concluded.
///
/// One record per column and per row of both check matrices, each weight against the declared
/// bound, so the worst record is the heaviest item. The offender, when there is one, is named by
/// matrix and item because a record index alone cannot say which of the four families it sits in.
#[derive(Debug, Clone, PartialEq)]
pub struct LdpcWeights<R> {
    /// The records, in the order examined: Z columns, Z rows, X columns, X rows, stopping at the
    /// first rejection.
    pub report: CheckReport<R>,
    /// The largest column weight seen.
    pub max_column_weight: usize,
    /// The largest row weight seen.
    pub max_row_weight: usize,
    /// The first item over the bound, if any.
    pub offender: Option<(CheckMatrix, LdpcItem)>,
}

/// Both weights of both check matrices against one declared bound.
///
/// A column of `∂₂` or `δ₀` is one stabilizer generator and its weight is the number of qubits
/// that check acts on; a row is one qubit and its weight is the number of checks acting on it.
/// LDPC asks both to stay bounded as the code grows, so both are measured. Examination stops at
/// the first item over the bound, and the count reports what was actually visited. A code with no
/// Z checks examines no Z columns, which is visible as an empty half.
///
/// # Errors
///
/// [`QuantumError::CalculationError`] if the bound or a weight cannot be represented in `R`.
pub fn check_ldpc_weights<R, W>(
    code: &CssCode<W>,
    bound: usize,
) -> Result<LdpcWeights<R>, QuantumError>
where
    R: RealField + FromPrimitive,
    W: NaturalNumber,
{
    let threshold = as_real::<R>(bound)?;
    let mut checks = Vec::new();
    let mut max_column_weight = 0usize;
    let mut max_row_weight = 0usize;
    let mut offender = None;

    for (matrix, generators) in [
        (CheckMatrix::Z, &code.z_generators),
        (CheckMatrix::X, &code.x_generators),
    ] {
        let mut row_weights = alloc::vec![0usize; code.n];
        // Columns: one check each.
        for (c, g) in generators.iter().enumerate() {
            let w = g.weight();
            max_column_weight = max_column_weight.max(w);
            for q in g.support() {
                row_weights[q] += 1;
            }
            let check = Check::new(CheckItem::Index(checks.len()), as_real::<R>(w)?, threshold);
            let accepted = check.accepted;
            checks.push(check);
            if !accepted {
                offender = Some((matrix, LdpcItem::Column(c)));
                return Ok(finish(checks, max_column_weight, max_row_weight, offender));
            }
        }
        // Rows: one qubit each.
        for (r, &w) in row_weights.iter().enumerate() {
            max_row_weight = max_row_weight.max(w);
            let check = Check::new(CheckItem::Index(checks.len()), as_real::<R>(w)?, threshold);
            let accepted = check.accepted;
            checks.push(check);
            if !accepted {
                offender = Some((matrix, LdpcItem::Row(r)));
                return Ok(finish(checks, max_column_weight, max_row_weight, offender));
            }
        }
    }
    Ok(finish(checks, max_column_weight, max_row_weight, offender))
}

fn finish<R: RealField>(
    checks: Vec<Check<R>>,
    max_column_weight: usize,
    max_row_weight: usize,
    offender: Option<(CheckMatrix, LdpcItem)>,
) -> LdpcWeights<R> {
    LdpcWeights {
        report: CheckReport::from_checks(checks),
        max_column_weight,
        max_row_weight,
        offender,
    }
}

fn as_real<R: RealField + FromPrimitive>(count: usize) -> Result<R, QuantumError> {
    R::from_usize(count).ok_or_else(|| {
        QuantumError::CalculationError(format!("the scalar cannot represent the count {count}"))
    })
}
