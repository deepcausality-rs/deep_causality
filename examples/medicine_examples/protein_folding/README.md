# Protein Folding: Generalized Master Equation

This example simulates protein folding with the Generalized Master Equation (GME), whose memory kernels make the dynamics non-Markovian.

## How to Run

```bash
cargo run -p medicine_examples --example protein_folding
```

---

## Engineering Value

Protein folding simulation serves:
- **Drug discovery**: misfolding diseases (Alzheimer's, Parkinson's)
- **Bioengineering**: proteins designed for specific functions
- **Computational biology**: 3D structure predicted from sequence

The GME captures **memory effects**: proteins "remember" recent conformations.

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

The bars show the population flowing from Unfolded to Native over time.

---

## Adapting This Example

1. **More states**: add intermediates to model a detailed folding pathway
2. **Different kinetics**: adjust the transition matrix for fast or slow folders
3. **Memory effects**: tune the memory kernels for other physical regimes
4. **Temperature dependence**: add Arrhenius-type rate modifications

---

## Key APIs Used

- `generalized_master_equation()`: non-Markovian dynamics
- `Probability`: type-safe probability values in [0,1]
- `CausalTensor`: transition and memory kernel matrices
