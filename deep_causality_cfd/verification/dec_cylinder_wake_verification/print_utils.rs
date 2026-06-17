/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Display + analysis layer for the cylinder-wake run: the kinetic-energy proxy, the shedding
//! Strouhal estimate, and the wake-probe CSV write (through the IO effect).
//!
//! All arithmetic runs at the working precision [`FloatType`]; values are cast to `f64` only at the
//! `eprintln!` display boundary (the `{:.*e}` formats need `f64`). The CSV is written at native
//! precision through the generic [`write_xy_csv`].

use crate::FloatType;
use crate::config::{
    BLOCKAGE, CaseGeometry, DROPOUT_EVERY, MERGE_FRACTION, NY, RE_D, SENSOR_SIGMA, U_BULK, ft,
};
use deep_causality_cfd::{IoAction, fail, write_xy_csv};
use deep_causality_num::{One, Zero};

/// Print the run header: the case / cell / sensor lines on stderr, then the CSV column header on
/// stdout. Working-precision quantities downcast to `f64` for the `{:.*e}`/`{:.*}` formats.
pub fn print_header(geom: &CaseGeometry, n_dropouts: usize) {
    eprintln!(
        "# cut-cell cylinder wake: grid {}×{NY}, D/H={BLOCKAGE}, Re_D={RE_D}, nu={:.3e}, dt={:.3e}",
        geom.nx,
        Into::<f64>::into(geom.nu),
        Into::<f64>::into(geom.dt),
    );
    eprintln!(
        "# cells: {} solid, {} cut; fluid area {:.4}; merge floor {MERGE_FRACTION}",
        geom.n_solid,
        geom.n_cut,
        Into::<f64>::into(geom.fluid_area),
    );
    eprintln!(
        "# sensor-fed top wall: U≈{U_BULK} (1σ {SENSOR_SIGMA}), dropout every {DROPOUT_EVERY} steps ({n_dropouts} total)"
    );
    println!("step,t,kinetic_energy,max_speed,div_residual,v_probe");
}

/// Kinetic energy proxy `½ Σ (u_e / h)²` over the edge cochain (uniform-metric form), in native
/// [`FloatType`].
pub fn kinetic_energy(u: &[FloatType], h: FloatType) -> FloatType {
    ft(0.5)
        * u.iter().fold(FloatType::zero(), |acc, &e| {
            let x = e / h;
            acc + x * x
        })
}

/// Estimate the shedding Strouhal `St = f·D / U` from the wake probe's mean-crossing rate over the
/// developed (second-half) signal. The crossing detection, period, and Strouhal are computed in
/// native [`FloatType`]; only the closing report downcasts to `f64`.
pub fn report_strouhal(series: &[(FloatType, FloatType)], diameter: FloatType, u_ref: FloatType) {
    if series.len() < 8 {
        eprintln!("# Strouhal: insufficient samples");
        return;
    }
    let half = series.len() / 2;
    let tail = &series[half..];
    let sum = tail.iter().fold(FloatType::zero(), |a, (_, v)| a + *v);
    let mean = sum / ft(tail.len() as f64);

    // Up-crossings of the mean ⇒ one per period.
    let mut crossings: Vec<FloatType> = Vec::new();
    for w in tail.windows(2) {
        let (t0, v0) = w[0];
        let (t1, v1) = w[1];
        if v0 - mean <= FloatType::zero() && v1 - mean > FloatType::zero() {
            let frac = (mean - v0) / (v1 - v0);
            crossings.push(t0 + frac * (t1 - t0));
        }
    }
    if crossings.len() < 2 {
        eprintln!("# Strouhal: no clear shedding detected in the developed signal");
        return;
    }
    let period = (*crossings.last().unwrap() - crossings[0]) / ft((crossings.len() - 1) as f64);
    let freq = FloatType::one() / period;
    let strouhal = freq * diameter / u_ref;
    eprintln!(
        "# shedding: period {:.4}, f {:.4}, St = f·D/U ≈ {:.4} \
         (confined/periodic cylinder — not the isolated-cylinder reference)",
        Into::<f64>::into(period),
        Into::<f64>::into(freq),
        Into::<f64>::into(strouhal),
    );
}

/// Persist the full wake-probe time series to CSV through the IO effect, at native precision:
/// `write_xy_csv` builds a deferred `IoAction`; `run` executes the write once, at the edge (an IO
/// failure surfaces as a `CausalityError`).
pub fn write_probe_csv(path: &str, series: &[(FloatType, FloatType)]) {
    write_xy_csv(path, ["t", "v_probe"], series)
        .run()
        .unwrap_or_else(|e| fail("wake-probe csv", e));
    eprintln!("# wrote {} wake-probe samples to {path}", series.len());
}
