/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::model_types::{FleetProcess, ProcessedReadings};

pub fn print_summary(process: &FleetProcess<ProcessedReadings>) {
    match &process.error {
        Some(err) => println!("❌ Pipeline error: {err:?}"),
        None => println!("✅ Pipeline complete."),
    }

    println!("\n--- Final FleetState ---");
    println!("  healthy_count:     {}", process.state.healthy_count);
    println!("  degraded_count:    {}", process.state.degraded_count);
    println!("  failed_count:      {}", process.state.failed_count);
    println!(
        "  total_uncertainty: {:.2}",
        process.state.total_uncertainty
    );
    println!(
        "  fused_temp:        {}",
        process
            .state
            .fused_temp
            .map(|t| format!("{t:.1}°C"))
            .unwrap_or_else(|| "n/a".into())
    );
    println!(
        "  verdict:           {}",
        process
            .state
            .verdict
            .as_ref()
            .map(|v| format!("{v:?}"))
            .unwrap_or_else(|| "n/a".into())
    );
    if process.state.anomalies.is_empty() {
        println!("  anomalies:         none");
    } else {
        println!("  anomalies:");
        for a in &process.state.anomalies {
            println!("    - {a}");
        }
    }

    println!("\n--- EffectLog ---");
    let log_text = format!("{:?}", process.logs);
    for line in log_text.split(',').map(|s| s.trim()) {
        if !line.is_empty() {
            println!("  {line}");
        }
    }
}
