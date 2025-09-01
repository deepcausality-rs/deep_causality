/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::Uncertain;

// Helper for approximate equality for f64
fn assert_approx_eq(a: f64, b: f64, epsilon: f64) {
    assert!(
        (a - b).abs() < epsilon,
        "{} is not approximately equal to {}",
        a,
        b
    );
}

use rusty_fork::rusty_fork_test;

rusty_fork_test! {
    #[test]
    fn integration_test_sensor_fusion_and_aggregation() {
    // Scenario: Two sensors measure a value, each with some noise.
    // We combine their readings by averaging and estimate the combined value's properties.

    let sensor1_mean = 50.0;
    let sensor1_std_dev = 2.0;
    let sensor2_mean = 52.0;
    let sensor2_std_dev = 1.5;

    // Sensor readings as uncertain values
    let sensor1_reading = Uncertain::<f64>::normal(sensor1_mean, sensor1_std_dev);
    let sensor2_reading = Uncertain::<f64>::normal(sensor2_mean, sensor2_std_dev);

    // Combined reading: simple average
    // (sensor1_reading + sensor2_reading) / Uncertain::point(2.0)
    // Note: Division by a constant requires the constant to be an Uncertain<f64>
    let combined_reading = (sensor1_reading + sensor2_reading) / Uncertain::<f64>::point(2.0);

    // Estimate the expected value (mean) of the combined reading
    let num_samples_expected_value = 10000;
    let estimated_mean = combined_reading.expected_value(num_samples_expected_value).expect("Expected value calculation failed");

    // The theoretical mean of the average of two independent normal distributions
    // is the average of their means.
    let theoretical_combined_mean = (sensor1_mean + sensor2_mean) / 2.0;

    // Assert that the estimated mean is close to the theoretical combined mean
    assert_approx_eq(estimated_mean, theoretical_combined_mean, 0.5); // Allow some tolerance

    // Estimate the standard deviation of the combined reading
    let num_samples_std_dev = 10000;
    let estimated_std_dev = combined_reading.standard_deviation(num_samples_std_dev).expect("Expected value calculation failed");

    // The theoretical variance of the average of two independent normal distributions
    // is (Var1 + Var2) / n^2, where n=2.
    // Var1 = std_dev1^2, Var2 = std_dev2^2
    let theoretical_combined_variance = (sensor1_std_dev.powi(2) + sensor2_std_dev.powi(2)) / 4.0;
    let theoretical_combined_std_dev = theoretical_combined_variance.sqrt();

    // Assert that the estimated standard deviation is close to the theoretical combined std_dev
    assert_approx_eq(estimated_std_dev, theoretical_combined_std_dev, 0.1); // Allow some tolerance

}

#[test]
fn integration_test_decision_making_under_uncertainty(){
    // Scenario: A system needs to decide if a condition is true, given uncertain inputs.

    // Input A: Uniformly distributed value
    let input_a_low = 10.0;
    let input_a_high = 20.0;
    let input_a = Uncertain::<f64>::uniform(input_a_low, input_a_high);

    // Input B: Normally distributed value
    let input_b_mean = 25.0;
    let input_b_std_dev = 3.0;
    let input_b = Uncertain::<f64>::normal(input_b_mean, input_b_std_dev);

    // Condition: (Input A * 2.0) > Input B
    // First, map Input A to (Input A * 2.0)
    let input_a_doubled = input_a.map(|x| x * 2.0); // Range 20.0 to 40.0

    // Then, compare input_a_doubled with input_b
    let condition = input_a_doubled.gt_uncertain(&input_b); // This returns Uncertain<bool>

    // Decision 1: Use to_bool with high confidence.
    // For this scenario, input_a_doubled (mean 30) is generally greater than input_b (mean 25).
    // So, with high confidence, the condition should be true.
    let decision_high_confidence = condition.to_bool(0.99).expect("Expected value calculation failed");
    assert!(decision_high_confidence);

    // Decision 2: Use probability_exceeds with a threshold.
    // What is the probability that the condition is true?
    let estimated_prob_condition_true = condition.estimate_probability(10000).expect("Expected value calculation failed");

    // Since input_a_doubled (mean 30) is generally higher than input_b (mean 25),
    // the probability of (A*2 > B) should be significantly greater than 0.5.
    assert!(estimated_prob_condition_true > 0.7); // Expect a high probability

    // Let's create a condition that is likely false
    let input_c_mean = 50.0;
    let input_c_std_dev = 5.0;
    let input_c = Uncertain::<f64>::normal(input_c_mean, input_c_std_dev);

    let condition_false = input_a_doubled.lt_uncertain(&input_c); // (A*2) < C (20-40 vs 50)

    let decision_false_high_confidence = condition_false.to_bool(0.99).expect("Expected value calculation failed");
    assert!(decision_false_high_confidence);
}
}
