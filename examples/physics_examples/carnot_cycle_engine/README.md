# Carnot Cycle Heat Engine

This example runs a four-stroke Carnot cycle as one causal-monad pipeline and checks that the cycle closes at the Carnot efficiency limit.

## How to Run

```bash
cargo run -p physics_examples --example carnot_cycle_engine
```

## Expected Output

```
=== Carnot Heat Engine Simulation ===

Start (Point A)                | P=415700.0 Pa | V=0.0100 m³ | T=500.0 K | S=+0.00 J/K
Isothermal Expansion (A→B)     | P=207850.0 Pa | V=0.0200 m³ | T=500.0 K | S=+5.76 J/K
Adiabatic Expansion (B→C)      | P= 57960.0 Pa | V=0.0430 m³ | T=300.0 K | S=+5.76 J/K
Isothermal Compression (C→D)   | P=115919.9 Pa | V=0.0215 m³ | T=300.0 K | S=+0.00 J/K
Adiabatic Compression (D→A)    | P=415700.0 Pa | V=0.0100 m³ | T=500.0 K | S=+0.00 J/K

=== Cycle Complete ===
Total Work Done: 1152.57 J
Carnot Efficiency Limit: 40.0%
```

---

## Physics Overview

The **Carnot Cycle** sets the upper limit of efficiency for any heat engine running between two thermal reservoirs. It consists of four reversible processes:

### The Four Stages

| Stage | Process | Description |
|-------|---------|-------------|
| A→B | **Isothermal Expansion** | Gas expands at constant $T_H$, absorbs heat $Q_{in}$ from the hot reservoir, and does work. |
| B→C | **Adiabatic Expansion** | Gas expands with no heat exchange ($Q=0$); temperature drops from $T_H$ to $T_C$. Gas does work. |
| C→D | **Isothermal Compression** | Gas is compressed at constant $T_C$ and rejects heat $Q_{out}$ to the cold reservoir. Work is done on the gas. |
| D→A | **Adiabatic Compression** | Gas is compressed with no heat exchange; temperature rises from $T_C$ to $T_H$. Work is done on the gas. |

### Key Equations

**Carnot Efficiency:**
$$\eta_{Carnot} = 1 - \frac{T_C}{T_H}$$

**Ideal Gas Law:**
$$PV = nRT$$

**Adiabatic Process (for monatomic ideal gas, $\gamma = 5/3$):**
$$T V^{\gamma-1} = \text{constant}$$

**Isothermal Work:**
$$W = nRT \ln\left(\frac{V_f}{V_i}\right)$$

**Adiabatic Work:**
$$W = C_V (T_i - T_f) = \frac{3}{2}nR(T_i - T_f)$$

---

## Simulation Configuration

| Parameter | Value | Description |
|-----------|-------|-------------|
| $n$ | 1.0 mol | Amount of substance |
| $T_H$ | 500 K | Hot reservoir temperature |
| $T_C$ | 300 K | Cold reservoir temperature |
| $V_A$ | 0.01 m³ | Initial volume (10 L) |
| $P_A$ | 415,700 Pa | Initial pressure (~4 atm) |
| Expansion Ratio | 2.0 | $V_B/V_A$ |

---

## Causal Chain Architecture

`CausalFlow::value` from `deep_causality_core` starts the pipeline with the state at point A, and each stroke is one `.bind()`:

```
Initial State → Step 1 (A→B) → Step 2 (B→C) → Step 3 (C→D) → Step 4 (D→A) → Final State
     ↓              ↓              ↓              ↓              ↓
  EngineState   EngineState   EngineState   EngineState   EngineState
```

### EngineState Structure

Each state holds:
- **Pressure** ($P$): gas pressure in Pascals
- **Volume** ($V$): gas volume in m³
- **Temperature** ($T$): gas temperature in Kelvin
- **Entropy** ($S$): cumulative entropy change in J/K
- **Work Done**: cumulative work output in Joules
- **Heat Absorbed**: cumulative heat taken from the hot reservoir in Joules
- **Phase**: the stage name

### Key Design Patterns

1. **Monadic Composition**: Each stroke reads the last `EngineState` on the trace and appends the next one.
2. **Derived End Point**: The closing stroke D→A derives its end point from the adiabat $T V^{\gamma-1} = \text{constant}$ instead of restating the starting values.
3. **Closure Check**: `ideal_gas_law` recovers $R$ from the end state, and the run compares it with CODATA and the returned volume with $V_A$.
4. **Efficiency Check**: The measured $W / Q_{in}$ is compared with `carnot_efficiency`.

---

## Key APIs Used

| API | Purpose |
|-----|---------|
| `deep_causality_physics::Pressure` | Type-safe pressure values (Pa) |
| `deep_causality_physics::Volume` | Type-safe volume values (m³) |
| `deep_causality_physics::Temperature` | Type-safe temperature values (K) |
| `deep_causality_physics::AmountOfSubstance` | Type-safe amount (mol) |
| `deep_causality_physics::carnot_efficiency` | Computes the theoretical efficiency limit |
| `deep_causality_physics::ideal_gas_law` | Verifies P, V, n, T consistency |
| `deep_causality_core::CausalFlow` | Monadic effect propagation |

---

## Thermodynamic Verification

The run checks four thermodynamic principles:

1. **Entropy Conservation**: Entropy returns to its initial value (S=0) after the cycle, as reversibility requires
2. **State Cycle Closure**: The system returns to its initial state (P, V, T)
3. **Efficiency Bound**: Net work output (~1153 J) matches the 40% Carnot limit
4. **Energy Conservation**: Work equals net heat transfer ($W = Q_{in} - Q_{out}$)
