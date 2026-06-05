/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::model::Report;

pub fn print_report(r: &Report) {
    println!("Forecasting a chaotic convective flow (Lorenz / Rayleigh-Bénard truncation).");
    println!("Rk4 dt=0.005, identical initial state (1,1,1); same scheme at every precision, so");
    println!("truncation cancels and the distance to the Float106 run is the roundoff growth alone.\n");

    println!("     t    |  f32 vs F106  |  f64 vs F106");
    println!("  --------+---------------+--------------");
    for row in &r.rows {
        println!("   {:>5.1}  |   {:>9}   |   {:>9}", row.t, row.d_f32, row.d_f64);
    }

    println!("\nForecast horizon (lead time before the flow state is off by one state-space unit):");
    print_horizon("  f32      ", r.h_f32);
    print_horizon("  f64      ", r.h_f64);
    println!(
        "  Float106   beyond T=60 here; the law below puts it near t ≈ {:.0}",
        Report::horizon_law(1e-32)
    );

    println!("\nThe horizon follows ln(1/ε)/λ with λ ≈ 0.906, so it grows linearly in correct digits:");
    println!(
        "    f32  (ε≈1.2e-7 ):  t ≈ {:>4.1}",
        Report::horizon_law(1.2e-7)
    );
    println!(
        "    f64  (ε≈2.2e-16):  t ≈ {:>4.1}",
        Report::horizon_law(2.2e-16)
    );
    println!(
        "    F106 (ε≈1.0e-32):  t ≈ {:>4.1}",
        Report::horizon_law(1.0e-32)
    );

    println!(
        "\nEach ~16 extra digits roughly doubles the trustworthy forecast time. Past its horizon a"
    );
    println!(
        "turbulence prediction is numerical fiction, however fine the step: for a chaotic flow the"
    );
    println!(
        "only way to forecast further ahead is more precision. Here Float106, reached by a type"
    );
    println!("parameter, is the lever that buys the extra lead time.");
}

fn print_horizon(label: &str, horizon: Option<f64>) {
    match horizon {
        Some(t) => println!("{label}  t ≈ {t:.1}"),
        None => println!("{label}  not reached within the simulated window"),
    }
}
