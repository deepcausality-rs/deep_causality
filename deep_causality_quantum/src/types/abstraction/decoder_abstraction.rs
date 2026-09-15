/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The decoder as an abstraction from a physical circuit to its detector error model.
//!
//! `DecoderAbstraction` is `Abstraction<CircuitModel, DemModel>`. The low level is the circuit that
//! produces the measurement record; the high level is the detector error model the decoder
//! assumes; `τ` is the decoder's reading of the record, a classical stochastic matrix from the
//! circuit's outcome strings to the model's detector and observable strings, lifted into QC as a
//! morphism with one scalar block per non-zero entry (the FStoch embedding). The crate contains no
//! decoding algorithm and no `Decoder` trait: `τ` enters as data and is validated once.
//!
//! The signature carries the `Io` query and one fault query per circuit location the caller names:
//! a Pauli injected at the location on the low level against the mechanism that represents it
//! fired once more on the high level. A location the model does not represent is given a phantom
//! mechanism, one that never fires and flips nothing, so its square asks whether the model may
//! ignore that location. A correlated error the model omits shifts the whole record, so every
//! square carries the nominal mismatch, of the order of the omitted probability, while the square
//! at the omitted location compares a shifted record with an unshifted prediction and its residual
//! is of the order of `√2`: the worst failure names the location. Naturality is therefore
//! decoder-model validation by openings, not by statistics. Attribution ranks the faults of a set by the residual their injection produces
//! against the model's nominal prediction.

use crate::QuantumError;
use crate::types::abstraction::abstraction::Abstraction;
use crate::types::abstraction::fault_set::{Fault, FaultSet, PauliKind};
use crate::types::abstraction::naturality::NaturalityReport;
use crate::types::abstraction::qc_model::QcModel;
use crate::types::abstraction::query::Query;
use crate::types::abstraction::type_alignment::TypeAlignment;
use crate::types::circuit_model::{CircuitModel, NumericCaps, QcMorphism};
use crate::types::decision::{CheckItem, Tolerance};
use crate::types::qcm::dem_model::DemModel;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::CausalTensor;

/// The decoder's stochastic matrix as a QC morphism between classical types: `matrix[x][y]` is
/// the probability of the high-level string `y` given the low-level string `x`, strings indexed
/// with the first wire most significant.
///
/// # Errors
///
/// [`QuantumError::DimensionMismatch`] when the product of either side's outcome counts
/// overflows or the shape does not match the counts; [`QuantumError::NonCptpChannel`] when a row
/// does not sum to one within the state tolerance or an entry is negative or non-finite.
pub fn stochastic_morphism<R>(
    matrix: &[Vec<f64>],
    low_counts: &[usize],
    high_counts: &[usize],
) -> Result<QcMorphism<R>, QuantumError>
where
    R: RealField + FromPrimitive + Default + fmt::Debug,
{
    let string_count = |counts: &[usize], side: &str| {
        counts
            .iter()
            .try_fold(1usize, |acc, &c| acc.checked_mul(c))
            .ok_or_else(|| {
                QuantumError::DimensionMismatch(format!(
                    "the {side} outcome counts {counts:?} overflow the string count"
                ))
            })
    };
    let n_low = string_count(low_counts, "low")?;
    let n_high = string_count(high_counts, "high")?;
    if matrix.len() != n_low || matrix.iter().any(|row| row.len() != n_high) {
        return Err(QuantumError::DimensionMismatch(format!(
            "the decoder matrix must be {n_low} × {n_high} for outcome counts {low_counts:?} → {high_counts:?}"
        )));
    }
    let tol = 1e-9f64;
    let digits = |mut index: usize, counts: &[usize]| -> Vec<usize> {
        let mut out = vec![0usize; counts.len()];
        for (k, &c) in counts.iter().enumerate().rev() {
            out[k] = index % c;
            index /= c;
        }
        out
    };
    let mut out = QcMorphism::new(1, 1, low_counts.to_vec(), high_counts.to_vec())?;
    for (x, row) in matrix.iter().enumerate() {
        let sum: f64 = row.iter().sum();
        if row.iter().any(|p| !p.is_finite() || *p < 0.0) || (sum - 1.0).abs() > tol {
            return Err(QuantumError::NonCptpChannel(format!(
                "the decoder matrix is not stochastic: row {x} has entries {row:?} summing to {sum}"
            )));
        }
        for (y, &p) in row.iter().enumerate() {
            if p <= 0.0 {
                continue;
            }
            let amplitude = R::from_f64(p.sqrt()).ok_or_else(|| {
                QuantumError::CalculationError("the scalar cannot represent the probability".into())
            })?;
            out.push(
                digits(x, low_counts),
                digits(y, high_counts),
                vec![CausalTensor::from_slice(
                    &[Complex::new(amplitude, R::zero())],
                    &[1, 1],
                )],
            )?;
        }
    }
    Ok(out)
}

/// One failing square of the decoder abstraction.
#[derive(Debug, Clone, PartialEq)]
pub struct DecoderFailure<R> {
    /// The high-level query.
    pub query: Query,
    /// The circuit location the query injects at; `None` for the `Io` square.
    pub location: Option<Fault>,
    /// Whether the model represents the location with a mechanism of its own.
    pub modelled: bool,
    /// The residual.
    pub residual: R,
}

/// The decoder abstraction's report: the naturality report and the failures named by location.
#[derive(Debug, Clone, PartialEq)]
pub struct DecoderReport<R> {
    /// The naturality report over the signature.
    pub naturality: NaturalityReport<R>,
    /// The failing squares, in signature order.
    pub failures: Vec<DecoderFailure<R>>,
}

impl<R: RealField> DecoderReport<R> {
    /// Whether the decoder's model commutes with the circuit on every query.
    pub fn holds(&self) -> bool {
        self.failures.is_empty()
    }

    /// The failure with the largest residual, the first among equals: the location the model
    /// misrepresents most.
    pub fn worst(&self) -> Option<&DecoderFailure<R>> {
        self.failures
            .iter()
            .fold(None, |acc: Option<&DecoderFailure<R>>, f| match acc {
                Some(a) if a.residual >= f.residual => Some(a),
                _ => Some(f),
            })
    }
}

impl<R: RealField + fmt::Debug> fmt::Display for DecoderReport<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "decoder model validation: {} squares, {}",
            self.naturality.report.examined(),
            if self.holds() {
                "every square commutes"
            } else {
                "rejected"
            }
        )?;
        if let Some(worst) = self.worst() {
            match &worst.location {
                Some(loc) => write!(f, "; worst at {loc}")?,
                None => write!(f, "; worst at the nominal square")?,
            }
        }
        for failure in &self.failures {
            match &failure.location {
                Some(loc) => write!(
                    f,
                    "; {} ({}) residual {:?}",
                    loc,
                    if failure.modelled {
                        "modelled"
                    } else {
                        "not in the model"
                    },
                    failure.residual
                )?,
                None => write!(f, "; the nominal square, residual {:?}", failure.residual)?,
            }
        }
        Ok(())
    }
}

/// The attribution of a logical fault: the faults of a set ranked by the residual their injection
/// produces against the model's nominal prediction, largest first.
#[derive(Debug, Clone, PartialEq)]
pub struct Attribution<R> {
    /// `(fault, residual)`, descending by residual, ties in set order.
    pub ranked: Vec<(Fault, R)>,
}

impl<R: RealField> Attribution<R> {
    /// The top-ranked fault.
    pub fn first(&self) -> Option<&(Fault, R)> {
        self.ranked.first()
    }
}

impl<R: RealField + fmt::Debug> fmt::Display for Attribution<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "logical attribution over {} faults:", self.ranked.len())?;
        for (rank, (fault, residual)) in self.ranked.iter().enumerate() {
            writeln!(f, "  {}. {fault}: residual {residual:?}", rank + 1)?;
        }
        Ok(())
    }
}

/// The decoder as an abstraction from a circuit to its detector error model.
#[derive(Debug, Clone)]
pub struct DecoderAbstraction<R: RealField> {
    inner: Abstraction<R, CircuitModel<R>, DemModel>,
    /// Per signature query after `Io`: the circuit location and whether the model represents it.
    locations: Vec<(Fault, bool)>,
}

impl<R> DecoderAbstraction<R>
where
    R: RealField + FromPrimitive + Default + fmt::Debug,
{
    /// The abstraction from `circuit` to `dem` through the decoder's stochastic matrix `tau`
    /// (`tau[x][y]`, low string `x` to high string `y`), over the named circuit `locations`, each a
    /// fault on the circuit with the index of the mechanism representing it or `None` when the
    /// model has none, in which case the model receives a phantom mechanism for it.
    ///
    /// # Errors
    ///
    /// [`stochastic_morphism`]'s errors; [`QuantumError::DimensionMismatch`] on a mechanism index
    /// outside the model; the constructors' errors.
    pub fn new(
        circuit: CircuitModel<R>,
        mut dem: DemModel,
        tau: &[Vec<f64>],
        locations: Vec<(Fault, Option<usize>)>,
    ) -> Result<Self, QuantumError> {
        let low_type = circuit.query_type(&Query::Io)?;
        let high_type = QcModel::<R>::query_type(&dem, &Query::Io)?;
        let morphism = stochastic_morphism::<R>(
            tau,
            &low_type.classical_out_counts,
            &high_type.classical_out_counts,
        )?;
        let alignment = TypeAlignment::new(Vec::new())?.with_classical_output(morphism)?;
        let mut map = vec![(Query::Io, Query::Io)];
        let mut named = Vec::with_capacity(locations.len());
        for (fault, mechanism) in locations {
            let index = match mechanism {
                Some(k) => {
                    if k >= dem.mechanisms().len() {
                        return Err(QuantumError::DimensionMismatch(format!(
                            "location {fault} names mechanism {k}, but the model has {}",
                            dem.mechanisms().len()
                        )));
                    }
                    k
                }
                None => dem.push_phantom()?,
            };
            map.push((
                Query::Fault(Fault::new(None, vec![(index, PauliKind::X)])?),
                Query::Fault(fault.clone()),
            ));
            named.push((fault, mechanism.is_some()));
        }
        Ok(Self {
            inner: Abstraction::new(circuit, dem, alignment, map)?,
            locations: named,
        })
    }

    /// The underlying abstraction.
    pub fn abstraction(&self) -> &Abstraction<R, CircuitModel<R>, DemModel> {
        &self.inner
    }

    /// The circuit locations of the signature after `Io`, with whether the model represents them.
    pub fn locations(&self) -> &[(Fault, bool)] {
        &self.locations
    }

    /// Decoder-model validation: the naturality check over the signature, each failing square
    /// named by its circuit location.
    ///
    /// # Errors
    ///
    /// As [`Abstraction::check_naturality`].
    pub fn check(&self, caps: &NumericCaps) -> Result<DecoderReport<R>, QuantumError> {
        let naturality = self.inner.check_naturality(caps)?;
        let queries = self.inner.signature().queries();
        let mut failures = Vec::new();
        for check in naturality.report.checks() {
            if check.accepted {
                continue;
            }
            let CheckItem::Index(i) = check.item else {
                continue;
            };
            let (location, modelled) = if i == 0 {
                (None, true)
            } else {
                let (fault, modelled) = &self.locations[i - 1];
                (Some(fault.clone()), *modelled)
            };
            failures.push(DecoderFailure {
                query: queries[i].clone(),
                location,
                modelled,
                residual: check.measured,
            });
        }
        Ok(DecoderReport {
            naturality,
            failures,
        })
    }

    /// Logical attribution over a fault set: every fault injected on the circuit against the
    /// model's nominal prediction, ranked by residual.
    ///
    /// # Errors
    ///
    /// As [`Abstraction::check_fault_tolerance`].
    pub fn attribute(
        &self,
        faults: &FaultSet,
        caps: &NumericCaps,
    ) -> Result<Attribution<R>, QuantumError> {
        let report = self.inner.check_fault_tolerance(faults, caps)?;
        let mut ranked: Vec<(Fault, R)> = report
            .records
            .into_iter()
            .map(|r| (r.fault, r.residual))
            .collect();
        // A stable sort keeps ties in set order.
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(core::cmp::Ordering::Equal));
        Ok(Attribution { ranked })
    }

    /// The tolerance a classical square is decided at, for callers reading residuals.
    pub fn tolerance() -> R {
        Tolerance::<R>::state()
            .threshold(1, R::one())
            .unwrap_or_else(|| R::epsilon().sqrt())
    }
}
