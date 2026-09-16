/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Protein folding with memory
//!
//! A protein chain folds by passing through partly folded intermediates on its way to the native
//! shape it works in. Modelling that as a Markov chain says the next conformation depends only on
//! the current one. Real chains carry memory: a segment that has just formed a contact behaves
//! differently from one that arrived in the same shape by chance.
//!
//! The **generalized master equation** carries that memory. Alongside the single-step transition
//! operator it sums a set of memory kernels against the distributions the chain held at earlier
//! times, so the past enters the next step directly.
//!
//! Three categorical operations run the distribution bookkeeping:
//!
//! ```text
//! fold      distribution → its total mass    a reduction over the states
//! fmap      distribution → rescaled entries  one division per state
//! sequence  Vec<Result>  → Result<Vec>       one fallible distribution
//! ```
//!
//! The memory term adds mass the transition operator alone conserves, so each step is renormalised.
//! `fold` totals it, `fmap` divides it out, and `sequence` turns the vector of fallible re-wraps
//! into a single result, so a value leaving `[0, 1]` surfaces as one error for the distribution.

mod model;
mod utils_print;

use deep_causality_num::Float106;
use model::{
    MEMORY_DEPTH, advance, markov_operator, memory_kernels, native_fraction, unfolded_state,
};
use utils_print::{print_distribution, print_header, print_summary};

/// How many steps of the master equation the run takes.
const TIME_STEPS: usize = 15;

/// How often the run prints the distribution.
const REPORT_EVERY: usize = 3;

/// The working scalar. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16`; the transition
/// operator, the memory kernels and every distribution recompute at that precision.
///
/// It sits at [`Float106`] by default on purpose. A hard-coded `f64` anywhere in the program is
/// invisible while the alias *is* `f64`, and shows up here as a compile error the moment the two
/// types differ.
pub type FloatType = Float106;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header(TIME_STEPS);

    let operator = markov_operator()?;
    let kernels = memory_kernels()?;

    let mut state = unfolded_state()?;
    print_distribution(0, &state);

    // The chain starts with no past, so the history is seeded with the initial distribution. It
    // then slides forward one step at a time, always holding the last `MEMORY_DEPTH` entries.
    let mut history = vec![state.clone(); MEMORY_DEPTH];

    for step in 1..=TIME_STEPS {
        state = advance(&state, &history, &operator, &kernels)?;

        history.remove(0);
        history.push(state.clone());

        if step % REPORT_EVERY == 0 {
            print_distribution(step, &state);
        }
    }

    print_summary(native_fraction(&state)?);
    Ok(())
}
