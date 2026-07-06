/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalMonad, CausalityError, CausalityErrorEnum, EffectLog, EffectValue};
use core::fmt::Debug;
use deep_causality_haft::LogAppend;

mod alternatable_context;
mod alternatable_state;
mod alternatable_value;
mod display;
mod explain;
mod getters;
pub mod hkt;
mod intervenable;
mod predicates;

/// The fundamental unit of causal computation in DeepCausality.
///
/// `CausalEffectPropagationProcess` encapsulates the state of a computation moving through a causal
/// graph. It unifies value propagation, state management, context awareness, error handling, and
/// comprehensive logging into a single, monadic structure.
///
/// # Concepts
///
/// *   **Outcome**: The value-XOR-error channel — `Result<EffectValue<Value>, Error>`, the
///     `Either E (Maybe T)` encoding. A process either carries an effect value (possibly
///     `EffectValue::None`) or an error, never both: the W-invariant
///     (`error present ⇒ no value`) holds **by construction**, not by discipline.
/// *   **State**: Persistent data that evolves as the process moves through the graph (Markovian state).
/// *   **Context**: Read-only configuration or environment data available to all steps.
/// *   **Error**: A failure state that short-circuits further computation but preserves logs.
/// *   **Logs**: An append-only history of every step, essential for auditability and explainability.
///
/// # Rationale
///
/// In complex causal reasoning, it is not enough to just know the final result. We must know *how*
/// that result was reached, what invalid states were encountered, and what context was active.
/// This struct implements the [Monad](https://en.wikipedia.org/wiki/Monad_(functional_programming)) pattern
/// to handle these concerns automatically, allowing users to focus on the domain logic ("Business Logic")
/// rather than plumbing (error checking, logging).
///
/// Because value and error live in ONE channel, the three monad laws (left identity, right
/// identity, associativity) hold **unconditionally** — including on errored carriers — and the
/// error short-circuit is a left zero (`bind(raise e, f) = raise e`, `f` never runs). This is
/// precondition P2 of the Causal Algebra program
/// (`openspec/notes/causal-algebra/Formalization.md` §2), machine-checked in
/// `lean/DeepCausalityFormal/Core/CausalMonad.lean`.
///
/// All fields are private; construct via [`new`](Self::new) (total — every representable state
/// is valid) or the named constructors, decompose via [`into_parts`](Self::into_parts), and read
/// via the getters in `getters.rs`.
#[derive(Debug, PartialEq, Clone)]
pub struct CausalEffectPropagationProcess<Value, State, Context, Error, Log> {
    /// The value-XOR-error channel: an effect value, or the error that ended the computation.
    pub(crate) outcome: Result<EffectValue<Value>, Error>,
    /// The current state of the process (e.g., accumulated risk, counters).
    pub(crate) state: State,
    /// The optional execution context (e.g., global config, reference data).
    pub(crate) context: Option<Context>,
    /// The audit log containing the history of operations.
    pub(crate) logs: Log,
}

impl<Value, State, Context, Error, Log>
    CausalEffectPropagationProcess<Value, State, Context, Error, Log>
{
    /// Total constructor over the single-channel representation.
    ///
    /// With value-XOR-error encoded as one `Result`, every combination of arguments is a
    /// valid process — there is nothing to validate and no way to construct the formerly
    /// representable invalid state (value AND error).
    pub const fn new(
        outcome: Result<EffectValue<Value>, Error>,
        state: State,
        context: Option<Context>,
        logs: Log,
    ) -> Self {
        Self {
            outcome,
            state,
            context,
            logs,
        }
    }

    /// Decomposes the process into its channels: `(outcome, state, context, logs)`.
    ///
    /// The inverse of [`new`](Self::new); the by-value counterpart to the getters.
    pub fn into_parts(
        self,
    ) -> (
        Result<EffectValue<Value>, Error>,
        State,
        Option<Context>,
        Log,
    ) {
        (self.outcome, self.state, self.context, self.logs)
    }

    /// Consumes the process and returns the carried scalar, if any.
    ///
    /// This is the terminal "give me the result value" accessor: it yields `Some(v)` only when
    /// the process carries `Ok(EffectValue::Value(v))`, and `None` for an errored process or any
    /// non-`Value` effect (`None`, `ContextualLink`, `RelayTo`, `Map`). It is the by-value
    /// counterpart to [`value`](Self::value) (which lends `Option<&EffectValue<Value>>`), and
    /// mirrors [`EffectValue::into_value`]. Use it at the end of a chain when you want the plain
    /// value out for display or comparison rather than a reference into the channel.
    pub fn into_value(self) -> Option<Value> {
        self.outcome.ok().and_then(EffectValue::into_value)
    }
}

impl<Value, State, Context, Error, Log>
    CausalEffectPropagationProcess<Value, State, Context, Error, Log>
where
    Log: LogAppend + Default,
    Error: Clone,
{
    /// Chains a stateful, context-aware computation.
    ///
    /// This is the primary method for building Markovian process chains, as the
    /// function `f` receives the value, state, and context from the previous step.
    ///
    /// Error short-circuits as a left zero: on an errored process, `f` is NOT invoked and the
    /// process is returned reassembled verbatim (error, state, context, and logs preserved),
    /// which is what makes right identity `bind(m, pure) = m` hold unconditionally.
    pub fn bind<F, NewValue>(
        self,
        f: F,
    ) -> CausalEffectPropagationProcess<NewValue, State, Context, Error, Log>
    where
        F: FnOnce(
            EffectValue<Value>,
            State,
            Option<Context>,
        ) -> CausalEffectPropagationProcess<NewValue, State, Context, Error, Log>,
    {
        match self.outcome {
            Err(error) => CausalEffectPropagationProcess {
                outcome: Err(error),
                state: self.state,
                context: self.context,
                logs: self.logs,
            },
            Ok(value) => {
                let mut next_process = f(value, self.state, self.context);

                let mut combined_logs = self.logs;
                combined_logs.append(&mut next_process.logs);
                next_process.logs = combined_logs;

                next_process
            }
        }
    }
}

impl<Value, State, Context, Error, Log>
    CausalEffectPropagationProcess<Value, State, Context, Error, Log>
where
    Log: Clone,
    Error: Clone,
{
    /// Lifts a stateless effect into a stateful process by providing an initial state and context.
    ///
    /// This is the primary entry point for starting a stateful computation chain from a
    /// simple, pre-existing effect.
    ///
    /// # Arguments
    /// * `effect`: The stateless `PropagatingEffect` (where State and Context are `()`).
    /// * `initial_state`: The starting state for the new process.
    /// * `initial_context`: The optional starting context for the new process.
    ///
    /// # Returns
    /// A new `CausalEffectPropagationProcess` ready for stateful operations.
    pub fn with_state(
        effect: CausalEffectPropagationProcess<Value, (), (), Error, Log>,
        initial_state: State,
        initial_context: Option<Context>,
    ) -> Self {
        Self {
            outcome: effect.outcome,
            state: initial_state,
            context: initial_context,
            logs: effect.logs,
        }
    }
}

// These constructors are generic over State and Context and can be used by any alias.
impl<Value, State, Context>
    CausalEffectPropagationProcess<Value, State, Context, CausalityError, EffectLog>
where
    Value: Default + Clone + Debug,
    State: Default + Clone + Debug,
    Context: Clone + Debug,
{
    /// Creates a new process that explicitly contains an error.
    /// The state is set to default.
    pub fn from_error(err: CausalityError) -> Self {
        Self {
            outcome: Err(err),
            state: State::default(),
            context: None,
            logs: EffectLog::new(),
        }
    }

    /// Creates a new process with `EffectValue::None`, default state, and no error.
    pub fn none() -> Self {
        Self {
            outcome: Ok(EffectValue::None),
            state: State::default(),
            context: None,
            logs: EffectLog::new(),
        }
    }

    /// Lifts a pure value into a process with a default state.
    pub fn pure(value: Value) -> Self {
        <Self as CausalMonad>::pure(value)
    }

    /// Maps the carried value with `f`, preserving state, context, and logs.
    ///
    /// This is the fluent `Functor` operation on the carrier, the value-only counterpart to
    /// [`bind`](Self::bind). It never panics. An error carrier short-circuits: the error and
    /// logs are preserved and `f` is not invoked. A `None` or `ContextualLink` carrier passes
    /// through unchanged. The dispatch variants `RelayTo` and `Map` embed a `PropagatingEffect`
    /// whose value type cannot be retyped by a value-level map, so `fmap` over them surfaces a
    /// `ValueNotAvailable` error rather than silently dropping the routing command; reach for
    /// [`bind`](Self::bind) when you need to act on those variants.
    /// (P1 seam: when `RelayTo`/`Map` leave `EffectValue`, only that match arm disappears.)
    pub fn fmap<NewValue, F>(
        self,
        f: F,
    ) -> CausalEffectPropagationProcess<NewValue, State, Context, CausalityError, EffectLog>
    where
        F: FnOnce(Value) -> NewValue,
    {
        let outcome = match self.outcome {
            // Error short-circuits: `f` is not invoked; the error is preserved.
            Err(error) => Err(error),
            Ok(EffectValue::Value(v)) => Ok(EffectValue::Value(f(v))),
            Ok(EffectValue::None) => Ok(EffectValue::None),
            Ok(EffectValue::ContextualLink(a, b)) => Ok(EffectValue::ContextualLink(a, b)),
            // RelayTo / Map carry a `PropagatingEffect<Value>` that a value-level `fmap` cannot
            // retype to `NewValue`. Surface this instead of dropping the dispatch command.
            Ok(_) => Err(CausalityError::new(CausalityErrorEnum::ValueNotAvailable)),
        };

        CausalEffectPropagationProcess {
            outcome,
            state: self.state,
            context: self.context,
            logs: self.logs,
        }
    }

    /// Creates a new process from a given `EffectValue`.
    /// The state is set to default.
    pub fn from_effect_value(effect_value: EffectValue<Value>) -> Self {
        Self {
            outcome: Ok(effect_value),
            state: State::default(),
            context: None,
            logs: EffectLog::new(),
        }
    }

    pub fn from_value(value: Value) -> Self {
        Self {
            outcome: Ok(EffectValue::Value(value)),
            state: State::default(),
            context: None,
            logs: EffectLog::new(),
        }
    }

    /// Creates a new process from a given `EffectValue` and `EffectLog`.
    /// The state is set to default.
    pub fn from_effect_value_with_log(value: EffectValue<Value>, logs: EffectLog) -> Self {
        Self {
            outcome: Ok(value),
            state: State::default(),
            context: None,
            logs,
        }
    }

    pub fn from_value_with_log(value: Value, logs: EffectLog) -> Self {
        Self {
            outcome: Ok(EffectValue::Value(value)),
            state: State::default(),
            context: None,
            logs,
        }
    }
}

impl<Value, State, Context, Log>
    CausalEffectPropagationProcess<Value, State, Context, CausalityError, Log>
where
    Log: LogAppend + Default,
{
    /// Chains a computation while automatically unwrapping the inner `EffectValue`.
    ///
    /// If the `EffectValue` is `None`, this method short-circuits with a `CausalityError`
    /// containing the provided `err_msg`. This simplifies the common pattern of:
    /// `bind -> match effect_value { Some(v) => f(v), None => Error }`
    ///
    /// On an errored process, the continuation is NOT invoked (left zero).
    pub fn bind_or_error<F, NewValue>(
        self,
        f: F,
        err_msg: &str,
    ) -> CausalEffectPropagationProcess<NewValue, State, Context, CausalityError, Log>
    where
        F: FnOnce(
            Value,
            State,
            Option<Context>,
        )
            -> CausalEffectPropagationProcess<NewValue, State, Context, CausalityError, Log>,
    {
        match self.outcome {
            Err(error) => CausalEffectPropagationProcess {
                outcome: Err(error),
                state: self.state,
                context: self.context,
                logs: self.logs,
            },
            Ok(effect_value) => match effect_value.into_value() {
                Some(v) => {
                    let mut next_process = f(v, self.state, self.context);
                    let mut combined_logs = self.logs;
                    combined_logs.append(&mut next_process.logs);
                    next_process.logs = combined_logs;
                    next_process
                }
                None => CausalEffectPropagationProcess {
                    outcome: Err(CausalityError(crate::CausalityErrorEnum::Custom(
                        err_msg.into(),
                    ))),
                    state: self.state,
                    context: self.context,
                    logs: self.logs,
                },
            },
        }
    }
}
