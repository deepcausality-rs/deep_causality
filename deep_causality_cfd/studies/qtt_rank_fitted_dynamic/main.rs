// SPDX-License-Identifier: MIT
// Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

//! QTT rank study — the **dynamic** lever: does a *marched* shock stay low tensor-train rank when the
//! coordinate is **aligned** to it? (Resolution 5, the Tier-B make-or-break.)
//!
//! `qtt_rank_nonlinear` measured that a *captured* 2-D curved shock raises rank as it forms (7 → 20), and
//! `qtt_rank_3d` measured the Cartesian upper bound `χ ~ √side` (unbounded in resolution). Both left the
//! decisive cell OPEN: does a marcher that keeps the feature **aligned to a coordinate axis** hold the
//! bond bounded *and resolution-independent* over the march — the thing Resolution 5 claims "by
//! construction"?
//!
//! Three marched cases (all viscous Burgers, the canonical shock-former), each at two resolutions so the
//! **resolution scaling** is visible — that is the real test, not a single number:
//!
//!   1. **Cartesian, CURVED** (a radial bump → curved shock): the misaligned control. Expect the peak
//!      bond to **grow with resolution** (the √side threat, dynamically).
//!   2. **Axis-ALIGNED, planar** (a 1-D front broadcast across `y`): the feature sits on a grid axis.
//!      Expect the peak bond to stay **low and flat** in resolution — alignment defeats the growth
//!      *under marching*, not just statically.
//!   3. **STATIC body-fitted polar march** (`BodyFittedCoordinate`, set once): a radial front marched in
//!      the curvilinear frame with the chain-rule physical gradient. The measured result is that it
//!      **grows with resolution**, no better than the capture. Cartesian fluxes evolve the front off the
//!      fixed coordinate, so the rank climbs. This is the empirical case for Resolution 5's **feedback
//!      re-pinning (D9)**: a one-time fitted coordinate is not enough; alignment must be maintained.
//!
//! Together the three say: alignment bounds the bond under marching (case 2), and holding that alignment
//! takes active re-pinning, not a static chart (case 3 vs case 2). Self-verifying: the gates encode the
//! finding and exit non-zero on regression. As a study, the headline is the measured magnitudes below.

use deep_causality_cfd::{BodyFittedCoordinate, gradient_x, gradient_y, laplacian_2d, quantize_2d};
use deep_causality_tensor::{CausalTensor, TensorTrain, TensorTrainOperator, Truncation};

const TAU: f64 = std::f64::consts::TAU;

fn main() {
    let mut failures: Vec<String> = Vec::new();
    let tol = 1e-8;

    println!(
        "=== QTT dynamic rank lever (Res 5): does a MARCHED shock stay low-rank when aligned? ===\n"
    );

    // Misaligned control (curved) and aligned (planar), each at two resolutions.
    let cart6 = burgers_cart(6, tol, Ic::Curved);
    let cart7 = burgers_cart(7, tol, Ic::Curved);
    let align6 = burgers_cart(6, tol, Ic::Planar);
    let align7 = burgers_cart(7, tol, Ic::Planar);
    // The real body-fitted polar march.
    let fit6 = burgers_fitted(6, tol);
    let fit7 = burgers_fitted(7, tol);

    println!("  case                              L=6     L=7   (peak max_bond over the march)");
    println!(
        "  Cartesian CURVED  (misaligned) :  {cart6:>4}    {cart7:>4}   <- the threat: grows with resolution"
    );
    println!(
        "  Axis-ALIGNED planar            :  {align6:>4}    {align7:>4}   <- alignment: low & flat (the lever)"
    );
    println!(
        "  STATIC body-fitted polar march :  {fit6:>4}    {fit7:>4}   <- set once: grows -> needs D9 re-pin"
    );

    // Gate FD-A: the misaligned curved shock's peak bond GROWS with resolution (the √side threat is
    // real and dynamic, reproducing qtt_rank_nonlinear / qtt_rank_3d on a marcher).
    if cart7 <= cart6 {
        failures.push(format!(
            "FD-A: curved-shock peak bond did not grow with resolution (L6={cart6}, L7={cart7})"
        ));
    }

    // Gate FD-B: an axis-ALIGNED marched shock stays low AND resolution-flat — alignment holds the rank
    // bounded under marching (the dynamic version of the static lever).
    if align6 > 8 || align7 > 8 || align7 > align6 + 1 {
        failures.push(format!(
            "FD-B: aligned marched shock not low/flat in resolution (L6={align6}, L7={align7})"
        ));
    }

    // Gate FD-C: a STATIC body-fitted coordinate is NOT self-bounding. With Cartesian fluxes the
    // marched radial front develops angular structure, so its bond grows with resolution like the
    // misaligned capture. This is the measured case for Res-5 feedback re-pinning (D9): a one-time
    // fitted coordinate is insufficient; alignment must be actively maintained.
    if fit7 <= fit6 {
        failures.push(format!(
            "FD-C: static body-fitted march did not grow with resolution (L6={fit6}, L7={fit7}); \
             expected growth, which is what motivates feedback re-pinning"
        ));
    }

    println!("\n--- reading ---");
    println!("  Three measurements, one conclusion about what bounds the rank under marching.");
    println!(
        "  The misaligned curved shock grows {cart6} -> {cart7} with resolution. The captured threat is"
    );
    println!("  real and dynamic, closing the cell qtt_rank_nonlinear left open.");
    println!(
        "  The axis-aligned front holds {align6} / {align7}, flat in resolution. When the feature stays on a"
    );
    println!(
        "  grid axis throughout, the marcher bounds the bond by construction. That is the Res-5 lever."
    );
    println!(
        "  The STATIC body-fitted coordinate grows {fit6} -> {fit7}, no better than the capture. A one-time"
    );
    println!(
        "  fitted coordinate does not stay aligned once Cartesian fluxes evolve the front, so the rank"
    );
    println!(
        "  climbs. This is the empirical case for D9: the coordinate must be re-pinned to the live front"
    );
    println!(
        "  each step (feedback), not set once. Alignment is the lever; maintaining it is the mechanism."
    );

    if failures.is_empty() {
        println!(
            "\nALL GATES PASSED — alignment bounds the bond dynamically; a static fit needs D9 re-pinning."
        );
    } else {
        eprintln!("\nFAILED GATES:");
        for f in &failures {
            eprintln!("  - {f}");
        }
        std::process::exit(1);
    }
}

/// Initial-condition family for the Cartesian marcher.
#[derive(Clone, Copy)]
enum Ic {
    /// A smooth radial bump → a *curved* (misaligned) shock.
    Curved,
    /// A 1-D wave broadcast across `y` → a *planar* (axis-aligned) shock.
    Planar,
}

/// March 2-D viscous Burgers `u_t + ½∂_x(u²) + ½∂_y(u²) = ν∇²u` on a Cartesian `2^l × 2^l` grid and
/// return the peak `max_bond` over the march.
fn burgers_cart(l: usize, tol: f64, ic: Ic) -> usize {
    let trunc = Truncation::<f64>::by_tol(tol).expect("tol");
    let side = 1usize << l;
    let dx = 1.0 / side as f64;
    let dt = 0.2 * dx;
    let nu = 1.0 * dx; // diffusion number ν·dt/dx² = 0.2 ≤ 0.25 (2-D explicit limit)
    let steps = (0.35 / dt) as usize;

    let gx = gradient_x::<f64>(l, l, dx, &trunc).expect("gx");
    let gy = gradient_y::<f64>(l, l, dx, &trunc).expect("gy");
    let lap = laplacian_2d::<f64>(l, l, dx, dx, &trunc).expect("lap2d");

    let mut data = vec![0.0f64; side * side];
    for ix in 0..side {
        for iy in 0..side {
            let x = ix as f64 * dx;
            let y = iy as f64 * dx;
            data[ix * side + iy] = match ic {
                Ic::Curved => {
                    let r2 = (x - 0.35).powi(2) + (y - 0.5).powi(2);
                    (-r2 / (2.0 * 0.08_f64.powi(2))).exp()
                }
                Ic::Planar => 0.5 + 0.4 * (TAU * x).sin(),
            };
        }
    }
    let mut u = quantize_2d(
        &CausalTensor::new(data, vec![side, side]).expect("dense"),
        &trunc,
    )
    .expect("encode");

    let mut peak = u.max_bond();
    for _ in 1..=steps {
        let u2 = u.hadamard_rounded(&u, &trunc).expect("u^2");
        let fx = gx.apply(&u2, &trunc).expect("fx");
        let fy = gy.apply(&u2, &trunc).expect("fy");
        let conv = fx.add(&fy).expect("flux").scale(-0.5);
        let visc = lap.apply(&u, &trunc).expect("visc").scale(nu);
        let rate = conv.add(&visc).expect("rate");
        u = u
            .add(&rate.scale(dt))
            .expect("euler")
            .round(&trunc)
            .expect("round");
        peak = peak.max(u.max_bond());
    }
    peak
}

/// March 2-D viscous Burgers in the **body-fitted polar coordinate**: a radial front advected by the
/// Cartesian-flux divergence assembled via the chain-rule physical gradient, with a computational-space
/// Laplacian as the stabilizer (a study cares about the bond evolution, not solution fidelity). Returns
/// the peak `max_bond` over the march.
fn burgers_fitted(l: usize, tol: f64) -> usize {
    let trunc = Truncation::<f64>::by_tol(tol).expect("tol");
    let n = 1usize << l;
    let (r0, dr) = (1.0f64, 1.0f64); // annulus r ∈ [1, 2]
    let coord = BodyFittedCoordinate::<f64>::new(l, l, r0, dr, 0.0, TAU, trunc).expect("coord");

    // Smooth radial bump centred mid-annulus: a function of η only ⇒ low-rank initial data.
    let rc = 1.5;
    let w = 0.12;
    let mut u = coord
        .sample(|_xi, eta| {
            let r = r0 + eta * dr;
            0.6 * (-((r - rc) / w).powi(2)).exp()
        })
        .expect("sample");

    // Stabilizer: computational-space Laplacian (spacing 1/N in each of ξ, η).
    let h = 1.0 / n as f64;
    let lap = laplacian_2d::<f64>(l, l, h, h, &trunc).expect("lap2d");

    // Smallest physical cell ≈ min(dr/N, r0·Δθ/N); advection-limited dt with margin.
    let cell = (dr / n as f64).min(r0 * TAU / n as f64);
    let dt = 0.2 * cell;
    let nu = 1.0 * h; // gentle computational-space stabilization
    let steps = (0.30 / dt) as usize;

    let mut peak = u.max_bond();
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
        peak = peak.max(u.max_bond());
    }
    peak
}
