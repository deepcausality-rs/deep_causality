
# DeepCausality Architecture Migration Review

This document outlines the architectural changes implemented during the migration of `deep_causality` to the `deep_causality_core` arity-5 monadic system.

## 1. Core Architectural Changes

### 1.1 Causaloid Generic Simplification (9 → 4)
The most significant change is the simplification of the `Causaloid` struct. Previously, the struct required 9 generic parameters to strictly define every aspect of the context (Data, Space, Time, Spacetime, Symbolic, etc.), leading to verbose signatures and tight coupling.

#### **Before (9 Generics)**
```rust
pub struct Causaloid<I, O, D, S, T, ST, SYM, VS, VT>
where
    I: Default,
    O: Default + Debug,
    D: Datable + Datable + Clone,
    S: Spatial<VS> + Clone,
    // ... many more bounds ...
{ ... }
```

#### **After (4 Generics)**
The new architecture abstracts the specific context components into a single `C` generic and introduces `PS` for internal process state.

```rust
pub struct Causaloid<I, O, PS, C>
where
    I: Default,
    O: Default + Debug,
    PS: Default + Clone, // Process State
    C: Clone,            // Context (abstract)
{ ... }
```

**Benefits:**
- **Decoupling:** `Causaloid` no longer needs to know the internal structure of the `Context` unless specified.
- **Flexibility:** `C` can be `Arc<RwLock<Context<...>>>` or any other type (e.g., `()`).
- **Readability:** Type aliases and function signatures are significantly shorter.

### 1.2 The Unified Effect System (`PropagatingProcess`)
The system moved from a fragmented approach (stateless `PropagatingEffect` + separate context handling) to a unified **State monad** pattern using `PropagatingProcess`.

#### **Before**
Effects were primarily stateless wrappers around data, with `Result` often wrapping them for error handling. Context was passed as an explicit reference argument.

#### **After**
`PropagatingProcess<V, S, C>` serves as the central Monadic container.
- **V (Value):** The data being computed (e.g., `bool`, `f64`).
- **S (State):** Internal state accumulated during the process (e.g., `()`, `Vec<History>`).
- **C (Context):** The shared environment.

**Relationship:**
`PropagatingEffect<T>` still exists in `deep_causality_core` effectively as a projection of `PropagatingProcess` (where State and Context are void/ignored) to support legacy systems like the `CSM` (Causal State Machine) which are primarily stateless/reactive.

### 1.3 Contextual Logic Refactoring

Functional logic for causaloids has been updated to return the simpler monadic type directly, removing standard `Result` nesting.

#### **Before**
```rust
type ContextualCausalFn<...> = 
    fn(EffectValue<I>, &Context<...>) // Reference to context
    -> Result<PropagatingEffect<O>, CausalityError>; // Result wrapper
```

#### **After**
```rust
type ContextualCausalFn<I, O, PS, C> = 
    fn(EffectValue<I>, PS, Option<C>) // Owned/Option context, explicit state
    -> PropagatingProcess<O, PS, C>;  // Direct Monad return (contains Value or Error)
```

**Key Changes:**
1.  **Direct Return:** Functions no longer return `Result<..., Error>`. `PropagatingProcess` itself can be in an error state (via `from_error`), allowing for uninterrupted monadic chaining (`bind`).
2.  **State Passing:** The `PS` parameter allows functions to receive and transition state explicitly.

---

## 2. Migration Examples

### 2.1 Updating Constraints
Usage of `Causaloid` or generic functions now requires adding the `Clone` bound to `I` and `O`.

```diff
- where I: Default, O: Default + Debug
+ where I: Default + Clone, O: Default + Debug + Clone
```
*Reason: The monadic `bind` operations in `deep_causality_core` require cloning values during propagation.*

### 2.2 Updating Causal Functions
Example from `test_utils.rs`:

```rust
// NEW Implementation
fn causal_fn(
    obs: EffectValue<bool>,
    _state: (),                         // Added State param
    context: Option<Arc<RwLock<...>>>   // Context is Option<C>
) -> PropagatingProcess<bool, (), Arc<RwLock<...>>> // Return Process directly
{
    if context.is_none() {
        // Return error via Monad, not Result::Err
        return PropagatingProcess::from_error(CausalityError(...)); 
    }
    
    // ... logic ...

    // Return value via pure
    PropagatingProcess::pure(result) 
}
```

### 2.3 Evaluation Bridge (`causable_utils.rs`)
Since much of the surrounding framework (like `CausalState`) expects stateless `PropagatingEffect`s, the core evaluation logic acts as a bridge:

1.  **Execute:** Calls `ContextualCausalFn` using `PropagatingProcess`.
2.  **Extract:** Accesses `.value` from the resulting process.
3.  **Convert:** Wraps the value into a `PropagatingEffect` (stateless) for consumption by the legacy CSM.

```rust
// Concept
let process = context_fn(input, state, context);
match process.value.into_value() {
    Some(val) => PropagatingEffect::pure(val),
    None => PropagatingEffect::from_error(...)
}
```

## 3. Verification

The migration is verified by:
1.  **Generic Bounds:** All `Causaloid` implementations now satisfy the 4-generic signature.
2.  **Compilation:** `cargo check --lib -p deep_causality` passes with 0 errors.
3.  **Tests:** `cargo test -p deep_causality` passes, confirming that the new process-based logic correctly resolves to deterministic outcomes expected by the existing test suite.

## 4. CSM Architecture Migration

Similar to `Causaloid`, the Causal State Machine (CSM) types have been simplified to use the unified Context generic.

### 4.1 CSM Generic Simplification (9 → 3)

The `CSM`, `CausalState`, and related aliases (`StateAction`, `CSMMap`) have been refactored to remove the explicit dependency on `D`, `S`, `T`, `ST`, `SYM`, `VS`, `VT`.

#### **Before (9 Generics)**
```rust
pub struct CSM<I, O, D, S, T, ST, SYM, VS, VT>
where ...
{
    state_actions: Arc<RwLock<CSMMap<I, O, D, S, T, ST, SYM, VS, VT>>>,
}
```

#### **After (3 Generics)**
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

### 4.2 Migration Checklist for CSM Users
1.  Update `CSM::new` calls: You no longer need to specify the 9 generic parameters. `C` is inferred or explicitly set to your context type (e.g., `Arc<RwLock<Context<...>>>` or `()`).
2.  Update `CausalState` construction: Similar to `CSM`, it now takes `Causaloid<I, O, (), C>`.
3.  Type Aliases: If you used `StateAction` or `CSMMap` in your code, update them to `StateAction<I, O, C>` and `CSMMap<I, O, C>`.

## 5. Generative System Architecture Migration

### 5.1 CausalSystemState Simplification (9 → 3)

The `CausalSystemState` struct used in the generative system has been simplified.

#### **Before (9 Generics)**
```rust
pub struct CausalSystemState<I, O, D, S, T, ST, SYM, VS, VT>
{
    pub causaloids: HashMap<u64, Causaloid<I, O, ..., Arc<RwLock<Context<...>>>>>,
    pub contexts: HashMap<u64, Context<...>>,
}
```

#### **After (3 Generics)**
```rust
pub struct CausalSystemState<I, O, C>
where
    I: Default,
    O: Default + Debug,
    C: Clone,
{
    pub causaloids: HashMap<u64, Causaloid<I, O, (), Arc<RwLock<C>>>>,
    pub contexts: HashMap<u64, C>,
}
```
### 5.2 Operation Enum Simplification (9 → 4)

The `Operation` enum and `OpTree` alias have also been simplified to abstract over the specific Context and Node types.

#### **Before (9 Generics)**
```rust
pub enum Operation<I, O, D, S, T, ST, SYM, VS, VT> {
    // ... variants strictly typed to D, S, T ...
}
```

#### **After (4 Generics)**
```rust
pub enum Operation<I, O, C, N>
where
    I: Default + Clone,
    O: Default + Clone + Debug,
    C: Clone, // Context Type (e.g., Context<...>)
    N: Clone, // Node Type (e.g., Contextoid<...>)
{
    // ...
    CreateCausaloid(CausaloidId, Causaloid<I, O, (), Arc<RwLock<C>>>),
    AddContextoidToContext {
        context_id: ContextId,
        contextoid: N,
    },
    // ...
}
```

**Usage:**
When defining an `OpTree` or calling `Model::evolve`, you now supply the concrete Context and Node types:
```rust
OpTree<I, O, Context<...>, Contextoid<...>>
```

**Note:** The `Operation` enum itself is simplified to `<I, O, C, N>`. However, methods like `Interpreter::execute` still generic over the 9 parameters (`I, O, D, S, T...`) because they must deconstruct the `Context` type to perform operations on its internal components (like adding a `Contextoid`). This complexity is hidden from the high-level `Operation` structure but exists in the execution layer.
### 5.3 Model Type Simplification (9 → 3)

The `Model` struct has been simplified to decouple it from strict Context/Node hierarchies in its definition.

#### **Before (9 Generics)**
```rust
pub struct Model<I, O, D, S, T, ST, SYM, VS, VT> {
    causaloid: Arc<Causaloid<I, O, ..., Arc<RwLock<Context<...>>>>>,
    context: Option<Arc<RwLock<Context<...>>>>,
}
```

#### **After (3 Generics)**
```rust
pub struct Model<I, O, C>
where
    I: Default,
    O: Default + Debug,
    C: Clone,
{
    causaloid: Arc<Causaloid<I, O, (), Arc<RwLock<C>>>>,
    context: Option<Arc<RwLock<C>>>,
}
```

**Note on `evolve`:**
The `evolve` method is primarily implemented for the concrete `Context` type to allow interoperability with the `Interpreter`, which currently requires specific Context structure. You must specialize your model usage (like `BaseModel` or `UniformModel`) or ensure your generic `C` matches `Context<...>` when calling `evolve`.
