/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::carriers::Channel;
use crate::types::decision::Tolerance;
use crate::types::qgates::channel::{choi_from_kraus, kraus_from_choi};
use crate::types::qgates::operator_linalg::frobenius_norm;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::{CausalTensor, Tensor};

/// The two caps of the numeric semantics, both counted and refused before allocating.
///
/// `max_entries` bounds the composite Choi operator, `2^(2n + 2k)` entries for a channel from `n`
/// to `k` qubits, and also the working storage of a program evaluation. `max_operators` bounds the
/// Kraus family a program carries, the product of the operator counts of its noise boxes and the
/// outcome counts of its measurements. A fault-set query inserts one error channel of at most four
/// operators, so the fault path stays far below the second cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumericCaps {
    /// Default `2^24`, so `n + k ≤ 12`.
    pub max_entries: u64,
    /// Default `2^12`.
    pub max_operators: u64,
}

impl Default for NumericCaps {
    fn default() -> Self {
        Self {
            max_entries: 1 << 24,
            max_operators: 1 << 12,
        }
    }
}

/// Classical values on a list of classical wires, in wire order.
pub type ClassicalValues = Vec<usize>;

/// A morphism of QC carried at the Kraus level: for each pair of classical input values `x` and
/// classical output values `y`, a family of Kraus operators `d_out × d_in`, the completely positive
/// map `f(y | x)` of Lorenz & Tull's Example 57. The Choi operator of a block is formed only when
/// asked for, under the entry cap.
///
/// A channel is the block `((), ())`; a classical stochastic matrix is one scalar per `(x, y)`; a
/// circuit with measurements is one block per outcome string. Two morphisms are compared by the
/// Frobenius norm over the direct sum of their block Choi operators, a block missing on one side
/// counting as zero.
#[derive(Debug, Clone, PartialEq)]
pub struct QcMorphism<R: RealField> {
    d_in: usize,
    d_out: usize,
    classical_in: Vec<usize>,
    classical_out: Vec<usize>,
    blocks: BTreeMap<(ClassicalValues, ClassicalValues), Vec<CausalTensor<Complex<R>>>>,
}

impl<R> QcMorphism<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    /// An empty morphism of the given type: quantum dimensions and the outcome counts of the
    /// classical input and output wires.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] on a zero dimension or a zero outcome count.
    pub fn new(
        d_in: usize,
        d_out: usize,
        classical_in: Vec<usize>,
        classical_out: Vec<usize>,
    ) -> Result<Self, QuantumError> {
        if d_in == 0 || d_out == 0 {
            return Err(QuantumError::DimensionMismatch(
                "a morphism needs non-zero quantum dimensions".into(),
            ));
        }
        if classical_in.iter().chain(&classical_out).any(|&c| c == 0) {
            return Err(QuantumError::DimensionMismatch(
                "a classical wire needs at least one outcome".into(),
            ));
        }
        Ok(Self {
            d_in,
            d_out,
            classical_in,
            classical_out,
            blocks: BTreeMap::new(),
        })
    }

    /// A purely quantum morphism from a Kraus family.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] on an empty family or on operators of unequal or
    /// non-matrix shape.
    pub fn from_kraus(kraus: &[CausalTensor<Complex<R>>]) -> Result<Self, QuantumError> {
        let first = kraus.first().ok_or_else(|| {
            QuantumError::DimensionMismatch("a morphism needs at least one Kraus operator".into())
        })?;
        let shape = first.shape();
        if shape.len() != 2 {
            return Err(QuantumError::DimensionMismatch(format!(
                "Kraus operators must be matrices, got shape {shape:?}"
            )));
        }
        let mut me = Self::new(shape[1], shape[0], Vec::new(), Vec::new())?;
        me.push(Vec::new(), Vec::new(), kraus.to_vec())?;
        Ok(me)
    }

    /// A purely quantum morphism from a [`Channel`], through its Kraus family when it holds one and
    /// through `kraus_from_choi` at the numerical-rank tolerance otherwise.
    ///
    /// # Errors
    ///
    /// The errors of `kraus_from_choi` on a channel that holds only its Choi operator.
    pub fn from_channel(channel: &Channel<R>) -> Result<Self, QuantumError> {
        match channel.kraus() {
            Some(k) => Self::from_kraus(k),
            None => {
                let d = channel.d_in() * channel.d_out();
                let tol = Tolerance::<R>::numerical_rank()
                    .threshold(d, R::one())
                    .ok_or_else(|| {
                        QuantumError::CalculationError(
                            "the numerical-rank member has no threshold at this dimension".into(),
                        )
                    })?;
                let kraus = kraus_from_choi(channel.choi(), channel.d_in(), channel.d_out(), tol)?;
                Self::from_kraus(&kraus)
            }
        }
    }

    /// The Kraus operators of every block, in block order.
    pub fn kraus(&self) -> Vec<CausalTensor<Complex<R>>> {
        self.blocks.values().flatten().cloned().collect()
    }

    /// The identity morphism on a `d`-dimensional quantum system.
    pub fn identity(d: usize) -> Result<Self, QuantumError> {
        let mut data = vec![Complex::new(R::zero(), R::zero()); d * d];
        for i in 0..d {
            data[i * d + i] = Complex::new(R::one(), R::zero());
        }
        Self::from_kraus(&[CausalTensor::from_slice(&data, &[d, d])])
    }

    /// Appends Kraus operators to the block `(x, y)`.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] if `x` or `y` has the wrong length or a value at or
    /// above its wire's outcome count, or if an operator is not `d_out × d_in`.
    pub fn push(
        &mut self,
        x: ClassicalValues,
        y: ClassicalValues,
        kraus: Vec<CausalTensor<Complex<R>>>,
    ) -> Result<(), QuantumError> {
        check_values(&x, &self.classical_in, "input")?;
        check_values(&y, &self.classical_out, "output")?;
        for k in &kraus {
            let shape = k.shape();
            if shape.len() != 2 || shape[0] != self.d_out || shape[1] != self.d_in {
                return Err(QuantumError::DimensionMismatch(format!(
                    "a Kraus operator of shape {shape:?} does not fit a {} × {} morphism",
                    self.d_out, self.d_in
                )));
            }
        }
        self.blocks.entry((x, y)).or_default().extend(kraus);
        Ok(())
    }

    /// The quantum input dimension.
    pub fn d_in(&self) -> usize {
        self.d_in
    }

    /// The quantum output dimension.
    pub fn d_out(&self) -> usize {
        self.d_out
    }

    /// The outcome counts of the classical input wires.
    pub fn classical_in(&self) -> &[usize] {
        &self.classical_in
    }

    /// The outcome counts of the classical output wires.
    pub fn classical_out(&self) -> &[usize] {
        &self.classical_out
    }

    /// The blocks, keyed by `(x, y)`.
    pub fn blocks(
        &self,
    ) -> &BTreeMap<(ClassicalValues, ClassicalValues), Vec<CausalTensor<Complex<R>>>> {
        &self.blocks
    }

    /// The number of Kraus operators over every block.
    pub fn operator_count(&self) -> u64 {
        self.blocks.values().map(|k| k.len() as u64).sum()
    }

    /// The input qubit count, the ceiling of `log₂ d_in`.
    pub fn input_qubits(&self) -> usize {
        ceil_log2(self.d_in)
    }

    /// The output qubit count, the ceiling of `log₂ d_out`.
    pub fn output_qubits(&self) -> usize {
        ceil_log2(self.d_out)
    }

    /// The entry count of the composite Choi operators this morphism would form: `(d_in·d_out)²`
    /// per block, saturating rather than overflowing.
    pub fn choi_entries(&self) -> u64 {
        let d = (self.d_in as u64).saturating_mul(self.d_out as u64);
        d.saturating_mul(d)
            .saturating_mul(self.blocks.len().max(1) as u64)
    }

    /// Refuses the Choi formation when its entry count exceeds the cap.
    fn check_entry_cap(&self, caps: &NumericCaps) -> Result<u64, QuantumError> {
        let entries = self.choi_entries();
        if entries > caps.max_entries {
            return Err(QuantumError::NaturalityDimensionExceeded(
                self.input_qubits(),
                self.output_qubits(),
                entries,
                caps.max_entries,
            ));
        }
        Ok(entries)
    }

    /// Sequential composition: `self` then `next`. The classical outputs of `self` feed the
    /// classical inputs of `next` in order; the quantum output of `self` is the quantum input of
    /// `next`. The composite's Kraus operators are the products `N · S` over every matched pair of
    /// blocks, and both the operator count and the storage they take are checked against the caps
    /// before any product is formed.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] on a quantum or classical type mismatch;
    /// [`QuantumError::KrausFamilyExceeded`] above the operator cap;
    /// [`QuantumError::NaturalityDimensionExceeded`] when the composite's operators would hold
    /// more entries than the entry cap.
    pub fn then(&self, next: &Self, caps: &NumericCaps) -> Result<Self, QuantumError> {
        if self.d_out != next.d_in {
            return Err(QuantumError::DimensionMismatch(format!(
                "cannot compose: first morphism outputs dimension {}, second inputs {}",
                self.d_out, next.d_in
            )));
        }
        if self.classical_out != next.classical_in {
            return Err(QuantumError::DimensionMismatch(format!(
                "cannot compose: classical outputs {:?} do not match classical inputs {:?}",
                self.classical_out, next.classical_in
            )));
        }
        let mut count: u64 = 0;
        for ((_, y), s) in &self.blocks {
            for ((y2, _), n) in &next.blocks {
                if y == y2 {
                    count = count.saturating_add((s.len() as u64).saturating_mul(n.len() as u64));
                }
            }
        }
        if count > caps.max_operators {
            return Err(QuantumError::KrausFamilyExceeded(count, caps.max_operators));
        }
        check_storage_cap(count, self.d_in, next.d_out, caps)?;
        let mut out = Self::new(
            self.d_in,
            next.d_out,
            self.classical_in.clone(),
            next.classical_out.clone(),
        )?;
        for ((x, y), s) in &self.blocks {
            for ((y2, z), n) in &next.blocks {
                if y != y2 {
                    continue;
                }
                let mut products = Vec::with_capacity(s.len() * n.len());
                for sk in s {
                    for nk in n {
                        let p = nk.matmul(sk).map_err(|e| {
                            QuantumError::CalculationError(format!("matmul: {e:?}"))
                        })?;
                        products.push(p);
                    }
                }
                out.push(x.clone(), z.clone(), products)?;
            }
        }
        Ok(out)
    }

    /// Parallel composition `self ⊗ other`: quantum legs `self` then `other` (so `self`'s index is
    /// the more significant), classical wires concatenated, Kraus operators the Kronecker products
    /// over every pair of blocks.
    ///
    /// # Errors
    ///
    /// [`QuantumError::KrausFamilyExceeded`] above the operator cap;
    /// [`QuantumError::NaturalityDimensionExceeded`] when the products would hold more entries
    /// than the entry cap; [`QuantumError::DimensionMismatch`] when the composite dimension
    /// overflows; the Kronecker's shape errors.
    pub fn tensor(&self, other: &Self, caps: &NumericCaps) -> Result<Self, QuantumError> {
        let count = self.operator_count().saturating_mul(other.operator_count());
        if count > caps.max_operators {
            return Err(QuantumError::KrausFamilyExceeded(count, caps.max_operators));
        }
        let overflow = || {
            QuantumError::DimensionMismatch(
                "the dimensions of a parallel composition overflow usize".into(),
            )
        };
        let d_in = self.d_in.checked_mul(other.d_in).ok_or_else(overflow)?;
        let d_out = self.d_out.checked_mul(other.d_out).ok_or_else(overflow)?;
        check_storage_cap(count, d_in, d_out, caps)?;
        let mut classical_in = self.classical_in.clone();
        classical_in.extend(&other.classical_in);
        let mut classical_out = self.classical_out.clone();
        classical_out.extend(&other.classical_out);
        let mut out = Self::new(d_in, d_out, classical_in, classical_out)?;
        for ((xa, ya), ka) in &self.blocks {
            for ((xb, yb), kb) in &other.blocks {
                let mut x = xa.clone();
                x.extend(xb);
                let mut y = ya.clone();
                y.extend(yb);
                let mut products = Vec::with_capacity(ka.len() * kb.len());
                for a in ka {
                    for b in kb {
                        products.push(a.kronecker(b).map_err(|e| {
                            QuantumError::CalculationError(format!("kronecker: {e:?}"))
                        })?);
                    }
                }
                out.push(x, y, products)?;
            }
        }
        Ok(out)
    }

    /// The identity on a list of classical wires: one scalar block per value, on the trivial
    /// quantum system.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] on a zero outcome count.
    pub fn classical_identity(counts: &[usize]) -> Result<Self, QuantumError> {
        let mut out = Self::new(1, 1, counts.to_vec(), counts.to_vec())?;
        let one = CausalTensor::from_slice(&[Complex::new(R::one(), R::zero())], &[1, 1]);
        let mut values = vec![0usize; counts.len()];
        loop {
            out.push(values.clone(), values.clone(), vec![one.clone()])?;
            let mut k = counts.len();
            loop {
                if k == 0 {
                    return Ok(out);
                }
                k -= 1;
                values[k] += 1;
                if values[k] < counts[k] {
                    break;
                }
                values[k] = 0;
            }
        }
    }

    /// A pure state as a morphism from the trivial system: one Kraus operator `d × 1`.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] on an empty ket.
    pub fn state(ket: &[Complex<R>]) -> Result<Self, QuantumError> {
        if ket.is_empty() {
            return Err(QuantumError::DimensionMismatch(
                "a state needs an amplitude".into(),
            ));
        }
        Self::from_kraus(&[CausalTensor::from_slice(ket, &[ket.len(), 1])])
    }

    /// The same morphism with its quantum legs reordered: `in_dims` and `out_dims` are the current
    /// leg dimensions in order, `in_order[k]` names which current input leg becomes the `k`-th, and
    /// likewise `out_order`.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] if a dimension list does not multiply to the morphism's
    /// dimension or an order is not a permutation.
    pub fn permute_legs(
        &self,
        in_dims: &[usize],
        in_order: &[usize],
        out_dims: &[usize],
        out_order: &[usize],
    ) -> Result<Self, QuantumError> {
        let map_in = leg_index_map(in_dims, in_order, self.d_in)?;
        let map_out = leg_index_map(out_dims, out_order, self.d_out)?;
        let mut out = Self::new(
            self.d_in,
            self.d_out,
            self.classical_in.clone(),
            self.classical_out.clone(),
        )?;
        let zero = Complex::new(R::zero(), R::zero());
        for ((x, y), kraus) in &self.blocks {
            let mut moved = Vec::with_capacity(kraus.len());
            for k in kraus {
                // (P_out K P_in†)[map_out(r), map_in(c)] = K[r, c]: a gather, not two products
                // with permutation matrices, which cost d_out · d_in · (d_out + d_in).
                let src = k.as_slice();
                let mut data = vec![zero; self.d_out * self.d_in];
                for r in 0..self.d_out {
                    for c in 0..self.d_in {
                        data[map_out[r] * self.d_in + map_in[c]] = src[r * self.d_in + c];
                    }
                }
                moved.push(CausalTensor::from_slice(&data, &[self.d_out, self.d_in]));
            }
            out.push(x.clone(), y.clone(), moved)?;
        }
        Ok(out)
    }

    /// The Choi operator of every block, formed through `choi_from_kraus`, with the entry count it
    /// took.
    ///
    /// # Errors
    ///
    /// [`QuantumError::NaturalityDimensionExceeded`] above the entry cap, before allocating.
    #[allow(clippy::type_complexity)]
    pub fn choi_blocks(
        &self,
        caps: &NumericCaps,
    ) -> Result<
        (
            BTreeMap<(ClassicalValues, ClassicalValues), CausalTensor<Complex<R>>>,
            u64,
        ),
        QuantumError,
    > {
        let entries = self.check_entry_cap(caps)?;
        let mut out = BTreeMap::new();
        for (key, kraus) in &self.blocks {
            out.insert(key.clone(), choi_from_kraus(kraus)?);
        }
        Ok((out, entries))
    }

    /// The Frobenius distance between two morphisms of one type over the direct sum of their block
    /// Choi operators, and the entries formed to compute it.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] if the types differ;
    /// [`QuantumError::NaturalityDimensionExceeded`] above the entry cap on either side.
    pub fn frobenius_distance(
        &self,
        other: &Self,
        caps: &NumericCaps,
    ) -> Result<(R, u64), QuantumError> {
        if self.d_in != other.d_in
            || self.d_out != other.d_out
            || self.classical_in != other.classical_in
            || self.classical_out != other.classical_out
        {
            return Err(QuantumError::DimensionMismatch(
                "the two morphisms have different types and cannot be compared".into(),
            ));
        }
        let (a, ea) = self.choi_blocks(caps)?;
        let (b, eb) = other.choi_blocks(caps)?;
        let mut sum_sq = R::zero();
        let keys: alloc::collections::BTreeSet<_> = a.keys().chain(b.keys()).cloned().collect();
        for key in keys {
            let residual = match (a.get(&key), b.get(&key)) {
                (Some(ja), Some(jb)) => frobenius_norm(&(ja.clone() - jb.clone())),
                (Some(j), None) | (None, Some(j)) => frobenius_norm(j),
                (None, None) => R::zero(),
            };
            sum_sq += residual * residual;
        }
        Ok((sum_sq.sqrt(), ea.saturating_add(eb)))
    }
}

fn check_values(values: &[usize], counts: &[usize], side: &str) -> Result<(), QuantumError> {
    if values.len() != counts.len() {
        return Err(QuantumError::DimensionMismatch(format!(
            "{} values {:?} do not match the {} classical wires",
            side,
            values,
            counts.len()
        )));
    }
    if let Some((v, c)) = values.iter().zip(counts).find(|(v, c)| *v >= *c) {
        return Err(QuantumError::DimensionMismatch(format!(
            "{side} value {v} is at or above its wire's outcome count {c}"
        )));
    }
    Ok(())
}

/// The smallest `n` with `2^n ≥ d`.
/// Refuses a family of `count` operators of `d_out × d_in` entries above the entry cap, before
/// any of them is formed.
fn check_storage_cap(
    count: u64,
    d_in: usize,
    d_out: usize,
    caps: &NumericCaps,
) -> Result<(), QuantumError> {
    let entries = count
        .saturating_mul(d_out as u64)
        .saturating_mul(d_in as u64);
    if entries > caps.max_entries {
        return Err(QuantumError::NaturalityDimensionExceeded(
            ceil_log2(d_in),
            ceil_log2(d_out),
            entries,
            caps.max_entries,
        ));
    }
    Ok(())
}

pub(crate) fn ceil_log2(d: usize) -> usize {
    let mut n = 0usize;
    while (1usize << n) < d {
        n += 1;
    }
    n
}

/// The index map of a leg permutation: `map[old]` is the position of the basis state `old` over
/// `dims` after the legs are reordered so that `order[k]` is the `k`-th leg, first leg most
/// significant.
pub(crate) fn leg_index_map(
    dims: &[usize],
    order: &[usize],
    d: usize,
) -> Result<Vec<usize>, QuantumError> {
    let product: usize = dims.iter().product();
    if product != d || order.len() != dims.len() {
        return Err(QuantumError::DimensionMismatch(format!(
            "leg dimensions {dims:?} and order {order:?} do not describe a {d}-dimensional system"
        )));
    }
    let mut seen = vec![false; dims.len()];
    for &o in order {
        if o >= dims.len() || seen[o] {
            return Err(QuantumError::DimensionMismatch(format!(
                "{order:?} is not a permutation of the legs"
            )));
        }
        seen[o] = true;
    }
    let new_dims: Vec<usize> = order.iter().map(|&o| dims[o]).collect();
    let mut map = vec![0usize; d];
    for (old, slot) in map.iter_mut().enumerate() {
        // Digits of `old` over `dims`.
        let mut digits = vec![0usize; dims.len()];
        let mut rest = old;
        for k in (0..dims.len()).rev() {
            digits[k] = rest % dims[k];
            rest /= dims[k];
        }
        let mut new = 0usize;
        for (k, &o) in order.iter().enumerate() {
            new = new * new_dims[k] + digits[o];
        }
        *slot = new;
    }
    Ok(map)
}
