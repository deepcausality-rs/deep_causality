# Generalized Master Equation Kernel Specification

**Status:** Draft
**Context:** Protein Folding, Non-Markovian Dynamics, Open Quantum Systems.

## 1. Overview
The `generalized_master_equation_kernel` implements the generalized master equation (GME) for systems with memory effects (non-Markovianity). Unlike the standard Master Equation ($\frac{d}{dt}P(t) = \mathcal{L}P(t)$), the GME integrates over the system's history using a memory kernel $\mathcal{K}(t - \tau)$.

## 2. Mathematical Definition
The GME governing the time evolution of the probability vector (or density matrix diagonal) $P(t)$ is:

$$ \frac{d}{dt} P(t) = \int_0^t \mathcal{K}(t - \tau) P(\tau) d\tau $$

In a discrete time-step simulation with step $\Delta t$, this can be approximated or combined with a Markovian term:

$$ P_{n+1} = T \cdot P_n + \sum_{k=0}^{m} \mathcal{K}_k \cdot P_{n-k} \cdot \Delta t $$

Where:
*   $T$: Markovian transition matrix (if present).
*   $\mathcal{K}_k$: Memory kernel tensor at lag $k$.
*   $m$: Memory depth (history length).

## 3. Implementation Details

### Required Types
*   **`Probability`**: New struct to be added in `deep_causality_physics/src/dynamics/quantities.rs`.
    *   Wraps `f64`.
    *   Validation: Range [0.0, 1.0].

### Function Signature
```rust
use crate::Probability;

pub fn generalized_master_equation_kernel(
    state: &[Probability],
    history: &[Vec<Probability>],
    markov_operator: Option<&CausalTensor<f64>>,
    memory_kernel: &[CausalTensor<f64>]
) -> Result<Vec<Probability>, PhysicsError>
```

### Logic
1.  **Validation:** Ensure `history` length matches `memory_kernel` length.
2.  **Markov Step:** If `markov_operator` is provided, compute $P_{initial} = T \times state$. Else $P_{initial} = 0$. Note: Tensor ops work on `f64`, so explicit `value()` extraction and wrapping required.
3.  **Memory Integration:**
    *   Iterate $k$ from $0$ to $m$.
    *   Compute correction: $\Delta P_k = \mathcal{K}_k \times history[k]$.
    *   Accumulate: $P_{new} = P_{initial} + \sum \Delta P_k$.
4.  **Output:** Wrap results in `Probability`.

## 4. Verification
*   **Markov Limit:** If `memory_kernel` is empty or zero, result matches standard Markov chain.
*   **Memory Effect:** Non-zero memory dampens/oscillates decay.

## 5. Integration

### Location
*   **File:** `deep_causality_physics/src/dynamics/estimation.rs`

### Wrapper
*   **File:** `deep_causality_physics/src/dynamics/wrappers.rs`

```rust
use deep_causality_core::{CausalityError, PropagatingEffect};
use crate::dynamics::estimation;
use crate::Probability;

pub fn generalized_master_equation(
    state: &[Probability],
    history: &[Vec<Probability>],
    markov_operator: Option<&CausalTensor<f64>>,
    memory_kernel: &[CausalTensor<f64>]
) -> PropagatingEffect<Vec<Probability>>
{
    match estimation::generalized_master_equation_kernel(state, history, markov_operator, memory_kernel) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
```

## 6. Test Strategy (100% Coverage)

### Unit Tests (`generalized_master_equation_kernel`)
1.  **Zero State:** Input zero state, empty history. Expect zero output.
2.  **Markov Limit:** Non-zero state, valid Markov operator, **empty** memory kernel. Expect result = $T \cdot P$.
3.  **Memory Only:** Zero Markov operator, non-zero history, valid memory kernel. Expect result = $\sum K_k \cdot P_{t-k}$.
4.  **Combined Dynamics:** Valid T, valid history, valid memory. Verify output equals manual calculation ($T \cdot P + \sum K \cdot Hist$).
5.  **Validation Error:** Mismatched history and kernel lengths. Expect `PhysicsError`.
6.  **Dimension Mismatch:** State dimension != Operator dimension. Expect `PhysicsError`.

### Wrapper Tests (`generalized_master_equation`)
7.  **Success Propagation:** Valid inputs. Verify `PropagatingEffect` contains correct value.
8.  **Error Propagation:** Invalid inputs (e.g., mismatch). Verify `PropagatingEffect` contains `CausalityError`.
