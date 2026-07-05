/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Display + analysis layer for the cavity run: the centerline profiles, the Ghia-table RMSE, the
//! streamfunction vortex centers, and the centerline CSV writes (through the IO effect).
//!
//! All arithmetic runs at the working precision [`FloatType`]: the marched field stays at
//! `FloatType`, and the Ghia `f64` reference specifications lift into `FloatType` once through [`ft`]
//! before comparison. Values downcast to `f64` only at the display boundary (the `{:.*}` formats and
//! the CSV strings). Lattice indices and physical region coordinates stay in `usize`/`f64`.

use crate::FloatType;
use crate::config::{GHIA_U, GHIA_V, GHIA_VORTICES, ft};
use crate::fail;
use deep_causality_cfd::IoAction;
// The centerline dump is a formatted string report (custom precision, empty reference cells for
// the profile rows), not a typed numeric table, so it uses core's low-level string writer
// directly rather than the DSL's typed `write_rows`.
use deep_causality_core::write_csv;
use deep_causality_num::{One, Zero};

/// The centerline velocity profiles `(u(0.5, y_j), v(x_i, 0.5))` at the vertex stations, from the
/// edge cochain, in native [`FloatType`].
pub fn centerline_profiles(
    u: &[FloatType],
    n: usize,
    h: FloatType,
) -> (Vec<FloatType>, Vec<FloatType>) {
    let nx_edges = (n - 1) * n;
    let center = (n - 1) / 2;
    let x_edge = |i: usize, j: usize| u[j * (n - 1) + i] / h;
    let y_edge = |i: usize, j: usize| u[nx_edges + j * n + i] / h;
    let u_profile: Vec<FloatType> = (0..n)
        .map(|j| ft(0.5) * (x_edge(center - 1, j) + x_edge(center, j)))
        .collect();
    let v_profile: Vec<FloatType> = (0..n)
        .map(|i| ft(0.5) * (y_edge(i, center - 1) + y_edge(i, center)))
        .collect();
    (u_profile, v_profile)
}

/// Linear interpolation of a vertex-station profile to coordinate `s`, in native [`FloatType`]. The
/// fractional position is split into an integer index (a `usize` cast) and a `FloatType` weight.
pub fn interp_profile(profile: &[FloatType], h: FloatType, s: FloatType) -> FloatType {
    let n = profile.len();
    let pos = s / h;
    let i0 = (Into::<f64>::into(pos).floor() as usize).min(n - 2);
    let w = pos - ft(i0 as f64);
    profile[i0] * (FloatType::one() - w) + profile[i0 + 1] * w
}

/// Pooled centerline RMSE against the Ghia tables, in native [`FloatType`] (the `f64` reference values
/// lift through [`ft`]).
pub fn centerline_rmse(
    u_profile: &[FloatType],
    v_profile: &[FloatType],
    h: FloatType,
) -> FloatType {
    let mut sq = FloatType::zero();
    for &(y, u_ref) in &GHIA_U {
        let d = interp_profile(u_profile, h, ft(y)) - ft(u_ref);
        sq += d * d;
    }
    for &(x, v_ref) in &GHIA_V {
        let d = interp_profile(v_profile, h, ft(x)) - ft(v_ref);
        sq += d * d;
    }
    (sq / ft((GHIA_U.len() + GHIA_V.len()) as f64)).sqrt()
}

/// Render the full cavity analysis: write the centerline CSVs, then print the RMSE and the detected
/// vortex centers (primary + bottom corner eddies) against the Ghia references. All arithmetic is
/// native [`FloatType`]; the prints downcast to `f64`.
pub fn render(u_form: &[FloatType], n: usize, h: FloatType) {
    let (u_profile, v_profile) = centerline_profiles(u_form, n, h);
    let interp =
        |profile: &[FloatType], s: FloatType| -> FloatType { interp_profile(profile, h, s) };

    write_centerline_csv(
        "cavity_centerline_u.csv",
        "y,u_computed,u_ghia,diff",
        &u_profile,
        h,
        &GHIA_U,
        &interp,
    );
    write_centerline_csv(
        "cavity_centerline_v.csv",
        "x,v_computed,v_ghia,diff",
        &v_profile,
        h,
        &GHIA_V,
        &interp,
    );

    let rmse = centerline_rmse(&u_profile, &v_profile, h);
    println!("# centerline RMSE vs Ghia: {:.4}", Into::<f64>::into(rmse));

    // --- Vortex centers (streamfunction extrema) ------------------------
    // ψ = 0 on the walls (they are streamlines); integrate up each column: Δψ across the vertical
    // edge (i, j)→(i, j+1) is ∫u_x dy ≈ the average of the four flanking x-edge velocities times h.
    let x_edge = |i: usize, j: usize| u_form[j * (n - 1) + i] / h;
    let mut psi = vec![FloatType::zero(); n * n];
    for i in 0..n {
        for j in 0..(n - 1) {
            let il = i.saturating_sub(1).min(n - 2);
            let ir = i.min(n - 2);
            let u_mid =
                ft(0.25) * (x_edge(il, j) + x_edge(ir, j) + x_edge(il, j + 1) + x_edge(ir, j + 1));
            psi[(j + 1) * n + i] = psi[j * n + i] + u_mid * h;
        }
    }

    println!("# vortex centers (streamfunction extrema) vs Ghia (1982):");
    println!("vortex,x,y,psi,ghia_x,ghia_y");
    // Primary: the global |ψ| extremum; corner eddies: the opposite-signed extrema in the bottom
    // corner quadrants.
    let (pi_, pj, p_psi) = extremum(&psi, n, |_, _| true, None);
    println!(
        "primary,{:.4},{:.4},{:+.4e},{:.4},{:.4}",
        Into::<f64>::into(ft(pi_ as f64) * h),
        Into::<f64>::into(ft(pj as f64) * h),
        Into::<f64>::into(p_psi),
        GHIA_VORTICES[0].1,
        GHIA_VORTICES[0].2
    );
    let corner_sign = -p_psi.signum();
    for (slot, region) in [
        (1usize, [0.0, 0.3, 0.0, 0.3]),
        (2usize, [0.7, 1.0, 0.0, 0.3]),
    ] {
        let (name, gx, gy) = GHIA_VORTICES[slot];
        let (ci, cj, c_psi) = extremum(
            &psi,
            n,
            |x, y| x >= region[0] && x <= region[1] && y >= region[2] && y <= region[3],
            Some(corner_sign),
        );
        if c_psi == FloatType::zero() {
            // The corner eddy did not separate at this resolution/horizon; the reporting run (129²,
            // t_end ≥ 150) resolves it.
            println!("{name},unresolved,unresolved,,{gx:.4},{gy:.4}");
        } else {
            println!(
                "{name},{:.4},{:.4},{:+.4e},{gx:.4},{gy:.4}",
                Into::<f64>::into(ft(ci as f64) * h),
                Into::<f64>::into(ft(cj as f64) * h),
                Into::<f64>::into(c_psi)
            );
        }
    }
}

/// The interior vertex maximizing `|ψ|` within `region` (physical coordinates in [0, 1], in `f64`);
/// `sign` restricts to one rotation sense. The streamfunction values are native [`FloatType`].
fn extremum(
    psi: &[FloatType],
    n: usize,
    region: impl Fn(f64, f64) -> bool,
    sign: Option<FloatType>,
) -> (usize, usize, FloatType) {
    let h = 1.0 / (n - 1) as f64; // physical coordinates for region selection (f64)
    let mut best = (0usize, 0usize, FloatType::zero());
    for j in 1..(n - 1) {
        for i in 1..(n - 1) {
            let (x, y) = (i as f64 * h, j as f64 * h);
            if !region(x, y) {
                continue;
            }
            let v = psi[j * n + i];
            if let Some(s) = sign
                && v.signum() != s
            {
                continue;
            }
            if v.abs() > best.2.abs() {
                best = (i, j, v);
            }
        }
    }
    best
}

/// Write a centerline CSV through the IO effect. Every field is rendered with the same specifiers as
/// the pre-DSL writer (working-precision values downcast to `f64` for the `{:.*}` formats), so the
/// bytes are byte-for-byte identical at `f64`; `write_csv` builds a deferred action and `run` executes
/// it once, at the edge (an IO failure surfaces as a `CausalityError`).
#[allow(clippy::too_many_arguments)]
fn write_centerline_csv(
    path: &str,
    header: &str,
    profile: &[FloatType],
    h: FloatType,
    ghia: &[(f64, f64); 17],
    interp: &impl Fn(&[FloatType], FloatType) -> FloatType,
) {
    let header_fields: Vec<String> = header.split(',').map(|s| s.to_string()).collect();
    let mut rows: Vec<Vec<String>> = Vec::with_capacity(ghia.len() + profile.len());
    // The Ghia stations with reference values and differences.
    for &(s, reference) in ghia {
        let computed = interp(profile, ft(s));
        let diff = computed - ft(reference);
        rows.push(vec![
            format!("{s:.4}"),
            format!("{:.6}", Into::<f64>::into(computed)),
            format!("{reference:.5}"),
            format!("{:+.6}", Into::<f64>::into(diff)),
        ]);
    }
    // The full computed profile (reference column empty).
    for (j, value) in profile.iter().enumerate() {
        rows.push(vec![
            format!("{:.4}", Into::<f64>::into(ft(j as f64) * h)),
            format!("{:.6}", Into::<f64>::into(*value)),
            String::new(),
            String::new(),
        ]);
    }
    write_csv(path, header_fields, rows)
        .run()
        .unwrap_or_else(|e| fail("centerline csv", e));
    println!("# wrote {path}");
}
