# Action Plan: `deep_causality_core` Refinements

This document outlines an actionable plan to address feedback regarding safety, robustness, and clarity of the
`deep_causality_core` crate. The plan is derived from simulated reviews by engineers from Google (Safety), Airbus (
Embedded), and a Stanford professor (Academia).

The core objectives are:

1. **Enhance Safety & Robustness**: Eliminate all panics and heap-allocated errors in the core logic.
2. **Improve Clarity for Safety-Critical Use**: Provide a clear example for `no_std`, real-time systems.
3. **Refine Documentation**: Clarify the distinct roles of the `ControlFlowBuilder` and the monadic effect system.

---

## 1. Refactor `CausalityError` to be a Fixed-Size Type

The current `CausalityError(String)` is unsuitable for high-assurance systems due to its reliance on heap allocation. It
will be replaced with a struct containing a fixed-size enum.

### Step 1.1: Define the New Error Enum

A new `CausalityErrorEnum` will be created to represent all possible error states within the `deep_causality_core`
crate.

**File**: `deep_causality_core/src/errors/causality_error_enum.rs` (new file)

```rust
/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CausalityErrorEnum {
    // Generic Errors
    Unspecified = 0,
    InternalLogicError = 1, // For logic paths that should be unreachable
    TypeConversionError = 2, // For failures in `FromProtocol`

    // Graph Execution Errors
    StartNodeOutOfBounds = 10,
    MaxStepsExceeded = 11,
    GraphExecutionProducedNoResult = 12,

    // Add other specific errors as they are identified
}

impl Default for CausalityErrorEnum {
    fn default() -> Self {
        Self::Unspecified
    }
}
```

### Step 1.2: Update the `CausalityError` Struct

The main error struct will be modified to use the new enum.

**File**: `deep_causality_core/src/errors/causality_error.rs`

```rust
// BEFORE
use core::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct CausalityError(pub String);

impl CausalityError {
    pub fn new(message: String) -> Self {
        CausalityError(message)
    }
}

// AFTER
use crate::errors::causality_error_enum::CausalityErrorEnum;
use core::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CausalityError(pub CausalityErrorEnum);

impl CausalityError {
    pub fn new(error_enum: CausalityErrorEnum) -> Self {
        CausalityError(error_enum)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CausalityError {}

impl Display for CausalityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // Delegate to the debug representation of the inner enum.
        write!(f, "{:?}", self.0)
    }
}
```

### Step 1.3: Update Dependent Code

All code that currently creates a `CausalityError` with a `String` must be updated. This includes the
`CausalProtocol::error` method, which will need its signature changed to be compatible with a static error type.

---

## 2. Eliminate Panics in Monadic Implementations

The use of `expect()` in `fmap` and `bind` implementations violates the principles of robust, panic-free code. These
will be replaced with proper error handling.

### Step 2.1: Refactor `fmap`

**File**: `deep_causality_core/src/types/propagating_effect/hkt.rs` (and equivalents)

The plan is to add a check. If the effect is already in an error state or its value is `None` (which would be an
inconsistent state), the function `f` is not called, and an error is propagated.

**Plan for `fmap` in `Functor` implementation:**

```rust
// BEFORE
fn fmap<A, B, Func>(
    m_a: /* ... */,
    f: Func,
) -> /* ... */
where
    Func: FnOnce(A) -> B,
{
    CausalEffectPropagationProcess {
        value: EffectValue::Value(f(m_a
            .value
            .into_value()
            // THIS IS A PANIC
            .expect("Functor fmap on a non-error effect should contain a value"))),
        // ...
    }
}

// AFTER (Conceptual)
fn fmap<A, B, Func>(
    m_a: /* ... */,
    f: Func,
) -> /* ... */
where
    Func: FnOnce(A) -> B,
    // Error type needs to be compatible
{
    if m_a.is_err() {
        // Just pass the error through, without applying f
        return m_a.into_new_type(); // Helper to change generic type
    }
    
    match m_a.value.into_value() {
        Some(val) => CausalEffectPropagationProcess {
            value: EffectValue::Value(f(val)),
            state: m_a.state,
            context: m_a.context,
            error: m_a.error,
            logs: m_a.logs,
        },
        None => {
            // This is an inconsistent state, an effect without error should have a value
            let mut new_effect = m_a.into_new_type();
            new_effect.error = Some(CausalityError::new(CausalityErrorEnum::InternalLogicError));
            new_effect
        }
    }
}
```

### Step 2.2: Refactor `bind`

A similar pattern will be applied to `bind`. The logic will explicitly handle cases where an effect lacks a value
without being in an error state, treating it as a new `InternalLogicError`.

---

## 3. Add Avionics Example for Hard Real-Time Systems

To demonstrate the crate's suitability for safety-critical applications, a new, self-contained example will be created.
It will be `no_std` and use fixed-size types (`f32`, arrays) to model a flight control scenario.

### Step 3.1: Create New Example File

**File**: `deep_causality_core/examples/control_flow_strict_zst.rs` (new file)

```rust
/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![no_std]
extern crate alloc;

use alloc::collections::VecDeque;
use deep_causality_core::{
    CausalProtocol, ControlFlowBuilder, FromProtocol, ToProtocol,
};

// 1. Define a static, fixed-size error enum for the flight system.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlightSystemError {
    SensorFailure,
    ValueOutOfRange,
    ProtocolMismatch,
}

// 2. Define a fixed-size data protocol using arrays instead of Vecs.
// This protocol is `Copy`-able and has a statically known size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FlightControlProtocol {
    AttitudeRequest(bool),       // Input: request sensor data
    AttitudeReading([f32; 3]),   // Sensor data: [roll, pitch, yaw]
    ControlSurfaceUpdate([f32; 3]), // Output: [aileron, elevator, rudder]
    Error(FlightSystemError),
}

// 3. Implement the main CausalProtocol trait for our flight protocol.
// Note the error function takes our static error enum.
impl CausalProtocol for FlightControlProtocol {
    fn error<E: core::fmt::Display>(_msg: &E) -> Self {
        // In a real system, you'd map the generic error to a specific protocol error.
        // For this example, we'll use a dedicated variant.
        // The original error message is discarded to prevent allocation.
        Self::Error(FlightSystemError::ProtocolMismatch)
    }
}

// 4. Implement protocol conversions for all data types used in the graph.
// These are boilerplate implementations that wrap/unwrap data from the enum.
impl ToProtocol<FlightControlProtocol> for bool {
    fn to_protocol(self) -> FlightControlProtocol {
        FlightControlProtocol::AttitudeRequest(self)
    }
}
impl FromProtocol<FlightControlProtocol> for bool {
    type Error = FlightSystemError;
    fn from_protocol(p: FlightControlProtocol) -> Result<Self, Self::Error> {
        match p {
            FlightControlProtocol::AttitudeRequest(v) => Ok(v),
            _ => Err(FlightSystemError::ProtocolMismatch),
        }
    }
}

impl ToProtocol<FlightControlProtocol> for [f32; 3] {
    fn to_protocol(self) -> FlightControlProtocol {
        // Differentiate between sensor reading and control update based on context.
        // For simplicity, we default to AttitudeReading. A real system might need
        // a more complex protocol.
        FlightControlProtocol::AttitudeReading(self)
    }
}
impl FromProtocol<FlightControlProtocol> for [f32; 3] {
    type Error = FlightSystemError;
    fn from_protocol(p: FlightControlProtocol) -> Result<Self, Self::Error> {
        match p {
            FlightControlProtocol::AttitudeReading(v) => Ok(v),
            FlightControlProtocol::ControlSurfaceUpdate(v) => Ok(v),
            _ => Err(FlightSystemError::ProtocolMismatch),
        }
    }
}

// 5. Define the logic functions. These MUST be static functions, not closures.
// They operate on fixed-size types (`f32`, arrays).

// Reads from a sensor. Returns a fixed-size array.
fn read_attitude_sensor(request: bool) -> [f32; 3] {
    if request {
        [0.1, -0.2, 0.05] // Dummy roll, pitch, yaw
    } else {
        [0.0, 0.0, 0.0]
    }
}

// Analyzes sensor data to determine required control adjustments.
fn attitude_correction_logic(reading: [f32; 3]) -> [f32; 3] {
    // Simplified logic: invert the roll and pitch to correct.
    [-reading[0], -reading[1], 0.0]
}

// Final check to ensure control values are within safety limits.
fn check_control_limits(controls: [f32; 3]) -> [f32; 3] {
    // Clamp values to a safe range of [-1.0, 1.0]
    [
        controls[0].clamp(-1.0, 1.0),
        controls[1].clamp(-1.0, 1.0),
        controls[2].clamp(-1.0, 1.0),
    ]
}

// Main function to build and execute the graph.
pub fn main() {
    // This example will only work if compiled with `strict-zst` feature.
    // Assumes `default-features = false`.
    
    let mut builder = ControlFlowBuilder::<FlightControlProtocol>::new();

    // Add nodes using static function items.
    let n_sensor = builder.add_node(read_attitude_sensor);
    let n_logic = builder.add_node(attitude_correction_logic);
    let n_limits = builder.add_node(check_control_limits);

    // Connect nodes. The compiler verifies that `[f32; 3]` flows correctly.
    builder.connect(n_sensor, n_logic);
    builder.connect(n_logic, n_limits);
    
    let graph = builder.build();

    // Execute with a pre-allocated queue to prevent allocation in the loop.
    let mut queue = VecDeque::with_capacity(10);
    let input = true.to_protocol(); // Start by requesting a sensor read.
    
    let result = graph.execute(input, n_sensor.id(), 10, &mut queue);

    // The final result should be the clamped control surface update.
    assert_eq!(
        result.unwrap(),
        FlightControlProtocol::ControlSurfaceUpdate([-0.1, 0.2, 0.0])
    );
}

```

### Step 3.2: Add Example to `Cargo.toml`

The new example will be registered in `deep_causality_core/Cargo.toml`.

```toml
[[example]]
name = "control_flow_strict_zst"
path = "examples/control_flow_strict_zst.rs"
required-features = ["strict-zst"]
```

---

## 4. Update README Documentation

The `deep_causality_core/README.md` will be updated to clarify the purpose and separation of its two main components.

**Proposed Addition to `README.md`:**

<hr>

### Architecture: Two Tools for Two Jobs

`deep_causality_core` provides two distinct, powerful abstractions for structuring complex logic. They can be used
independently.

#### 1. The `ControlFlowBuilder`: For Correct-by-Construction Systems

* **What It Is**: A tool for defining a **static, compile-time verified** execution graph. Nodes are functions, and
  edges are data flows between them.
* **Key Feature**: **Safety**. The Rust compiler ensures you can only connect an output of type `T` to an input of type
  `T`. This eliminates an entire class of runtime integration errors.
* **Best For**:
    * **Safety-Critical & Embedded Systems**: With the `strict-zst` feature, it guarantees zero heap allocations and no
      dynamic dispatch in the execution path, making it suitable for environments requiring formal verification and WCET
      analysis (e.g., avionics, automotive).
    * **ETL Pipelines**: Defining fixed data transformation pipelines where the flow is static and reliability is
      paramount.
* **Independence**: It has **no dependency** on the monadic effect system. It is a standalone tool for building robust,
  static graphs.

#### 2. The Monadic Effect System: For Dynamic Causal Reasoning

* **What It Is**: A flexible, functional foundation for modeling processes using monadic types like
  `PropagatingEffect` (stateless) and `PropagatingProcess` (stateful).
* **Key Feature**: **Flexibility & Composability**. It allows for dynamic chaining of operations (`bind`), state
  propagation, context-awareness, and causal interventions.
* **Best For**:
    * The foundation of the main `deep_causality` library.
    * Complex simulations where state and context evolve.
    * Systems that need to reason about and dynamically respond to events.
* **Relationship**: This system is the engine that enables the advanced causal reasoning capabilities of the wider
  DeepCausality ecosystem.

**How to Choose**:

* If you need to define a **fixed, ultra-reliable data-flow graph** and cannot tolerate runtime errors or allocations,
  use the **`ControlFlowBuilder`**.
* If you are building a system that requires **dynamic, stateful, and context-aware reasoning**, use the **Monadic
  Effect System**.
* The two can be used together; for instance, a node within a `ControlFlowBuilder` graph could itself execute a complex
  computation managed by the monadic system.

---
