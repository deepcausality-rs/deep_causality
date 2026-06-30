// SPDX-License-Identifier: MIT
// Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

//! QTT re-pinning marcher — the positive half of the Resolution-5 lever, and the heart of Stage 4.
//!
//! `qtt_rank_fitted_dynamic` measured the negative half: a **static** body-fitted coordinate is not
//! enough. Under Cartesian fluxes a marched radial front drifts off a fixed curvilinear chart and the
//! bond climbs with resolution (25 → 35), no better than the capture. The fix Resolution 5 / D9
//! prescribes is **feedback re-pinning**: track the live front and move the coordinate with it so the
//! feature stays coordinate-stationary.
//!
//! This study prototypes that. Each step, after one Burgers update in the polar frame, it locates the
//! front (steepest radial gradient), rolls the field back to a fixed computational band `η = ½`, and
//! slides the annulus inner radius `r₀` so the front's physical radius maps to that band — then rebuilds
//! the low-rank metric for the new `r₀`. The front therefore never moves in computational space; its
//! representation, and so its bond, should stay bounded and resolution-flat where the static chart grew.
//!
//! Cases: static polar march vs re-pinned polar march, each at 64² and 128², so the **resolution growth**
//! is the headline. Self-verifying: gates exit non-zero on regression.

use deep_causality_cfd::{
    BodyFittedCoordinate, dequantize_2d, gradient_y, laplacian_2d, quantize_2d,
};
use deep_causality_tensor::{CausalTensor, TensorTrain, TensorTrainOperator, Truncation};

const TAU: f64 = std::f64::consts::TAU;

fn main() {
    let mut failures: Vec<String> = Vec::new();
    let tol = 1e-8;

    println!(
        "=== QTT re-pinning marcher (Res 5 / D9, Stage-4 core): what actually bounds the marched rank? ===\n"
    );

    // Part 1: marching Cartesian fluxes through the curved front, static vs re-pinned.
    let (stat6, _) = march_polar(6, tol, false);
    let (stat7, _) = march_polar(7, tol, false);
    let (rep6, n6) = march_polar(6, tol, true);
    let (rep7, n7) = march_polar(7, tol, true);
    // Part 2: coordinate-aligned (radial) transport on the same re-pinned tracked interface.
    let fit6 = march_radial_fitted(6, tol);
    let fit7 = march_radial_fitted(7, tol);

    let grow_static = stat7 as i64 - stat6 as i64;
    let grow_repin = rep7 as i64 - rep6 as i64;

    println!("  Part 1 — Cartesian fluxes marched THROUGH the curved front:");
    println!(
        "    STATIC body-fitted    :  {stat6:>4}    {stat7:>4}    (+{grow_static} with resolution)"
    );
    println!(
        "    RE-PINNED (D9)        :  {rep6:>4}    {rep7:>4}    (+{grow_repin}; {n6} re-pins at L6, {n7} at L7)"
    );
    println!("  Part 2 — coordinate-ALIGNED radial transport on a re-pinned tracked interface:");
    println!(
        "    FITTED + tracked      :  {fit6:>4}    {fit7:>4}    (the front as an aligned interface)\n"
    );

    // Gate RP-A: marching Cartesian fluxes through the curved front grows with resolution (the problem).
    if stat7 <= stat6 {
        failures.push(format!(
            "RP-A: Cartesian-flux polar march did not grow with resolution (L6={stat6}, L7={stat7})"
        ));
    }
    // Gate RP-B: re-pinning actually engaged (the feedback mechanism ran).
    if n6 == 0 || n7 == 0 {
        failures.push(format!("RP-B: re-pin never engaged (n6={n6}, n7={n7})"));
    }
    // Gate RP-C (the negative finding): re-pinning the coordinate ALONE does not curb the growth — the
    // rank driver is the angular structure a Cartesian-flux march injects through the front, not the
    // front's drift. So re-pinning is necessary but not sufficient.
    if grow_repin < grow_static {
        failures.push(format!(
            "RP-C: re-pin-alone unexpectedly curbed the Cartesian-flux growth (static +{grow_static}, repin +{grow_repin}); the expected finding is that it does not"
        ));
    }
    // Gate RP-D (the positive finding): aligning the transport with the fitted coordinate (radial, with
    // the front as a tracked interface) holds the bond at O(1) and resolution-flat — far below Part 1.
    if fit6 > 8 || fit7 > 8 || fit7 > fit6 + 1 || fit7 >= rep7 {
        failures.push(format!(
            "RP-D: coordinate-aligned tracked transport not bounded/flat (L6={fit6}, L7={fit7}, vs repin {rep7})"
        ));
    }

    println!("--- reading ---");
    println!(
        "  Part 1: a STATIC body-fitted chart grows +{grow_static} from 64^2 to 128^2, and re-pinning the"
    );
    println!(
        "  coordinate to the live front (+{grow_repin}, {n7} re-pins at L7) does NOT curb it. So the rank driver"
    );
    println!(
        "  is not the front's drift; it is the angular structure a Cartesian-flux march injects by carrying"
    );
    println!("  fluxes THROUGH a curved front. Re-pinning is necessary but not sufficient.");
    println!(
        "  Part 2: align the transport with the coordinate (radial flux, the front as a tracked interface)"
    );
    println!(
        "  and the same re-pinned marcher holds bond {fit6}/{fit7}, flat in resolution. THAT is the Res-5"
    );
    println!(
        "  lever working dynamically. The Stage-4 mechanism is therefore re-pin AND treat the front as an"
    );
    println!(
        "  exact Rankine-Hugoniot interface (smooth each side), not march Cartesian fluxes across it."
    );

    if failures.is_empty() {
        println!(
            "\nALL GATES PASSED — re-pin is necessary; aligning the flux to the coordinate is what bounds the rank."
        );
    } else {
        eprintln!("\nFAILED GATES:");
        for f in &failures {
            eprintln!("  - {f}");
        }
        std::process::exit(1);
    }
}

/// Coordinate-aligned (radial) transport of a tracked interface in the re-pinned polar frame: the front
/// is moved along `η` only (no Cartesian flux carried through it), with the coordinate re-pinned to hold
/// it at `η = ½`. The field stays a function of `η` (an aligned interface), so its bond stays `O(1)`.
/// Returns the peak `max_bond`.
fn march_radial_fitted(l: usize, tol: f64) -> usize {
    let trunc = Truncation::<f64>::by_tol(tol).expect("tol");
    let n = 1usize << l;
    let target = n / 2;
    let delta = 3.0 / n as f64; // ~3-cell interface

    // A radial interface (post-shock high → freestream low) at η = ½: a function of η. The metric is not
    // needed for the radial update, so the coordinate is used only to lay down the initial condition.
    let mut u = BodyFittedCoordinate::<f64>::new(l, l, 1.0, 1.0, 0.0, TAU, trunc)
        .expect("coord")
        .sample(|_xi, eta| 0.5 * (1.0 - ((eta - 0.5) / delta).tanh()))
        .expect("ic");

    let h = 1.0 / n as f64;
    let gy = gradient_y::<f64>(l, l, h, &trunc).expect("gy"); // ∂/∂η (computational)
    let lap = laplacian_2d::<f64>(l, l, h, h, &trunc).expect("lap");
    let dt = 0.2 * h;
    let nu = 1.0 * h;
    let speed = 1.0; // outward radial advection so the interface drifts and the re-pin engages
    let steps = 200usize;

    let mut peak = u.max_bond();
    for _ in 1..=steps {
        // Radial advection u_t = -speed·∂u/∂η + ν·∇²u (no flux carried transverse to the coordinate).
        let adv = gy.apply(&u, &trunc).expect("adv").scale(-speed);
        let visc = lap.apply(&u, &trunc).expect("visc").scale(nu);
        let rate = adv.add(&visc).expect("rate");
        u = u
            .add(&rate.scale(dt))
            .expect("euler")
            .round(&trunc)
            .expect("round");
        if let Some(jstar) = front_eta(&u, l) {
            let shift = target as isize - jstar as isize;
            if shift != 0 {
                u = roll_eta(&u, l, shift, &trunc);
            }
        }
        peak = peak.max(u.max_bond());
    }
    peak
}

/// March a radial Burgers front in the body-fitted polar frame. With `repin`, track the front each step
/// and slide the coordinate to keep it at the fixed band `η = ½`. Returns `(peak_bond, n_repins)`.
fn march_polar(l: usize, tol: f64, repin: bool) -> (usize, usize) {
    let trunc = Truncation::<f64>::by_tol(tol).expect("tol");
    let n = 1usize << l;
    let dr = 1.0f64;
    let mut r0 = 1.0f64;
    let rc = 1.5f64;
    let w = 0.12f64;
    let target = n / 2;

    let mut coord = BodyFittedCoordinate::<f64>::new(l, l, r0, dr, 0.0, TAU, trunc).expect("coord");
    let mut u = coord
        .sample(|_xi, eta| 0.6 * (-(((r0 + eta * dr) - rc) / w).powi(2)).exp())
        .expect("ic");

    // Computational-space stabilizer (spacing fixed; independent of r0).
    let h = 1.0 / n as f64;
    let lap = laplacian_2d::<f64>(l, l, h, h, &trunc).expect("lap");
    let cell = (dr / n as f64).min(r0 * TAU / n as f64);
    let dt = 0.2 * cell;
    let nu = 1.0 * h;
    let steps = (0.30 / dt) as usize;

    let mut peak = u.max_bond();
    let mut n_repin = 0usize;
    for _ in 1..=steps {
        let u2 = u.hadamard_rounded(&u, &trunc).expect("u^2");
        let (fx, fy) = coord.physical_gradient(&u2).expect("phys grad");
        let conv = fx.add(&fy).expect("flux").scale(-0.5);
        let visc = lap.apply(&u, &trunc).expect("visc").scale(nu);
        let rate = conv.add(&visc).expect("rate");
        u = u
            .add(&rate.scale(dt))
            .expect("euler")
            .round(&trunc)
            .expect("round");

        let front = if repin { front_eta(&u, l) } else { None };
        if let Some(jstar) = front {
            let shift = target as isize - jstar as isize;
            if shift != 0 {
                u = roll_eta(&u, l, shift, &trunc);
                r0 += (jstar as f64 - target as f64) / n as f64 * dr;
                coord = BodyFittedCoordinate::<f64>::new(l, l, r0, dr, 0.0, TAU, trunc)
                    .expect("recoord");
                n_repin += 1;
            }
        }
        peak = peak.max(u.max_bond());
    }
    (peak, n_repin)
}

/// Locate the front: the interior `η` index of steepest `ξ`-averaged radial gradient.
fn front_eta(u: &deep_causality_tensor::CausalTensorTrain<f64>, l: usize) -> Option<usize> {
    let n = 1usize << l;
    let dense = dequantize_2d(u, l, l).expect("dequantize");
    let s = dense.as_slice();
    let mut prof = vec![0.0f64; n];
    for ix in 0..n {
        for (j, p) in prof.iter_mut().enumerate() {
            *p += s[ix * n + j];
        }
    }
    let mut jstar = None;
    let mut best = -1.0;
    for j in 2..n - 2 {
        let g = (prof[j + 1] - prof[j - 1]).abs();
        if g > best {
            best = g;
            jstar = Some(j);
        }
    }
    jstar
}

/// Cyclically roll the field by `shift` cells along `η` (a rank-preserving relabel) and re-encode.
fn roll_eta(
    u: &deep_causality_tensor::CausalTensorTrain<f64>,
    l: usize,
    shift: isize,
    trunc: &Truncation<f64>,
) -> deep_causality_tensor::CausalTensorTrain<f64> {
    let n = 1usize << l;
    let dense = dequantize_2d(u, l, l).expect("dequantize");
    let s = dense.as_slice();
    let mut rolled = vec![0.0f64; n * n];
    for ix in 0..n {
        for j in 0..n {
            let src = ((j as isize - shift).rem_euclid(n as isize)) as usize;
            rolled[ix * n + j] = s[ix * n + src];
        }
    }
    quantize_2d(
        &CausalTensor::new(rolled, vec![n, n]).expect("dense"),
        trunc,
    )
    .expect("re-encode")
}
