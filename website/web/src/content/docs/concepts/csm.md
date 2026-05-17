---
title: Causal State Machine
description: A registry of state-action pairs whose transitions are driven by causal evaluation rather than fixed thresholds.
section: concepts
order: 9
---

The Causal State Machine (CSM) is the connector between a Causaloid's verdict and an effect on the outside world. It lives in `deep_causality::types::csm_types` and is built around two ideas: a state is "active" when its Causaloid evaluates to an active effect; an action is a function that runs when its paired state is active.

## What it is

A CSM holds a thread-safe map of `(CausalState, CausalAction)` pairs:

```rust
pub struct CSM<I, O, C>
where
    I: Default + Clone,
    O: CsmEvaluable + Default + Debug + Clone,
    C: Clone,
{
    state_actions: Arc<RwLock<CSMMap<I, O, C>>>,
}
```

The constructor takes a slice of pairs (`csm/mod.rs`). The map sits behind `Arc<RwLock<...>>`, so the CSM is shareable across threads and can grow or shrink at runtime through `add_state`, `remove_state`, and `update_state`, each implemented in its own submodule.

## States and actions

A `CausalState<I, O, C>` is small. It carries:

- an integer `id`;
- a `version` for tracking iterations of the same logical state;
- a `PropagatingEffect<I>` of pre-bound data;
- the `Causaloid` that decides whether this state is active;
- an optional `UncertainParameter` for states whose Causaloid emits an uncertain effect.

A `CausalAction` is smaller still:

```rust
pub struct CausalAction {
    action: fn() -> Result<(), ActionError>,
    description: &'static str,
    version: usize,
}
```

`action.fire()` invokes the function pointer. The action surface is kept minimal on purpose: the conditional logic lives in the Causaloid, not in the action.

## Evaluation

Two entry points are exposed:

- `eval_single_state(id, data)` runs the Causaloid for one state against caller-supplied data.
- `eval_all_states()` walks every registered state and runs its Causaloid against the data already bound to the state.

In both cases the effect returned by the Causaloid is inspected:

```rust
let is_active = match &effect.value {
    EffectValue::Value(val) => val
        .is_active(state.uncertain_parameter().as_ref())
        .map_err(CsmError::Causal)?,
    // Other variants (RelayTo, Error, etc.) are inactive for action firing.
    _ => false,
};

if is_active {
    self.fire_action_with_ethos_check(state, action, effect)?;
}
```

`is_active` is delegated to the `CsmEvaluable` trait. A deterministic effect resolves to `true` or `false` directly; an uncertain effect runs its hypothesis test against the state's `UncertainParameter`. Anything that is not a value (relay, error, none) is treated as inactive and fires nothing.

## When to reach for it

A CSM fits whenever the question "should this fire?" has to follow a causal verdict rather than a fixed threshold. Common shapes:

- sensor monitoring with thresholds that depend on context, not on the latest reading alone;
- alert routing where the alert depends on a pattern across signals;
- control loops in which an action should run only when an uncertain condition clears a confidence bar.

The CSM does not own a scheduler. The host application decides when to call `eval_*`. That keeps it cheap to embed in async runtimes, batch jobs, or hard real-time loops.

## See also

- Example: [sensor monitoring with CSM](/examples/sensor-monitoring-csm/) (source: `examples/csm_examples/csm_basic/`).
- Concept: [Causaloid](/docs/concepts/causaloid/), the unit that supplies the verdict each state acts on.
- Concept: [Effect Ethos](/docs/concepts/effect-ethos/), for policy checks that should gate an action before it fires.
- Concept: [Uncertainty](/docs/concepts/uncertainty/), for the hypothesis tests applied when a state's Causaloid emits an uncertain effect.
