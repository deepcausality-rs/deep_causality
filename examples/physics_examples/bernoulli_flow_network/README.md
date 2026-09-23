# Bernoulli Flow Network

This example sends water through a pipe network and chains the fluid state across the segments with the causal monad, then checks that every segment conserves the total head.

## How to Run

```bash
cargo run -p physics_examples --example bernoulli_flow_network
```

## Expected Output

```
=== Bernoulli Flow Network Simulation ===

Fluid: Water (1000 kg/m³)
Flow Rate: 0.10 m³/s

[0] Reservoir       | P=200000.0 Pa | v=0.00 m/s | h=10.0 m
[1]       Main Pipe | P=194933.9 Pa | v=3.18 m/s | h=10.0 m
[2]         Venturi | P=118943.1 Pa | v=12.73 m/s | h=10.0 m  (Venturi: pressure drops as velocity increases)
[3]   Ground Outlet | P=293000.4 Pa | v=3.18 m/s | h=0.0 m   (Drop: potential energy → pressure)

=== Simulation Complete ===
Final State: Ground Outlet at P=293000.4 Pa, v=3.18 m/s, h=0.0 m
```

---

## Physics Overview

The simulation follows a fluid parcel through the network and applies **Bernoulli's Principle** at each segment transition:

$$P_1 + \frac{1}{2}\rho v_1^2 + \rho g h_1 = P_2 + \frac{1}{2}\rho v_2^2 + \rho g h_2$$

The equation conserves energy along a streamline in an ideal (inviscid, incompressible) fluid.

### Network Segments

| Segment | Description | Diameter | Height | Key Physics |
|---------|-------------|----------|--------|-------------|
| 0 | **Reservoir** | Large (10 m) | 10 m | Starting point, high pressure potential |
| 1 | **Main Pipe** | 0.2 m | 10 m | Velocity rises as the flow enters the narrower pipe |
| 2 | **Venturi** | 0.1 m | 10 m | **Venturi Effect**: pressure drops as velocity rises in the constriction |
| 3 | **Ground Outlet** | 0.2 m | 0 m | **Hydrostatic Gain**: potential energy (height) converts to pressure |

### Key Principles Demonstrated

1. **Continuity Equation**: $A_1 v_1 = A_2 v_2 = Q$ (constant volumetric flow rate)
   - A smaller pipe area forces a higher velocity to keep the flow constant
   
2. **Venturi Effect** (Segment 1→2): 
   - Diameter halves (0.2m → 0.1m), so the area shrinks by a factor of 4
   - Velocity quadruples (3.18 → 12.73 m/s)
   - Pressure drops (195 kPa → 119 kPa)

3. **Hydrostatic Pressure** (Segment 2→3):
   - Height drops from 10m to 0m
   - Potential energy $\rho g h = 1000 \times 9.8 \times 10 ≈ 98$ kPa converts to pressure
   - The outlet ends with the highest pressure in the network

---

## Simulation Configuration

| Parameter | Value | Description |
|-----------|-------|-------------|
| Fluid Density | 1000 kg/m³ | Water at STP |
| Flow Rate | 0.1 m³/s | 100 L/s volumetric flow |
| Initial Pressure | 200 kPa | ~2 atm |
| Initial Height | 10 m | Reservoir elevation |

---

## Causal Chain Architecture

`CausalFlow::value` starts the pipeline with the reservoir state, and one `.bind` per segment appends the next fluid state to the trace:

```
Reservoir → Main Pipe → Venturi → Ground Outlet
    ↓           ↓          ↓          ↓
FluidState  FluidState  FluidState  FluidState
```

### FluidState Structure

Each state holds:
- **Pressure** ($P$): static pressure in Pascals
- **Velocity** ($v$): flow velocity in m/s
- **Height** ($h$): elevation in meters
- **Label**: the segment name

### Key Design Patterns

1. **Value channel**: The trace `Vec<FluidState>` travels in the value channel; each segment reads the last state and appends its own.
2. **Error channel**: A failed quantity constructor or `bernoulli_pressure` call becomes the flow's error, and the run prints it.
3. **Head check**: After the pipeline, the run compares each segment's total head $P + \frac{1}{2}\rho v^2 + \rho g h$ with the reservoir's.

---

## Key APIs Used

| API | Purpose |
|-----|---------|
| `deep_causality_physics::bernoulli_pressure` | Computes the new pressure from the Bernoulli equation |
| `deep_causality_physics::Pressure` | Type-safe pressure values (Pa) |
| `deep_causality_physics::Speed` | Type-safe velocity values (m/s) |
| `deep_causality_physics::Length` | Type-safe distance values (m) |
| `deep_causality_physics::Density` | Type-safe density values (kg/m³) |
| `deep_causality_core::CausalFlow` | Monadic effect propagation |

---

## Physical Verification

The results check out by hand:

1. **Continuity**: At d=0.2m, A=π(0.1)²≈0.0314 m², so v=Q/A=0.1/0.0314≈3.18 m/s ✓
2. **Venturi**: At d=0.1m, A=π(0.05)²≈0.00785 m², so v=0.1/0.00785≈12.73 m/s ✓
3. **Energy Conservation**: Pressure gain at outlet ≈ ρgh = 1000×9.8×10 = 98 kPa

The final pressure (~293 kPa) exceeds the initial (~200 kPa) because the elevation drop converts potential energy into pressure.
