/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Monad law 3 (`core.causaloid.encapsulation_flat`) exercised over the
//! arity-5 STATE channel carrying complex-MATRIX payloads at
//! `Complex<Float106>` — nested and flat evaluation of an operator-valued
//! process agree up to numerical tolerance (task 2.6, spec
//! quantum-operator-layer "Encapsulation-equals-flat exercised over
//! matrix-valued state").

use deep_causality_core::{
    CausalEffect, CausalEffectPropagationProcess, CausalityError, EffectLog, PropagatingEffect,
};
use deep_causality_num::Float106;
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{identity_matrix, matrix_trace};
use deep_causality_tensor::{CausalTensor, Tensor};

type F = Float106;
type CF = Complex<F>;
type MatState = CausalTensor<CF>;
type Proc = CausalEffectPropagationProcess<F, MatState, (), CausalityError, EffectLog>;

fn f(x: f64) -> F {
    Float106::from_f64(x)
}

fn c(re: f64, im: f64) -> CF {
    Complex::new(f(re), f(im))
}

fn mat2(data: [[(f64, f64); 2]; 2]) -> MatState {
    let d: Vec<CF> = data
        .iter()
        .flat_map(|row| row.iter().map(|(re, im)| c(*re, *im)))
        .collect();
    CausalTensor::new(d, vec![2, 2]).unwrap()
}

fn sigma_x() -> MatState {
    mat2([[(0., 0.), (1., 0.)], [(1., 0.), (0., 0.)]])
}

fn sigma_y() -> MatState {
    mat2([[(0., 0.), (0., -1.)], [(0., 1.), (0., 0.)]])
}

fn hadamard() -> MatState {
    let h = 1.0 / 2.0_f64.sqrt();
    mat2([[(h, 0.), (h, 0.)], [(h, 0.), (-h, 0.)]])
}

fn conjugate_by(u: &MatState, m: &MatState) -> MatState {
    u.matmul(m).unwrap().matmul(&u.dagger().unwrap()).unwrap()
}

fn max_abs_diff(a: &MatState, b: &MatState) -> f64 {
    assert_eq!(
        a.shape(),
        b.shape(),
        "max_abs_diff shape mismatch: {:?} vs {:?}",
        a.shape(),
        b.shape()
    );
    a.as_slice()
        .iter()
        .zip(b.as_slice())
        .map(|(x, y)| {
            let dr: f64 = (x.re - y.re).into();
            let di: f64 = (x.im - y.im).into();
            (dr * dr + di * di).sqrt()
        })
        .fold(0.0, f64::max)
}

/// Seeds a matrix-state process: value = running trace, state = ρ.
fn seed(rho: MatState) -> Proc {
    let tr = matrix_trace(&rho).unwrap().re;
    CausalEffectPropagationProcess::with_state(PropagatingEffect::from_value(tr), rho, None)
}

/// A process step: conjugate the STATE matrix by `u` and add the resulting
/// trace onto the value channel — an operator-valued Markov step.
fn step(u: MatState) -> impl FnOnce(CausalEffect<F>, MatState, Option<()>) -> Proc {
    move |effect, state, _ctx| {
        let new_state = conjugate_by(&u, &state);
        let tr = matrix_trace(&new_state).unwrap().re;
        let val = effect.into_value().unwrap_or_else(F::default);
        CausalEffectPropagationProcess::with_state(
            PropagatingEffect::from_value(val + tr),
            new_state,
            None,
        )
    }
}

/// A genuinely non-unitary step: mix the state with its σx-conjugate
/// (a dephasing-like channel), still trace-preserving.
fn channel_step() -> impl FnOnce(CausalEffect<F>, MatState, Option<()>) -> Proc {
    move |effect, state, _ctx| {
        let flipped = conjugate_by(&sigma_x(), &state);
        let half = c(0.5, 0.0);
        let mixed_data: Vec<CF> = state
            .as_slice()
            .iter()
            .zip(flipped.as_slice())
            .map(|(a, b)| {
                let s = Complex::new(a.re + b.re, a.im + b.im);
                Complex::new(
                    s.re * half.re - s.im * half.im,
                    s.re * half.im + s.im * half.re,
                )
            })
            .collect();
        let mixed = CausalTensor::new(mixed_data, vec![2, 2]).unwrap();
        let tr = matrix_trace(&mixed).unwrap().re;
        let val = effect.into_value().unwrap_or_else(F::default);
        CausalEffectPropagationProcess::with_state(
            PropagatingEffect::from_value(val + tr),
            mixed,
            None,
        )
    }
}

fn initial_rho() -> MatState {
    // A non-trivial 2x2 density-like matrix (Hermitian, trace 1).
    mat2([[(0.7, 0.), (0.2, 0.1)], [(0.2, -0.1), (0.3, 0.)]])
}

#[test]
fn test_encapsulation_flat_on_matrix_state() {
    // Flat: one pass through all three operator steps.
    let flat = seed(initial_rho())
        .bind(step(hadamard()))
        .bind(step(sigma_y()))
        .bind(channel_step());

    // Nested: the same chain with the tail encapsulated in a single bound
    // computation — monad law 3 (associativity) on the matrix STATE channel:
    // (m >>= f) >>= g  ==  m >>= (λx. f x >>= g).
    let nested = seed(initial_rho()).bind(|e, s, ctx| {
        step(hadamard())(e, s, ctx)
            .bind(step(sigma_y()))
            .bind(channel_step())
    });

    let (f_out, fs, _, _) = flat.into_parts();
    let (n_out, ns, _, _) = nested.into_parts();
    let fv = f_out.unwrap().into_value().unwrap();
    let nv = n_out.unwrap().into_value().unwrap();

    let dv: f64 = (fv - nv).into();
    assert!(dv.abs() < 1e-30, "value channels diverged: {}", dv);
    assert!(
        max_abs_diff(&fs, &ns) < 1e-30,
        "matrix STATE channels diverged: {}",
        max_abs_diff(&fs, &ns)
    );
}

#[test]
fn test_two_stage_encapsulation_matches_one_pass() {
    // The graph-style wrapped/two-stage evaluation: run a prefix process to
    // completion, then seed a suffix process with its (value, state).
    let one_pass = seed(initial_rho())
        .bind(step(hadamard()))
        .bind(step(sigma_y()))
        .bind(channel_step());

    let prefix = seed(initial_rho()).bind(step(hadamard()));
    let (mid_out, mid_state, _, _) = prefix.into_parts();
    let mid_value = mid_out.unwrap().into_value().expect("prefix value");
    let suffix = CausalEffectPropagationProcess::with_state(
        PropagatingEffect::from_value(mid_value),
        mid_state,
        None,
    )
    .bind(step(sigma_y()))
    .bind(channel_step());

    let (o_out, o_state, _, _) = one_pass.into_parts();
    let (s_out, s_state, _, _) = suffix.into_parts();
    let dv: f64 =
        (o_out.unwrap().into_value().unwrap() - s_out.unwrap().into_value().unwrap()).into();
    assert!(dv.abs() < 1e-30);
    assert!(max_abs_diff(&o_state, &s_state) < 1e-30);
}

#[test]
fn test_matrix_state_error_short_circuit_preserves_state_type() {
    // The error left-zero on the matrix-state carrier: after an error, later
    // steps never run and the process stays typed over the matrix state.
    let err = CausalityError::new(deep_causality_core::CausalityErrorEnum::Custom(
        "boom".into(),
    ));
    let p: Proc = CausalEffectPropagationProcess::from_error(err);
    let after = p.bind(step(sigma_y()));
    assert!(after.is_err());
    // The identity of the state space is intact (default = empty tensor is
    // fine; the point is the typed channel survived the short circuit).
    let _ = identity_matrix::<F>(2);
}
