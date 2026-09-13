/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The numeric semantics functor: a [`CircuitModel`] evaluated to a [`QcMorphism`] at the Kraus
//! level. Every box acts on register state vectors through its small operator and the register's
//! strides; no operator on the whole register is ever formed, and the Choi operator is formed once,
//! for the composite channel from the declared inputs to the declared outputs, by the morphism.

use crate::QuantumError;
use crate::types::circuit_model::circuit_box::CircuitBox;
use crate::types::circuit_model::gate_unitary::gate_unitary;
use crate::types::circuit_model::model::CircuitModel;
use crate::types::circuit_model::qc_morphism::{NumericCaps, QcMorphism, ceil_log2};
use crate::types::circuit_model::wire::WireId;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::CausalTensor;

/// One classical branch of an evaluation: the values written so far and, for each quantum input
/// basis state, the register state vector it has been carried to.
struct Branch<R> {
    values: Vec<Option<usize>>,
    cols: Vec<Vec<Complex<R>>>,
}

impl<R> CircuitModel<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    /// The input-output semantics of the whole model: every box active, the declared inputs and
    /// outputs.
    ///
    /// # Errors
    ///
    /// As [`evaluate`](Self::evaluate).
    pub fn numeric_semantics(&self, caps: &NumericCaps) -> Result<QcMorphism<R>, QuantumError> {
        let active = vec![true; self.boxes().len()];
        self.evaluate(&active, self.inputs(), self.outputs(), caps)
    }

    /// The semantics of the model with the boxes `active` marks, the quantum wires `inputs` free
    /// and the wires `outputs` kept. Opening a node (Lorenz & Tull §7.2) deactivates its boxes and
    /// adds their wires to the inputs; this is the function that evaluates the result.
    ///
    /// # Errors
    ///
    /// [`QuantumError::NaturalityDimensionExceeded`] when the composite Choi or the working storage
    /// would exceed the entry cap, and [`QuantumError::KrausFamilyExceeded`] when the branch count
    /// would exceed the operator cap, both before allocating; [`QuantumError::CalculationError`]
    /// when an encoder reads a classical wire no active box wrote and no input supplies, or a
    /// classical output is never written; [`QuantumError::DimensionMismatch`] on a wire outside the
    /// model.
    pub fn evaluate(
        &self,
        active: &[bool],
        inputs: &[WireId],
        outputs: &[WireId],
        caps: &NumericCaps,
    ) -> Result<QcMorphism<R>, QuantumError> {
        if active.len() != self.boxes().len() {
            return Err(QuantumError::DimensionMismatch(format!(
                "the activity mask has {} entries for {} boxes",
                active.len(),
                self.boxes().len()
            )));
        }
        let legs = self.quantum_wires();
        let dims: Vec<usize> = legs
            .iter()
            .map(|&w| self.wires()[w].cardinality())
            .collect();
        let position = |w: WireId| -> Result<usize, QuantumError> {
            legs.binary_search(&w).map_err(|_| {
                QuantumError::DimensionMismatch(format!(
                    "wire {w} is not a quantum wire of the model"
                ))
            })
        };
        let mut strides = vec![1usize; dims.len()];
        for i in (0..dims.len().saturating_sub(1)).rev() {
            strides[i] = strides[i + 1] * dims[i + 1];
        }
        let d_total = dims
            .iter()
            .try_fold(1usize, |a, &d| a.checked_mul(d))
            .ok_or_else(|| {
                QuantumError::DimensionMismatch("register dimension overflows usize".into())
            })?;

        let mut input_pos: Vec<usize> = inputs
            .iter()
            .map(|&w| position(w))
            .collect::<Result<_, _>>()?;
        input_pos.sort_unstable();
        input_pos.dedup();
        let d_in = input_pos.iter().map(|&p| dims[p]).product::<usize>().max(1);

        let mut out_q: Vec<usize> = Vec::new();
        let mut out_c: Vec<WireId> = Vec::new();
        for &w in outputs {
            match self.wires().get(w) {
                Some(t) if t.is_quantum() => out_q.push(position(w)?),
                Some(_) => out_c.push(w),
                None => {
                    return Err(QuantumError::DimensionMismatch(format!(
                        "output {w} is not a wire of the model"
                    )));
                }
            }
        }
        out_q.sort_unstable();
        out_q.dedup();
        let d_out = out_q.iter().map(|&p| dims[p]).product::<usize>().max(1);
        let traced: Vec<usize> = (0..dims.len()).filter(|p| !out_q.contains(p)).collect();
        let d_traced = traced.iter().map(|&p| dims[p]).product::<usize>().max(1);

        // Classical inputs: read by an active box, written by none.
        let written: BTreeSet<WireId> = self
            .boxes()
            .iter()
            .enumerate()
            .filter(|(b, _)| active[*b])
            .filter_map(|(_, bx)| bx.classical_write())
            .collect();
        let mut classical_in: Vec<WireId> = self
            .boxes()
            .iter()
            .enumerate()
            .filter(|(b, _)| active[*b])
            .filter_map(|(_, bx)| bx.classical_read())
            .filter(|w| !written.contains(w))
            .collect();
        classical_in.sort_unstable();
        classical_in.dedup();
        let in_counts: Vec<usize> = classical_in
            .iter()
            .map(|&w| self.wires()[w].cardinality())
            .collect();
        let out_counts: Vec<usize> = out_c
            .iter()
            .map(|&w| self.wires()[w].cardinality())
            .collect();

        // Both caps, before anything is allocated. The entry cap is on the working storage, one
        // register state vector per input basis state per classical branch; the Choi operator is
        // capped where it is formed, in `QcMorphism::choi_blocks`, since the program's own Choi is
        // never formed. Every later branch expansion is checked against the same register.
        let register = Register {
            dims: &dims,
            strides: &strides,
            per_branch: (d_total as u64).saturating_mul(d_in as u64),
            n: ceil_log2(d_in),
            k: ceil_log2(d_out),
            caps,
        };
        let initial_branches = in_counts
            .iter()
            .fold(1u64, |a, &c| a.saturating_mul(c as u64));
        register.check_branches(initial_branches)?;
        // The assembled family has one operator per branch and traced basis state.
        register.check_operators(initial_branches.saturating_mul(d_traced as u64))?;

        // Initial branches: one per classical-input assignment, each carrying the input basis.
        let zero = Complex::new(R::zero(), R::zero());
        let one = Complex::new(R::one(), R::zero());
        let mut branches: Vec<Branch<R>> = Vec::new();
        let mut assignment = vec![0usize; classical_in.len()];
        loop {
            let mut values = vec![None; self.wires().len()];
            for (k, &w) in classical_in.iter().enumerate() {
                values[w] = Some(assignment[k]);
            }
            let mut cols = Vec::with_capacity(d_in);
            for i in 0..d_in {
                let mut v = vec![zero; d_total];
                let mut idx = 0usize;
                let mut rest = i;
                for &p in input_pos.iter().rev() {
                    idx += (rest % dims[p]) * strides[p];
                    rest /= dims[p];
                }
                v[idx] = one;
                cols.push(v);
            }
            branches.push(Branch { values, cols });
            if !advance(&mut assignment, &in_counts) {
                break;
            }
        }

        for (b, bx) in self.boxes().iter().enumerate() {
            if !active[b] {
                continue;
            }
            let positions: Vec<usize> = bx
                .quantum_wires()
                .iter()
                .map(|&w| position(w))
                .collect::<Result<_, _>>()?;
            let d_box: usize = positions.iter().map(|&p| dims[p]).product();
            match bx {
                CircuitBox::Unitary { wires, program } => {
                    for op in program {
                        let (local, m) = gate_unitary::<R>(op)?;
                        let gate_pos: Vec<usize> = local.iter().map(|&q| positions[q]).collect();
                        for br in &mut branches {
                            for col in &mut br.cols {
                                apply_small(col, &dims, &strides, &gate_pos, &m);
                            }
                        }
                    }
                    let _ = wires;
                }
                CircuitBox::Channel { channel, .. } => {
                    let kraus = QcMorphism::from_channel(channel)?.kraus();
                    branches = branch_over(&branches, &kraus, None, &positions, &register)?;
                }
                CircuitBox::Kraus { kraus, .. } => {
                    branches = branch_over(&branches, kraus, None, &positions, &register)?;
                }
                CircuitBox::Instrument { outcome, kraus, .. } => {
                    // Every outcome family opens its branches: the whole count is checked before
                    // the first is formed.
                    let opened = kraus
                        .iter()
                        .fold(0u64, |a, fam| a.saturating_add(fam.len() as u64))
                        .saturating_mul(branches.len() as u64);
                    register.check_branches(opened)?;
                    let mut next: Vec<Branch<R>> = Vec::new();
                    for (y, fam) in kraus.iter().enumerate() {
                        let mut tagged = branch_over(
                            &branches,
                            fam,
                            Some((*outcome, y)),
                            &positions,
                            &register,
                        )?;
                        next.append(&mut tagged);
                    }
                    branches = next;
                }
                CircuitBox::Measurement { outcome, .. } => {
                    register
                        .check_branches((branches.len() as u64).saturating_mul(d_box as u64))?;
                    let mut next: Vec<Branch<R>> = Vec::new();
                    for y in 0..d_box {
                        // `|0⟩⟨y|`: keep the amplitude where the wires read `y`, move it to `|0⟩`.
                        let mut m = vec![zero; d_box * d_box];
                        m[y] = one;
                        let k = CausalTensor::from_slice(&m, &[d_box, d_box]);
                        let mut tagged = branch_over(
                            &branches,
                            &[k],
                            Some((*outcome, y)),
                            &positions,
                            &register,
                        )?;
                        next.append(&mut tagged);
                    }
                    branches = next;
                }
                CircuitBox::Encoder { input, states, .. } => {
                    for br in &mut branches {
                        let x = br.values[*input].ok_or_else(|| {
                            QuantumError::CalculationError(format!(
                                "box {b} (encoder) reads classical wire {input} before any box wrote it"
                            ))
                        })?;
                        // `|ψ_x⟩⟨0…0|` on the box's wires.
                        let psi = states[x].as_slice();
                        let mut m = vec![zero; d_box * d_box];
                        for (j, a) in psi.iter().enumerate() {
                            m[j * d_box] = *a;
                        }
                        let k = CausalTensor::from_slice(&m, &[d_box, d_box]);
                        for col in &mut br.cols {
                            apply_small(col, &dims, &strides, &positions, &k);
                        }
                    }
                }
            }
        }

        // Assemble the morphism: for each branch and each traced index, one Kraus operator; the
        // family's size is checked before the first operator is formed.
        register.check_operators((branches.len() as u64).saturating_mul(d_traced as u64))?;
        let mut qc = QcMorphism::new(d_in, d_out, in_counts.clone(), out_counts)?;
        let compose = |digits: &[(usize, usize)]| -> usize {
            digits.iter().map(|&(p, v)| v * strides[p]).sum()
        };
        for br in &branches {
            let x: Vec<usize> = classical_in
                .iter()
                .map(|&w| br.values[w].unwrap_or(0))
                .collect();
            let mut y = Vec::with_capacity(out_c.len());
            for &w in &out_c {
                y.push(br.values[w].ok_or_else(|| {
                    QuantumError::CalculationError(format!(
                        "classical output {w} was never written by an active box"
                    ))
                })?);
            }
            let mut kraus = Vec::with_capacity(d_traced);
            for t in 0..d_traced {
                let mut t_digits = Vec::with_capacity(traced.len());
                let mut rest = t;
                for &p in traced.iter().rev() {
                    t_digits.push((p, rest % dims[p]));
                    rest /= dims[p];
                }
                let t_off = compose(&t_digits);
                let mut data = vec![zero; d_out * d_in];
                for o in 0..d_out {
                    let mut o_digits = Vec::with_capacity(out_q.len());
                    let mut rest = o;
                    for &p in out_q.iter().rev() {
                        o_digits.push((p, rest % dims[p]));
                        rest /= dims[p];
                    }
                    let idx = compose(&o_digits) + t_off;
                    for (i, col) in br.cols.iter().enumerate() {
                        data[o * d_in + i] = col[idx];
                    }
                }
                kraus.push(CausalTensor::from_slice(&data, &[d_out, d_in]));
            }
            qc.push(x, y, kraus)?;
        }
        Ok(qc)
    }
}

fn clone_branch<R: Clone>(b: &Branch<R>) -> Branch<R> {
    Branch {
        values: b.values.clone(),
        cols: b.cols.clone(),
    }
}

/// The register an evaluation runs on, with what one classical branch costs and the caps every
/// branch expansion is checked against.
struct Register<'a> {
    dims: &'a [usize],
    strides: &'a [usize],
    /// Entries one branch holds: the register dimension times the input basis size.
    per_branch: u64,
    /// Input and output qubit counts, for the error.
    n: usize,
    k: usize,
    caps: &'a NumericCaps,
}

impl Register<'_> {
    /// Refuses `count` branches when their storage exceeds the entry cap or their number the
    /// operator cap.
    fn check_branches(&self, count: u64) -> Result<(), QuantumError> {
        let entries = count.saturating_mul(self.per_branch);
        if entries > self.caps.max_entries {
            return Err(QuantumError::NaturalityDimensionExceeded(
                self.n,
                self.k,
                entries,
                self.caps.max_entries,
            ));
        }
        self.check_operators(count)
    }

    /// Refuses a Kraus family of `count` operators above the operator cap.
    fn check_operators(&self, count: u64) -> Result<(), QuantumError> {
        if count > self.caps.max_operators {
            return Err(QuantumError::KrausFamilyExceeded(
                count,
                self.caps.max_operators,
            ));
        }
        Ok(())
    }
}

/// Every branch times every operator of a family, tagging the outcome wire when given. The count
/// is checked against both caps before the products are formed.
fn branch_over<R>(
    branches: &[Branch<R>],
    family: &[CausalTensor<Complex<R>>],
    tag: Option<(WireId, usize)>,
    positions: &[usize],
    register: &Register<'_>,
) -> Result<Vec<Branch<R>>, QuantumError>
where
    R: RealField,
{
    let count = (branches.len() as u64).saturating_mul(family.len() as u64);
    register.check_branches(count)?;
    let mut out = Vec::with_capacity(count as usize);
    for br in branches {
        for k in family {
            let mut nb = clone_branch(br);
            if let Some((w, y)) = tag {
                nb.values[w] = Some(y);
            }
            for col in &mut nb.cols {
                apply_small(col, register.dims, register.strides, positions, k);
            }
            out.push(nb);
        }
    }
    Ok(out)
}

/// Applies a `d_b × d_b` operator on the legs at `positions` (first most significant) to a register
/// state vector, through the register's strides. `O(d_total · d_b)` per call.
fn apply_small<R>(
    col: &mut [Complex<R>],
    dims: &[usize],
    strides: &[usize],
    positions: &[usize],
    m: &CausalTensor<Complex<R>>,
) where
    R: RealField,
{
    let d_b: usize = positions.iter().map(|&p| dims[p]).product();
    let ms = m.as_slice();
    // Local strides of the box's index over its positions.
    let mut local_strides = vec![1usize; positions.len()];
    for i in (0..positions.len().saturating_sub(1)).rev() {
        local_strides[i] = local_strides[i + 1] * dims[positions[i + 1]];
    }
    // Register offset of each local index.
    let offsets: Vec<usize> = (0..d_b)
        .map(|j| {
            positions
                .iter()
                .enumerate()
                .map(|(k, &p)| ((j / local_strides[k]) % dims[p]) * strides[p])
                .sum()
        })
        .collect();
    let zero = Complex::new(R::zero(), R::zero());
    let mut out = vec![zero; col.len()];
    for (idx, &amp) in col.iter().enumerate() {
        // Local digit of this index and the base with the box's digits cleared.
        let mut j = 0usize;
        let mut base = idx;
        for (k, &p) in positions.iter().enumerate() {
            let digit = (idx / strides[p]) % dims[p];
            j += digit * local_strides[k];
            base -= digit * strides[p];
        }
        for jp in 0..d_b {
            let coefficient = ms[jp * d_b + j];
            out[base + offsets[jp]] += coefficient * amp;
        }
    }
    col.copy_from_slice(&out);
}

/// Advances a mixed-radix odometer; `false` when it wraps to all zeros.
fn advance(digits: &mut [usize], radix: &[usize]) -> bool {
    for k in (0..digits.len()).rev() {
        digits[k] += 1;
        if digits[k] < radix[k] {
            return true;
        }
        digits[k] = 0;
    }
    false
}
