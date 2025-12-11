# Quantum Counterfactual: The Dead Qubit

This example demonstrates quantum error correction via "time-travel debugging" - rewinding quantum state history to fix bit-flip errors.

## How to Run

```bash
cargo run -p physics_examples --example quantum_counterfactual
```

---

## Engineering Value

Quantum error correction is essential for:
- **Fault-tolerant Quantum Computing**: Qubits are fragile and need protection
- **Quantum Memory**: Long-term storage requires error mitigation
- **Debugging Quantum Algorithms**: Understanding where errors occur

This example shows how monadic state threading enables "debugging" by inspecting and rewinding history.

---

## Causal Chain

```text
[t=1] Apply Gate      → Simulate bit-flip error (|0⟩ → |1⟩)
         ↓
[t=2] Measure Syndrome → Detect P(|1⟩) > 0.9 → ERROR!
         ↓
[t=3] Rewind History   → Pop bad state from history
         ↓
[t=4] Apply Correction → Apply X gate to restore |0⟩
         ↓
[VERIFY] Final State   → P(|0⟩) = 0.98 → SUCCESS
```

---

## Key Concepts

### History-Aware Computation

The `CausalEffectPropagationProcess` carries a `QuantumHistory` state through the chain:

```rust
struct QuantumHistory {
    states: Vec<HilbertState>,  // History of quantum states
}
```

### Error Detection

Syndrome measurement checks if the qubit has flipped:
```rust
let prob_1 = current_state.as_inner().data()[1].norm_sqr();
if prob_1 > 0.9 {
    // Error detected!
}
```

### Counterfactual Correction

"Time travel" by popping the corrupted state and applying correction:
```rust
hist.states.pop();  // Rewind
hist.states.push(corrected_state);  // Apply fix
```

---

## Adapting This Example

1. **Multi-qubit systems**: Extend `HilbertState` to more dimensions
2. **Different error types**: Simulate phase-flip or depolarizing errors
3. **Error correction codes**: Implement Shor, Steane, or surface codes
4. **Continuous monitoring**: Add periodic syndrome checks

---

## Key APIs Used

- `CausalEffectPropagationProcess::with_state()` - Thread state through computation
- `HilbertState` - Quantum state vector with complex amplitudes
- `.bind()` - Monadic composition of quantum operations
