---
title: Glossary
description: Canonical terminology for DeepCausality. The names land here once; everything else references this page.
sidebar:
  order: 99
---

This page is the single source of truth for terminology. The other concept pages link here. The [EPP preprint](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/effect_propagation_process/epp.pdf) and its companion volumes are the formal authority; this page is the operational synopsis.

## Core terms

**Causaloid**: A self-contained unit of causality. Wraps a causal function and metadata. Composes recursively into Singleton, Collection, and Graph forms that share the same type. See [Causaloid](/concepts/causaloid/).

**Causaloid Graph**: A directed graph whose nodes are Causaloids and whose edges express the order of evaluation. The result of evaluating the graph is the effect produced at its terminal node(s).

**Causal Discovery Language (CDL)**: A typestate-builder pipeline that ingests observational data and produces a discovery report whose recommendations inform the construction of a `CausaloidGraph`. Lives in [`deep_causality_discovery`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_discovery). See [CDL](/concepts/cdl/).

**Causal Function**: The pure function a Causaloid wraps. Two signatures: stateless (`CausalFn<I, O>`) takes the input alone; contextual (`ContextualCausalFn<I, O, STATE, CTX>`) also receives a Context.

**Causal Monad**: The `pure`/`bind` algebra that composes Causal Effect Propagation Processes. A trait (`CausalMonad`) implemented by the carrier effect, not a separate type. The axiom `m₂ = m₁ >>= f` is its state-threading `bind` operation. See [Causal Monad](/concepts/causal-monad/).

**Causal Reasoning**: The act of running one or more Causaloids against a Context and consuming the resulting propagating effect.

**Causal State Machine (CSM)**: A higher-level construct that links recognized causal states to deterministic actions. Used in the Effect Ethos for separating inference from action. Introduced in the [Teleology preprint](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/teleology_effect_propagation_process/epp_teleology.pdf).

**Context**: An explicit hypergraph encoding the environment in which Causaloids operate. Nodes are Contextoids; edges are typed relations. See [Context](/concepts/context/).

**Contextoid**: The atomic node of a Context. Carries a typed payload: `Datoid`, `Spaceoid`, `Tempoid`, `SpaceTempoid`, or `Symboid`. Non-recursive by design.

**Contextual Fabric**: The monograph's name for the union of Contexts and Context Hypergraphs in which effects propagate. The Rust code exposes this through `BaseContext` and its generic relatives.

**Dynamic Causality**: The library's framing of causality as a process whose structure (or context, or both) can evolve. The default operating mode. See [Dynamic Causality](/concepts/dynamic-causality/).

**Effect Ethos**: The verification layer that checks every causal effect against a set of named Teloids. Implements defeasible deontic inference with Lex Posterior, Lex Specialis, and Lex Superior. Lives in [`deep_causality_ethos`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_ethos). See [Effect Ethos](/concepts/effect-ethos/).

**Effect Log**: An append-only audit log carried by every `CausalEffectPropagationProcess`. Every Causaloid invocation contributes one entry; `bind` merges logs across the chain.

**Effect Propagation Process (EPP)**: Both a concept and a literal type in code. The concept: the directed flow of effects through a Causaloid chain. The type: `CausalEffectPropagationProcess<Value, State, Context, Error, Log>` in [`deep_causality_core`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_core). See [Effect Propagation Process](/concepts/effect-propagation-process/).

**Causal Effect (`CausalEffect`)**: The success channel of a propagating effect — a value, an absence (none), or a control command (`RelayTo`). An opaque newtype over the free monad on the control-operation functor. (Replaces the earlier `EffectValue` enum; its unused `ContextualLink` and `Map` variants were removed.)

**Evidence**: A unit of factual data in the monograph's ontology. In code, evidence enters the system as Contextoids of type `Datoid`.

**Higher-Kinded Types (HKT)**: Type-level functions that take types as arguments and return types. The library encodes them via the witness pattern (`HKT3`, `HKT5`) defined in [`deep_causality_haft`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_haft). See [HKT](/concepts/hkt/).

**Propagating Effect**: The stateless carrier alias `PropagatingEffect<T> = CausalEffectPropagationProcess<T, (), (), CausalityError, EffectLog>`. The everyday return type of a Causaloid's function; it implements the [Causal Monad](/concepts/causal-monad/) trait. Its stateful sibling is `PropagatingProcess<T, S, C>`.

**Teloid**: The atomic deontic rule inside an Effect Ethos. Encodes a modality (obligatory, impermissible, optional), a condition, and a Context query. Defined in the [Teleology preprint](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/teleology_effect_propagation_process/epp_teleology.pdf).
