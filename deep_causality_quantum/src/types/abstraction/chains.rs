/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Three abstraction chains over CSS codes, each a pair of links whose composite obeys the law of
//! [`Abstraction::compose`]: a concatenated code, code switching through a gadget, and a
//! distillation round. Every chain is an example with checks, not a theorem: it claims the residual
//! it measures and the bound the law records, nothing more.
//!
//! **Concatenation.** The inner code's `n` physical qubits are grouped into blocks of the outer
//! code's `k` logical qubits and each block is encoded by the outer code. The low-level model
//! carries the inner encoder on the wires that stand for the middle qubits, one outer encoder per
//! block, and the inner program with every gate replaced by the outer code's emitted program for
//! it, on the block's wires: Table 1's gates through [`CodeAbstraction::program`], `CCZ` and
//! `C^{m−1}Z` through the outer code's `C^{m−1}Z̄` emitter (Haruna, Table 1 row 6). A gate on
//! middle qubits in different blocks would need a transversal gadget between code blocks this
//! construction does not carry and is refused. The inner `T̄` on a representative of weight two
//! or more carries `CS†` (Haruna Eq. 3.56), and Table 1 has no controlled-`S` row, so the outer
//! code has no `CS̄†` emitter and the concatenation of `T̄` is refused by that name. The first
//! link `L → M` aligns each block's logical wires by the
//! identity on the input side and the block through the outer recovery on the output side; the
//! second link is the inner code's abstraction.
//!
//! **Code switching.** The low-level model encodes into code A, decodes it again, runs a noise
//! channel on one logical wire, encodes into code B and runs B's program. The gadget is the
//! decode-and-re-encode; the first link `L → M` aligns it by the physical identity with M the
//! code-B model, so its residual is the noise the gadget introduces, and the second link is the
//! code-B abstraction. A noiseless gadget composes exactly.
//!
//! **Distillation round.** The low-level model encodes into the code, depolarises every physical
//! qubit with probability `p`, and runs the encoded magic-state rotation `T̄ H̄` on every logical
//! qubit; the middle model is the same without the noise and the high level is `T H` on the
//! logical qubits. The first link measures the noise the ideal recovery does not remove, which is
//! the residual a distillation round would have to drive down; the construction is a non-strict
//! quantum-to-quantum abstraction and the paper (§7.1) defers this case.

use crate::QuantumError;
use crate::types::abstraction::abstraction::Abstraction;
use crate::types::abstraction::code_abstraction::{CodeAbstraction, LogicalGate};
use crate::types::abstraction::composition::Composed;
use crate::types::abstraction::ideal_recovery::IdealRecovery;
use crate::types::abstraction::query::Query;
use crate::types::abstraction::type_alignment::{AlignmentSide, AlignmentSpec, TypeAlignment};
use crate::types::circuit_model::{CircuitBox, CircuitModel, NumericCaps, QcMorphism, WireType};
use crate::types::qgates::gates_haruna::logical_multi_cz;
use crate::types::qgates::operator_linalg::identity_matrix;
use crate::types::qpu::circuit::GateOp;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_homology::{ChainComplex, Gf2Chain};
use deep_causality_num::{FromPrimitive, NaturalNumber};
use deep_causality_num_complex::Complex;
use deep_causality_tensor::CausalTensor;

/// Two links of circuit models, `L → M` and `M → H`.
#[derive(Debug, Clone)]
pub struct Chain<R: RealField> {
    /// The first link.
    pub first: Abstraction<R, CircuitModel<R>, CircuitModel<R>>,
    /// The second link.
    pub second: Abstraction<R, CircuitModel<R>, CircuitModel<R>>,
}

impl<R> Chain<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    /// The composite with its law.
    ///
    /// # Errors
    ///
    /// As [`Abstraction::compose`].
    pub fn compose(
        self,
        caps: &NumericCaps,
    ) -> Result<Composed<R, CircuitModel<R>, CircuitModel<R>>, QuantumError> {
        self.first.compose(self.second, caps)
    }
}

/// The single-qubit depolarising channel with probability `p`, as the Kraus family
/// `√(1 − p) I, √(p/3) X, √(p/3) Y, √(p/3) Z`; `p = 3/4` is the completely depolarising channel
/// `ρ ↦ Tr(ρ) I/2`.
///
/// # Errors
///
/// [`QuantumError::DimensionMismatch`] unless `0 ≤ p ≤ 1`.
pub fn depolarizing_kraus<R>(p: R) -> Result<Vec<CausalTensor<Complex<R>>>, QuantumError>
where
    R: RealField + FromPrimitive,
{
    if p < R::zero() || p > R::one() {
        return Err(QuantumError::DimensionMismatch(
            "a depolarising probability lies in [0, 1]".into(),
        ));
    }
    let zero = Complex::new(R::zero(), R::zero());
    let three = R::from_usize(3).ok_or_else(|| {
        QuantumError::CalculationError("failed to represent depolarizing denominator 3".into())
    })?;
    let stay = Complex::new((R::one() - p).sqrt(), R::zero());
    let flip = Complex::new((p / three).sqrt(), R::zero());
    let i_flip = Complex::new(R::zero(), (p / three).sqrt());
    let m = |data: [Complex<R>; 4]| CausalTensor::from_slice(&data, &[2, 2]);
    Ok(vec![
        m([stay, zero, zero, stay]),
        m([zero, flip, flip, zero]),
        m([zero, -i_flip, i_flip, zero]),
        m([flip, zero, zero, -flip]),
    ])
}

/// A gate with every qubit index moved by `offset`.
fn shifted(op: &GateOp, offset: usize) -> GateOp {
    match op {
        GateOp::H(q) => GateOp::H(q + offset),
        GateOp::X(q) => GateOp::X(q + offset),
        GateOp::Y(q) => GateOp::Y(q + offset),
        GateOp::Z(q) => GateOp::Z(q + offset),
        GateOp::S(q) => GateOp::S(q + offset),
        GateOp::Sdg(q) => GateOp::Sdg(q + offset),
        GateOp::T(q) => GateOp::T(q + offset),
        GateOp::Tdg(q) => GateOp::Tdg(q + offset),
        GateOp::Cnot { control, target } => GateOp::Cnot {
            control: control + offset,
            target: target + offset,
        },
        GateOp::Cz { control, target } => GateOp::Cz {
            control: control + offset,
            target: target + offset,
        },
        GateOp::Csdg { control, target } => GateOp::Csdg {
            control: control + offset,
            target: target + offset,
        },
        GateOp::Ccz { q0, q1, q2 } => GateOp::Ccz {
            q0: q0 + offset,
            q1: q1 + offset,
            q2: q2 + offset,
        },
        GateOp::Cmz { qubits } => GateOp::Cmz {
            qubits: qubits.iter().map(|q| q + offset).collect(),
        },
    }
}

/// A program on the middle qubits encoded by the outer code: middle qubit `j` is logical qubit
/// `j mod k` of block `j / k`, and each gate becomes the outer code's emitted program for it on
/// the block's wires. `Y` is `X` then `Z` up to a global phase; `CCZ` and `C^{m−1}Z` within a
/// block are the outer code's `C^{m−1}Z̄` on the block's representatives (Haruna, Table 1 row 6).
///
/// # Errors
///
/// [`QuantumError::DimensionMismatch`] if the outer code has no logical qubits;
/// [`QuantumError::CalculationError`] for a gate across blocks, for `CS†`, which needs a `CS̄†`
/// emitter Table 1 does not have, or for a gate kind the outer code has no emitter for; the
/// emitters' errors.
pub fn encode_program<W, R>(
    outer: &CodeAbstraction<W>,
    program: &[GateOp],
) -> Result<Vec<GateOp>, QuantumError>
where
    W: NaturalNumber,
    R: RealField + FromPrimitive,
{
    let n = outer.basis().len();
    let k = outer.basis().num_logical_qubits();
    if k == 0 {
        return Err(QuantumError::DimensionMismatch(
            "the outer code has no logical qubits, so a middle qubit has no block to stand in"
                .into(),
        ));
    }
    let place = |q: usize| (q / k, q % k);
    // The block a gate's qubits share and their logical indices in it.
    let one_block = |op: &GateOp, qubits: &[usize]| -> Result<(usize, Vec<usize>), QuantumError> {
        let (&first, rest) = qubits
            .split_first()
            .ok_or_else(|| QuantumError::CalculationError(format!("{op:?} names no qubit")))?;
        let (block, i) = place(first);
        let mut logical = vec![i];
        for &q in rest {
            let (b, j) = place(q);
            if b != block {
                return Err(QuantumError::CalculationError(format!(
                    "{op:?} acts across outer blocks {block} and {b}; a transversal gadget \
                     between code blocks is not part of this construction"
                )));
            }
            logical.push(j);
        }
        Ok((block, logical))
    };
    let mut out = Vec::new();
    for op in program {
        match op {
            GateOp::X(q)
            | GateOp::Y(q)
            | GateOp::Z(q)
            | GateOp::S(q)
            | GateOp::T(q)
            | GateOp::H(q) => {
                let (block, i) = place(*q);
                let gates: Vec<LogicalGate> = match op {
                    GateOp::X(_) => vec![LogicalGate::X(i)],
                    GateOp::Y(_) => vec![LogicalGate::Z(i), LogicalGate::X(i)],
                    GateOp::Z(_) => vec![LogicalGate::Z(i)],
                    GateOp::S(_) => vec![LogicalGate::S(i)],
                    GateOp::T(_) => vec![LogicalGate::T(i)],
                    _ => vec![LogicalGate::H(i)],
                };
                for g in gates {
                    out.extend(
                        outer
                            .program::<R>(&g)?
                            .iter()
                            .map(|o| shifted(o, block * n)),
                    );
                }
            }
            GateOp::Cz { control, target } => {
                let (block, logical) = one_block(op, &[*control, *target])?;
                out.extend(
                    outer
                        .program::<R>(&LogicalGate::Cz(logical[0], logical[1]))?
                        .iter()
                        .map(|o| shifted(o, block * n)),
                );
            }
            GateOp::Ccz { q0, q1, q2 } => {
                out.extend(encode_multi_cz::<W, _>(
                    outer,
                    op,
                    &[*q0, *q1, *q2],
                    &one_block,
                )?);
            }
            GateOp::Cmz { qubits } => {
                out.extend(encode_multi_cz::<W, _>(outer, op, qubits, &one_block)?);
            }
            GateOp::Csdg { .. } => {
                return Err(QuantumError::CalculationError(format!(
                    "{op:?} needs a logical CS̄† emitter, a controlled-S† on two representatives, \
                     which the outer code does not have: Haruna's Table 1 has no controlled-S row"
                )));
            }
            other => {
                return Err(QuantumError::CalculationError(format!(
                    "{other:?} has no emitter in the outer code; X, Y, Z, S, T, H, CZ, CCZ and \
                     C^(m-1)Z within a block are encoded"
                )));
            }
        }
    }
    Ok(out)
}

/// `C^{m−1}Z` on middle qubits within one block as the outer code's `C^{m−1}Z̄` on the block's
/// representatives, shifted onto the block's wires.
fn encode_multi_cz<W, F>(
    outer: &CodeAbstraction<W>,
    op: &GateOp,
    qubits: &[usize],
    one_block: F,
) -> Result<Vec<GateOp>, QuantumError>
where
    W: NaturalNumber,
    F: Fn(&GateOp, &[usize]) -> Result<(usize, Vec<usize>), QuantumError>,
{
    let n = outer.basis().len();
    let (block, logical) = one_block(op, qubits)?;
    let chains: Vec<&Gf2Chain<W>> = logical
        .iter()
        .map(|&i| &outer.basis().homology()[i])
        .collect();
    Ok(logical_multi_cz(&chains)?
        .iter()
        .map(|o| shifted(o, block * n))
        .collect())
}

/// The logical program of a Table 1 gate on the high level.
fn logical_of(gate: &LogicalGate) -> Vec<GateOp> {
    match gate {
        LogicalGate::Z(i) => vec![GateOp::Z(*i)],
        LogicalGate::X(i) => vec![GateOp::X(*i)],
        LogicalGate::S(i) => vec![GateOp::S(*i)],
        LogicalGate::T(i) => vec![GateOp::T(*i)],
        LogicalGate::H(i) => vec![GateOp::H(*i)],
        LogicalGate::Cz(i, j) => vec![GateOp::Cz {
            control: *i,
            target: *j,
        }],
    }
}

/// The concatenation of `inner` with `outer` for one logical gate. See the module documentation.
///
/// # Errors
///
/// [`QuantumError::DimensionMismatch`] if the outer code has no logical qubits or the inner
/// code's qubit count is not a multiple of the outer code's logical count; [`encode_program`]'s
/// errors; the constructors'.
pub fn concatenated_code<W, KI, KO, R>(
    inner: &KI,
    outer: &KO,
    gate: &LogicalGate,
) -> Result<Chain<R>, QuantumError>
where
    W: NaturalNumber,
    KI: ChainComplex + ?Sized,
    KO: ChainComplex + ?Sized,
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    let inner_code = CodeAbstraction::<W>::new(inner, vec![gate.clone()])?;
    let outer_code = CodeAbstraction::<W>::new(outer, vec![])?;
    let n_in = inner_code.basis().len();
    let k_in = inner_code.basis().num_logical_qubits();
    let n_out = outer_code.basis().len();
    let k_out = outer_code.basis().num_logical_qubits();
    if k_out == 0 {
        return Err(QuantumError::DimensionMismatch(
            "the outer code has no logical qubits, so the inner code's qubits have no blocks to \
             fill"
                .into(),
        ));
    }
    if n_in % k_out != 0 {
        return Err(QuantumError::DimensionMismatch(format!(
            "the inner code's {n_in} qubits do not fill blocks of the outer code's {k_out} logical qubits"
        )));
    }
    let blocks = n_in / k_out;
    let inner_program = inner_code.program::<R>(gate)?;
    let second =
        inner_code.numeric_abstraction_of::<R>(&logical_of(gate), inner_program.clone(), false)?;
    let middle = second.low().clone();

    let inner_recovery = IdealRecovery::<R>::from_basis(inner_code.basis())?;
    let outer_recovery = IdealRecovery::<R>::from_basis(outer_code.basis())?;
    let n_low = blocks * n_out;
    // Middle qubit `j` stands on the low-level wire `(j / k_out) · n_out + j mod k_out`, the
    // `(j mod k_out)`-th logical input of its outer block; the inner encoder acts on those wires
    // in middle order before the outer encoders act on the blocks.
    let middle_wire = |j: usize| (j / k_out) * n_out + j % k_out;
    let mut boxes: Vec<CircuitBox<R>> = Vec::with_capacity(blocks + 2);
    boxes.push(CircuitBox::Kraus {
        wires: (0..n_in).map(middle_wire).collect(),
        kraus: inner_recovery.encoder().kraus(),
    });
    for b in 0..blocks {
        boxes.push(CircuitBox::Kraus {
            wires: (b * n_out..(b + 1) * n_out).collect(),
            kraus: outer_recovery.encoder().kraus(),
        });
    }
    boxes.push(CircuitBox::Unitary {
        wires: (0..n_low).collect(),
        program: encode_program::<W, R>(&outer_code, &inner_program)?,
    });
    // The low-level inputs are the outer-encoder inputs of the blocks that carry the inner code's
    // logical inputs; the other blocks start at |0…0⟩ as the middle qubits they encode do.
    let inputs: Vec<usize> = (0..k_in).map(middle_wire).collect();
    let low = CircuitModel::ungrouped(
        vec![WireType::qubit(); n_low],
        boxes,
        inputs,
        (0..n_low).collect(),
    )?;

    let identity = QcMorphism::from_kraus(&[identity_matrix::<R>(1 << k_out)])?;
    let mut entries: Vec<(AlignmentSide, AlignmentSpec<R>)> = Vec::new();
    for b in 0..blocks {
        let middle_wires: Vec<usize> = (b * k_out..(b + 1) * k_out).collect();
        let low_logical: Vec<usize> = (b * n_out..b * n_out + k_out).collect();
        let low_block: Vec<usize> = (b * n_out..(b + 1) * n_out).collect();
        entries.push((
            AlignmentSide::Input,
            (
                middle_wires.clone(),
                low_logical,
                identity.clone(),
                identity.clone(),
            ),
        ));
        entries.push((
            AlignmentSide::Output,
            (
                middle_wires,
                low_block,
                outer_recovery.recovery().clone(),
                outer_recovery.isometry().clone(),
            ),
        ));
    }
    let first = Abstraction::new(
        low,
        middle,
        TypeAlignment::new_sided(entries)?,
        vec![(Query::Io, Query::Io)],
    )?;
    Ok(Chain { first, second })
}

/// The link `L → M` whose `k` logical inputs and `n` physical outputs align by the identity, over
/// the `Io` query: the shape code switching and a distillation round share.
fn identity_link<R>(
    low: CircuitModel<R>,
    middle: CircuitModel<R>,
    k: usize,
    n: usize,
) -> Result<Abstraction<R, CircuitModel<R>, CircuitModel<R>>, QuantumError>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    let physical_identity = QcMorphism::from_kraus(&[identity_matrix::<R>(1 << n)])?;
    let logical_identity = QcMorphism::from_kraus(&[identity_matrix::<R>(1 << k)])?;
    Abstraction::new(
        low,
        middle,
        TypeAlignment::new_sided(vec![
            (
                AlignmentSide::Input,
                (
                    (0..k).collect(),
                    (0..k).collect(),
                    logical_identity.clone(),
                    logical_identity,
                ),
            ),
            (
                AlignmentSide::Output,
                (
                    (0..n).collect(),
                    (0..n).collect(),
                    physical_identity.clone(),
                    physical_identity,
                ),
            ),
        ])?,
        vec![(Query::Io, Query::Io)],
    )
}

/// Code switching from `from` into `into` for one logical gate, with `gadget_noise` a Kraus family
/// on one qubit applied to the first logical wire between the decoder and the re-encoder, or empty
/// for a noiseless gadget. See the module documentation.
///
/// # Errors
///
/// [`QuantumError::DimensionMismatch`] if the codes encode different logical counts; the
/// constructors'.
pub fn code_switching<W, KA, KB, R>(
    from: &KA,
    into: &KB,
    gate: &LogicalGate,
    gadget_noise: Vec<CausalTensor<Complex<R>>>,
) -> Result<Chain<R>, QuantumError>
where
    W: NaturalNumber,
    KA: ChainComplex + ?Sized,
    KB: ChainComplex + ?Sized,
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    let code_a = CodeAbstraction::<W>::new(from, vec![])?;
    let code_b = CodeAbstraction::<W>::new(into, vec![gate.clone()])?;
    let (n_a, k_a) = (code_a.basis().len(), code_a.basis().num_logical_qubits());
    let (n_b, k_b) = (code_b.basis().len(), code_b.basis().num_logical_qubits());
    if k_a != k_b {
        return Err(QuantumError::DimensionMismatch(format!(
            "code switching needs equal logical counts, not {k_a} and {k_b}"
        )));
    }
    let n = n_a.max(n_b);
    let program_b = code_b.program::<R>(gate)?;
    let second = code_b.numeric_abstraction_of::<R>(&logical_of(gate), program_b.clone(), false)?;
    let middle = second.low().clone();
    let recovery_a = IdealRecovery::<R>::from_basis(code_a.basis())?;
    let recovery_b = IdealRecovery::<R>::from_basis(code_b.basis())?;
    let encoder_a = recovery_a.encoder().kraus();
    let decoder_a: Vec<CausalTensor<Complex<R>>> = encoder_a
        .iter()
        .map(|k| {
            k.dagger()
                .map_err(|e| QuantumError::CalculationError(format!("dagger: {e:?}")))
        })
        .collect::<Result<_, _>>()?;
    let mut boxes: Vec<CircuitBox<R>> = vec![
        CircuitBox::Kraus {
            wires: (0..n_a).collect(),
            kraus: encoder_a,
        },
        CircuitBox::Kraus {
            wires: (0..n_a).collect(),
            kraus: decoder_a,
        },
    ];
    if !gadget_noise.is_empty() {
        boxes.push(CircuitBox::Kraus {
            wires: vec![0],
            kraus: gadget_noise,
        });
    }
    boxes.push(CircuitBox::Kraus {
        wires: (0..n_b).collect(),
        kraus: recovery_b.encoder().kraus(),
    });
    boxes.push(CircuitBox::Unitary {
        wires: (0..n_b).collect(),
        program: program_b,
    });
    // The middle model has n_b wires; the low-level model needs the wider of the two codes.
    let (low, middle) = if n == n_b {
        (
            CircuitModel::ungrouped(
                vec![WireType::qubit(); n],
                boxes,
                (0..k_a).collect(),
                (0..n).collect(),
            )?,
            middle,
        )
    } else {
        return Err(QuantumError::DimensionMismatch(format!(
            "code switching into a narrower code ({n_b} qubits from {n_a}) is not part of this construction"
        )));
    };
    let first = identity_link(low, middle, k_a, n)?;
    Ok(Chain { first, second })
}

/// A distillation round on `complex` with depolarising probability `p` on every physical qubit.
/// See the module documentation.
///
/// # Errors
///
/// [`depolarizing_kraus`]'s range error; the emitters' errors for `H̄` and `T̄`; the constructors'.
pub fn distillation_round<W, K, R>(complex: &K, p: R) -> Result<Chain<R>, QuantumError>
where
    W: NaturalNumber,
    K: ChainComplex + ?Sized,
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    let code = CodeAbstraction::<W>::new(complex, vec![])?;
    let (n, k) = (code.basis().len(), code.basis().num_logical_qubits());
    let mut logical = Vec::with_capacity(2 * k);
    let mut physical = Vec::new();
    for i in 0..k {
        logical.push(GateOp::H(i));
        logical.push(GateOp::T(i));
        physical.extend(code.program::<R>(&LogicalGate::H(i))?);
        physical.extend(code.program::<R>(&LogicalGate::T(i))?);
    }
    let second = code.numeric_abstraction_of::<R>(&logical, physical.clone(), false)?;
    let middle = second.low().clone();
    let recovery = IdealRecovery::<R>::from_basis(code.basis())?;
    let noise = depolarizing_kraus::<R>(p)?;
    let mut boxes: Vec<CircuitBox<R>> = vec![CircuitBox::Kraus {
        wires: (0..n).collect(),
        kraus: recovery.encoder().kraus(),
    }];
    for w in 0..n {
        boxes.push(CircuitBox::Kraus {
            wires: vec![w],
            kraus: noise.clone(),
        });
    }
    boxes.push(CircuitBox::Unitary {
        wires: (0..n).collect(),
        program: physical,
    });
    let low = CircuitModel::ungrouped(
        vec![WireType::qubit(); n],
        boxes,
        (0..k).collect(),
        (0..n).collect(),
    )?;
    let first = identity_link(low, middle, k, n)?;
    Ok(Chain { first, second })
}
