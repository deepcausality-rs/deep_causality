/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Fault tolerance as the naturality check over the signature enlarged by a fault set.
//!
//! On the exact path a fault is a Pauli pushed through the gate's program by
//! [`ExactProgram::conjugate`]: a Clifford layer through the tableau, a diagonal Table 1 gate in
//! the algebra of its logical `Z̄(γᵢ)`s, where the remainder `(P† G P) G†` is a [`GaugeFieldGate`]
//! on the same blocks (Haruna, arXiv:2511.15224, Eq. 3.63; design D7). After the fault's own Pauli
//! is recovered, the remainder acts on the code space as `exp(2πi · Δg(p̂))` with `p̂` the logical
//! parity operators, and since the blocks are independent homology classes it is a non-trivial
//! logical unitary exactly when `Δg` is not constant on `{0,1}^m`. A fault is therefore tolerated
//! when the remainder is constant and the propagated Pauli has no more weight than the fault had,
//! so that a recovery built for the fault set's weight still corrects it. The witness is the
//! flipped parity pattern with the remainder's phase table, or the propagated Pauli and the logical
//! operator it carries.
//!
//! On the numeric path a fault is a `Query::Fault` on the low-level circuit against the high-level
//! `Io`, compared as every naturality square is, in Frobenius norm on the Choi operators. The two
//! paths answer neighbouring questions: the exact path asks whether the gate spreads the fault, the
//! numeric one whether the abstraction's `τ` absorbs the spread fault. They agree when `τ` corrects
//! every error of the set's weight.
//!
//! No threshold, distance or asymptotic suppression is claimed: a report states its fault set, its
//! residual or remainder per fault, and the path that decided each.
//!
//! [`GaugeFieldGate`]: crate::types::qcode::gauge_field_gate::GaugeFieldGate

use crate::QuantumError;
use crate::types::abstraction::abstraction::Abstraction;
use crate::types::abstraction::code_abstraction::{CodeAbstraction, LogicalGate};
use crate::types::abstraction::fault_set::{FAULT_SET_CAP, Fault, FaultOrigin, FaultSet};
use crate::types::abstraction::qc_model::QcModel;
use crate::types::abstraction::query::Query;
use crate::types::circuit_model::{CircuitModel, ExactProgram, NumericCaps, SemanticsPath};
use crate::types::decision::{Check, CheckItem, CheckReport, Tolerance};
use crate::types::qcode::logical_pauli::LogicalPauli;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use deep_causality_algebra::RealField;
use deep_causality_num::{FromPrimitive, NaturalNumber};

/// One fault's verdict.
#[derive(Debug, Clone, PartialEq)]
pub struct FaultRecord<R> {
    /// The fault.
    pub fault: Fault,
    /// Whether the fault is tolerated.
    pub tolerated: bool,
    /// The path that decided it.
    pub path: SemanticsPath,
    /// Zero or one on the exact path; the Frobenius residual on the numeric path.
    pub residual: R,
    /// The Pauli terms of the remainder on the exact path, one for the identity; zero on the
    /// numeric path.
    pub remainder_terms: usize,
    /// The weight of the propagated Pauli on the exact path.
    pub propagated_weight: Option<usize>,
    /// Why the fault is not tolerated, when it is not.
    pub witness: Option<String>,
}

/// The fault-tolerance report of one gate or one abstraction under a fault set.
#[derive(Debug, Clone, PartialEq)]
pub struct FaultToleranceReport<R> {
    /// One check per fault, in the set's order.
    pub report: CheckReport<R>,
    /// One record per fault, in the set's order.
    pub records: Vec<FaultRecord<R>>,
    /// The fault set's constructor.
    pub origin: FaultOrigin,
    /// The fault set's size.
    pub count: usize,
}

impl<R: RealField> FaultToleranceReport<R> {
    /// The record with the largest residual, the first among equals.
    pub fn worst(&self) -> Option<&FaultRecord<R>> {
        self.records
            .iter()
            .fold(None, |acc: Option<&FaultRecord<R>>, r| match acc {
                Some(a) if a.residual >= r.residual => Some(a),
                _ => Some(r),
            })
    }

    /// How many faults are tolerated.
    pub fn tolerated(&self) -> usize {
        self.records.iter().filter(|r| r.tolerated).count()
    }

    /// Whether every fault is tolerated; an empty set holds vacuously.
    pub fn holds(&self) -> bool {
        self.report.accepted()
    }

    /// How many records each path decided, `(exact, numeric)`.
    pub fn paths(&self) -> (usize, usize) {
        let exact = self
            .records
            .iter()
            .filter(|r| r.path == SemanticsPath::Exact)
            .count();
        (exact, self.records.len() - exact)
    }
}

impl<R: RealField + fmt::Debug> fmt::Display for FaultToleranceReport<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (exact, numeric) = self.paths();
        write!(
            f,
            "fault tolerance under {} ({} faults): {} tolerated, {} not; verdict {:?}; paths {exact} exact, {numeric} numeric",
            self.origin,
            self.count,
            self.tolerated(),
            self.records.len() - self.tolerated(),
            self.report.verdict()
        )?;
        if let Some(w) = self.worst() {
            write!(f, "; worst residual {:?} at {}", w.residual, w.fault)?;
            if let Some(wit) = &w.witness {
                write!(f, ": {wit}")?;
            }
        }
        Ok(())
    }
}

/// The Haruna filter: which Table 1 gates of a code tolerate every weight-one Pauli fault.
#[derive(Debug, Clone, PartialEq)]
pub struct HarunaFilter<R> {
    /// Every gate's report, in the abstraction's gate order.
    pub verdicts: Vec<(LogicalGate, FaultToleranceReport<R>)>,
    /// The gates that hold.
    pub holds: Vec<LogicalGate>,
}

impl<R: RealField + fmt::Debug> fmt::Display for HarunaFilter<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (gate, report) in &self.verdicts {
            let (exact, numeric) = report.paths();
            let path = if numeric == 0 {
                "exact"
            } else if exact == 0 {
                "numeric"
            } else {
                "mixed"
            };
            writeln!(
                f,
                "{}: {} ({} of {} faults tolerated, {path})",
                gate.name(),
                if report.holds() { "holds" } else { "fails" },
                report.tolerated(),
                report.count
            )?;
        }
        Ok(())
    }
}

/// The number of qubits a Pauli acts on.
fn pauli_weight<W: NaturalNumber>(p: &LogicalPauli<W>) -> usize {
    let support: BTreeSet<usize> = p.x().support().chain(p.z().support()).collect();
    support.len()
}

fn turns_table(phases: &[crate::types::qcode::diagonal_phase::Turns]) -> String {
    let parts: Vec<String> = phases
        .iter()
        .map(|t| format!("{}/{}", t.numer(), t.denom()))
        .collect();
    format!("[{}]", parts.join(", "))
}

impl<W: NaturalNumber> CodeAbstraction<W> {
    /// The gate's program in the propagator's normal form: a diagonal gate as its `O_k` on its
    /// blocks, `X̄` and `H̄` as Clifford layers of their emitted programs.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] if the gate names a logical qubit the code does not
    /// have; the emitters' errors.
    pub fn exact_program<R>(&self, gate: &LogicalGate) -> Result<ExactProgram<W>, QuantumError>
    where
        R: RealField + FromPrimitive,
    {
        self.check_gate(gate)?;
        match self.expected_gauge(gate) {
            Some(g) => Ok(ExactProgram::diagonal(g?)),
            None => ExactProgram::clifford(self.basis().len(), self.program::<R>(gate)?),
        }
    }

    /// The exact fault-tolerance check of one gate under a fault set: every fault pushed through
    /// the gate's program, tolerated when the remainder is constant and the propagated Pauli has no
    /// more weight than the fault. Every record reads `SemanticsPath::Exact`.
    ///
    /// # Errors
    ///
    /// [`QuantumError::NoPropagationNormalForm`] if the program leaves the normal form; the
    /// emitters' and the basis's errors; a fault over the wrong register.
    pub fn check_fault_tolerance<R>(
        &self,
        gate: &LogicalGate,
        faults: &FaultSet,
    ) -> Result<FaultToleranceReport<R>, QuantumError>
    where
        R: RealField + FromPrimitive,
    {
        let n = self.basis().len();
        let program = self.exact_program::<R>(gate)?;
        let mut checks = Vec::with_capacity(faults.len());
        let mut records = Vec::with_capacity(faults.len());
        for (i, fault) in faults.faults().iter().enumerate() {
            let pauli = fault.as_logical_pauli::<W>(n)?;
            let propagated = program.conjugate(&pauli)?;
            let constant = propagated.remainder.is_constant();
            let weight = pauli_weight(&propagated.pauli);
            let within = weight <= fault.weight();
            let tolerated = constant && within;
            let remainder_terms = if constant {
                1
            } else {
                let eps = R::epsilon().sqrt();
                propagated
                    .remainder
                    .pauli_coefficients::<R>()?
                    .iter()
                    .filter(|(_, c)| (c.re * c.re + c.im * c.im).sqrt() > eps)
                    .count()
            };
            let witness = if tolerated {
                None
            } else if !constant {
                Some(format!(
                    "the remainder on flipped parities {:#b} has the phase table {}, a non-trivial logical unitary",
                    propagated.remainder.parity_flips(pauli.x())?,
                    turns_table(propagated.remainder.phases())
                ))
            } else {
                let carried = propagated.pauli.compose(&pauli)?;
                let logical = match self.basis().is_logically_trivial(&carried) {
                    Ok(true) => "a stabilizer",
                    Ok(false) => "a non-trivial logical operator",
                    Err(_) => "an operator outside the normalizer",
                };
                Some(format!(
                    "propagates to a Pauli of weight {weight} on X {:?} Z {:?}, carrying {logical}",
                    propagated.pauli.x().support().collect::<Vec<_>>(),
                    propagated.pauli.z().support().collect::<Vec<_>>()
                ))
            };
            let residual = if tolerated { R::zero() } else { R::one() };
            checks.push(Check::new(CheckItem::Index(i), residual, R::zero()));
            records.push(FaultRecord {
                fault: fault.clone(),
                tolerated,
                path: SemanticsPath::Exact,
                residual,
                remainder_terms,
                propagated_weight: Some(weight),
                witness,
            });
        }
        Ok(FaultToleranceReport {
            report: CheckReport::from_checks(checks),
            records,
            origin: faults.origin(),
            count: faults.len(),
        })
    }

    /// The Haruna filter: `check_fault_tolerance` under `pauli_weight(1)` on every physical qubit,
    /// for every gate of the abstraction, and the subset that holds. The faults sit between the
    /// encoder and the program of the Example 58 model, after its node 0, which the exact path does
    /// not consult.
    ///
    /// # Errors
    ///
    /// As [`check_fault_tolerance`](Self::check_fault_tolerance).
    pub fn haruna_filter<R>(&self) -> Result<HarunaFilter<R>, QuantumError>
    where
        R: RealField + FromPrimitive,
    {
        let n = self.basis().len();
        let locations: Vec<usize> = (0..n).collect();
        let faults = FaultSet::pauli_weight(&locations, Some(0), 1, FAULT_SET_CAP)?;
        let mut verdicts = Vec::with_capacity(self.gates().len());
        let mut holds = Vec::new();
        for gate in self.gates() {
            let report = self.check_fault_tolerance::<R>(gate, &faults)?;
            if report.holds() {
                holds.push(gate.clone());
            }
            verdicts.push((gate.clone(), report));
        }
        Ok(HarunaFilter { verdicts, holds })
    }
}

impl<R, H> Abstraction<R, CircuitModel<R>, H>
where
    R: RealField + FromPrimitive + Default + fmt::Debug,
    H: QcModel<R>,
{
    /// The numeric fault-tolerance check: every fault as a `Query::Fault` on the low-level
    /// circuit against the high-level `Io`, compared as a naturality square. Every record reads
    /// `SemanticsPath::Numeric`.
    ///
    /// # Errors
    ///
    /// The semantics' caps and the alignment's errors; a fault on a wire that is not quantum.
    pub fn check_fault_tolerance(
        &self,
        faults: &FaultSet,
        caps: &NumericCaps,
    ) -> Result<FaultToleranceReport<R>, QuantumError> {
        let mut checks = Vec::with_capacity(faults.len());
        let mut records = Vec::with_capacity(faults.len());
        for (i, fault) in faults.faults().iter().enumerate() {
            let low_q = Query::Fault(fault.clone());
            let (left, right) = self.square_with(&Query::Io, &low_q, caps)?;
            let (residual, _) = left.frobenius_distance(&right, caps)?;
            let dim = left.d_in() * left.d_out();
            let tolerance = Tolerance::<R>::state()
                .threshold(dim, R::one())
                .unwrap_or_else(|| R::epsilon().sqrt());
            let check = Check::new(CheckItem::Index(i), residual, tolerance);
            let tolerated = check.accepted;
            checks.push(check);
            records.push(FaultRecord {
                fault: fault.clone(),
                tolerated,
                path: SemanticsPath::Numeric,
                residual,
                remainder_terms: 0,
                propagated_weight: None,
                witness: (!tolerated).then(|| {
                    format!(
                        "the faulted square has residual {residual:?} against {tolerance:?} in frobenius-on-choi"
                    )
                }),
            });
        }
        Ok(FaultToleranceReport {
            report: CheckReport::from_checks(checks),
            records,
            origin: faults.origin(),
            count: faults.len(),
        })
    }
}
