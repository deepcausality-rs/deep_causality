/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CdlError, CdlWarningLog};
use deep_causality_haft::{HKT, HKT3, LogAppend, NoConstraint};

use std::marker::PhantomData;

mod applicative;
mod functor;
mod monad;

// --- CdlEffect Type Definition ---

#[derive(Debug, Clone)]
pub struct CdlEffect<T> {
    // E and W types fixed at the HKT witness level
    pub inner: Result<T, CdlError>, // Enforces valid state: either a Value or an Error.
    pub warnings: CdlWarningLog,    // Accumulated warnings, always present.
}

impl<T> CdlEffect<T> {
    /// Convenience method to print accumulated warnings.
    /// Prints "No Warnings" if empty, typically to stdout.
    pub fn print_warnings(&self) {
        if self.warnings.entries.is_empty() {
            println!("No Warnings");
        } else {
            println!("Pipeline Warnings:");
            for warning in &self.warnings.entries {
                println!(" - {:?}", warning);
            }
        }
    }

    /// Monadic bind operation: chains a function returning a new CdlEffect.
    /// This enables the fluent `.bind()` syntax used in examples.
    pub fn bind<B, F>(self, f: F) -> CdlEffect<B>
    where
        F: FnMut(T) -> CdlEffect<B>,
    {
        // Delegate to the Monad Witness implementation
        use deep_causality_haft::Monad;
        CdlEffectWitness::<CdlError, CdlWarningLog>::bind(self, f)
    }

    /// `FnOnce` Kleisli composition: like [`bind`](Self::bind), but the continuation
    /// runs at most once, so it may move owned values in (e.g. a loaded `BrcdInput`).
    ///
    /// Short-circuits on a prior error and threads (merges) the warning log, exactly
    /// like `bind`. This is the engine behind the fluent stage methods on
    /// `CdlEffect<CDL<State>>`, which let the pipeline read
    /// `.clean_data(..).feature_select(..)` without a `.bind(|cdl| cdl. …)` wrapper
    /// on every line.
    pub fn and_then<B, F>(self, f: F) -> CdlEffect<B>
    where
        F: FnOnce(T) -> CdlEffect<B>,
    {
        match self.inner {
            Err(e) => CdlEffect {
                inner: Err(e),
                warnings: self.warnings,
            },
            Ok(val) => {
                let mut next = f(val);
                let mut combined_warnings = self.warnings;
                combined_warnings.append(&mut next.warnings);
                CdlEffect {
                    inner: next.inner,
                    warnings: combined_warnings,
                }
            }
        }
    }
}

impl<T: std::fmt::Display> CdlEffect<T> {
    /// Prints the result and warnings if successful, or the error if failed.
    pub fn print_results(&self) {
        match &self.inner {
            Ok(val) => {
                println!("{}", val);
                self.print_warnings();
            }
            Err(e) => {
                eprintln!("❌ Pipeline Failed!");
                eprintln!("Error: {}", e);
            }
        }
    }
}

// --- HKT Witness Definition ---

/// A witness type that "fixes" the Error (E) and Warning Log (WLog) types
/// for `CdlEffect`, implementing the `HKT` and `HKT3` traits.
///
/// This creates a type-level function: `Type<T> = CdlEffect<T>` (where E and WLog are fixed).
///
/// It uses `PhantomData` to carry the fixed types without inhibiting standard traits
/// and a phantom generic placeholder.
///
/// The `Functor`, `Pure`, `Applicative`, and `Monad` instances for this witness live in
/// the sibling `functor`, `applicative`, and `monad` modules.
pub struct CdlEffectWitness<E, WLog>(PhantomData<(E, WLog)>);

// Implement HKT
impl<E, WLog> HKT for CdlEffectWitness<E, WLog> {
    type Constraint = NoConstraint;
    type Type<T> = CdlEffect<T>;
}

// Implement HKT3
impl<E, WLog> HKT3<E, WLog> for CdlEffectWitness<E, WLog>
where
    E: 'static,
    WLog: 'static,
{
    // The associated type that this witness "witnesses"
    type Type<T> = CdlEffect<T>;
}
