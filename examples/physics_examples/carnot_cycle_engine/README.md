# Carnot Cycle Heat Engine

A simulation of the theoretical limit of thermodynamic efficiency using Causal Monads.

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

The **Carnot Cycle** represents the theoretical upper limit of efficiency for any heat engine operating between two thermal reservoirs. It consists of 4 reversible processes:

### The Four Stages

| Stage | Process | Description |
|-------|---------|-------------|
| A→B | **Isothermal Expansion** | Gas expands at constant $T_H$. Heat $Q_{in}$ absorbed from hot reservoir. Work done by gas. |
| B→C | **Adiabatic Expansion** | Gas expands with no heat exchange ($Q=0$). Temperature drops from $T_H$ to $T_C$. Work done by gas. |
| C→D | **Isothermal Compression** | Gas compressed at constant $T_C$. Heat $Q_{out}$ rejected to cold reservoir. Work done on gas. |
| D→A | **Adiabatic Compression** | Gas compressed with no heat exchange. Temperature rises from $T_C$ to $T_H$. Work done on gas. |

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
| Compression Ratio | 2.0 | $V_B/V_A$ |

---

## Causal Chain Architecture

The simulation models the Carnot cycle as a **causal propagation process** using `CausalEffectPropagationProcess` from `deep_causality_core`. Each thermodynamic stage is represented as a `.bind()` operation:

```
Initial State → Step 1 (A→B) → Step 2 (B→C) → Step 3 (C→D) → Step 4 (D→A) → Final State
     ↓              ↓              ↓              ↓              ↓
  EngineState   EngineState   EngineState   EngineState   EngineState
```

### EngineState Structure

Each state in the cycle tracks:
- **Pressure** ($P$): Current gas pressure in Pascals
- **Volume** ($V$): Current gas volume in m³
- **Temperature** ($T$): Current gas temperature in Kelvin
- **Entropy** ($S$): Cumulative entropy change in J/K
- **Work Done**: Cumulative work output in Joules
- **Phase**: Human-readable description of current stage

### Key Design Patterns

1. **Monadic Composition**: Each stage transforms the engine state and propagates it to the next stage
2. **Closure Captures**: Configuration parameters are captured from the outer scope for use in bind closures
3. **Value Propagation**: The `EngineState` flows through the chain as the monadic value
4. **Physics Validation**: Ideal Gas Law is verified at critical points

---

## Key APIs Used

| API | Purpose |
|-----|---------|
| `deep_causality_physics::Pressure` | Type-safe pressure values (Pa) |
| `deep_causality_physics::Volume` | Type-safe volume values (m³) |
| `deep_causality_physics::Temperature` | Type-safe temperature values (K) |
| `deep_causality_physics::AmountOfSubstance` | Type-safe amount (mol) |
| `deep_causality_physics::carnot_efficiency` | Computes theoretical efficiency limit |
| `deep_causality_physics::ideal_gas_law` | Verifies P, V, n, T consistency |
| `deep_causality_core::CausalEffectPropagationProcess` | Monadic effect propagation |

---

## Thermodynamic Verification

The simulation demonstrates key thermodynamic principles:

1. **Entropy Conservation**: After completing the cycle, entropy returns to its initial value (S=0), confirming reversibility
2. **State Cycle Closure**: The system returns exactly to its initial state (P, V, T)
3. **Efficiency Bound**: Net work output (~1153 J) is consistent with the 40% Carnot limit
4. **Energy Conservation**: Work equals net heat transfer ($W = Q_{in} - Q_{out}$)
