# Protein Folding: Generalized Master Equation

This example simulates protein folding dynamics using the Generalized Master Equation (GME) with memory kernels for non-Markovian behavior.

## How to Run

```bash
cargo run -p medicine_examples --example protein_folding
```

---

## Engineering Value

Protein folding simulation is crucial for:
- **Drug Discovery**: Understanding protein misfolding diseases (Alzheimer's, Parkinson's)
- **Bioengineering**: Designing proteins with specific functions
- **Computational Biology**: Predicting 3D structure from sequence

The GME approach captures **memory effects** - proteins "remember" recent conformations.

---

## Physics Background

### Generalized Master Equation

Standard Markov: `P(t+Δt) = T · P(t)`

GME adds memory: `P(t+Δt) = T · P(t) + Σ K(τ) · P(t-τ)`

Where:
- **T**: Markov transition matrix (instantaneous transitions)
- **K(τ)**: Memory kernel (history-dependent corrections)
- **P(t)**: Probability distribution over conformational states

### Conformational States

```
[0] Unfolded → [1] Intermediate 1 → [2] Intermediate 2 → [3] Native (Folded)
```

---

## Causal Chain

```text
[Init]    100% Unfolded state
             ↓
[t=1..15] Apply GME step:
             │
             ├─ Markov: T · P(t)
             │
             └─ Memory: Σ K_k · P(t-k)
             ↓
[Result]  ~65% Native state → Protein folded!
```

---

## Output Interpretation

```
[t= 1] Distribution:
  Unfolded:  70.00% ██████████████
  Intermed1:  30.00% ██████
```

Visual bars show the population flowing from Unfolded → Native over time.

---

## Adapting This Example

1. **More states**: Model detailed folding pathway with more intermediates
2. **Different kinetics**: Adjust transition matrix for fast/slow folders
3. **Memory effects**: Tune memory kernels for different physical regimes
4. **Temperature dependence**: Add Arrhenius-type rate modifications

---

## Key APIs Used

- `generalized_master_equation()` - Non-Markovian dynamics
- `Probability` - Type-safe probability values [0,1]
- `CausalTensor` - Transition and memory kernel matrices
