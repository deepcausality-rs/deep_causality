# Diving Decompression Planner

SCUBA diving decompression sickness prevention using Bühlmann ZH-L16C algorithm with CNS oxygen toxicity tracking.

## Quick Start

```bash
cargo run -p medicine_examples --example diving_decompression
```

## Scientific Background

### Decompression Sickness (DCS)

Occurs when dissolved nitrogen forms bubbles during rapid ascent. The Bühlmann model simulates 16 tissue compartments with different absorption rates (half-times: 5–635 minutes).

### CNS Oxygen Toxicity

At depth, oxygen partial pressure (ppO2) increases. NOAA limits track exposure:
- ppO2 > 1.0 bar: CNS% accumulates
- 80% CNS: Warning threshold
- 100% CNS: Seizure risk

## APIs Demonstrated

| API | Purpose |
|-----|---------|
| `CausalTensor<f64>` | 16-element tissue tension vector |
| `CausalEffectPropagationProcess` | Monadic dive phase chaining |
| `Pressure`, `Length` | Type-safe physics quantities |

## Key Formulas

**Schreiner Equation** (tissue loading):
```
P_t = P_i + (P_0 - P_i) × e^(-kt)
```

**Ascent Ceiling**:
```
P_ceiling = (P_tissue - a) × b
```

**CNS Accumulation**:
```
CNS% = Σ (time_at_ppO2 / max_time_for_ppO2) × 100
```

## Output

Generates dive table for 10m–50m depths showing:
- No Decompression Limit (NDL)
- ppO2 and CNS%
- Safety stops and deco stops

## Adaptation Ideas

- Add Nitrox support (adjust F_O2)
- Multi-dive residual nitrogen tracking
