/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Fault sets: finite signatures of low-level fault queries, Lorenz & Tull's combs that are not
//! Do-queries, counted and capped before anything is allocated.
//!
//! A fault is a Pauli error inserted at a location of the low-level circuit: after a named node,
//! or before the first box when no node is named. `pauli_weight(t)` enumerates every Pauli of
//! weight exactly `t` on the given locations, `C(n, t) · 3^t` faults, and refuses above a cap with
//! the count and the cap in the error. Realistic sets come from a detector error model's
//! mechanisms through [`FaultSet::from_dem`], which takes the mechanisms' Pauli supports as the
//! model lists them.

use crate::QuantumError;
use crate::types::abstraction::query::Query;
use crate::types::circuit_model::{NodeId, WireId};
use crate::types::qcode::logical_pauli::LogicalPauli;
use crate::types::qpu::circuit::GateOp;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use deep_causality_homology::Gf2Chain;
use deep_causality_num::NaturalNumber;

/// The default cap on the number of faults a set may enumerate.
pub const FAULT_SET_CAP: u64 = 1 << 16;

/// A single-qubit Pauli error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PauliKind {
    /// A bit flip.
    X,
    /// A bit and phase flip.
    Y,
    /// A phase flip.
    Z,
}

impl PauliKind {
    /// The three kinds, in order.
    pub const ALL: [PauliKind; 3] = [PauliKind::X, PauliKind::Y, PauliKind::Z];

    /// The gate applying this error on local qubit `q`.
    pub fn gate(self, q: usize) -> GateOp {
        match self {
            PauliKind::X => GateOp::X(q),
            PauliKind::Y => GateOp::Y(q),
            PauliKind::Z => GateOp::Z(q),
        }
    }

    /// Whether the error has an X part.
    pub fn has_x(self) -> bool {
        matches!(self, PauliKind::X | PauliKind::Y)
    }

    /// Whether the error has a Z part.
    pub fn has_z(self) -> bool {
        matches!(self, PauliKind::Z | PauliKind::Y)
    }

    /// The letter.
    pub fn letter(self) -> char {
        match self {
            PauliKind::X => 'X',
            PauliKind::Y => 'Y',
            PauliKind::Z => 'Z',
        }
    }
}

/// One fault: a Pauli error on distinct wires, inserted after a node of the low-level circuit or,
/// with no node named, before its first box.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Fault {
    after: Option<NodeId>,
    errors: Vec<(WireId, PauliKind)>,
}

impl Fault {
    /// A fault from its errors, sorted by wire.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] if a wire is named twice or the fault is empty.
    pub fn new(
        after: Option<NodeId>,
        mut errors: Vec<(WireId, PauliKind)>,
    ) -> Result<Self, QuantumError> {
        if errors.is_empty() {
            return Err(QuantumError::DimensionMismatch(
                "a fault needs at least one error".into(),
            ));
        }
        errors.sort_unstable();
        if let Some(w) = errors.windows(2).find(|p| p[0].0 == p[1].0) {
            return Err(QuantumError::DimensionMismatch(format!(
                "wire {} carries two errors in one fault",
                w[0].0
            )));
        }
        Ok(Self { after, errors })
    }

    /// The node the error follows, if any.
    pub fn after(&self) -> Option<NodeId> {
        self.after
    }

    /// The errors, sorted by wire.
    pub fn errors(&self) -> &[(WireId, PauliKind)] {
        &self.errors
    }

    /// The number of wires with an error.
    pub fn weight(&self) -> usize {
        self.errors.len()
    }

    /// The wires with an error, ascending.
    pub fn wires(&self) -> Vec<WireId> {
        self.errors.iter().map(|(w, _)| *w).collect()
    }

    /// The gate program applying the errors on the fault's wires in ascending order, with local
    /// qubit indices.
    pub fn program(&self) -> Vec<GateOp> {
        self.errors
            .iter()
            .enumerate()
            .map(|(i, (_, p))| p.gate(i))
            .collect()
    }

    /// The fault as a Pauli over `n` qubits, wire `w` being qubit `w`.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] if a wire is not below `n`.
    pub fn as_logical_pauli<W: NaturalNumber>(
        &self,
        n: usize,
    ) -> Result<LogicalPauli<W>, QuantumError> {
        if let Some((w, _)) = self.errors.iter().find(|(w, _)| *w >= n) {
            return Err(QuantumError::DimensionMismatch(format!(
                "the fault names wire {w}, but the register has {n} qubits"
            )));
        }
        let xs: Vec<usize> = self
            .errors
            .iter()
            .filter(|(_, p)| p.has_x())
            .map(|(w, _)| *w)
            .collect();
        let zs: Vec<usize> = self
            .errors
            .iter()
            .filter(|(_, p)| p.has_z())
            .map(|(w, _)| *w)
            .collect();
        let x = Gf2Chain::from_support(n, 1, &xs)
            .map_err(|e| QuantumError::DimensionMismatch(format!("{e}")))?;
        let z = Gf2Chain::from_support(n, 1, &zs)
            .map_err(|e| QuantumError::DimensionMismatch(format!("{e}")))?;
        LogicalPauli::new(x, z)
    }

    /// A short name, `X3 Z5 after node 1`.
    pub fn name(&self) -> String {
        let mut s = String::new();
        for (i, (w, p)) in self.errors.iter().enumerate() {
            if i > 0 {
                s.push(' ');
            }
            s.push(p.letter());
            s.push_str(&format!("{w}"));
        }
        match self.after {
            Some(n) => format!("{s} after node {n}"),
            None => format!("{s} at the inputs"),
        }
    }
}

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name())
    }
}

/// Which constructor produced a fault set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FaultOrigin {
    /// Every Pauli of the given weight on the locations.
    PauliWeight(usize),
    /// Faults listed by the caller.
    Declared,
    /// The mechanisms of a detector error model.
    Dem,
}

impl fmt::Display for FaultOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FaultOrigin::PauliWeight(t) => write!(f, "pauli_weight({t})"),
            FaultOrigin::Declared => f.write_str("declared"),
            FaultOrigin::Dem => f.write_str("from_dem"),
        }
    }
}

/// A finite signature of fault queries over the low-level model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaultSet {
    faults: Vec<Fault>,
    origin: FaultOrigin,
}

impl FaultSet {
    /// The number of faults `pauli_weight(t)` enumerates on `n` locations, `C(n, t) · 3^t`, or
    /// `None` when it does not fit in a `u64`.
    pub fn count_pauli_weight(n: usize, t: usize) -> Option<u64> {
        if t > n {
            return Some(0);
        }
        let mut binom: u64 = 1;
        for i in 0..t {
            // C(n, i + 1) = C(n, i) · (n − i) / (i + 1), exact at every step.
            binom = binom.checked_mul((n - i) as u64)? / (i as u64 + 1);
        }
        let mut power: u64 = 1;
        for _ in 0..t {
            power = power.checked_mul(3)?;
        }
        binom.checked_mul(power)
    }

    /// Every Pauli of weight exactly `t` on the locations, inserted after `after`, refused above
    /// `cap` before any fault is allocated.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] naming the count and the cap when the count exceeds the
    /// cap or overflows; [`QuantumError::DimensionMismatch`] on repeated locations or `t = 0`.
    pub fn pauli_weight(
        locations: &[WireId],
        after: Option<NodeId>,
        t: usize,
        cap: u64,
    ) -> Result<Self, QuantumError> {
        if t == 0 {
            return Err(QuantumError::DimensionMismatch(
                "pauli_weight needs a weight of at least one".into(),
            ));
        }
        let distinct: BTreeSet<WireId> = locations.iter().copied().collect();
        if distinct.len() != locations.len() {
            return Err(QuantumError::DimensionMismatch(
                "the fault locations repeat a wire".into(),
            ));
        }
        let n = locations.len();
        let count = Self::count_pauli_weight(n, t).ok_or_else(|| {
            QuantumError::CalculationError(format!(
                "pauli_weight({t}) on {n} locations: C({n}, {t}) · 3^{t} overflows the count; the cap is {cap}"
            ))
        })?;
        if count > cap {
            return Err(QuantumError::CalculationError(format!(
                "pauli_weight({t}) on {n} locations enumerates {count} faults, above the cap {cap}"
            )));
        }
        let sorted: Vec<WireId> = distinct.into_iter().collect();
        let mut faults = Vec::with_capacity(count as usize);
        for subset in combinations(n, t) {
            for pattern in 0..3usize.pow(t as u32) {
                let mut p = pattern;
                let mut errors = Vec::with_capacity(t);
                for &i in &subset {
                    errors.push((sorted[i], PauliKind::ALL[p % 3]));
                    p /= 3;
                }
                faults.push(Fault::new(after, errors)?);
            }
        }
        Ok(Self {
            faults,
            origin: FaultOrigin::PauliWeight(t),
        })
    }

    /// The listed faults, in order, duplicates removed.
    pub fn declared(faults: &[Fault]) -> Self {
        let mut seen = BTreeSet::new();
        let faults = faults
            .iter()
            .filter(|f| seen.insert((*f).clone()))
            .cloned()
            .collect();
        Self {
            faults,
            origin: FaultOrigin::Declared,
        }
    }

    /// One fault per mechanism of a detector error model, each mechanism given as the wires and
    /// Paulis it flips; empty mechanisms are skipped and duplicates removed.
    ///
    /// # Errors
    ///
    /// [`Fault::new`]'s errors.
    pub fn from_dem(
        mechanisms: &[Vec<(WireId, PauliKind)>],
        after: Option<NodeId>,
    ) -> Result<Self, QuantumError> {
        let mut seen = BTreeSet::new();
        let mut faults = Vec::new();
        for m in mechanisms.iter().filter(|m| !m.is_empty()) {
            let f = Fault::new(after, m.clone())?;
            if seen.insert(f.clone()) {
                faults.push(f);
            }
        }
        Ok(Self {
            faults,
            origin: FaultOrigin::Dem,
        })
    }

    /// The faults.
    pub fn faults(&self) -> &[Fault] {
        &self.faults
    }

    /// The constructor.
    pub fn origin(&self) -> FaultOrigin {
        self.origin
    }

    /// The number of faults.
    pub fn len(&self) -> usize {
        self.faults.len()
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.faults.is_empty()
    }

    /// The faults as low-level queries.
    pub fn queries(&self) -> Vec<Query> {
        self.faults.iter().cloned().map(Query::Fault).collect()
    }
}

impl fmt::Display for FaultSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({} faults)", self.origin, self.faults.len())
    }
}

/// The `k`-subsets of `0..n`, ascending.
fn combinations(n: usize, k: usize) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    if k > n {
        return out;
    }
    let mut idx: Vec<usize> = (0..k).collect();
    loop {
        out.push(idx.clone());
        let mut i = k;
        loop {
            if i == 0 {
                return out;
            }
            i -= 1;
            if idx[i] < n - k + i {
                idx[i] += 1;
                for j in i + 1..k {
                    idx[j] = idx[j - 1] + 1;
                }
                break;
            }
        }
    }
}
