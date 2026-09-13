/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The code-space isometry and the ideal recovery of a CSS code, as channels.
//!
//! These are the numeric `E` and `τ` of a code's type alignment. `E : 2^k → 2^n` sends the logical
//! basis state `|x̄⟩` to the code state `P X̄(γ̃)^x |0…0⟩`, normalised, where `P` projects onto the
//! stabilizer's `+1` eigenspace and the `X̄(γ̃ᵢ)` are the logical `X`s dual to the logical `Z̄(γᵢ)`.
//! `τ : 2^n → 2^k` is the Knill–Laflamme recovery of the code's own syndrome table: for each
//! syndrome `s`, a Pauli `C_s` of least weight producing it, and the Kraus operator
//! `K_s = E† C_s Π_s` with `Π_s` the syndrome projector. `Σ K_s† K_s = Σ Π_s = I`, and
//! `τ ∘ E = id` because `C_0 = I`. This is the abstraction's `τ` for a code, not a decoder for
//! experiments: it is the object Lorenz & Tull's Definition 14 asks for, built from the stabilizer
//! generators the crate already carries. Dense on `2^n`, so it stops at ten qubits.

use crate::QuantumError;
use crate::types::circuit_model::QcMorphism;
use crate::types::qcode::clifford_action::symplectic_dual_basis;
use crate::types::qcode::logical_equivalence::LogicalBasis;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_homology::Gf2Chain;
use deep_causality_num::{FromPrimitive, NaturalNumber};
use deep_causality_num_complex::Complex;
use deep_causality_tensor::CausalTensor;

/// The widest register the dense construction takes.
pub const IDEAL_RECOVERY_MAX_QUBITS: usize = 10;

/// The isometry and the recovery of a code.
#[derive(Debug, Clone, PartialEq)]
pub struct IdealRecovery<R: RealField> {
    isometry: QcMorphism<R>,
    encoder: QcMorphism<R>,
    recovery: QcMorphism<R>,
    syndromes: usize,
    max_correction_weight: usize,
    corrections: Vec<(Vec<bool>, Vec<bool>)>,
}

impl<R: RealField> IdealRecovery<R> {
    /// `E : 2^k → 2^n`, one Kraus operator.
    pub fn isometry(&self) -> &QcMorphism<R> {
        &self.isometry
    }

    /// The encoder as a unitary on the `n` physical qubits: `|x⟩ ⊗ |0…0⟩ ↦ |x̄⟩` for the logical
    /// value `x` on the first `k` wires, completed to a unitary by the remaining coset states of the
    /// `X`-stabilizer group, one per coset and character. This is the box a low-level model in the shape of Lorenz & Tull's Example 58
    /// starts with, so that its input type is the logical space.
    pub fn encoder(&self) -> &QcMorphism<R> {
        &self.encoder
    }

    /// `τ : 2^n → 2^k`, one Kraus operator per syndrome.
    pub fn recovery(&self) -> &QcMorphism<R> {
        &self.recovery
    }

    /// The number of syndromes, `2^(n − k)`.
    pub fn syndromes(&self) -> usize {
        self.syndromes
    }

    /// The largest weight among the corrections in the table.
    pub fn max_correction_weight(&self) -> usize {
        self.max_correction_weight
    }

    /// The correction table by syndrome index: the X and Z supports of the Pauli applied, so a
    /// caller can reproduce a recovery by hand.
    pub fn corrections(&self) -> &[(Vec<bool>, Vec<bool>)] {
        &self.corrections
    }
}

/// A Pauli on `n` qubits as its X and Z supports, applied to a state vector in place: `X` flips the
/// bit, `Z` signs it, `Y = iXZ` up to the phase this routine drops, which no Kraus operator's Choi
/// operator sees.
fn apply_pauli<R: RealField>(state: &mut [Complex<R>], n: usize, x: &[bool], z: &[bool]) {
    let mut out = vec![Complex::new(R::zero(), R::zero()); state.len()];
    let mut flip = 0usize;
    for (q, &f) in x.iter().enumerate() {
        if f {
            flip |= 1 << (n - 1 - q);
        }
    }
    for (idx, amp) in state.iter().enumerate() {
        let mut sign = R::one();
        for (q, &s) in z.iter().enumerate() {
            if s && idx & (1 << (n - 1 - q)) != 0 {
                sign = -sign;
            }
        }
        out[idx ^ flip] += *amp * Complex::new(sign, R::zero());
    }
    state.copy_from_slice(&out);
}

/// `(I + (−1)^s S)/2 |v⟩` for a stabilizer generator given as a chain, Z-type or X-type.
fn project<R: RealField>(
    state: &mut [Complex<R>],
    n: usize,
    generator: &[bool],
    z_type: bool,
    sign: bool,
) {
    let mut moved = state.to_vec();
    let none = vec![false; n];
    if z_type {
        apply_pauli(&mut moved, n, &none, generator);
    } else {
        apply_pauli(&mut moved, n, generator, &none);
    }
    let half = R::one() / (R::one() + R::one());
    let s = if sign { -R::one() } else { R::one() };
    for (a, m) in state.iter_mut().zip(moved) {
        *a = (*a + m * Complex::new(s, R::zero())) * Complex::new(half, R::zero());
    }
}

fn bits<W: NaturalNumber>(chain: &Gf2Chain<W>, n: usize) -> Vec<bool> {
    let mut v = vec![false; n];
    for q in chain.support() {
        v[q] = true;
    }
    v
}

impl<R> IdealRecovery<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    /// The isometry and the recovery of the code a [`LogicalBasis`] describes.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] above [`IDEAL_RECOVERY_MAX_QUBITS`];
    /// [`QuantumError::CalculationError`] if a logical basis state comes out zero or a syndrome has
    /// no correction up to weight `n`; the channel constructors' errors.
    pub fn from_basis<W: NaturalNumber>(basis: &LogicalBasis<W>) -> Result<Self, QuantumError> {
        let n = basis.len();
        let k = basis.num_logical_qubits();
        if n > IDEAL_RECOVERY_MAX_QUBITS {
            return Err(QuantumError::DimensionMismatch(format!(
                "the ideal recovery is dense on 2^{n}; it stops at {IDEAL_RECOVERY_MAX_QUBITS} qubits"
            )));
        }
        let duals = symplectic_dual_basis(basis.homology(), basis.cohomology())?;
        let z_gens: Vec<Vec<bool>> = basis.stabilizers().iter().map(|g| bits(g, n)).collect();
        let x_gens: Vec<Vec<bool>> = basis.x_stabilizers().iter().map(|g| bits(g, n)).collect();
        let d = 1usize << n;
        let dk = 1usize << k;
        let zero = Complex::new(R::zero(), R::zero());
        let none = vec![false; n];

        // Logical basis states as the columns of the isometry.
        let mut columns: Vec<Vec<Complex<R>>> = Vec::with_capacity(dk);
        for x in 0..dk {
            let mut v = vec![zero; d];
            v[0] = Complex::new(R::one(), R::zero());
            for (i, dual) in duals.iter().enumerate() {
                if x & (1 << (k - 1 - i)) != 0 {
                    apply_pauli(&mut v, n, &bits(dual, n), &none);
                }
            }
            for g in &z_gens {
                project(&mut v, n, g, true, false);
            }
            for g in &x_gens {
                project(&mut v, n, g, false, false);
            }
            let norm = v
                .iter()
                .fold(R::zero(), |a, c| a + c.re * c.re + c.im * c.im)
                .sqrt();
            if norm <= R::epsilon().sqrt() {
                return Err(QuantumError::CalculationError(format!(
                    "the logical basis state {x} projects to zero; the seed |0…0⟩ lies outside the code space"
                )));
            }
            let inv = Complex::new(R::one() / norm, R::zero());
            columns.push(v.into_iter().map(|c| c * inv).collect());
        }
        let mut e_data = vec![zero; d * dk];
        for (x, col) in columns.iter().enumerate() {
            for (row, amp) in col.iter().enumerate() {
                e_data[row * dk + x] = *amp;
            }
        }
        let isometry = QcMorphism::from_kraus(&[CausalTensor::from_slice(&e_data, &[d, dk])])?;

        // The encoder unitary. For a CSS code the basis states fall into cosets `c + S_X` of the
        // `X`-stabilizer group; the vectors `|c, χ⟩ = |S_X|^{-1/2} Σ_s χ(s) |c + s⟩`, one per
        // coset and character `χ` of `S_X`, are an orthonormal basis of the whole space, and the
        // code words are the trivial-character vectors of the cosets with zero `Z`-syndrome. The
        // code words sit at the input columns `|x⟩|0…0⟩`; the other vectors fill the remaining
        // columns in order.
        let group: Vec<usize> = {
            let mut g = vec![0usize];
            for generator in &x_gens {
                let mask = generator
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| **b)
                    .fold(0usize, |m, (q, _)| m | (1 << (n - 1 - q)));
                let mut more: Vec<usize> = g.iter().map(|s| s ^ mask).collect();
                more.retain(|m| !g.contains(m));
                g.extend(more);
            }
            g
        };
        let mut unitary_cols: Vec<Option<Vec<Complex<R>>>> = vec![None; d];
        for (x, col) in columns.iter().enumerate() {
            unitary_cols[x << (n - k)] = Some(col.clone());
        }
        let scale = Complex::new(
            R::one() / R::from_usize(group.len()).unwrap_or_else(R::one).sqrt(),
            R::zero(),
        );
        let free_slots: Vec<usize> = (0..d).filter(|c| unitary_cols[*c].is_none()).collect();
        let mut free = free_slots.into_iter();
        let mut seen = vec![false; d];
        for c in 0..d {
            if seen[c] {
                continue;
            }
            let coset: Vec<usize> = group.iter().map(|s| c ^ s).collect();
            for &m in &coset {
                seen[m] = true;
            }
            let mut characters: Vec<Vec<bool>> = Vec::new();
            for y in 0..d {
                let signs: Vec<bool> = group
                    .iter()
                    .map(|s| (y & s).count_ones() % 2 == 1)
                    .collect();
                if !characters.contains(&signs) {
                    characters.push(signs);
                }
                if characters.len() == group.len() {
                    break;
                }
            }
            for signs in &characters {
                let mut v = vec![zero; d];
                for (m, negative) in coset.iter().zip(signs) {
                    v[*m] = if *negative { -scale } else { scale };
                }
                let is_code_word = columns.iter().any(|col| {
                    let overlap = col
                        .iter()
                        .zip(&v)
                        .fold(zero, |acc, (a, b)| acc + Complex::new(a.re, -a.im) * *b);
                    (overlap.re * overlap.re + overlap.im * overlap.im)
                        > R::from_f64(0.5).unwrap_or_else(R::one)
                });
                if is_code_word {
                    continue;
                }
                let slot = free.next().ok_or_else(|| {
                    QuantumError::CalculationError(
                        "the coset basis has more vectors than the encoder has free columns".into(),
                    )
                })?;
                unitary_cols[slot] = Some(v);
            }
        }
        if free.next().is_some() {
            return Err(QuantumError::CalculationError(
                "the coset basis leaves an encoder column empty; a code word is not a coset state"
                    .into(),
            ));
        }
        let mut w_data = vec![zero; d * d];
        for (col, v) in unitary_cols.iter().enumerate() {
            for (row, amp) in v.as_ref().expect("filled").iter().enumerate() {
                w_data[row * d + col] = *amp;
            }
        }
        let encoder = QcMorphism::from_kraus(&[CausalTensor::from_slice(&w_data, &[d, d])])?;

        // Syndrome table: least-weight Pauli per syndrome, by weight.
        let checks = z_gens.len() + x_gens.len();
        let syndrome_of = |x: &[bool], z: &[bool]| -> usize {
            let mut s = 0usize;
            for (i, g) in z_gens.iter().enumerate() {
                // An X-part anticommutes with a Z-check on odd overlap.
                let parity = g.iter().zip(x).filter(|(a, b)| **a && **b).count() % 2;
                s |= parity << i;
            }
            for (i, g) in x_gens.iter().enumerate() {
                let parity = g.iter().zip(z).filter(|(a, b)| **a && **b).count() % 2;
                s |= parity << (z_gens.len() + i);
            }
            s
        };
        let mut table: BTreeMap<usize, (Vec<bool>, Vec<bool>)> = BTreeMap::new();
        table.insert(0, (none.clone(), none.clone()));
        let total = 1usize << checks;
        let mut max_weight = 0usize;
        'search: for weight in 1..=n {
            for qubits in combinations(n, weight) {
                for pattern in 0..(3usize.pow(weight as u32)) {
                    let mut x = none.clone();
                    let mut z = none.clone();
                    let mut p = pattern;
                    for &q in &qubits {
                        match p % 3 {
                            0 => x[q] = true,
                            1 => z[q] = true,
                            _ => {
                                x[q] = true;
                                z[q] = true;
                            }
                        }
                        p /= 3;
                    }
                    let s = syndrome_of(&x, &z);
                    if let alloc::collections::btree_map::Entry::Vacant(v) = table.entry(s) {
                        v.insert((x, z));
                        max_weight = weight;
                        if table.len() == total {
                            break 'search;
                        }
                    }
                }
            }
        }
        if table.len() != total {
            return Err(QuantumError::CalculationError(format!(
                "{} of {total} syndromes have a correction; the stabilizer generators are dependent or the search is incomplete",
                table.len()
            )));
        }

        // K_s = E† C_s Π_s: row x is (Π_s C_s |x̄⟩)†, since Paulis are Hermitian up to the phase
        // `apply_pauli` drops and the projectors are Hermitian.
        let mut kraus = Vec::with_capacity(total);
        for s in 0..total {
            let (x, z) = &table[&s];
            let mut k_data = vec![zero; dk * d];
            for (row, col) in columns.iter().enumerate() {
                let mut v = col.clone();
                apply_pauli(&mut v, n, x, z);
                for (i, g) in z_gens.iter().enumerate() {
                    project(&mut v, n, g, true, (s >> i) & 1 == 1);
                }
                for (i, g) in x_gens.iter().enumerate() {
                    project(&mut v, n, g, false, (s >> (z_gens.len() + i)) & 1 == 1);
                }
                for (c, amp) in v.iter().enumerate() {
                    k_data[row * d + c] = Complex::new(amp.re, -amp.im);
                }
            }
            kraus.push(CausalTensor::from_slice(&k_data, &[dk, d]));
        }
        let recovery = QcMorphism::from_kraus(&kraus)?;
        let corrections = (0..total).map(|s| table[&s].clone()).collect();
        Ok(Self {
            isometry,
            encoder,
            recovery,
            syndromes: total,
            max_correction_weight: max_weight,
            corrections,
        })
    }
}

/// The `k`-subsets of `0..n`, ascending.
fn combinations(n: usize, k: usize) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    let mut idx: Vec<usize> = (0..k).collect();
    if k > n {
        return out;
    }
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
