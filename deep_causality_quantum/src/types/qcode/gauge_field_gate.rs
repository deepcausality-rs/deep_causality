/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A diagonal logical gate as a function of the logical parities of its blocks.
//!
//! Haruna, *Note on Logical Gates by Gauge Field Formalism of Quantum Error Correction*,
//! arXiv:2511.15224, defines every diagonal gate of Table 1 as
//!
//! ```text
//! O_k(γ₁, …, γ_m) = exp(iπ/2^{k−1} · p₁ ⋯ p_m),   pᵢ = (I − Z̄(γᵢ))/2        (Eq. 3.63)
//! ```
//!
//! with `S̄` (3.14), `CZ̄` (3.37), `C^{m−1}Z̄` (3.49) and `T̄` (3.56) as instances, and derives the
//! physical decompositions of Table 1 by expanding modulo 2. Every such gate lies in the
//! commutative algebra the `m` Paulis `Z̄(γᵢ)` generate, which has dimension `2^m`, so it is fixed by
//! its phase on each of the `2^m` parity patterns. That table is this type. Conjugating a Pauli
//! `P = X^a Z^b` through the gate replaces each `Z̄(γᵢ)` by `(−1)^{⟨a, γᵢ⟩} Z̄(γᵢ)`, so the remainder
//! `(P† G P) G†` is another table on the same blocks, and the representative weight never enters.
//!
//! Phases are exact [`Turns`], reduced modulo one turn.

use crate::QuantumError;
use crate::types::qcode::diagonal_phase::{DiagonalPhase, Turns};
use crate::types::qpu::circuit::GateOp;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_homology::Gf2Chain;
use deep_causality_num::{FromPrimitive, Gf2, NaturalNumber};
use deep_causality_num_complex::Complex;
use deep_causality_num_rational::Rational;

/// The most blocks one gate carries, so the phase table stays `2^m ≤ 2^16` entries.
pub const MAX_GAUGE_BLOCKS: usize = 16;

/// A diagonal gate in the algebra of its blocks' logical `Z̄`s. See the module documentation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GaugeFieldGate<W> {
    len: usize,
    degree: usize,
    blocks: Vec<Gf2Chain<W>>,
    /// `phases[idx]` is the phase in turns on the parity pattern `idx`, bit `i` the parity of
    /// block `i`; reduced to `[0, 1)`.
    phases: Vec<Turns>,
}

/// `t` reduced to `[0, 1)` turns.
pub fn reduce_turns(t: Turns) -> Turns {
    let (mut n, mut d) = (*t.numer(), *t.denom());
    if d < 0 {
        n = -n;
        d = -d;
    }
    Rational::new(n.rem_euclid(d), d)
}

impl<W: NaturalNumber> GaugeFieldGate<W> {
    /// A gate from its blocks and its phase table, one entry per parity pattern.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] if the table has other than `2^m` entries, if the blocks
    /// are over different registers or degrees, or if there are more than [`MAX_GAUGE_BLOCKS`].
    pub fn new(
        len: usize,
        degree: usize,
        blocks: Vec<Gf2Chain<W>>,
        phases: Vec<Turns>,
    ) -> Result<Self, QuantumError> {
        if blocks.len() > MAX_GAUGE_BLOCKS {
            return Err(QuantumError::DimensionMismatch(format!(
                "{} blocks exceed the gate's limit of {MAX_GAUGE_BLOCKS}",
                blocks.len()
            )));
        }
        if phases.len() != 1usize << blocks.len() {
            return Err(QuantumError::DimensionMismatch(format!(
                "a gate on {} blocks needs {} phases, got {}",
                blocks.len(),
                1usize << blocks.len(),
                phases.len()
            )));
        }
        if let Some(b) = blocks
            .iter()
            .find(|b| b.len() != len || b.degree() != degree)
        {
            return Err(QuantumError::DimensionMismatch(format!(
                "a block over {} cells of degree {} in a gate over {len} cells of degree {degree}",
                b.len(),
                b.degree()
            )));
        }
        Ok(Self {
            len,
            degree,
            blocks,
            phases: phases.into_iter().map(reduce_turns).collect(),
        })
    }

    /// The identity: no blocks, phase zero.
    pub fn identity(len: usize, degree: usize) -> Self {
        Self {
            len,
            degree,
            blocks: Vec::new(),
            phases: vec![Rational::new(0, 1)],
        }
    }

    /// `O_k(γ₁, …, γ_m) = exp(iπ/2^{k−1} · p₁⋯p_m)`: `1/2^k` of a turn on the all-odd pattern.
    ///
    /// # Errors
    ///
    /// As [`new`](Self::new); [`QuantumError::DimensionMismatch`] if `blocks` is empty or `k`
    /// exceeds 60.
    pub fn o_k(blocks: Vec<Gf2Chain<W>>, k: u32) -> Result<Self, QuantumError> {
        let first = blocks.first().ok_or_else(|| {
            QuantumError::DimensionMismatch("a gate needs at least one block".into())
        })?;
        if k > 60 {
            return Err(QuantumError::DimensionMismatch(format!(
                "2^{k} does not fit the phase arithmetic"
            )));
        }
        let (len, degree) = (first.len(), first.degree());
        let m = blocks.len();
        let mut phases = vec![Rational::new(0, 1); 1usize << m];
        phases[(1usize << m) - 1] = Rational::new(1, 1i64 << k);
        Self::new(len, degree, blocks, phases)
    }

    /// `Z̄(γ)`, Table 1 row 1.
    pub fn z(gamma: Gf2Chain<W>) -> Result<Self, QuantumError> {
        Self::o_k(vec![gamma], 1)
    }

    /// `S̄(γ)`, Table 1 row 3 and Eq. (3.14).
    pub fn s(gamma: Gf2Chain<W>) -> Result<Self, QuantumError> {
        Self::o_k(vec![gamma], 2)
    }

    /// `T̄(γ)`, Table 1 row 7 and Eq. (3.56).
    pub fn t(gamma: Gf2Chain<W>) -> Result<Self, QuantumError> {
        Self::o_k(vec![gamma], 3)
    }

    /// `CZ̄(γ₁, γ₂)`, Table 1 row 5 and Eq. (3.37).
    pub fn cz(gamma1: Gf2Chain<W>, gamma2: Gf2Chain<W>) -> Result<Self, QuantumError> {
        Self::o_k(vec![gamma1, gamma2], 1)
    }

    /// `C^{m−1}Z̄(γ₁, …, γ_m)`, Table 1 row 6 and Eq. (3.49).
    pub fn multi_cz(blocks: Vec<Gf2Chain<W>>) -> Result<Self, QuantumError> {
        Self::o_k(blocks, 1)
    }

    /// A single-block gate from a [`DiagonalPhase`], which must be a parity function: the
    /// polynomial `Q(n)/M` must agree with `Q(n mod 2)/M` modulo one turn for every `n` up to the
    /// chain's weight. The Table 1 polynomials are, by construction; a polynomial that is not
    /// cannot be carried here.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] naming the first `n` at which the polynomial is not a
    /// parity function.
    pub fn from_diagonal_phase(gate: &DiagonalPhase<W>) -> Result<Self, QuantumError> {
        let even = reduce_turns(gate.phase_at(0));
        let odd = reduce_turns(gate.phase_at(1));
        for n in 2..=(gate.chain().weight() as u64) {
            let expect = if n % 2 == 0 { even } else { odd };
            if reduce_turns(gate.phase_at(n)) != expect {
                return Err(QuantumError::CalculationError(format!(
                    "the phase polynomial is not a parity function: at overlap {n} it gives {:?}, the parity gives {expect:?}",
                    reduce_turns(gate.phase_at(n))
                )));
            }
        }
        Self::new(
            gate.chain().len(),
            gate.chain().degree(),
            vec![gate.chain().clone()],
            vec![even, odd],
        )
    }

    /// A diagonal Clifford program (`Z`, `S`, `S†`, `CZ`, two-qubit `Cmz`) as a gate whose blocks
    /// are the single qubits it touches, for products with logical gates.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] on a gate that is not diagonal Clifford;
    /// [`QuantumError::DimensionMismatch`] beyond the block limit.
    pub fn from_diagonal_clifford(num_qubits: usize, ops: &[GateOp]) -> Result<Self, QuantumError> {
        let mut acc = Self::identity(num_qubits, 1);
        for op in ops {
            let chain = |q: usize| {
                Gf2Chain::<W>::from_support(num_qubits, 1, &[q])
                    .map_err(|e| QuantumError::DimensionMismatch(format!("{e}")))
            };
            let g = match op {
                GateOp::Z(q) => Self::o_k(vec![chain(*q)?], 1)?,
                GateOp::S(q) => Self::o_k(vec![chain(*q)?], 2)?,
                GateOp::Sdg(q) => {
                    let mut g = Self::o_k(vec![chain(*q)?], 2)?;
                    g.phases = g.phases.into_iter().map(|p| reduce_turns(-p)).collect();
                    g
                }
                GateOp::Cz { control, target } => {
                    Self::o_k(vec![chain(*control)?, chain(*target)?], 1)?
                }
                GateOp::Cmz { qubits } if qubits.len() <= 2 => {
                    let blocks = qubits
                        .iter()
                        .map(|&q| chain(q))
                        .collect::<Result<Vec<_>, _>>()?;
                    Self::o_k(blocks, 1)?
                }
                other => {
                    return Err(QuantumError::CalculationError(format!(
                        "{other:?} is not a diagonal Clifford gate"
                    )));
                }
            };
            acc = acc.product(&g)?;
        }
        Ok(acc)
    }

    /// The gate a diagonal physical program implements, read off the program rather than assumed:
    /// the phase of every computational basis state over the gates' support is summed from the
    /// gates that fire on it, and the result must depend on the state only through the parities of
    /// `blocks`. This is how an emitted Table 1 program is checked against the gauge-field
    /// expression it was derived from, and it fails on a program that omits a factor.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] if a gate is not diagonal, acts on a qubit outside the
    /// blocks, or if two basis states with the same block parities carry different phases, naming
    /// the state; [`QuantumError::DimensionMismatch`] if the support exceeds 20 qubits or the
    /// blocks' parities are not independent on it.
    pub fn from_diagonal_program(
        num_qubits: usize,
        ops: &[GateOp],
        blocks: Vec<Gf2Chain<W>>,
    ) -> Result<Self, QuantumError> {
        let first = blocks.first().ok_or_else(|| {
            QuantumError::DimensionMismatch("a gate needs at least one block".into())
        })?;
        let (len, degree) = (first.len(), first.degree());
        if len != num_qubits {
            return Err(QuantumError::DimensionMismatch(format!(
                "the blocks are over {len} qubits, the program over {num_qubits}"
            )));
        }
        let m = blocks.len();
        // Support of the program and the turns each gate contributes when all its qubits are set.
        let mut support: Vec<usize> = Vec::new();
        let mut gates: Vec<(Vec<usize>, Turns)> = Vec::with_capacity(ops.len());
        for op in ops {
            let turns = match op {
                GateOp::Z(_) | GateOp::Cz { .. } | GateOp::Ccz { .. } | GateOp::Cmz { .. } => {
                    Rational::new(1, 2)
                }
                GateOp::S(_) => Rational::new(1, 4),
                GateOp::Sdg(_) | GateOp::Csdg { .. } => Rational::new(3, 4),
                GateOp::T(_) => Rational::new(1, 8),
                GateOp::Tdg(_) => Rational::new(7, 8),
                other => {
                    return Err(QuantumError::CalculationError(format!(
                        "{other:?} is not diagonal; a gauge-field gate reads diagonal programs only"
                    )));
                }
            };
            let qubits = op.qubits();
            for &q in &qubits {
                if q >= num_qubits {
                    return Err(QuantumError::DimensionMismatch(format!(
                        "{op:?} names qubit {q} on a {num_qubits}-qubit register"
                    )));
                }
                if !blocks.iter().any(|b| b.support().any(|s| s == q)) {
                    return Err(QuantumError::CalculationError(format!(
                        "{op:?} acts on qubit {q}, which lies in no block; the phase would depend on \
                         more than the blocks' parities"
                    )));
                }
                if !support.contains(&q) {
                    support.push(q);
                }
            }
            gates.push((qubits, turns));
        }
        support.sort_unstable();
        if support.len() > 20 {
            return Err(QuantumError::DimensionMismatch(format!(
                "the program's support has {} qubits; the enumeration stops at 20",
                support.len()
            )));
        }
        let position = |q: usize| support.iter().position(|&s| s == q).expect("in support");
        let block_masks: Vec<usize> = blocks
            .iter()
            .map(|b| {
                b.support()
                    .filter(|q| support.contains(q))
                    .fold(0usize, |acc, q| acc | (1 << position(q)))
            })
            .collect();
        let gate_masks: Vec<(usize, Turns)> = gates
            .iter()
            .map(|(qs, t)| {
                (
                    qs.iter().fold(0usize, |acc, &q| acc | (1 << position(q))),
                    *t,
                )
            })
            .collect();
        let mut table: Vec<Option<Turns>> = vec![None; 1usize << m];
        for x in 0..(1usize << support.len()) {
            let mut phase = Rational::new(0, 1);
            for (mask, t) in &gate_masks {
                if x & mask == *mask {
                    phase = reduce_turns(phase + *t);
                }
            }
            let mut pattern = 0usize;
            for (i, mask) in block_masks.iter().enumerate() {
                if (x & mask).count_ones() % 2 == 1 {
                    pattern |= 1 << i;
                }
            }
            match table[pattern] {
                None => table[pattern] = Some(phase),
                Some(seen) if seen == phase => {}
                Some(seen) => {
                    return Err(QuantumError::CalculationError(format!(
                        "the program is not a function of the block parities: basis state {x:#b} \
                         over the support carries {phase:?} where pattern {pattern:#b} carried {seen:?}"
                    )));
                }
            }
        }
        // A block the program never touches is a free parity: the phase cannot depend on it, so
        // its patterns copy the pattern with that bit cleared. A block the program touches whose
        // parity cannot be set independently of the others is an error.
        let reach_mask = block_masks
            .iter()
            .enumerate()
            .filter(|(_, mask)| **mask != 0)
            .fold(0usize, |acc, (i, _)| acc | (1 << i));
        let mut phases = Vec::with_capacity(1usize << m);
        for pattern in 0..(1usize << m) {
            let reachable = pattern & reach_mask;
            let t = table[reachable].ok_or_else(|| {
                QuantumError::DimensionMismatch(format!(
                    "parity pattern {reachable:#b} is unreachable on the program's support; the \
                     blocks' parities are not independent there"
                ))
            })?;
            phases.push(t);
        }
        Self::new(len, degree, blocks, phases)
    }

    /// The register width.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the register is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The chain degree of the blocks.
    pub fn degree(&self) -> usize {
        self.degree
    }

    /// The blocks.
    pub fn blocks(&self) -> &[Gf2Chain<W>] {
        &self.blocks
    }

    /// The phase table, one entry per parity pattern, in `[0, 1)` turns.
    pub fn phases(&self) -> &[Turns] {
        &self.phases
    }

    /// The block count `m`.
    pub fn num_blocks(&self) -> usize {
        self.blocks.len()
    }

    /// The phase on a parity pattern, bit `i` the parity of block `i`.
    pub fn phase_at(&self, parities: usize) -> Turns {
        self.phases[parities & ((1usize << self.blocks.len()) - 1)]
    }

    /// Which block parities a Pauli with X-part `x` flips: bit `i` set when `⟨x, γᵢ⟩ = 1`.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] if `x` is over another register.
    pub fn parity_flips(&self, x: &Gf2Chain<W>) -> Result<usize, QuantumError> {
        let mut flips = 0usize;
        for (i, b) in self.blocks.iter().enumerate() {
            let p = x
                .inner(b)
                .map_err(|e| QuantumError::DimensionMismatch(format!("{e}")))?;
            if p == Gf2::ONE {
                flips |= 1 << i;
            }
        }
        Ok(flips)
    }

    /// The remainder `(P† G P) G†` of conjugating a Pauli with X-part `x` through this gate:
    /// the table `φ(idx ⊕ flips) − φ(idx)`. Constant, and then the identity up to a global phase,
    /// exactly when no flipped block's parity enters the phase.
    ///
    /// # Errors
    ///
    /// As [`parity_flips`](Self::parity_flips).
    pub fn conjugated_by_pauli(&self, x: &Gf2Chain<W>) -> Result<Self, QuantumError> {
        let flips = self.parity_flips(x)?;
        let phases = (0..self.phases.len())
            .map(|idx| reduce_turns(self.phases[idx ^ flips] - self.phases[idx]))
            .collect();
        Ok(Self {
            len: self.len,
            degree: self.degree,
            blocks: self.blocks.clone(),
            phases,
        })
    }

    /// Whether every parity pattern carries the same phase, so the gate is a scalar.
    pub fn is_constant(&self) -> bool {
        self.phases.iter().all(|p| *p == self.phases[0])
    }

    /// The product of two gates: the union of their blocks, phases added pattern by pattern.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] on different registers or beyond the block limit.
    pub fn product(&self, other: &Self) -> Result<Self, QuantumError> {
        if self.len != other.len
            || (self.degree != other.degree && !self.blocks.is_empty() && !other.blocks.is_empty())
        {
            return Err(QuantumError::DimensionMismatch(format!(
                "gates over {} and {} cells cannot be multiplied",
                self.len, other.len
            )));
        }
        let mut blocks: Vec<Gf2Chain<W>> = self.blocks.clone();
        let mut other_map = Vec::with_capacity(other.blocks.len());
        for b in &other.blocks {
            match blocks.iter().position(|c| c == b) {
                Some(p) => other_map.push(p),
                None => {
                    blocks.push(b.clone());
                    other_map.push(blocks.len() - 1);
                }
            }
        }
        if blocks.len() > MAX_GAUGE_BLOCKS {
            return Err(QuantumError::DimensionMismatch(format!(
                "the product would carry {} blocks, above the limit of {MAX_GAUGE_BLOCKS}",
                blocks.len()
            )));
        }
        let m = blocks.len();
        let self_m = self.blocks.len();
        let mut phases = Vec::with_capacity(1usize << m);
        for idx in 0..(1usize << m) {
            let self_idx = idx & ((1usize << self_m) - 1);
            let mut other_idx = 0usize;
            for (j, &p) in other_map.iter().enumerate() {
                if idx & (1 << p) != 0 {
                    other_idx |= 1 << j;
                }
            }
            phases.push(reduce_turns(
                self.phases[self_idx] + other.phases[other_idx],
            ));
        }
        let degree = if self.blocks.is_empty() {
            other.degree
        } else {
            self.degree
        };
        Ok(Self {
            len: self.len,
            degree,
            blocks,
            phases,
        })
    }

    /// The gate's expansion in the Paulis `Z̄(γ_S)`, `S` a subset of the blocks as a bitmask:
    /// `c_S = 2^{−m} Σ_ε e^{2πi φ(ε)} (−1)^{|S ∩ ε|}`. Every `S` is returned; a caller filters by
    /// modulus. Numeric, for reports; the decision path compares the table itself.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] if the scalar cannot represent a phase.
    pub fn pauli_coefficients<R>(&self) -> Result<Vec<(usize, Complex<R>)>, QuantumError>
    where
        R: RealField + FromPrimitive,
    {
        let m = self.blocks.len();
        let size = 1usize << m;
        let two_pi = R::from_f64(core::f64::consts::TAU).ok_or_else(|| {
            QuantumError::CalculationError("the scalar cannot represent 2π".into())
        })?;
        let scale = R::from_usize(size).ok_or_else(|| {
            QuantumError::CalculationError("the scalar cannot represent the pattern count".into())
        })?;
        let unit: Vec<Complex<R>> = self
            .phases
            .iter()
            .map(|p| {
                let t = R::from_f64(*p.numer() as f64 / *p.denom() as f64).ok_or_else(|| {
                    QuantumError::CalculationError("the scalar cannot represent a phase".into())
                })?;
                let angle = two_pi * t;
                Ok(Complex::new(angle.cos(), angle.sin()))
            })
            .collect::<Result<_, QuantumError>>()?;
        let mut out = Vec::with_capacity(size);
        for s in 0..size {
            let mut acc = Complex::new(R::zero(), R::zero());
            for (eps, u) in unit.iter().enumerate() {
                let sign = if (s & eps).count_ones() % 2 == 0 {
                    R::one()
                } else {
                    -R::one()
                };
                acc += *u * Complex::new(sign, R::zero());
            }
            out.push((s, acc * Complex::new(R::one() / scale, R::zero())));
        }
        Ok(out)
    }
}
