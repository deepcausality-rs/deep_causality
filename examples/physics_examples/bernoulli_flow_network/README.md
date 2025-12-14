# Bernoulli Flow Network

This example simulates a fluid dynamics network using Causal Monads to chain state updates across pipe segments.

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

The simulation tracks a fluid parcel moving through a pipe network, applying **Bernoulli's Principle** at each segment transition:

$$P_1 + \frac{1}{2}\rho v_1^2 + \rho g h_1 = P_2 + \frac{1}{2}\rho v_2^2 + \rho g h_2$$

This equation expresses conservation of energy along a streamline in an ideal (inviscid, incompressible) fluid.

### Network Segments

| Segment | Description | Diameter | Height | Key Physics |
|---------|-------------|----------|--------|-------------|
| 0 | **Reservoir** | Large (10 m) | 10 m | Starting point with high pressure potential |
| 1 | **Main Pipe** | 0.2 m | 10 m | Velocity increases as flow enters narrower pipe |
| 2 | **Venturi** | 0.1 m | 10 m | **Venturi Effect**: pressure drops as velocity increases in constriction |
| 3 | **Ground Outlet** | 0.2 m | 0 m | **Hydrostatic Gain**: potential energy (height) converts to pressure |

### Key Principles Demonstrated

1. **Continuity Equation**: $A_1 v_1 = A_2 v_2 = Q$ (constant volumetric flow rate)
   - As pipe area decreases, velocity must increase to maintain constant flow
   
2. **Venturi Effect** (Segment 1→2): 
   - Diameter halves (0.2m → 0.1m), area decreases by factor of 4
   - Velocity quadruples (3.18 → 12.73 m/s)
   - Pressure drops significantly (195 kPa → 119 kPa)

3. **Hydrostatic Pressure** (Segment 2→3):
   - Height drops from 10m to 0m
   - Potential energy $\rho g h = 1000 \times 9.8 \times 10 ≈ 98$ kPa converts to pressure
   - Final pressure is highest despite starting lower

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

The simulation uses `CausalEffectPropagationProcess` to model fluid state propagation:

```
Reservoir → Main Pipe → Venturi → Ground Outlet
    ↓           ↓          ↓          ↓
FluidState  FluidState  FluidState  FluidState
```

### FluidState Structure

Each state tracks:
- **Pressure** ($P$): Current static pressure in Pascals
- **Velocity** ($v$): Current flow velocity in m/s
- **Height** ($h$): Current elevation in meters
- **Diameter** ($D$): Pipe diameter at this segment

### Key Design Patterns

1. **First Bind**: Uses the `state` parameter (initial FluidState from `with_state`)
2. **Subsequent Binds**: Use the first closure parameter (`prev_state.into_value()`) which is the propagating FluidState from the previous segment
3. **Captured Variables**: `density` and `flow_rate_volumetric` are captured from outer scope

---

## Key APIs Used

| API | Purpose |
|-----|---------|
| `deep_causality_physics::bernoulli_pressure` | Computes new pressure using Bernoulli equation |
| `deep_causality_physics::Pressure` | Type-safe pressure values (Pa) |
| `deep_causality_physics::Speed` | Type-safe velocity values (m/s) |
| `deep_causality_physics::Length` | Type-safe distance values (m) |
| `deep_causality_physics::Density` | Type-safe density values (kg/m³) |
| `deep_causality_core::CausalEffectPropagationProcess` | Monadic effect propagation |

---

## Physical Verification

The simulation results can be verified:

1. **Continuity**: At d=0.2m, A=π(0.1)²≈0.0314 m², so v=Q/A=0.1/0.0314≈3.18 m/s ✓
2. **Venturi**: At d=0.1m, A=π(0.05)²≈0.00785 m², so v=0.1/0.00785≈12.73 m/s ✓
3. **Energy Conservation**: Pressure gain at outlet ≈ ρgh = 1000×9.8×10 = 98 kPa

The final pressure (~293 kPa) is higher than initial (~200 kPa) because the elevation drop converts potential energy into pressure.
