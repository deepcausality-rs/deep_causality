/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::{Uncertain, UncertainError};
use std::collections::HashMap;

/// Sensor data processing with comprehensive error handling
///
/// This example demonstrates how to handle common real-world scenarios:
/// - Sensor failures and missing data
/// - Out-of-range readings and calibration drift
/// - Network timeouts and communication errors
/// - Data validation and outlier detection
/// - Graceful degradation and fallback strategies
fn main() -> Result<(), UncertainError> {
    println!("🔧 Robust Sensor Data Processing with Error Handling");
    println!("===================================================\n");

    // Simulate a multi-sensor system with various failure modes
    let sensor_readings = simulate_sensor_data();

    println!("📊 Raw Sensor Data Status:");
    print_sensor_status(&sensor_readings);

    // Process sensor data with error handling and validation
    let processed_data = process_sensor_data_robust(&sensor_readings)?;

    println!("\n🛡️  Error Handling and Data Validation:");
    validate_and_process(&processed_data)?;

    println!("\n⚙️  Sensor Fusion with Uncertainty Propagation:");
    sensor_fusion_with_errors(&processed_data)?;

    println!("\n🚨 Anomaly Detection and Outlier Handling:");
    detect_and_handle_anomalies(&sensor_readings)?;

    println!("\n🔄 Fallback Strategies and Graceful Degradation:");
    demonstrate_fallback_strategies(&sensor_readings)?;

    println!("\n📈 Long-term Reliability Assessment:");
    assess_system_reliability(&sensor_readings)?;

    Ok(())
}

#[derive(Debug, Clone)]
struct SensorReading {
    id: String,
    value: Option<f64>,
    timestamp: u64,
    status: SensorStatus,
    uncertainty: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
enum SensorStatus {
    Healthy,
    Degraded,
    Failed,
    OutOfRange,
    CalibrationDrift,
    CommunicationError,
}

fn simulate_sensor_data() -> HashMap<String, SensorReading> {
    let mut sensors = HashMap::new();

    // Temperature sensors with various issues
    sensors.insert(
        "temp_1".to_string(),
        SensorReading {
            id: "temp_1".to_string(),
            value: Some(23.2),
            timestamp: 1000,
            status: SensorStatus::Healthy,
            uncertainty: Some(0.5),
        },
    );

    sensors.insert(
        "temp_2".to_string(),
        SensorReading {
            id: "temp_2".to_string(),
            value: Some(85.7), // Unrealistic reading - likely sensor failure
            timestamp: 1002,
            status: SensorStatus::OutOfRange,
            uncertainty: Some(5.0), // High uncertainty due to suspected failure
        },
    );

    sensors.insert(
        "temp_3".to_string(),
        SensorReading {
            id: "temp_3".to_string(),
            value: None, // Communication timeout
            timestamp: 995,
            status: SensorStatus::CommunicationError,
            uncertainty: None,
        },
    );

    // Pressure sensors
    sensors.insert(
        "pressure_1".to_string(),
        SensorReading {
            id: "pressure_1".to_string(),
            value: Some(1013.25),
            timestamp: 1001,
            status: SensorStatus::Healthy,
            uncertainty: Some(2.0),
        },
    );

    sensors.insert(
        "pressure_2".to_string(),
        SensorReading {
            id: "pressure_2".to_string(),
            value: Some(1015.8),
            timestamp: 1003,
            status: SensorStatus::CalibrationDrift, // Systematic bias detected
            uncertainty: Some(8.0), // Increased uncertainty due to calibration issues
        },
    );

    // Humidity sensors
    sensors.insert(
        "humidity_1".to_string(),
        SensorReading {
            id: "humidity_1".to_string(),
            value: Some(45.2),
            timestamp: 999,
            status: SensorStatus::Degraded, // Aging sensor with reduced accuracy
            uncertainty: Some(3.5),
        },
    );

    sensors.insert(
        "humidity_2".to_string(),
        SensorReading {
            id: "humidity_2".to_string(),
            value: None, // Complete sensor failure
            timestamp: 980,
            status: SensorStatus::Failed,
            uncertainty: None,
        },
    );

    sensors
}

fn print_sensor_status(sensors: &HashMap<String, SensorReading>) {
    for reading in sensors.values() {
        let value_str = match reading.value {
            Some(v) => format!("{:.1}", v),
            None => "N/A".to_string(),
        };

        let uncertainty_str = match reading.uncertainty {
            Some(u) => format!("±{:.1}", u),
            None => "unknown".to_string(),
        };

        let status_icon = match reading.status {
            SensorStatus::Healthy => "✅",
            SensorStatus::Degraded => "⚠️ ",
            SensorStatus::Failed => "❌",
            SensorStatus::OutOfRange => "⚡",
            SensorStatus::CalibrationDrift => "🔧",
            SensorStatus::CommunicationError => "📡",
        };

        println!(
            "   {} {}: {} {} ({:?}) [t={}]",
            status_icon, reading.id, value_str, uncertainty_str, reading.status, reading.timestamp
        );
    }
}

fn process_sensor_data_robust(
    sensors: &HashMap<String, SensorReading>,
) -> Result<HashMap<String, Result<Uncertain<f64>, String>>, UncertainError> {
    let mut processed = HashMap::new();

    for (id, reading) in sensors {
        let result = match (&reading.status, reading.value, reading.uncertainty) {
            // Healthy sensors: use as-is
            (SensorStatus::Healthy, Some(value), Some(uncertainty)) => {
                Ok(Uncertain::normal(value, uncertainty))
            }

            // Degraded sensors: increase uncertainty
            (SensorStatus::Degraded, Some(value), Some(uncertainty)) => {
                let degraded_uncertainty = uncertainty * 2.0;
                Ok(Uncertain::normal(value, degraded_uncertainty))
            }

            // Out-of-range sensors: try to salvage with high uncertainty
            (SensorStatus::OutOfRange, Some(value), _) => {
                if is_physically_plausible(id, value) {
                    // Use reading but with very high uncertainty
                    Ok(Uncertain::normal(value, 10.0))
                } else {
                    Err(format!(
                        "Sensor {} reading {} is physically implausible",
                        id, value
                    ))
                }
            }

            // Calibration drift: apply correction with increased uncertainty
            (SensorStatus::CalibrationDrift, Some(value), Some(uncertainty)) => {
                let corrected_value = apply_calibration_correction(id, value);
                let drift_uncertainty = uncertainty * 1.5 + 2.0;
                Ok(Uncertain::normal(corrected_value, drift_uncertainty))
            }

            // Failed or communication error sensors
            (SensorStatus::Failed | SensorStatus::CommunicationError, _, _) => {
                Err(format!("Sensor {} is unavailable", id))
            }

            _ => Err(format!("Sensor {} has invalid data configuration", id)),
        };

        processed.insert(id.clone(), result);
    }

    Ok(processed)
}

fn is_physically_plausible(sensor_id: &str, value: f64) -> bool {
    match sensor_id {
        id if id.starts_with("temp") => (-50.0..=100.0).contains(&value),
        id if id.starts_with("pressure") => (800.0..=1200.0).contains(&value),
        id if id.starts_with("humidity") => (0.0..=100.0).contains(&value),
        _ => true,
    }
}

fn apply_calibration_correction(sensor_id: &str, value: f64) -> f64 {
    match sensor_id {
        "pressure_2" => value - 2.3,
        id if id.starts_with("temp") => value * 0.98 + 0.5,
        _ => value,
    }
}

fn validate_and_process(
    processed_data: &HashMap<String, Result<Uncertain<f64>, String>>,
) -> Result<(), UncertainError> {
    let mut healthy_sensors = 0;
    let mut _failed_sensors = 0;
    let mut total_uncertainty = 0.0;

    for (id, result) in processed_data {
        match result {
            Ok(uncertain_value) => {
                healthy_sensors += 1;

                // Use expected_value and standard_deviation
                let mean = uncertain_value.expected_value(100)?;
                let std_dev = uncertain_value.standard_deviation(100)?;
                total_uncertainty += std_dev;

                println!("   ✅ {}: μ={:.1}, σ={:.1}", id, mean, std_dev);

                // Validate uncertainty bounds using implicit_conditional
                let high_uncertainty_evidence = Uncertain::<f64>::point(uncertain_value.standard_deviation(100)?).greater_than(5.0);
                if high_uncertainty_evidence.implicit_conditional()? {
                    println!("      ⚠️  High uncertainty detected - consider sensor maintenance");
                }
            }
            Err(error) => {
                _failed_sensors += 1;
                println!("   ❌ {}: {}", id, error);
            }
        }
    }

    let total_sensors = processed_data.len();
    let reliability = healthy_sensors as f64 / total_sensors as f64 * 100.0;
    let avg_uncertainty = if healthy_sensors > 0 {
        total_uncertainty / healthy_sensors as f64
    } else {
        0.0
    };

    println!(
        "\n   📊 System Health: {}/{} sensors operational ({:.1}%)",
        healthy_sensors, total_sensors, reliability
    );
    println!("   📊 Average uncertainty: {:.1}", avg_uncertainty);

    if reliability < 60.0 {
        println!("   🚨 CRITICAL: System reliability below acceptable threshold!");
    } else if reliability < 80.0 {
        println!("   ⚠️  WARNING: System reliability degraded");
    }
    Ok(())
}

fn sensor_fusion_with_errors(
    processed_data: &HashMap<String, Result<Uncertain<f64>, String>>,
) -> Result<(), UncertainError> {
    println!("   Temperature fusion with error handling:");

    // Collect all healthy temperature sensors
    let temp_sensors: Vec<_> = processed_data
        .iter()
        .filter(|(id, _)| id.starts_with("temp"))
        .filter_map(|(id, result)| match result {
            Ok(uncertain) => Some((id, uncertain)),
            Err(_) => None,
        })
        .collect();

    if temp_sensors.is_empty() {
        println!("      ❌ No healthy temperature sensors available!");
        return Ok(());
    }

    if temp_sensors.len() == 1 {
        println!("      ⚠️  Only one temperature sensor available - no redundancy");
        let (_id, temp) = temp_sensors[0];
        let mean = temp.expected_value(1000)?;
        println!("      📊 Single sensor reading: {:.1}°C", mean);
        return Ok(());
    }

    // Multi-sensor fusion with uncertainty weighting
    println!("      ✅ Fusing {} temperature sensors", temp_sensors.len());

    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;

    for (id, temp_uncertain) in &temp_sensors {
        let mean = temp_uncertain.expected_value(1000)?;
        let std_dev = temp_uncertain.standard_deviation(1000)?;

        // Weight by inverse uncertainty (lower uncertainty = higher weight)
        let weight = 1.0 / (std_dev + 0.1);
        weighted_sum += mean * weight;
        total_weight += weight;

        println!("         {}: {:.1}°C ±{:.1} (weight: {:.2})", id, mean, std_dev, weight);
    }

    let fused_temperature = weighted_sum / total_weight;
    // Fused uncertainty is more complex to calculate directly from the graph
    // For simplicity, we'll approximate or use a fixed value for now.
    // A more accurate fused uncertainty would require a dedicated statistical method.
    let fused_uncertainty = 1.0 / total_weight.sqrt(); // Simplified approximation

    println!("      🎯 Fused temperature: {:.1}°C ±{:.1}", fused_temperature, fused_uncertainty);

    // Detect sensor disagreement
    let temp_values: Vec<f64> = temp_sensors
        .iter()
        .map(|(_, temp)| temp.expected_value(100).unwrap().clone())
        .collect();

    if temp_values.len() > 1 {
        let max_diff = temp_values
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
            - temp_values
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();

        if max_diff > 5.0 {
            println!(
                "      ⚠️  Large sensor disagreement ({:.1}°C) - possible sensor failure",
                max_diff
            );
        }
    }
    Ok(())
}

fn detect_and_handle_anomalies(
    sensors: &HashMap<String, SensorReading>,
) -> Result<(), UncertainError> {
    for (id, reading) in sensors {
        if let Some(value) = reading.value {
            let is_anomaly = detect_anomaly(id, value, &reading.status);

            if is_anomaly {
                println!("   🚨 Anomaly detected in {}: {}", id, value);

                // Suggest corrective actions
                match reading.status {
                    SensorStatus::OutOfRange => {
                        println!(
                            "      💡 Action: Check sensor calibration and physical installation"
                        );
                    }
                    SensorStatus::CalibrationDrift => {
                        println!("      💡 Action: Schedule sensor recalibration");
                    }
                    SensorStatus::Degraded => {
                        println!("      💡 Action: Replace sensor within maintenance window");
                    }
                    _ => {
                        println!(
                            "      💡 Action: Investigate sensor and environmental conditions"
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

fn detect_anomaly(sensor_id: &str, value: f64, status: &SensorStatus) -> bool {
    let out_of_normal_range = match sensor_id {
        id if id.starts_with("temp") => !Uncertain::<f64>::point(value).within_range(15.0, 35.0).implicit_conditional().unwrap(),
        id if id.starts_with("pressure") => !Uncertain::<f64>::point(value).within_range(980.0, 1050.0).implicit_conditional().unwrap(),
        id if id.starts_with("humidity") => !Uncertain::<f64>::point(value).within_range(20.0, 80.0).implicit_conditional().unwrap(),
        _ => false,
    };

    let sensor_status_indicates_problem = matches!(
        status,
        SensorStatus::Failed | SensorStatus::OutOfRange | SensorStatus::CalibrationDrift
    );

    out_of_normal_range || sensor_status_indicates_problem
}

fn demonstrate_fallback_strategies(
    sensors: &HashMap<String, SensorReading>,
) -> Result<(), UncertainError> {
    println!("   Implementing fallback strategies:");

    // Strategy 1: Use historical data when sensor fails
    let temp_sensors: Vec<_> = sensors
        .iter()
        .filter(|(id, _)| id.starts_with("temp"))
        .collect();

    let healthy_temp_count = temp_sensors
        .iter()
        .filter(|(_, reading)| reading.status == SensorStatus::Healthy)
        .count();

    if healthy_temp_count == 0 {
        println!("      🔄 No healthy temperature sensors - using historical model");
        let historical_temp = Uncertain::normal(22.0, 3.0);
        let mean = historical_temp.expected_value(100)?;
        println!("         Historical estimate: {:.1}°C ±3.0 (high uncertainty)", mean);
    }

    // Strategy 2: Cross-validation between sensor types
    println!("      🔄 Cross-validation between sensor types:");

    if let (Some(pressure_reading), Some(temp_reading)) = (
        sensors.get("pressure_1"),
        sensors.get("temp_1"),
    ) {
        if let (Some(pressure), Some(temp)) = (pressure_reading.value, temp_reading.value) {
            let expected_temp_from_pressure = estimate_temperature_from_pressure(pressure);
            let temp_diff = (temp - expected_temp_from_pressure).abs();

            if Uncertain::<f64>::point(temp_diff).greater_than(10.0).implicit_conditional()? {
                println!(
                    "         ⚠️  Temperature-pressure correlation check failed"
                );
                println!(
                    "         Expected temp: {:.1}°C, Measured: {:.1}°C",
                    expected_temp_from_pressure, temp
                );
            } else {
                println!("         ✅ Temperature-pressure correlation validated");
            }
        }
    }

    // Strategy 3: Graceful degradation
    println!("      🔄 Graceful degradation modes:");
    let operational_sensors = sensors
        .values()
        .filter(|r| matches!(r.status, SensorStatus::Healthy | SensorStatus::Degraded))
        .count();

    match operational_sensors {
        0 => println!("         🚨 EMERGENCY MODE: All sensors failed - system shutdown required"),
        1..=2 => println!("         ⚠️  REDUCED MODE: Limited sensors - increased uncertainty"),
        3..=4 => println!("         📊 NORMAL MODE: Adequate sensor coverage"),
        _ => println!("         ✅ FULL MODE: All sensors operational"),
    }
    Ok(())
}

fn estimate_temperature_from_pressure(pressure: f64) -> f64 {
    20.0 + (pressure - 1013.25) * 0.02
}

fn assess_system_reliability(
    sensors: &HashMap<String, SensorReading>,
) -> Result<(), UncertainError> {
    println!("   Long-term reliability metrics:");

    let total_sensors = sensors.len() as f64;
    let healthy_sensors = sensors
        .values()
        .filter(|r| r.status == SensorStatus::Healthy)
        .count() as f64;
    let degraded_sensors = sensors
        .values()
        .filter(|r| r.status == SensorStatus::Degraded)
        .count() as f64;
    let failed_sensors = sensors
        .values()
        .filter(|r| {
            matches!(
                r.status,
                SensorStatus::Failed | SensorStatus::CommunicationError
            )
        })
        .count() as f64;

    let system_availability = (healthy_sensors + degraded_sensors) / total_sensors * 100.0;
    let system_health = healthy_sensors / total_sensors * 100.0;

    println!("      📊 System availability: {:.1}%", system_availability);
    println!("      📊 System health: {:.1}%", system_health);
    println!("      📊 Failed sensors: {:.0}/{:.0}", failed_sensors, total_sensors);

    if Uncertain::<f64>::point(degraded_sensors).greater_than(0.0).implicit_conditional()? {
        let maintenance_urgency = degraded_sensors / total_sensors * 100.0;
        println!(
            "      🔧 Maintenance needed for {:.0} sensors ({:.1}% of system)",
            degraded_sensors, maintenance_urgency
        );

        if Uncertain::<f64>::point(maintenance_urgency).greater_than(30.0).implicit_conditional()? {
            println!("         🚨 HIGH PRIORITY: Schedule immediate maintenance");
        } else if Uncertain::<f64>::point(maintenance_urgency).greater_than(15.0).implicit_conditional()? {
            println!("         ⚠️  MEDIUM PRIORITY: Schedule maintenance within week");
        } else {
            println!("         📅 LOW PRIORITY: Include in next routine maintenance");
        }
    }

    let risk_level = if Uncertain::<f64>::point(system_health).less_than(50.0).implicit_conditional()? {
        "CRITICAL"
    } else if Uncertain::<f64>::point(system_health).less_than(70.0).implicit_conditional()? {
        "HIGH"
    } else if Uncertain::<f64>::point(system_health).less_than(85.0).implicit_conditional()? {
        "MEDIUM"
    } else {
        "LOW"
    };

    println!("      🎯 Overall system risk: {}", risk_level);

    if Uncertain::<f64>::point(system_health).less_than(70.0).implicit_conditional()? {
        println!("      💡 Recommendations:");
        println!("         - Implement redundant sensor deployment");
        println!("         - Increase monitoring frequency");
        println!("         - Review maintenance procedures");
    }
    Ok(())
}
