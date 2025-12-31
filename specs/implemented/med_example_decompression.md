# Decompression Sickness Example Specification

## Overview

This specification describes a SCUBA diving decompression sickness prevention example that models safe ascending rates and mandatory decompression stops using DeepCausality. The example implements a simplified Bühlmann ZH-L16 decompression algorithm to calculate nitrogen tissue loading and safe ascent profiles for recreational dive depths (10m–50m).

---

## Scientific Background

### Decompression Sickness (DCS)

Decompression sickness occurs when dissolved inert gases (primarily nitrogen) form bubbles in the body during rapid pressure reduction (ascent). The governing physics involves:

1. **Henry's Law**: Gas solubility is proportional to ambient pressure
2. **Diffusion Kinetics**: Gas exchange between blood and tissues follows exponential saturation curves
3. **Supersaturation Limits**: Maximum tolerable pressure gradient before bubble formation

### Bühlmann ZH-L16 Algorithm

The Bühlmann model represents the body as 16 hypothetical tissue compartments, each with a unique:
- **Half-time (τ)**: Time for tissue to reach 50% saturation (5–640 minutes)
- **M-value**: Maximum tolerable supersaturation ratio

**Core Equation** (gas tension in tissue compartment):
```
P_t(t) = P_0 + (P_i - P_0) × (1 - 2^(-t/τ))
```
Where:
- `P_t(t)` = tissue nitrogen partial pressure at time t
- `P_0` = initial tissue tension
- `P_i` = inspired nitrogen partial pressure (depth-dependent)
- `τ` = compartment half-time

**Ascent Ceiling** (minimum safe ambient pressure):
```
P_amb_min = (P_t - a) × b
```
Where `a` and `b` are tissue-specific coefficients derived empirically.

### CNS Oxygen Toxicity

Central Nervous System (CNS) oxygen toxicity occurs when oxygen partial pressure (ppO2) exceeds safe thresholds. At depth, air breathing produces elevated ppO2 that must be tracked:

**ppO2 Calculation**:
```
ppO2 = P_amb × F_O2 = (1 + depth/10) × 0.21
```

For air at depth:
- 10m: ppO2 = 0.42 bar (safe)
- 30m: ppO2 = 0.84 bar (safe)
- 50m: ppO2 = 1.26 bar (caution zone)
- 66m: ppO2 = 1.60 bar (recreational limit)

**NOAA Exposure Limits** (single dive):

| ppO2 (bar) | Max Time (min) | Notes |
|------------|----------------|-------|
| 1.60       | 45             | Emergency/deco only |
| 1.50       | 120            | Technical diving |
| 1.40       | 150            | Recreational max |
| 1.30       | 180            | Standard air limit |
| 1.20       | 210            | Conservative |
| 1.10       | 240            | Extended exposure |
| 1.00       | 300            | Shallow diving |

**CNS% Calculation** (oxygen clock):
```
CNS% = Σ (time_at_ppO2 / max_time_for_ppO2) × 100
```

- **Warning threshold**: 80% CNS
- **Critical threshold**: 100% CNS (risk of seizure)
- **Surface half-life**: 90 minutes (CNS% reduces by 50% every 90 min)

---

## Tensor vs Topology: Implementation Analysis

### Option A: Tensor-Based Implementation

**Approach**: Use `CausalTensor<f64>` to represent tissue compartments as a 1D tensor of 16 elements, where each element tracks nitrogen partial pressure.

**Structure**:
```rust
// Tissue tensions: [16] tensor for 16 compartments
let tissue_tensions: CausalTensor<f64> = CausalTensor::new(vec![...], vec![16])?;

// Half-times: [16] tensor
let half_times: CausalTensor<f64> = CausalTensor::new(HALF_TIMES, vec![16])?;
```

**Pros**:
- ✅ Simple, direct mapping: 1 tensor element = 1 tissue compartment
- ✅ Efficient vectorized arithmetic (element-wise operations)
- ✅ Low overhead, minimal memory allocation
- ✅ Matches the parallel compartment assumption of Bühlmann model
- ✅ Easy to extend to multi-dive tracking (2D tensor: [dives × compartments])

**Cons**:
- ❌ No inherent topological structure
- ❌ Cannot model spatial diffusion gradients within tissues
- ❌ No natural representation of compartment connectivity

---

### Option B: Topology-Based Implementation

**Approach**: Use `Manifold<f64>` with `SimplicialComplex` where vertices represent tissue compartments and edges represent diffusion pathways.

**Structure**:
```rust
// 16 vertices (tissue compartments) with connectivity
let complex = SimplicialComplexBuilder::new()
    .with_vertices(16)
    .with_edges(blood_tissue_connections)
    .build()?;

let tissue_manifold = Manifold::new(complex, tissue_data, cursor)?;

// Use Laplacian for diffusion modeling
let diffusion_rate = tissue_manifold.laplacian(0);
```

**Pros**:
- ✅ Rich structure for modeling inter-tissue gas exchange pathways
- ✅ Natural use of Laplacian operator for diffusion equations
- ✅ Can model blood-tissue barrier permeabilities as edge weights
- ✅ Extensible to true physiological tissue networks

**Cons**:
- ❌ Bühlmann model assumes **parallel compartments** (no inter-tissue exchange)
- ❌ Significant overhead for simple parallel compartment model
- ❌ Laplacian not needed when compartments are independent
- ❌ More complex setup with diminishing returns for this use case

---

### Recommendation: **Tensor-Based Implementation**

**Rationale**: The Bühlmann decompression model explicitly assumes parallel, independent tissue compartments with no inter-compartment gas diffusion. This maps directly to a 1D tensor where:
- Each element represents one tissue compartment
- Element-wise exponential operations compute gas loading
- No graph structure is required

The topology approach would be appropriate if modeling:
- Perfusion-diffusion coupled models
- Spatial gas distribution within tissues
- Countercurrent blood flow effects

For the standard Bühlmann algorithm, tensor is both **more accurate** (matches model assumptions) and **simpler** (no unnecessary topological structures).

> **Decision**: Use `CausalTensor<f64>` for tissue compartment tensions with `CausalEffectPropagationProcess` for monadic time evolution.

---

## Implementation Design

### File Structure

```
examples/medicine_examples/
├── decompression/
│   ├── main.rs            # Entry point and simulation loop
│   └── README.md          # Documentation
```

### Core Data Types

```rust
/// Represents a diver's physiological state
#[derive(Debug, Clone, Default)]
struct DiverState {
    /// Current depth in meters
    depth: Length,
    /// Elapsed bottom time in minutes
    bottom_time: f64,
    /// Nitrogen tissue tensions [16 compartments] in bar
    tissue_tensions: CausalTensor<f64>,
    /// CNS oxygen toxicity percentage (0-100+)
    cns_percent: f64,
    /// Current ascent phase description
    phase: String,
}

/// Dive profile parameters
struct DiveProfile {
    max_depth: Length,          // Target depth (10-50m)
    bottom_time: f64,           // Time at depth (minutes)
    descent_rate: f64,          // meters/min (standard: 18-30 m/min)
    ascent_rate: f64,           // meters/min (safe: 9-18 m/min)
    cns_percent: f64,           // Final CNS oxygen toxicity %
}

/// Decompression stop
struct DecoStop {
    depth: Length,              // Stop depth
    duration: f64,              // Minutes at stop
}
```

### Constants (Bühlmann ZH-L16C)

```rust
/// Tissue compartment half-times (minutes) for N2
const HALF_TIMES: [f64; 16] = [
    5.0, 8.0, 12.5, 18.5, 27.0, 38.3, 54.3, 77.0,
    109.0, 146.0, 187.0, 239.0, 305.0, 390.0, 498.0, 635.0
];

/// M-value 'a' coefficients (bar)
const A_COEFFICIENTS: [f64; 16] = [
    1.1696, 1.0000, 0.8618, 0.7562, 0.6200, 0.5043, 0.4410, 0.4000,
    0.3750, 0.3500, 0.3295, 0.3065, 0.2835, 0.2610, 0.2480, 0.2327
];

/// M-value 'b' coefficients (dimensionless)
const B_COEFFICIENTS: [f64; 16] = [
    0.5578, 0.6514, 0.7222, 0.7825, 0.8126, 0.8434, 0.8693, 0.8910,
    0.9092, 0.9222, 0.9319, 0.9403, 0.9477, 0.9544, 0.9602, 0.9653
];

/// Surface nitrogen partial pressure (bar) - ~79% of 1 atm
const SURFACE_N2_PP: f64 = 0.79;

/// Standard gradient factor settings (conservative recreational diving)
const GF_LOW: f64 = 0.30;   // 30% at deepest stop
const GF_HIGH: f64 = 0.85;  // 85% at surface

/// Oxygen fraction in air
const F_O2: f64 = 0.21;

/// NOAA CNS oxygen toxicity limits: (ppO2 threshold, max_time_minutes)
const CNS_LIMITS: [(f64, f64); 7] = [
    (1.60, 45.0),
    (1.50, 120.0),
    (1.40, 150.0),
    (1.30, 180.0),
    (1.20, 210.0),
    (1.10, 240.0),
    (1.00, 300.0),
];

/// CNS surface interval half-life (minutes)
const CNS_HALF_LIFE: f64 = 90.0;
```

### Core Physics Functions

```rust
/// Calculates ambient pressure at depth
/// P = P_surface + ρgh = 1.0 + depth/10 (bar, for seawater)
fn ambient_pressure(depth: &Length) -> Pressure {
    // 1 bar surface + 0.1 bar per meter depth
    Pressure::new(1.0 + depth.value() / 10.0).unwrap()
}

/// Calculates inspired nitrogen partial pressure at depth
/// P_i = (P_amb - P_water_vapor) × F_N2
fn inspired_n2_pp(depth: &Length) -> f64 {
    let p_amb = ambient_pressure(depth).value();
    let p_water_vapor = 0.0627; // bar at 37°C
    let f_n2 = 0.79;            // fraction N2 in air
    (p_amb - p_water_vapor) * f_n2
}

/// Schreiner equation: tissue gas loading over time
/// P_t = P_i + (P_0 - P_i) × e^(-kt)
/// where k = ln(2) / half_time
fn tissue_loading(
    p_initial: f64,
    p_inspired: f64,
    time_minutes: f64,
    half_time: f64,
) -> f64 {
    let k = (2.0_f64).ln() / half_time;
    p_inspired + (p_initial - p_inspired) * (-k * time_minutes).exp()
}

/// Calculates ascent ceiling (minimum safe depth) for a tissue
/// P_ceiling = (P_tissue - a) × b
fn tissue_ceiling(tissue_tension: f64, a: f64, b: f64, gf: f64) -> f64 {
    let m_value = tissue_tension / b + a;
    let allowed_pp = tissue_tension - gf * (tissue_tension - m_value);
    // Convert ceiling pressure to depth, clamp to surface
    ((allowed_pp - 1.0) * 10.0).max(0.0)
}

/// Calculates oxygen partial pressure at depth
fn oxygen_pp(depth: &Length) -> f64 {
    ambient_pressure(depth).value() * F_O2
}

/// Finds the maximum exposure time for a given ppO2 from NOAA table
/// Uses linear interpolation between table entries
fn max_cns_time(pp_o2: f64) -> f64 {
    if pp_o2 < 1.0 {
        return f64::INFINITY; // No CNS concern below 1.0 bar
    }
    
    // Find bounding entries and interpolate
    for i in 0..CNS_LIMITS.len() - 1 {
        let (pp_high, time_high) = CNS_LIMITS[i];
        let (pp_low, time_low) = CNS_LIMITS[i + 1];
        
        if pp_o2 >= pp_low && pp_o2 <= pp_high {
            // Linear interpolation
            let ratio = (pp_o2 - pp_low) / (pp_high - pp_low);
            return time_low + ratio * (time_high - time_low);
        }
    }
    
    // Above 1.6 bar - use strictest limit
    if pp_o2 > 1.6 {
        return 45.0 * (1.6 / pp_o2); // Extrapolate conservatively
    }
    
    300.0 // Default to safest limit
}

/// Calculates CNS% accumulation for time spent at a depth
fn cns_accumulation(depth: &Length, time_minutes: f64) -> f64 {
    let pp_o2 = oxygen_pp(depth);
    let max_time = max_cns_time(pp_o2);
    
    if max_time.is_infinite() {
        return 0.0;
    }
    
    (time_minutes / max_time) * 100.0
}

/// Calculates CNS% decay during surface interval
fn cns_decay(current_cns: f64, surface_time_minutes: f64) -> f64 {
    let decay_factor = 0.5_f64.powf(surface_time_minutes / CNS_HALF_LIFE);
    current_cns * decay_factor
}
```

### Simulation Flow

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SCUBA Decompression Planner ===\n");

    // Process dive profiles from 10m to 50m in 5m intervals
    let depths = [10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 40.0, 45.0, 50.0];

    for max_depth in depths {
        let profile = calculate_safe_dive_profile(max_depth)?;
        print_dive_table_entry(&profile);
    }

    // Detailed simulation for 30m dive
    println!("\n=== Detailed 30m Dive Simulation ===\n");
    simulate_dive(30.0, 20.0)?; // 30m for 20 minutes

    Ok(())
}
```

### Dive Profile Calculation

```rust
fn calculate_safe_dive_profile(max_depth: f64) -> Result<DiveProfile, PhysicsError> {
    let depth = Length::new(max_depth)?;

    // Standard recreational bottom times (NDL - No Decompression Limit)
    // Based on conservative gradient factors
    let bottom_time = estimate_ndl(max_depth);

    // Safe ascent rate: 9 m/min (PADI standard)
    let ascent_rate = 9.0;

    // Calculate decompression stops if needed
    let deco_stops = calculate_deco_stops(max_depth, bottom_time)?;

    Ok(DiveProfile {
        max_depth: depth,
        bottom_time,
        descent_rate: 18.0,
        ascent_rate,
        deco_stops,
    })
}
```

### Monadic Time Evolution

```rust
fn simulate_dive(max_depth: f64, bottom_time: f64) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tissue tensions at surface equilibrium
    let initial_tensions = vec![SURFACE_N2_PP; 16];
    let tissue_tensions = CausalTensor::new(initial_tensions, vec![16])?;

    let initial_state = DiverState {
        depth: Length::new(0.0)?,
        bottom_time: 0.0,
        tissue_tensions,
        phase: "Surface".to_string(),
    };

    // Phase 1: Descent
    let process = CausalEffectPropagationProcess::with_state(
        CausalEffectPropagationProcess::pure(()),
        initial_state,
        None,
    )
    .bind(|_, state, _| {
        // Compute tissue loading during descent
        let descent_time = max_depth / 18.0; // 18 m/min
        let new_tensions = compute_descent_loading(&state, max_depth, descent_time);
        
        CausalEffectPropagationProcess::pure(DiverState {
            depth: Length::new(max_depth).unwrap(),
            tissue_tensions: new_tensions,
            phase: "At Depth".to_string(),
            ..state
        })
    })
    // Phase 2: Bottom time
    .bind(|prev, _, _| {
        let state = prev.into_value().unwrap();
        let new_tensions = compute_bottom_loading(&state, bottom_time);
        
        CausalEffectPropagationProcess::pure(DiverState {
            bottom_time,
            tissue_tensions: new_tensions,
            phase: "Bottom Phase Complete".to_string(),
            ..state
        })
    })
    // Phase 3: Ascent with stops
    .bind(|prev, _, _| {
        let state = prev.into_value().unwrap();
        execute_ascent(&state)
    });

    // Print results
    print_tissue_tensions(&process.state().tissue_tensions);
    
    Ok(())
}
```

---

## Dive Table Output

The example will generate a standard dive table showing:

| Depth | ppO2 (bar) | NDL (min) | Ascent Time | CNS% | Safety Stop | Deco Stop(s) |
|-------|------------|-----------|-------------|------|-------------|--------------|
| 10m   | 0.42       | 219       | 1.1 min     | 0%   | None        | None         |
| 15m   | 0.53       | 98        | 1.7 min     | 0%   | 3min @ 5m   | None         |
| 20m   | 0.63       | 56        | 2.2 min     | 0%   | 3min @ 5m   | None         |
| 25m   | 0.74       | 35        | 2.8 min     | 0%   | 3min @ 5m   | None         |
| 30m   | 0.84       | 20        | 3.3 min     | 0%   | 3min @ 5m   | None         |
| 35m   | 0.95       | 14        | 3.9 min     | 0%   | 3min @ 5m   | None         |
| 40m   | 1.05       | 9         | 4.4 min     | 3%   | 3min @ 5m   | 2min @ 6m    |
| 45m   | 1.16       | 8         | 5.0 min     | 4%   | 3min @ 5m   | 3min @ 6m    |
| 50m   | 1.26       | 6         | 5.6 min     | 5%   | 3min @ 5m   | 5min @ 6m    |

> **Notes**:
> - **NDL** = No Decompression Limit (maximum time at depth without mandatory stops)
> - **Deco Stop** format: `Xmin @ Ym` = stop at Y meters depth for X minutes during ascent
>   - Example: `5min @ 6m` means pause at 6m for 5 minutes before continuing to surface
>   - Multiple stops are cumulative (e.g., `3min @ 9m` then `5min @ 6m` then `3min @ 5m`)
> - ppO2 calculated as (1 + depth/10) × 0.21 for air
> - CNS% only accumulates when ppO2 > 1.0 bar (depths > 38m on air)
> - At 50m, ppO2 = 1.26 bar, well within safe recreational limits (< 1.4 bar)

---

## Expected Console Output

```
=== SCUBA Decompression Planner ===

Tissue Compartment Half-Times (min): [5, 8, 12.5, ... , 635]
Gradient Factors: GF_low=30%, GF_high=85%

=== Dive Table ===
┌───────┬───────────┬─────────────┬─────────────┬──────────────┐
│ Depth │ NDL (min) │ Ascent Time │ Safety Stop │ Deco Stop(s) │
├───────┼───────────┼─────────────┼─────────────┼──────────────┤
│  10m  │    219    │   1.1 min   │    None     │     None     │
│  15m  │     98    │   1.7 min   │  3min @ 5m  │     None     │
│  ...  │    ...    │     ...     │     ...     │     ...      │
│  50m  │      6    │   5.6 min   │  3min @ 5m  │  5min @ 6m   │
└───────┴───────────┴─────────────┴─────────────┴──────────────┘

=== Detailed 30m Dive Simulation ===

[DESCENT] 0m → 30m in 1.7 min (18 m/min)
  Tissue Loading: [0.79, 0.79, 0.79, ...] → [1.02, 0.98, 0.94, ...]

[BOTTOM] 20 min at 30m
  Tissue Loading: [1.02, 0.98, 0.94, ...] → [2.45, 2.12, 1.78, ...]
  Controlling Compartment: #4 (τ=18.5 min)
  Ascent Ceiling: 3.2m

[ASCENT] 30m → 5m at 9 m/min
  [25m] Ceiling: 2.8m ✓
  [20m] Ceiling: 2.5m ✓
  [15m] Ceiling: 2.1m ✓
  [10m] Ceiling: 1.8m ✓
  [ 5m] Safety Stop: 3 min

[SURFACE] Final tissue tensions:
  Compartment #1 (τ=5):    1.42 bar (82% saturated)
  Compartment #4 (τ=18.5): 1.89 bar (94% saturated) ← Controlling
  Compartment #16 (τ=635): 0.81 bar (< 5% loaded)

[O2 TOXICITY] CNS Oxygen Status:
  ppO2 at 30m: 0.84 bar (< 1.0 bar threshold)
  CNS% accumulated: 0% (no accumulation below ppO2 1.0)
  Status: ✓ SAFE (well below 80% warning threshold)

[COMPLETE] Dive completed safely. Total runtime: 26.7 min
```

---

## Verification Plan

### Unit Tests
```bash
# Run all medicine example tests
cargo test -p medicine_examples --lib
```

### Integration Test
```bash
# Run the decompression example
cargo run -p medicine_examples --example decompression
```

### Manual Verification
1. Compare NDL values with PADI/SSI recreational dive tables
2. Verify tissue loading follows exponential saturation curves
3. Confirm ascent ceilings are respected
4. Validate safety stop recommendations

---

## Cargo.toml Entry

```toml
[[example]]
name = "decompression"
path = "decompression/main.rs"
```

---

## Future Extensions

1. **Multi-dive profiles**: Track residual nitrogen across surface intervals (CNS% decays with 90-min half-life)
2. **Tissue visualization**: Real-time compartment saturation and CNS% graphs
3. **HDT (Hyperbaric Data Table)**: Import standard dive planning tables
4. **Deep stops**: Implement Pyle stops for technical diving

---

## Summary

| Aspect | Choice | Rationale |
|--------|--------|-----------|
| **Data Structure** | `CausalTensor<f64>` | Matches parallel compartment model |
| **Time Evolution** | `CausalEffectPropagationProcess` | Monadic chaining of dive phases |
| **Physics Base** | Bühlmann ZH-L16C | Industry-standard algorithm |
| **O2 Toxicity** | NOAA CNS limits | Standard ppO2 exposure tracking |
| **Target Depths** | 10m–50m @ 5m intervals | Recreational diving range |
| **Conservatism** | GF 30/85, CNS < 80% | Conservative recreational factors |

This implementation demonstrates DeepCausality's capability to model time-dependent diffusion kinetics with exponential saturation curves—a pattern applicable to pharmacokinetics, environmental modeling, and other multi-compartment systems.
