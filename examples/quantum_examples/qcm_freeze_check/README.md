# A Quantum Causal Model, and the Check That Decides Whether It Is One

This example checks whether a graph carrying quantum operators is a quantum causal model, and shows
where the library enforces that condition: at the freeze boundary.

```bash
cargo run -p quantum_examples --example qcm_freeze_check
```

## The problem

A quantum causal model describes a process as a graph whose nodes carry Choi–Jamiołkowski factors.
The factors that share a Hilbert leg have to **pairwise commute**.

The model's Markov condition requires those factors to be simultaneously assignable, and
non-commuting operators have no joint assignment. A graph that fails the condition is not a quantum
causal model.

## Freezing is where the check belongs

A causal graph in this library is **dynamic** while it is being built and **frozen** once it is ready
to run. At the freeze the structure is complete and nothing depends on it yet, so a structural check
there neither rejects a half-built graph nor finds the problem after a result has been used.

`freeze_quantum` runs the pairwise commutator checks at that boundary:

| Outcome | What happens |
|---|---|
| every shared-leg pair commutes | the graph freezes; the report names the pairs tested and the worst margin |
| some pair does not | the freeze **aborts**, the error names the pair, and the graph **rolls back to dynamic** |

A model that cannot be frozen is never left half-frozen. A failure costs the caller a rebuild, never
a result computed on a structure that does not hold.

## What the run does

Two models, differing in exactly one operator:

```text
sigma_z and diag(3, -1) on leg 0   commute        freezes, reports the margin
sigma_x and sigma_z     on leg 0   anticommute    aborts, names the pair, rolls back
```

Pauli `X` and `Z` anticommute, so `[σx, σz] = −2i σy` and its norm is as far from zero as a
single-qubit commutator gets. A pair that *nearly* commuted would test the tolerance instead of the
condition.

The commuting pair is `σz` with `diag(3, −1)`. Both are diagonal, and any two diagonal operators
commute exactly, so the margin is zero. A multiple of the identity would commute with everything
and demonstrate nothing about diagonality.

## Output

```text
[1] sigma_z and diag(3, -1) on leg 0
    expected: they commute
    froze cleanly
      pairs tested       1
      worst margin       0.000e0
      is_frozen()        true

[2] sigma_x and sigma_z on leg 0
    expected: they anticommute
    freeze aborted
      offending pair     nodes 0 and 1
      reason             ‖[ρ_0, ρ_1]‖_F exceeds the Q-TOL threshold (margin > 1)
      is_frozen()        false
```

The error names the pair, so a large model shows where to start looking.

## Precision is a parameter

```rust
pub type FloatType = Float106;
```

Every constant is declared at that type through `const_scalar_from_int!` and
`const_scalar_from_float!`. The alias is `Float106` so that a hard-coded `f64` fails to compile; with
the alias at `f64` it would go unnoticed.

All four scalars run and agree. The margin is either exactly zero or as large as a single-qubit
commutator can be, and a shorter mantissa changes neither.

`model.rs` writes out the nodes' predicate instead of taking it from the library's test helpers.
Those are fixed at `f64` and would pin the graph to one precision while the factors followed the
alias.

## What this example covers

The example states the validity condition as a step in the library's lifecycle: the condition is
structural, the freeze boundary settles structure, and a failed check leaves nothing behind.
Precision as a parameter follows from that form. Everything else stays simple: two nodes, one shared
leg, single-qubit factors given directly instead of derived from a channel, and one pairwise check.

A model of a real process adds what this leaves out: factors obtained from actual channels through
the Choi–Jamiołkowski isomorphism, several legs per node with overlapping supports, a tolerance
chosen from the numerical conditioning of those factors rather than left at its default, and the C₃
structural check that runs alongside this one.

## How to grow the example toward a real model

Each step keeps the structure already here.

- **Factors from channels.** Build the Choi matrix of an actual quantum channel instead of writing
  the operator down; the check and the freeze are unchanged.
- **More legs and more nodes.** `factors_on_shared_leg` declares one support per node. Declaring
  several turns one pairwise check into the full set the condition asks for.
- **A chosen tolerance.** `CommutatorTolerance::default()` is a starting point. Deriving it from the
  factors' norms suits a model with numerically obtained factors.
- **The `C₃` check alongside.** `freeze_quantum` already takes the structural argument this example
  passes empty, so adding a `C₃` relation exercises the other half of the freeze.

## Files

| File | Holds |
|---|---|
| `main.rs` | the alias and the two freeze scenarios |
| `model.rs` | the operators, the two-node graph, and the factor store |
| `constants.rs` | the numbers the operators are written with |
| `utils_print.rs` | the presentation, and the only `lower` calls |

## Key APIs used

- `freeze_quantum` — the freeze-boundary Markov commutativity check
- `ProcessFactors` / `FactorSupports` — the node-keyed factor store and its Hilbert-leg registry
- `CommutatorTolerance` — how far from commuting a pair may sit
- `QuantumErrorEnum::CommutatorNonZero` — the abort, naming the offending pair
