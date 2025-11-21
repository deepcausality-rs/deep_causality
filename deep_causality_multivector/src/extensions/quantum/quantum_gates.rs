/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::HilbertState;
use deep_causality_num::{Complex64, One, Zero};

/// A suite of standard Quantum Gates (Unitary Operators).
/// These are constructed as elements of the algebra (End(A)).
///
/// Mapping logic for Cl(0, n) metric:
/// * Pauli X ~ i * Gamma_1
/// * Pauli Y ~ i * Gamma_2
/// * Pauli Z ~ i * Gamma_1 * Gamma_2
///
/// (The factor of 'i' is required because Gamma^2 = -1 in Spin(10),
/// but Pauli Matrices must square to +1).
pub trait QuantumGates {
    /// The Identity Operator (No-op)
    fn gate_identity() -> Self;

    /// Pauli-X (NOT Gate).
    /// Flips |0> to |1>.
    fn gate_x() -> Self;

    /// Pauli-Y.
    /// Maps |0> -> i|1> and |1> -> -i|0>.
    fn gate_y() -> Self;

    /// Pauli-Z (Phase Flip).
    /// Leaves |0> unchanged, maps |1> -> -|1>.
    fn gate_z() -> Self;

    /// Hadamard Gate (H).
    /// Creates Superposition: |0> -> (|0> + |1>)/sqrt(2).
    /// Definition: H = (X + Z) / sqrt(2).
    fn gate_hadamard() -> Self;

    /// Phase Gate (S).
    /// Z-rotation by 90 degrees.
    fn gate_s() -> Self;

    /// T Gate (pi/8).
    /// Z-rotation by 45 degrees.
    fn gate_t() -> Self;
}

impl QuantumGates for HilbertState {
    /// The Identity Operator (No-op).
    ///
    /// # Physics
    /// The identity operator $I$ leaves any quantum state $|\psi\rangle$ unchanged: $I|\psi\rangle = |\psi\rangle$.
    /// It is a fundamental operator that represents "doing nothing" to the quantum system.
    ///
    /// # Math
    /// In Geometric Algebra, the identity element is the scalar $1$. When represented as a multivector,
    /// it has a coefficient of $1$ for the scalar blade (grade 0) and zero for all other blades.
    ///
    /// # Rust Details
    /// A new `HilbertState` is created with a `data` vector initialized to zeros, except for the first element
    /// (`data[0]`), which corresponds to the scalar component, set to `Complex64::one()`.
    /// The state is constructed using `new_spin10()`, ensuring it is part of the `Cl(0,10)` algebra.
    fn gate_identity() -> Self {
        // Scalar 1.0
        let mut data = vec![Complex64::zero(); 1024];
        data[0] = Complex64::one();
        Self::new_spin10(data).unwrap()
    }

    /// Pauli-X (NOT Gate).
    ///
    /// # Physics
    /// The Pauli-X gate is the quantum equivalent of the classical NOT gate. It performs a bit flip,
    /// transforming $|0\rangle \leftrightarrow |1\rangle$. It corresponds to a rotation around the X-axis
    /// of the Bloch sphere by $\pi$ radians.
    ///
    /// # Math (Clifford Algebra Mapping)
    /// In the `Cl(0,10)` algebra (where basis vectors $e_k$ square to $-1$), the Pauli-X operator
    /// is represented as $X = i e_1$.
    /// This choice ensures that $X^2 = (i e_1)(i e_1) = i^2 e_1^2 = (-1)(-1) = 1$, matching the
    /// physical property of the Pauli-X matrix.
    ///
    /// # Rust Details
    /// A `HilbertState` is initialized with all coefficients as zero. The coefficient for the
    /// first basis vector ($e_1$, corresponding to bitmap `1`) is set to the imaginary unit `i`.
    /// The state is constructed using `new_spin10()`, ensuring it operates within `Cl(0,10)`.
    fn gate_x() -> Self {
        let mut data = vec![Complex64::zero(); 1024];
        let i = Complex64::new(0.0, 1.0);

        // Index 1 corresponds to basis vector e1
        data[1] = i;

        Self::new_spin10(data).unwrap()
    }

    /// Pauli-Y.
    ///
    /// # Physics
    /// The Pauli-Y gate performs a transformation $|0\rangle \to i|1\rangle$ and $|1\rangle \to -i|0\rangle$.
    /// It corresponds to a rotation around the Y-axis of the Bloch sphere by $\pi$ radians.
    ///
    /// # Math (Clifford Algebra Mapping)
    /// In the `Cl(0,10)` algebra, the Pauli-Y operator is represented as $Y = i e_2$.
    /// This ensures that $Y^2 = (i e_2)(i e_2) = i^2 e_2^2 = (-1)(-1) = 1$, matching the
    /// physical property of the Pauli-Y matrix.
    ///
    /// # Rust Details
    /// A `HilbertState` is initialized with all coefficients as zero. The coefficient for the
    /// second basis vector ($e_2$, corresponding to bitmap `10` or index `2`) is set to the
    /// imaginary unit `i`.
    /// The state is constructed using `new_spin10()`.
    fn gate_y() -> Self {
        let mut data = vec![Complex64::zero(); 1024];
        let i = Complex64::new(0.0, 1.0);

        // Index 2 corresponds to basis vector e2
        data[2] = i;

        Self::new_spin10(data).unwrap()
    }

    /// Pauli-Z (Phase Flip).
    ///
    /// # Physics
    /// The Pauli-Z gate leaves the $|0\rangle$ state unchanged and maps $|1\rangle \to -|1\rangle$.
    /// It applies a $\pi$ phase shift to the $|1\rangle$ component. It corresponds to a rotation
    /// around the Z-axis of the Bloch sphere by $\pi$ radians.
    ///
    /// # Math (Clifford Algebra Mapping)
    /// In the `Cl(0,10)` algebra, the Pauli-Z operator is constructed from the geometric product
    /// of the X and Y operators. Specifically, $Z = -iXY$.
    /// Using the previous definitions $X = i e_1$ and $Y = i e_2$:
    /// $$ Z = -i(i e_1)(i e_2) = -i(i^2 e_1 e_2) = -i(-e_1 e_2) = i e_{12} $$
    /// This ensures $Z^2 = (i e_{12})(i e_{12}) = i^2 (e_{12})^2 = (-1)(-1) = 1$,
    /// matching the physical property of the Pauli-Z matrix.
    ///
    /// # Rust Details
    /// This implementation constructs the Z gate by first obtaining instances of the X and Y gates.
    /// It then computes their geometric product (`xy = x.as_inner() * y.as_inner()`).
    /// Finally, it scales this product by `-i` to match the definition $Z = -iXY$.
    /// The state is constructed using `new_spin10()`.
    fn gate_z() -> Self {
        // Z = i*e12. The basis blade e12 corresponds to the bitmap 0b11, which is index 3.
        let mut data = vec![Complex64::zero(); 1024];
        let i = Complex64::new(0.0, 1.0);
        data[3] = i; // Index 3 corresponds to e12
        Self::new_spin10(data).unwrap()
    }

    /// Hadamard Gate (H).
    ///
    /// # Physics
    /// The Hadamard gate is a single-qubit gate that creates superposition. It transforms
    /// a basis state into a superposition of $|0\rangle$ and $|1\rangle$.
    /// Specifically, $|0\rangle \to \frac{1}{\sqrt{2}}(|0\rangle + |1\rangle)$ and
    /// $|1\rangle \to \frac{1}{\sqrt{2}}(|0\rangle - |1\rangle)$.
    /// It effectively rotates a state on the Bloch sphere by $\pi$ around the axis $(\hat{x} + \hat{z})/\sqrt{2}$.
    ///
    /// # Math
    /// The Hadamard gate can be defined in terms of Pauli matrices:
    /// $$ H = \frac{1}{\sqrt{2}}(X + Z) $$
    ///
    /// # Rust Details
    /// This implementation directly follows the mathematical definition. It obtains instances of
    /// the Pauli-X and Pauli-Z gates, adds their underlying `CausalMultiVector` representations,
    /// and then scales the result by $1/\sqrt{2}$.
    /// The state is constructed using `new_spin10()`.
    fn gate_hadamard() -> Self {
        let x = Self::gate_x();
        let z = Self::gate_z();

        let sum = x.as_inner() + z.as_inner(); // X + Z
        let scale = Complex64::new(1.0 / 2.0f64.sqrt(), 0.0); // 1 / sqrt(2)

        let h_mv = sum * scale;

        Self::new_spin10(h_mv.data).unwrap()
    }

    /// Phase Gate (S).
    ///
    /// # Physics
    /// The S-gate, also known as the Phase gate, applies a phase shift of $\pi/2$ ($90^\circ$)
    /// around the Z-axis of the Bloch sphere. It transforms $|0\rangle \to |0\rangle$ and
    /// $|1\rangle \to i|1\rangle$.
    ///
    /// # Math
    /// The S-gate is a Z-rotation gate, which can be expressed in exponential form.
    /// The implementation here uses the form $S = e^{-i \frac{\pi}{4} Z}$.
    /// Given that $Z^2 = I$ (Identity), the exponential can be expanded as:
    /// $$ e^{-i\theta Z} = (\cos \theta) I - i (\sin \theta) Z $$
    /// For the S-gate, $\theta = \frac{\pi}{4}$.
    ///
    /// # Rust Details
    /// 1.  It obtains the Identity gate ($I$) and the Pauli-Z gate ($Z$).
    /// 2.  The angle `theta` is set to `FRAC_PI_4` ($\pi/4$).
    /// 3.  The terms `cos(theta) * I` and `-i * sin(theta) * Z` are calculated.
    /// 4.  These two multivectors are added to form the final S-gate operator.
    ///
    /// The new state is constructed using `new_spin10()`.
    fn gate_s() -> Self {
        let i_gate = Self::gate_identity();
        let z_gate = Self::gate_z();

        let theta = std::f64::consts::FRAC_PI_4; // pi/4 radians (45 degrees)
        let cos = Complex64::new(theta.cos(), 0.0);
        let neg_i_sin = Complex64::new(0.0, -theta.sin()); // -i * sin(theta)

        let term2 = z_gate.as_inner().clone() * neg_i_sin; // -i * sin(theta) * Z
        let term1 = i_gate.as_inner().clone() * cos; // cos(theta) * I

        let result = term1 + term2;

        Self::new_spin10(result.data).unwrap()
    }

    /// T Gate (pi/8).
    ///
    /// # Physics
    /// The T-gate, also known as the $\pi/8$ gate, applies a phase shift of $\pi/4$ ($45^\circ$)
    /// around the Z-axis of the Bloch sphere. It is a fundamental single-qubit gate,
    /// and a 'non-Clifford' gate, meaning it cannot be constructed from Hadamard and CNOT gates alone.
    /// It transforms $|0\rangle \to |0\rangle$ and $|1\rangle \to e^{i\pi/4}|1\rangle$.
    ///
    /// # Math
    /// The T-gate is a Z-rotation gate, expressed in exponential form.
    /// The implementation here uses the form $T = e^{-i \frac{\pi}{8} Z}$.
    /// Given that $Z^2 = I$ (Identity), the exponential can be expanded as:
    /// $$ e^{-i\theta Z} = (\cos \theta) I - i (\sin \theta) Z $$
    /// For the T-gate, $\theta = \frac{\pi}{8}$.
    ///
    /// # Rust Details
    /// 1.  It obtains the Identity gate ($I$) and the Pauli-Z gate ($Z$).
    /// 2.  The angle `theta` is set to `FRAC_PI_8` ($\pi/8$).
    /// 3.  The terms `cos(theta) * I` and `-i * sin(theta) * Z` are calculated.
    /// 4.  These two multivectors are added to form the final T-gate operator.
    ///
    /// The new state is constructed using `new_spin10()`.
    fn gate_t() -> Self {
        let i_gate = Self::gate_identity();
        let z_gate = Self::gate_z();

        let theta = std::f64::consts::FRAC_PI_8; // pi/8 radians (22.5 degrees)
        let cos = Complex64::new(theta.cos(), 0.0);
        let neg_i_sin = Complex64::new(0.0, -theta.sin()); // -i * sin(theta)

        let term2 = z_gate.as_inner().clone() * neg_i_sin; // -i * sin(theta) * Z
        let term1 = i_gate.as_inner().clone() * cos; // cos(theta) * I

        let result = term1 + term2;

        Self::new_spin10(result.data).unwrap()
    }
}
