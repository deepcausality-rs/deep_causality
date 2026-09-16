/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Tensor <-> Algebra Round-Trip Inside the Causal Monad
//!
//! A vector starts as a `CausalTensor<FloatType>` of shape `[3]`. It is lifted
//! into `Cl(3,0)` as a pure vector, rotated by a Clifford rotor, then lowered
//! back to a tensor. A final tensor dot product confirms the norm was preserved.
//!
//! Each step is a `bind` on `CausalEffectPropagationProcess`. The carried
//! value type changes between steps: `CausalTensor` -> `CausalMultiVector` ->
//! `CausalTensor` -> `FloatType`. The monad threads them in one straight line.
//!
//! ## APIs Demonstrated
//! - Fluent `.bind(|value, state, context| ...)` across heterogeneous value types
//! - `CausalMultiVector::geometric_product`
//! - `EinSumOp::dot_prod` for the norm check

use deep_causality_algebra::Real;
use deep_causality_haft::Pure;
use deep_causality_metric::Metric;
use deep_causality_multivector::CausalMultiVector;
use deep_causality_num::{Float106, const_scalar_from_int};
use deep_causality_tensor::{CausalTensor, EinSumOp, Tensor};
use mathematics_examples::effect_helpers::{Process, ProcessWitness, fail, ok, print_log};

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision.
pub type FloatType = Float106;

/// Small numbers, declared once at the working type rather than lifted at each use.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const TWO: FloatType = const_scalar_from_int!(FloatType, 2);
const THREE: FloatType = const_scalar_from_int!(FloatType, 3);
const FOUR: FloatType = const_scalar_from_int!(FloatType, 4);

fn main() {
    print_header();

    let initial = CausalTensor::new(vec![THREE, FOUR, ZERO], vec![3])
        .expect("three components in a rank-1 shape of 3");
    let initial_norm_sq: FloatType = initial.as_slice().iter().fold(ZERO, |acc, &v| acc + v * v);
    print_initial(initial.as_slice(), initial_norm_sq);

    // One straight-line monadic chain. The carried value type changes at every
    // step (Tensor -> MultiVector -> Tensor -> scalar); `bind` threads the
    // value, state, context, error, and log automatically. The closure receives
    // the upstream `CausalEffect`; an error in any step short-circuits the rest.
    let result: Process<FloatType> = ProcessWitness::pure(initial)
        .bind(|v, _, _| lift_to_algebra(v.into_value().expect("initial tensor")))
        .bind(|v, _, _| rotate_in_xy(v.into_value().expect("lifted multivector")))
        .bind(|v, _, _| lower_to_tensor(v.into_value().expect("rotated multivector")))
        .bind(|v, _, _| norm_squared(v.into_value().expect("lowered tensor")));

    print_log(result.logs());
    print_outcome(
        result.error().map(|e| e.to_string()),
        result.value_cloned(),
        initial_norm_sq,
    );
}

fn lift_to_algebra(v: CausalTensor<FloatType>) -> Process<CausalMultiVector<FloatType>> {
    let s = v.as_slice();
    if s.len() != 3 {
        return fail(format!("lift: expected length 3 vector, got {}", s.len()));
    }
    let zero = ZERO;
    // Cl(3,0) basis order: [1, e1, e2, e3, e12, e13, e23, e123]
    let coeffs = vec![zero, s[0], s[1], s[2], zero, zero, zero, zero];
    match cl3(coeffs, Metric::Euclidean(3)) {
        Ok(mv) => {
            let msg = format!("lift: tensor {:?} -> multivector vector", s);
            ok(mv, msg)
        }
        Err(e) => fail(e),
    }
}

/// A `Cl(3,0)` element, with the constructor's error carried as a message the chain can hold.
fn cl3(coeffs: Vec<FloatType>, metric: Metric) -> Result<CausalMultiVector<FloatType>, String> {
    CausalMultiVector::new(coeffs, metric)
        .map_err(|e| format!("building a Cl(3,0) element failed: {e:?}"))
}

fn rotate_in_xy(v: CausalMultiVector<FloatType>) -> Process<CausalMultiVector<FloatType>> {
    match rotate_step(v) {
        Ok(rotated) => ok(rotated, "rotate: 90 deg in e1^e2 plane"),
        Err(e) => fail(e),
    }
}

/// A 90-degree rotation in the `e1^e2` plane, as a rotor sandwich.
fn rotate_step(v: CausalMultiVector<FloatType>) -> Result<CausalMultiVector<FloatType>, String> {
    let metric = Metric::Euclidean(3);
    let half = FloatType::pi() / TWO / TWO;
    let (c, sn) = (half.cos(), half.sin());
    let zero = ZERO;

    let rotor = cl3(vec![c, zero, zero, zero, -sn, zero, zero, zero], metric)?;
    let rotor_rev = cl3(vec![c, zero, zero, zero, sn, zero, zero, zero], metric)?;
    Ok(rotor.geometric_product(&v).geometric_product(&rotor_rev))
}

fn lower_to_tensor(mv: CausalMultiVector<FloatType>) -> Process<CausalTensor<FloatType>> {
    let d = mv.data();
    match CausalTensor::new(vec![d[1], d[2], d[3]], vec![3]) {
        Ok(v) => {
            let msg = format!("lower: multivector -> tensor {:?}", v.as_slice());
            ok(v, msg)
        }
        Err(e) => fail(format!("lower: rebuilding the tensor failed: {e:?}")),
    }
}

fn norm_squared(v: CausalTensor<FloatType>) -> Process<FloatType> {
    let copy = v.clone();
    let ast = EinSumOp::dot_prod(v, copy);
    let result = match CausalTensor::ein_sum(&ast) {
        Ok(t) => t.as_slice()[0],
        Err(e) => return fail(format!("dot_prod failed: {:?}", e)),
    };
    ok(result, format!("norm^2 via tensor dot_prod = {}", result))
}

// -----------------------------------------------------------------------------------------
// Printing
//
// The working type prints itself. At `Float106` its own `Display` carries the extra digits
// this example exists to show, so nothing is lowered to `f64` on the way out.
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== Tensor <-> Algebra Round-Trip Inside the Causal Monad ===");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());
}

fn print_initial(vector: &[FloatType], norm_sq: FloatType) {
    println!("Initial vector: {vector:?}");
    println!("Initial |v|^2 = {norm_sq}\n");
}

fn print_outcome(error: Option<String>, final_norm_sq: Option<FloatType>, initial: FloatType) {
    match error {
        Some(e) => println!("\nChain errored: {e}"),
        None => {
            let final_norm_sq = final_norm_sq.expect("a chain with no error carries a value");
            let drift = (final_norm_sq - initial).abs();
            println!("\nFinal |v|^2 = {final_norm_sq}");
            println!("Round-trip drift = {drift} (should be at machine epsilon)");
        }
    }
}
