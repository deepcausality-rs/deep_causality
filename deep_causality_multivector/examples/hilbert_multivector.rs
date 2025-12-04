/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{HilbertState, MultiVector, QuantumGates, QuantumOps};
use deep_causality_num::{Complex64, Zero};

// -----------------------------------------------------------------------------------------
// ENGINEERING VALUE:
// Quantum Computing simulations often require complex linear algebra libraries.
// Geometric Algebra (specifically Spacetime Algebra or high-dimensional Clifford Algebra)
// provides a natural isomorphism to Quantum Mechanics (Spinors ~ Rotors).
//
// This example demonstrates simulating Quantum Gates (Pauli-X, Hadamard, etc.) and
// States (Qubits) using `HilbertState` (a wrapper around MultiVector).
// This approach allows researchers to verify quantum algorithms using geometric intuition
// and ensures unitary evolution is preserved by the algebraic structure.
// -----------------------------------------------------------------------------------------

// Helper to print a complex number with proper formatting
fn print_complex(label: &str, c: Complex64) {
    println!("{}: {:.4} + {:.4}i", label, c.re, c.im);
}

fn main() {
    println!("--- Quantum Physics Example in Clifford Algebra ---");
    println!("Demonstrating Quantum Gates and Operations with HilbertState in Cl(0,10)");
    println!();

    // --- 1. Initialize a Qubit State ---
    // In this Clifford Algebra context, we use gate_identity() to represent a base state,
    // analogous to |0> in standard quantum computing, or a vacuum state.
    println!("1. Initializing Base States:");
    let zero_state = HilbertState::gate_identity(); // Analogous to |0>
    println!("Initial 'zero_state' (represented by Identity operator):");
    // Display only first few coefficients as it's a large vector
    println!(
        "  Scalar part (data[0]): {:.4} + {:.4}i",
        zero_state.mv().data()[0].re,
        zero_state.mv().data()[0].im
    );
    println!("  Metric: {:?}", zero_state.mv().metric());
    println!();

    // --- 2. Apply Quantum Gates ---
    println!("2. Applying Quantum Gates:");

    // Pauli-X Gate
    let x_gate = HilbertState::gate_x();
    println!("  Pauli-X Gate created.");
    // Apply X gate to zero_state to get 'one_state'
    // Note: In GA, operators are elements of the algebra, so application is geometric product.
    let one_state = x_gate
        .clone()
        .into_inner()
        .geometric_product(zero_state.as_inner());
    let one_state = HilbertState::from_multivector(one_state); // Convert back to HilbertState
    println!("  Applied Pauli-X to 'zero_state' -> 'one_state':");
    println!(
        "    Scalar part (data[0]): {:.4} + {:.4}i",
        one_state.mv().data()[0].re,
        one_state.mv().data()[0].im
    );
    println!(
        "    Vector e1 part (data[1]): {:.4} + {:.4}i",
        one_state.mv().data()[1].re,
        one_state.mv().data()[1].im
    );
    println!();

    // Hadamard Gate
    let h_gate = HilbertState::gate_hadamard();
    println!("  Hadamard Gate created.");
    // Apply H gate to zero_state to get 'plus_state'
    let plus_state = h_gate
        .clone()
        .into_inner()
        .geometric_product(zero_state.as_inner());
    let plus_state = HilbertState::from_multivector(plus_state);
    println!("  Applied Hadamard to 'zero_state' -> 'plus_state':");
    println!(
        "    Scalar part (data[0]): {:.4} + {:.4}i",
        plus_state.mv().data()[0].re,
        plus_state.mv().data()[0].im
    );
    println!(
        "    Vector e1 part (data[1]): {:.4} + {:.4}i",
        plus_state.mv().data()[1].re,
        plus_state.mv().data()[1].im
    );
    println!(
        "    Bivector e12 part (data[3]): {:.4} + {:.4}i",
        plus_state.mv().data()[3].re,
        plus_state.mv().data()[3].im
    );
    println!();

    // --- 3. Demonstrate Quantum Operations ---
    println!("3. Demonstrating Quantum Operations:");

    // Hermitian Conjugate (dag)
    println!("  a. Hermitian Conjugate (dag):");
    let zero_state_dag = zero_state.dag();
    println!("    'zero_state'.dag() (scalar part):");
    print_complex("      Scalar part", zero_state_dag.mv().data()[0]);
    println!();

    let plus_state_dag = plus_state.dag();
    println!("    'plus_state'.dag() (scalar and e1 parts):");
    print_complex("      Scalar part", plus_state_dag.mv().data()[0]);
    print_complex("      Vector e1 part", plus_state_dag.mv().data()[1]);
    print_complex("      Bivector e12 part", plus_state_dag.mv().data()[3]);
    println!();

    // Inner Product (bracket)
    println!("  b. Inner Product (bracket):");
    print_complex(
        "    <zero_state | zero_state>",
        zero_state.bracket(&zero_state),
    ); // Should be 1
    print_complex("    <one_state | one_state>", one_state.bracket(&one_state)); // Should be -1 (due to GA mapping of X as state)
    print_complex(
        "    <zero_state | one_state>",
        zero_state.bracket(&one_state),
    ); // Should be 0
    // Based on test_expectation_value_x_on_x_state, bracket for X as a state with itself is -1.
    // So <plus_state|zero_state> = <(H|0>)|0> = <0|H|0> (if H is Hermitian which it is not, it's unitary).
    // H = (X+Z)/sqrt(2). If |0> is the identity, then <0|H|0> = ScalarPart(I.dag() * H * I) = ScalarPart(H) = 0.
    print_complex(
        "    <plus_state | zero_state>",
        plus_state.bracket(&zero_state),
    ); // Should be ~0 (Scalar part of H is 0)
    print_complex(
        "    <plus_state | plus_state>",
        plus_state.bracket(&plus_state),
    ); // Should be ~1
    println!();

    // Expectation Value
    println!("  c. Expectation Value:");
    let z_gate = HilbertState::gate_z();
    // In Cl(0,10), |0> is identity (scalar 1), Z is i*e12. <I|Z|I> = ScalarPart(I*Z*I) = ScalarPart(Z) = 0.
    print_complex(
        "    <zero_state | Z | zero_state>",
        zero_state.expectation_value(&z_gate),
    ); // Should be 0

    // <X | I | X> (X as state, I as operator) was -1 in tests
    print_complex(
        "    <X | I | X> (X as state, I as operator)",
        x_gate.expectation_value(&HilbertState::gate_identity()),
    );
    println!();

    // Normalization
    println!("  d. Normalization:");
    let mut unnormalized_data = vec![Complex64::zero(); 1024];
    unnormalized_data[0] = Complex64::new(2.0, 0.0); // 2 * Identity
    let unnormalized_state = HilbertState::new_spin10(unnormalized_data).unwrap();
    println!("    Unnormalized state (scalar part):");
    print_complex("      Scalar part", unnormalized_state.mv().data()[0]);

    let normalized_state = unnormalized_state.normalize();
    println!("    Normalized state (scalar part):");
    print_complex("      Scalar part", normalized_state.mv().data()[0]);
    print_complex(
        "    Bracket of normalized state with itself",
        normalized_state.bracket(&normalized_state),
    ); // Should be 1
    println!();

    // --- 4. Advanced: Example of Superposition and Measurement-like Operation ---
    // (Conceptual - not a full quantum simulator)
    println!("4. Conceptual Superposition and 'Measurement':");
    println!("  'plus_state' is a superposition state (~ (|0>+|1>)/sqrt(2)).");
    // These amplitudes will be the scalar part of the bracket
    print_complex(
        "  <plus_state | zero_state> : amplitude to be in |0> state",
        plus_state.bracket(&zero_state),
    );
    print_complex(
        "  <plus_state | one_state> : amplitude to be in |1> state",
        plus_state.bracket(&one_state),
    );
    println!();
}
