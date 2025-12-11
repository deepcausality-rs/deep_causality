# DeepCausality Ethos: The Programmable Deontic Layer

`deep_causality_ethos` provides the **Effect Ethos**, a programmable reasoning engine for verifying whether proposed actions align with safety, mission, and compliance norms.

It implements a **Defeasible Deontic Logic** system, meaning it handles rules about obligations and permissions that can conflict (and resolves those conflicts deterministically).

---

## 🏗️ Core Concept: The Teloid

The fundamental unit of the system is the **Teloid** (from Greek *telos*, meaning goal/end). A Teloid represents a **Norm** or **Rule** that governs behavior.

### Structure of a Teloid

| Component | Description |
|-----------|-------------|
| **Predicate** | A function determining *when* this norm applies (given Context + Action). |
| **Modality** | What the norm dictates: `Obligatory`, `Impermissible`, or `Optional(cost)`. |
| **Action ID** | The specific action this norm regulates. |
| **Heuristics** | `Specificity` and `Priority` scores used to resolve conflicts. |

```rust
// Logical equivalent of:
// "IF action is 'fire_laser' AND target is 'civilian' THEN status is 'Impermissible'"
let no_civilian_harm = Teloid::new_deterministic(
    1,
    "fire_laser".to_string(),
    |ctx, action| ctx.target_is_civilian(), 
    TeloidModal::Impermissible,
    ...
);
```

---

## 🧠 The Effect Ethos Engine

The `EffectEthos` struct is the reasoning engine. It orchestrates the verification process:

1.  **Input**: Takes a `Context` (state of the world) and a `ProposedAction`.
2.  **Activation**: Scans the `TeloidStore` to find all norms where the **Predicate** evaluates to `true`.
3.  **Conflict Resolution**: If active norms disagree (e.g., one says *Obligatory*, one says *Impermissible*), it resolves the conflict using Defeasible Logic rules.
4.  **Verdict**: Outputs a final decision with justification.

### Conflict Resolution Hierarchy

When norms class, the winner is decided by:
1.  **Specificity**: A more specific rule overrides a general rule (Lex Specialis).
2.  **Priority**: A higher priority rule overrides a lower priority rule (Lex Superior).
3.  **Recency**: (Optional) A newer rule may override an older one (Lex Posterior).

---

## ⚖️ Deontic Modalities

The system supports three core modalities for any action:

*   **`Obligatory`**: The action **MUST** be taken. (Failure to do so is a violation).
*   **`Impermissible`**: The action **MUST NOT** be taken. (Doing so is a violation).
*   **`Optional(cost)`**: The action is permitted, but incurs a specific "cost" (computational, resource, or ethical cost).

---

## 🔍 The Verdict

The output of the engine is a `Verdict`:

```rust
pub struct Verdict {
    pub outcome: TeloidModal,     // e.g., Impermissible
    pub justification: Vec<u64>,  // IDs of the Teloids that forced this outcome
}
```

This provides **Explicability**. The system doesn't just say "No"; it says "No, because Norm #42 (Safety Protocol) and Norm #15 (RoE) applied, and #42 overrode #15."

---

## 🚀 Use Cases

1.  **AI Safety**: Preventing autonomous systems from taking dangerous actions.
2.  **Robotics**: Implementing Asimov-style laws or operational safeguards.
3.  **Compliance**: Real-time checking of financial transactions against regulatory norms.
4.  **Game AI**: Managing complex faction rules or character behaviors.

---

## Summary

| Concept | Type | Purpose |
|---------|------|---------|
| Norm Unit | `Teloid` | Defines a single rule (If condition -> Outcome) |
| Engine | `EffectEthos` | Manages store, graph, and reasoning process |
| Logic | `TeloidModal` | `Obligatory`, `Impermissible`, `Optional` |
| Output | `Verdict` | Final decision + Justification trail |
