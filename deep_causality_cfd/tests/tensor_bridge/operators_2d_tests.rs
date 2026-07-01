/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_cfd::{dequantize_2d, gradient_x, gradient_y, laplacian_2d, quantize_2d};
use deep_causality_tensor::{
    CausalTensor, CausalTensorTrainOperator, TensorTrainOperator, Truncation,
};

const TAU: f64 = core::f64::consts::TAU;

fn full() -> Truncation<f64> {
    Truncation::by_bond(4096).unwrap()
}

fn field2d(nx: usize, ny: usize, f: impl Fn(usize, usize) -> f64) -> CausalTensor<f64> {
    let mut data = vec![0.0; nx * ny];
    for i in 0..nx {
        for j in 0..ny {
            data[i * ny + j] = f(i, j);
        }
    }
    CausalTensor::new(data, vec![nx, ny]).unwrap()
}

fn apply_2d(
    op: &CausalTensorTrainOperator<f64>,
    u: &CausalTensor<f64>,
    lx: usize,
    ly: usize,
) -> Vec<f64> {
    let q = quantize_2d(u, &full()).unwrap();
    let out = op.apply(&q, &full()).unwrap();
    dequantize_2d(&out, lx, ly).unwrap().as_slice().to_vec()
}

#[test]
fn round_trips_2d() {
    let (nx, ny) = (8usize, 8usize);
    let u = field2d(nx, ny, |i, j| {
        ((i * ny + j) as f64 * 0.3).sin() - 0.2 * j as f64
    });
    let q = quantize_2d(&u, &full()).unwrap();
    let back = dequantize_2d(&q, 3, 3).unwrap();
    assert_eq!(back.shape(), &[8, 8]);
    for (a, b) in back.as_slice().iter().zip(u.as_slice()) {
        assert!((a - b).abs() <= 1e-12, "2-D round-trip differs: {a} vs {b}");
    }
}

#[test]
fn axis_derivatives_hit_the_right_axis() {
    let (nx, ny, dx) = (8usize, 8usize, 0.5f64);
    // A field varying only along x.
    let u = field2d(nx, ny, |i, _j| (TAU * i as f64 / nx as f64).sin());
    let gx = gradient_x::<f64>(3, 3, dx, &full()).unwrap();
    let gy = gradient_y::<f64>(3, 3, dx, &full()).unwrap();
    let dudx = apply_2d(&gx, &u, 3, 3);
    let dudy = apply_2d(&gy, &u, 3, 3);
    let s = |i: usize| (TAU * i as f64 / nx as f64).sin();
    for i in 0..nx {
        let want = (s((i + 1) % nx) - s((i + nx - 1) % nx)) / (2.0 * dx);
        for j in 0..ny {
            assert!(
                (dudx[i * ny + j] - want).abs() <= 1e-9,
                "∂ₓ wrong at {i},{j}"
            );
            assert!(
                dudy[i * ny + j].abs() <= 1e-9,
                "∂ᵧ of x-only field not zero at {i},{j}"
            );
        }
    }
}

#[test]
fn laplacian_2d_matches_five_point_stencil() {
    let (nx, ny, dx) = (8usize, 8usize, 0.5f64);
    let u = |i: usize, j: usize| {
        (TAU * i as f64 / nx as f64).sin() + (TAU * j as f64 / ny as f64).cos()
    };
    let f = field2d(nx, ny, u);
    let lap = laplacian_2d::<f64>(3, 3, dx, dx, &full()).unwrap();
    let got = apply_2d(&lap, &f, 3, 3);
    for i in 0..nx {
        for j in 0..ny {
            let want = (u((i + 1) % nx, j)
                + u((i + nx - 1) % nx, j)
                + u(i, (j + 1) % ny)
                + u(i, (j + ny - 1) % ny)
                - 4.0 * u(i, j))
                / (dx * dx);
            assert!(
                (got[i * ny + j] - want).abs() <= 1e-9,
                "Δ wrong at {i},{j}: {} vs {want}",
                got[i * ny + j]
            );
        }
    }
}
