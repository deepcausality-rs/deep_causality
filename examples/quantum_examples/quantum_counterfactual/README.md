# A Quantum Counterfactual: What the Decoder Was Reading

This example runs the three-qubit repetition code, which recovers a state without looking at it,
then asks a counterfactual question: the decoder acted on a syndrome, so what would it have done had
the syndrome said something else?

```bash
cargo run -p quantum_examples --example quantum_counterfactual
```

## The problem

A qubit on its own cannot be error-checked. Every measurement that would reveal whether it flipped
also reveals `α` and `β` and collapses the protected superposition.

The repetition code spreads one logical qubit across three physical ones:

```text
α|0⟩ + β|1⟩   →   α|000⟩ + β|111⟩
```

and then ask only about the **relationship** between them. `⟨Z₀Z₁⟩` answers whether two qubits
disagree, and its value is exactly `+1` or `−1` whatever `α` and `β` are. It reports an error without
reporting the state.

Two such checks distinguish all four cases:

| `⟨Z₀Z₁⟩` | `⟨Z₁Z₂⟩` | Diagnosis |
|---|---|---|
| agree | agree | no error |
| differ | agree | qubit 0 flipped |
| differ | differ | qubit 1 flipped |
| agree | differ | qubit 2 flipped |

Applying `X` to the named qubit undoes the flip exactly.

## The counterfactual

The syndrome *causes* the recovery. Nothing else reaches the decoder, so a different syndrome would
have produced a different action.

`CausalFlow::alternate_value_if` is Pearl's do-operator. It replaces the measured value with the
value an intervention forces, and records the substitution. The two pipelines run the same steps on
the same corrupted register and differ by one line:

```text
observed        measure →                      recover
counterfactual  measure → do(syndrome := q0) → recover
```

Both recoveries run. One restores the state and one destroys it; since everything else about the
two runs is identical, the substituted value accounts for the difference.

## What the code demonstrates

| Operation | Carries |
|---|---|
| `bind` | each stage, threading the register through the state channel |
| `fold` | amplitudes → a parity, and amplitudes → a fidelity |
| `alternate_value_if` | the intervention, as the do-operator |

The register rides the **state** channel and the syndrome rides the **value** channel. The split
mirrors the code: the decoder sees the value channel and never the state, so error correction works
without measuring the protected qubit.

## Output

```text
Encoding
  logical      0.600|0> + 0.800|1>
  encoded      0.600|000> + 0.800|111>
  norm^2       1.000000000

The error: an X gate on qubit 1
  before       0.600|000> + 0.800|111>
  after        0.600|010> + 0.800|101>
  norm^2       1.000000000

The syndrome: two parity measurements
  check        clean      corrupted
  <Z0 Z1>     +1.000       -1.000
  <Z1 Z2>     +1.000       -1.000

Recovery from observed
  syndrome     (differ, differ)  ->  qubit 1
  state        0.600|000> + 0.800|111>
  fidelity     1.000000000

Recovery from do(syndrome := qubit 0)
  syndrome     (differ, agree )  ->  qubit 0
  state        0.600|110> + 0.800|001>
  fidelity     0.000000000

Outcome
  fidelity to the protected state
    observed syndrome        1.000000000
    intervened syndrome      0.000000000
    no correction at all     0.000000000
```

Three details in that output matter.

**The parities are `±1`, never `0.6` or `0.8`.** The code rests on this property. Every basis state
in the support of a code word agrees on the parity of any two qubits, so the weights sum to one and
the amplitudes cancel out of the answer.

**The error is a permutation.** The same eight numbers come out in a different order, because `X` on
qubit `k` moves the amplitude at index `i` to index `i XOR (1 << k)`. Nothing is substituted and no
amplitude is invented.

**The miscorrected state has norm 1.** Fidelity decides whether a recovery worked. A check that
some amplitude came out large would pass `0.600|110⟩ + 0.800|001⟩`, a normalised state orthogonal to
the protected one.

## Precision is a parameter

```rust
pub type FloatType = Float106;
```

Every constant is declared at that type through `const_scalar_from_int!`, and `3/5` and `4/5` are
derived from integers at the working precision. The alias is `Float106` so that a hard-coded `f64`
fails to compile; with the alias at `f64` it would go unnoticed. All four scalars run and agree
exactly, because the gates are permutations and the two fidelities are exactly `1` and `0`.

## What this example covers

The example states a correction as a causal process over the library's types; precision as a
parameter and categorical composition follow from that form. The decoder acts on the syndrome and
nothing else, which makes both the correction and the counterfactual possible. Everything else stays
simple: one bit-flip error on one known qubit, no phase errors, measurement as reading an expectation
instead of sampling and collapsing, and a four-row lookup for the decoder.

A fault-tolerant treatment adds what this leaves out: the repetition code protects against bit flips
only, so a real code such as Shor's or a surface code is needed to catch phase errors too; syndrome
extraction needs its own ancilla qubits and is itself noisy; and errors arrive continuously rather
than once, so the syndrome is measured repeatedly and the decoder works over a history of them.

## How to grow the example toward a fault-tolerant simulation

Each step keeps the structure already here.

- **Phase errors.** Add `Z` alongside `X` and move to the nine-qubit Shor code. `N_QUBITS` is a
  constant and `bit_flip` already generalises to any single-qubit Pauli.
- **Ancilla-based syndrome extraction.** Measure the parity with a helper qubit and a pair of
  `CNOT`s, as a device does, instead of reading the expectation directly.
- **Repeated rounds.** Wrap the measure-and-recover pair in `CausalFlow::iterate_n`, so the decoder
  works over a sequence of syndromes and can spot a syndrome measurement that was itself wrong.
- **Noisy syndromes.** The intervention already shows what a misread syndrome does. Replace the
  forced value with a sampled one from `deep_causality_uncertain` and the same pipeline becomes a
  study of decoder failure rates.

## Files

| File | Holds |
|---|---|
| `main.rs` | the alias, the two pipelines, and the one line that separates them |
| `model.rs` | the encoding, the `X` gate, the parity measurements, the decoder, the fidelity |
| `utils_print.rs` | the presentation, and the only `lower` calls |

## Key APIs used

- `CausalFlow::bind` — monadic composition of the stages
- `CausalFlow::alternate_value_if` — the do-operator that forces the syndrome
- `CausalEffectPropagationProcess::with_state` — threading the register through the state channel
- `HilbertState` — the three-qubit register, as eight complex amplitudes
