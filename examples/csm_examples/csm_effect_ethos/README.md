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
--- Effect Ethos Example ---

=== Part 1: CSM Evaluation (no ethos check) ===
Alert! High temperature detected!
CSM: Action triggered (temperature threshold exceeded)

=== Part 2: EffectEthos Deontic Reasoning ===
Proposed action: high_temp_alert
Ethos verdict outcome: Impermissible
Justification:
  - Norm #1: action 'high_temp_alert' is Impermissible

>>> Action is FORBIDDEN by the ethos!
The ethos prevents triggering this alert based on deontic rules.

--- Example Complete ---

Key insight: EffectEthos provides deontic reasoning (what SHOULD happen)
separate from causal reasoning (what WILL happen given causes).
```

The CSM fires the alert, and the `EffectEthos` rules the same action impermissible. The two checks run
side by side: the example does not gate the CSM on the ethos verdict.
