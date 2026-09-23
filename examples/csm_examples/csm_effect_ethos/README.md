# Example: Causal State Machine with Effect Ethos

This example pairs a **Causal State Machine (CSM)** with an **Effect Ethos**, which adds deontic (normative)
reasoning to a reactive system.

It models a temperature monitoring system that triggers an alert when the temperature exceeds a threshold. The
`EffectEthos` then evaluates the alert action against a set of predefined norms.

## How it Works

The example contrasts a purely reactive system with one governed by deontic rules.

### 1. The Components

* **`CausalState`**: A state that becomes active when its condition holds. Here, its `Causaloid` checks whether the
  input reading reaches the threshold `0.55`.
* **`CausalAction`**: An action that fires when the corresponding state is active. Here, a function that prints an
  alert to the console.
* **`CausalStateMachine (CSM)`**: Links states to actions. When a state evaluates to `true`, the CSM fires its
  associated action.
* **`EffectEthos`**: A deontic reasoning engine holding a set of norms (`Teloids`). It evaluates a `ProposedAction` and
  decides whether it is permissible, obligatory, or impermissible.
* **`Teloid` (Norm)**: A single rule within the ethos. The example defines a norm that makes the `high_temp_alert`
  action **Impermissible**, as when alerts are suppressed to prevent spamming or during a manual override.

### 2. The Scenario

The `main` function runs in two parts:

**A. CSM evaluation without the `EffectEthos`**

1. The `CSM` holds the high-temperature state and the alert action.
2. The `CSM` is evaluated with a reading of `0.6`, which exceeds the threshold.
3. The `CausalState` becomes active.
4. The `CSM` fires the `CausalAction`, which prints "Alert! High temperature detected!".

**B. Deontic evaluation with the `EffectEthos`**

1. The example builds a `ProposedAction` named `high_temp_alert`.
2. `EffectEthos::evaluate_action` checks it against the norms tagged `temperature`, using the shared context.
3. The active `high_temp_alert` norm renders the action **Impermissible**.
4. The example prints the verdict, the norms in its justification, and that the ethos forbids the alert.

## How to Run

From the root of the `deep_causality` project, run:

```bash
cargo run -p csm_examples --example csm_effect_ethos_example
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

The `EffectEthos` blocks an action that the causal model alone would have executed.
