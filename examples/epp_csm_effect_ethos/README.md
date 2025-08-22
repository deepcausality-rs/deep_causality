# Example: Causal State Machine with Effect Ethos

This example demonstrates the integration of a **Causal State Machine (CSM)** with an **Effect Ethos** to add a layer of
deontic (normative) reasoning to a reactive system.

It models a simple temperature monitoring system that triggers an alert when the temperature exceeds a threshold.
However, the action of sending the alert is subject to approval by the `EffectEthos`, which evaluates it against a set
of predefined norms.

## How it Works

The example is structured to highlight the difference between a purely reactive system and one governed by deontic
rules.

### 1. The Components

* **`CausalState`**: A state that becomes active when a specific condition is met. Here, it checks if the `temperature`
  data value is greater than `50.0`.
* **`CausalAction`**: An action that is triggered when the corresponding state is active. In this case, it's a simple
  function that prints an alert to the console.
* **`CausalStateMachine (CSM)`**: Manages the link between states and actions. When a state evaluates to `true`, the CSM
  triggers its associated action.
* **`EffectEthos`**: A deontic reasoning engine that contains a set of norms (`Teloids`). It evaluates a
  `ProposedAction` to determine if it is permissible, obligatory, or impermissible.
* **`Teloid` (Norm)**: A single rule within the ethos. In this example, we define a norm that makes the
  `High temp alert` **Impermissible**. This simulates a scenario where, for instance, alerts might be suppressed to
  prevent spamming or if a manual override is in effect.

### 2. The Scenario

The `main` function executes the same logic twice to provide a clear comparison:

**A. Execution without `EffectEthos`**

1. The `CSM` is initialized with the high-temperature state and the alert action, but **without** the `EffectEthos`.
2. The `CSM` is evaluated with a temperature of `60.0`, which exceeds the threshold.
3. The `CausalState` becomes active.
4. The `CSM` immediately fires the `CausalAction`, and the alert "Alert! High temperature detected!" is printed.
5. The result is `Ok(())`, as the action was executed without restriction.

**B. Execution with `EffectEthos`**

1. The `CSM` is initialized with the state, the action, **and** the `EffectEthos` containing the impermissibility norm.
   A `TeloidTag` (`High temp alert`) is provided to link the evaluation to the correct norm.
2. The `CSM` is evaluated again with a temperature of `60.0`.
3. The `CausalState` becomes active.
4. Instead of firing the action directly, the `CSM` creates a `ProposedAction` and submits it to the `EffectEthos` for
   evaluation.
5. The `EffectEthos` finds the active `High temp alert` norm, which renders the action **Impermissible**.
6. Because the action is forbidden, the `CSM` does **not** fire the action. Instead, it returns a `CsmError::Forbidden`
   containing a detailed explanation of the verdict.

## How to Run

To run the example, use the following command from the root of the `deep_causality` project:

```bash
cargo run --release --bin example_effect_ethos
```

### Expected Output

```text
--- Running CSM without EffectEthos ---
Alert! High temperature detected!
Result without ethos: Ok(())

--- Running CSM with EffectEthos ---
Result with ethos: Err(Forbidden("The final verdict is Impermissible....
The outcome is Impermissible because at least one impermissible norm was active and undefeated, which has the highest precedence."))
```

This output clearly illustrates the core concept: the `EffectEthos` successfully intercepted and blocked an action that
would have otherwise been executed, providing a powerful mechanism for building safer and more robust systems.
