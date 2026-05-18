---
title: Effect Ethos
description: The verification layer that checks every effect against the rules you have to honor.
section: concepts
order: 4
---

A Causaloid says what the system *infers*. The Effect Ethos says what the system is *allowed to act on*. They are different jobs, and the library keeps them in different crates.

## The problem

Inference and policy get conflated in most codebases. A trading rule fires; somewhere downstream a function checks for compliance flags; somewhere else a manual approval is recorded; somewhere else again a `if user.is_admin` clause silently changes the path. The rules are scattered, the audit is partial, and the day a regulator asks *which check failed and why*, the code cannot answer.

The Effect Ethos pulls those scattered checks into one structured object that can answer.

## What it is

The `EffectEthos` lives in the [`deep_causality_ethos`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_ethos) crate. The shape:

```rust
pub struct EffectEthos<D, S, T, ST, SYM, VS, VT>
where
    /* … same context bounds as Context … */
{
    teloid_store: TeloidStore<D, S, T, ST, SYM, VS, VT>,
    tag_index: TagIndex,
    teloid_graph: TeloidGraph,
    id_to_index_map: HashMap<TeloidID, usize>,
    is_verified: bool,
}
```

Two parts to read. The `teloid_store` holds the active rules. The `teloid_graph` and `tag_index` make the rules navigable, both by id and by category.

A `Teloid` is the atom inside the store. It is a computable unit of purpose, and it instantiates one rule from a defeasible deontic calculus (DDIC). Concretely a Teloid carries:

- A **deontic modality**: obligatory, impermissible, or optional.
- A **condition** under which the modality applies.
- A **scope tag** for indexing.
- An **id** that survives logging.
- A reference to the **Context query** the rule will evaluate against.

The [Teleology preprint](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/teleology_effect_propagation_process/epp_teleology.pdf) introduces Teloids as the answer to the question, "What stops an emergent system from inferring its way into a state you cannot let it act on?"

## Building an Ethos

```rust
use deep_causality_ethos::{EffectEthos, Verdict};

let ethos = EffectEthos::new()
    .forbid("pii_leaves_region",       rule::pii_leaves_region)
    .require("audit_log_present",      rule::audit_log_present)
    .require("approval_when_over_10k", rule::approval_when_over(10_000))
    .require("kyc_completed",          rule::kyc_completed);
```

The named rules are not decoration. The string is the stable identifier that shows up in every violation report, every audit log, and every rejection message returned to a caller.

## Verifying an action

```rust
let verdict = ethos.verify(&action)?;

match verdict {
    Verdict::Pass => commit(action)?,
    Verdict::Fail(violations) => {
        for v in &violations {
            audit_log.write(v)?;
        }
        return Err(reject_with(violations));
    }
}
```

`verify` evaluates every Teloid against the action. It does not stop at the first failure. The returned `Verdict::Fail` carries the full list of violations, each tagged with its Teloid name and the condition that failed. The audit log gets the complete picture; the caller gets a single rejection.

## Conflict resolution

Real rule sets contradict each other. Two requirements both apply, one says obligatory, the other says impermissible. The Effect Ethos resolves the contradiction with three principles drawn from legal reasoning:

- **Lex Posterior** — the later-issued rule wins over the earlier one.
- **Lex Specialis** — the more specific rule wins over the more general one.
- **Lex Superior** — the higher-priority rule wins over the lower-priority one.

These run in a fixed order when the Ethos is asked to reconcile a conflict. The combination is enough to handle most rule-set evolution in practice without giving up determinism.

## Why this is a separate concept

The Effect Ethos is not just the negation of the Causaloid graph. It can disagree with the Causaloid graph, and the disagreement is the point.

A Causaloid graph reasons forward from inputs to a propagating effect. It is concerned with *what is inferable*. An Ethos reasons against that effect from operational constraints. It is concerned with *what is permissible*. The two answers disagree often enough to be worth modelling separately. When they agree, the action commits. When they disagree, the rejection is structured and explainable.

The [Metaphysics preprint](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/metaphysics_effect_propagation_process/epp_metaphysics.pdf) frames this as the prospective guardrail for emergent systems: as the causal structure becomes able to evolve, the action layer needs an independent check that the evolved structure has not produced an output you cannot let leave the building.

## Common patterns

**Per-environment Ethoses.** Define one Ethos per deployment target (sandbox, staging, prod). The same Causaloids fire; the verdict differs because the Teloids differ.

**Layered Ethoses.** A general Ethos plus a regional Ethos plus a per-customer Ethos. Compose by chaining `verify` calls; reject if any layer fails.

**Time-windowed Teloids.** A Teloid can carry a temporal condition that consults the Context. A rule that applies only between two timestamps disappears outside that window without code changes.

**Replay.** Persist the action and the Context snapshot; re-run `verify` against a future version of the Ethos to detect rule-set drift.

## Where to look next

[Causaloid](/docs/concepts/causaloid/) is the inference layer the Ethos sits above. The API reference is on docs.rs at [`deep_causality_ethos`](https://docs.rs/deep_causality_ethos). The [Teleology preprint](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/teleology_effect_propagation_process/epp_teleology.pdf) is the formal treatment.
