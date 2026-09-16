/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # A quantum counterfactual: what the decoder was reading
//!
//! A qubit on its own cannot be error-checked. Every measurement that would reveal whether it
//! flipped also reveals `α` and `β`, and destroys the superposition being protected. The way out
//! is to spread one logical qubit across three physical ones,
//!
//! ```text
//! α|0⟩ + β|1⟩   →   α|000⟩ + β|111⟩
//! ```
//!
//! and then ask only about the *relationship* between them. `⟨Z₀Z₁⟩` answers whether two qubits
//! disagree, and its value is `+1` or `−1` whatever `α` and `β` are, so it reports an error without
//! reporting the state. Two such checks name which of the three qubits flipped, and applying `X`
//! there undoes it exactly.
//!
//! That is the whole of the three-qubit repetition code, and it is what this run performs: a real
//! gate, a real pair of parity measurements, and a correction chosen by a decoder rather than
//! written down in advance.
//!
//! # The counterfactual
//!
//! The recovery is caused by the syndrome. Nothing else reaches the decoder, so if the syndrome had
//! said something else the decoder would have acted on that instead — and the run can be made to
//! show it.
//!
//! [`CausalFlow::alternate_value_if`] is Pearl's do-operator. It substitutes the value the
//! measurement produced with the value an intervention forces, and records the substitution.
//! The two pipelines below are the same four steps and differ by that one line:
//!
//! ```text
//! observed        encode → flip → measure →                    recover
//! counterfactual  encode → flip → measure → do(syndrome := q0) → recover
//! ```
//!
//! Both recoveries run. One restores the state and one destroys it, and the difference is
//! attributable to the substituted value because everything else about the two runs is identical.
//!
//! # What the run does
//!
//! ```text
//! bind                 each stage, threading the register through the state channel
//! fold                 amplitudes → a parity, and amplitudes → a fidelity
//! alternate_value_if   the intervention, as the do-operator
//! ```

mod model;
mod utils_print;

use deep_causality_core::{
    CausalEffect, CausalEffectPropagationProcess, CausalFlow, CausalityError, PropagatingProcess,
};
use deep_causality_multivector::HilbertState;
use deep_causality_num::Float106;
use model::{
    FLIPPED_QUBIT, ONE, Syndrome, amplitudes, bit_flip, encode, exactness_tolerance, fidelity,
    magnitude, measure_syndrome, recover,
};
use utils_print::{
    print_encoding, print_error, print_header, print_outcome, print_parities, print_recovery,
};

/// The working scalar. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16`; the amplitudes,
/// the parities and the fidelities all recompute at that precision.
///
/// It sits at [`Float106`] by default on purpose. A hard-coded `f64` anywhere in the program is
/// invisible while the alias *is* `f64`, and shows up here as a compile error the moment the two
/// types differ.
pub type FloatType = Float106;

/// The register as it travels through the pipeline.
type Register = HilbertState<FloatType>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    // One logical qubit, spread across three.
    let (alpha, beta) = amplitudes();
    let protected = encode(alpha, beta)?;
    print_encoding(&protected);

    // The error: a real X gate on one register, which the decoder is not told about.
    let corrupted = bit_flip(&protected, FLIPPED_QUBIT)?;
    print_error(&protected, &corrupted, FLIPPED_QUBIT);
    print_parities(&protected, &corrupted);

    // The observed run: measure the syndrome, and recover from what it says.
    let observed = CausalFlow::process(corrupted.clone())
        .bind(stage_measure)
        .bind(stage_recover)
        .into_process();

    // The counterfactual: the same four steps, with the syndrome the decoder reads replaced by the
    // one an intervention forces. `alternate_value_if` is the do-operator, and it is the only
    // difference between this pipeline and the one above.
    let intervened = CausalFlow::process(corrupted.clone())
        .bind(stage_measure)
        .alternate_value_if(Syndrome::is_error, |_| Syndrome::NAMES_QUBIT_0)
        .bind(stage_recover)
        .into_process();

    let observed_state = observed.state();
    let intervened_state = intervened.state();

    print_recovery(
        "observed",
        measure_syndrome(&corrupted),
        observed_state,
        fidelity(&protected, observed_state),
    );
    print_recovery(
        "do(syndrome := qubit 0)",
        Syndrome::NAMES_QUBIT_0,
        intervened_state,
        fidelity(&protected, intervened_state),
    );

    let observed_fidelity = fidelity(&protected, observed_state);
    let intervened_fidelity = fidelity(&protected, intervened_state);

    print_outcome(
        observed_fidelity,
        intervened_fidelity,
        fidelity(&protected, &corrupted),
    );

    // The two claims the example makes: the observed syndrome restores the state, and the forced
    // one destroys it. Both hold at every scalar, so a miss is a defect and the run says so.
    let tolerance = exactness_tolerance();
    let restored = magnitude(observed_fidelity - ONE) < tolerance;
    let destroyed = intervened_fidelity < tolerance;

    if !(restored && destroyed) {
        return Err(format!(
            "the recovery claims failed: observed fidelity {}, intervened fidelity {}",
            deep_causality_num::lower(observed_fidelity),
            deep_causality_num::lower(intervened_fidelity)
        )
        .into());
    }

    Ok(())
}

/// Measures both parity checks and puts the syndrome on the value channel.
///
/// The register itself rides the state channel untouched, which is the point of measuring parities
/// rather than amplitudes: the stage learns whether two qubits disagree and learns nothing else.
fn stage_measure(
    _previous: CausalEffect<()>,
    register: Register,
    ctx: Option<()>,
) -> PropagatingProcess<Syndrome, Register, ()> {
    let syndrome = measure_syndrome(&register);
    let next = CausalEffectPropagationProcess::pure(syndrome);

    CausalEffectPropagationProcess::with_state(next, register, ctx)
}

/// Applies whatever correction the syndrome on the value channel calls for.
///
/// It reads the syndrome and nothing else, which is what makes the intervention above meaningful:
/// substituting that value is substituting everything the decoder has to go on.
fn stage_recover(
    syndrome: CausalEffect<Syndrome>,
    register: Register,
    ctx: Option<()>,
) -> PropagatingProcess<Syndrome, Register, ()> {
    let syndrome = syndrome.into_value().unwrap_or(Syndrome::CLEAN);

    match recover(&register, syndrome) {
        Ok(corrected) => {
            let next = CausalEffectPropagationProcess::pure(syndrome);
            CausalEffectPropagationProcess::with_state(next, corrected, ctx)
        }
        Err(e) => {
            let failed = CausalEffectPropagationProcess::from_error(CausalityError::new(
                deep_causality_core::CausalityErrorEnum::Custom(format!("{e}")),
            ));
            CausalEffectPropagationProcess::with_state(failed, register, ctx)
        }
    }
}
