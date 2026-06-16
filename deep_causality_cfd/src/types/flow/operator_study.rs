/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The **operator-accuracy** CfdFlow solver kind: a DEC operator swept over a set of
//! resolutions for its convergence order — no field march.
//!
//! The study evaluates a DEC operator on the analytic 2D Taylor–Green field (a Laplacian
//! eigenfield) at each resolution of a periodic torus, measures the relative `L²` error against
//! the operator's exact action, and reports the observed convergence orders `log₂(eₙ / e₂ₙ)`.
//! The **viscous** operator is the 1-form Hodge Laplacian `δd` (for the divergence-free field
//! `δu = 0`, so `Δu♭ = δd u♭`); its exact action on the `k = 1` Taylor–Green field is `2 u♭`
//! (the vector Laplacian `∇²u = −2u`, and `Δ = −∇²` on the flat torus). The diagonal-Hodge DEC
//! `δd` reproduces the eigenvalue to `O(h²)`, so the study reports second order.

use crate::traits::Solver;
use crate::types::CfdScalar;
use crate::types::flow::{CfdFlow, Report};
use crate::types::flow_config::Mesh;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, LatticeComplex, Manifold};

/// A DEC operator whose discretization accuracy is studied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// The viscous operator: the 1-form Hodge Laplacian `δd` (the `δd f` of the design note).
    Viscous,
}

impl CfdFlow {
    /// Begin an operator-accuracy study. The resolutions and operator are set fluently; `run`
    /// sweeps the operator over a periodic torus at each resolution and reports the per-resolution
    /// `operator_error` and the observed `convergence_order` between consecutive resolutions.
    pub fn operator_study<R: CfdScalar>(name: impl Into<String>) -> OperatorStudyBuilder<R> {
        OperatorStudyBuilder::new(name)
    }
}

/// Fluent builder for an operator-accuracy study.
pub struct OperatorStudyBuilder<R: CfdScalar> {
    name: String,
    resolutions: Vec<usize>,
    operator: Operator,
    _scalar: core::marker::PhantomData<R>,
}

impl<R: CfdScalar> OperatorStudyBuilder<R> {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            resolutions: vec![16, 32, 64],
            operator: Operator::Viscous,
            _scalar: core::marker::PhantomData,
        }
    }

    /// Set the resolution sweep (the per-axis cell counts of the periodic torus).
    pub fn resolutions(mut self, resolutions: impl Into<Vec<usize>>) -> Self {
        self.resolutions = resolutions.into();
        self
    }

    /// Select the operator under study (default [`Operator::Viscous`]).
    pub fn operator(mut self, operator: Operator) -> Self {
        self.operator = operator;
        self
    }

    /// Assemble and run the study.
    ///
    /// # Errors
    /// `PhysicsError::DimensionMismatch` when fewer than two resolutions are given (no order is
    /// defined), plus any materialization or operator failure.
    pub fn run(self) -> Result<Report<R>, PhysicsError> {
        OperatorStudyCase {
            name: self.name,
            resolutions: self.resolutions,
            operator: self.operator,
            _scalar: core::marker::PhantomData::<R>,
        }
        .run()
    }
}

/// An owned operator-accuracy case.
struct OperatorStudyCase<R: CfdScalar> {
    name: String,
    resolutions: Vec<usize>,
    operator: Operator,
    _scalar: core::marker::PhantomData<R>,
}

impl<R: CfdScalar> Solver<R> for OperatorStudyCase<R> {
    fn run(self) -> Result<Report<R>, PhysicsError> {
        if self.resolutions.len() < 2 {
            return Err(PhysicsError::DimensionMismatch(
                "operator_study: at least two resolutions are required for a convergence order"
                    .into(),
            ));
        }

        let mut errors: Vec<R> = Vec::with_capacity(self.resolutions.len());
        for &n in &self.resolutions {
            errors.push(operator_error::<R>(self.operator, n)?);
        }

        // Observed order between consecutive resolutions: log₂(eₙ / e₂ₙ).
        let two = R::from_f64(2.0).expect("2.0 lifts into every real field");
        let ln2 = two.ln();
        let mut orders: Vec<R> = Vec::with_capacity(errors.len() - 1);
        for w in errors.windows(2) {
            let order = if w[1] > R::zero() {
                (w[0] / w[1]).ln() / ln2
            } else {
                R::zero()
            };
            orders.push(order);
        }

        let mut report = Report::new(self.name);
        report.add_series("operator_error", errors);
        report.add_series("convergence_order", orders);
        Ok(report)
    }
}

/// The relative `L²` error of the operator's discrete action against its exact action on the
/// analytic Taylor–Green field, on an `n × n` periodic torus of spacing `2π/n`.
fn operator_error<R: CfdScalar>(operator: Operator, n: usize) -> Result<R, PhysicsError> {
    let two_pi = R::from_f64(2.0 * core::f64::consts::PI).expect("2π lifts into R");
    let h = two_pi / R::from_usize(n).expect("a resolution lifts into R");
    let (manifold, _) = Mesh::<2, R>::periodic_cube(n).spacing(h).materialize()?;

    // The analytic Taylor–Green velocity as an edge 1-form.
    let vertex = taylor_green_2d::<R>(&manifold, h);
    let edge = manifold
        .de_rham(&vertex)
        .map_err(|e| PhysicsError::TopologyError(format!("de_rham failed: {e}")))?;

    match operator {
        Operator::Viscous => {
            // δd on the edge 1-form; exact action on the k = 1 eigenfield is 2 u♭.
            let du = manifold.exterior_derivative_of(edge.as_slice(), 1);
            let dde = manifold.codifferential_of(du.as_slice(), 2);
            let two = R::from_f64(2.0).expect("2.0 lifts into R");
            Ok(relative_l2_against_scaled(
                dde.as_slice(),
                edge.as_slice(),
                two,
            ))
        }
    }
}

/// The 2D Taylor–Green vertex field `u = (cos x sin y, −sin x cos y)` on the lattice of spacing
/// `h`, in the `de_rham` input layout (`vertex · D + axis`). A divergence-free Laplacian
/// eigenfield with eigenvalue `−2`.
fn taylor_green_2d<R: CfdScalar>(
    manifold: &Manifold<LatticeComplex<2, R>, R>,
    h: R,
) -> CausalTensor<R> {
    let n0 = manifold.complex().num_cells(0);
    let mut v = vec![R::zero(); 2 * n0];
    for (vi, cell) in manifold.complex().iter_cells(0).enumerate() {
        let p = cell.position();
        let x = R::from_usize(p[0]).expect("a lattice index lifts into R") * h;
        let y = R::from_usize(p[1]).expect("a lattice index lifts into R") * h;
        v[2 * vi] = x.cos() * y.sin();
        v[2 * vi + 1] = R::zero() - x.sin() * y.cos();
    }
    CausalTensor::new(v, vec![2 * n0]).expect("a 1-D vertex tensor cannot fail to allocate")
}

/// `‖computed − scale·reference‖₂ / ‖scale·reference‖₂`.
fn relative_l2_against_scaled<R: CfdScalar>(computed: &[R], reference: &[R], scale: R) -> R {
    let mut num = R::zero();
    let mut den = R::zero();
    for (&c, &r) in computed.iter().zip(reference.iter()) {
        let target = scale * r;
        let d = c - target;
        num += d * d;
        den += target * target;
    }
    if den > R::zero() {
        (num / den).sqrt()
    } else {
        R::zero()
    }
}
