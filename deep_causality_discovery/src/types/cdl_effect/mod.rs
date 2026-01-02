/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CDL, CdlConfig, CdlError, CdlWarningLog, NoData};
use deep_causality_haft::{
    Applicative, Effect3, Functor, HKT, HKT3, LogAppend, Monad, NoConstraint, Pure, Satisfies,
};

use std::marker::PhantomData;

// --- 3.2. CdlEffect Type Definition ---

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
        CdlEffectWitness::<CdlError, CdlWarningLog>::bind(self, f)
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

// --- 3.1. HKT Witness Definitions ---

/// A witness type that "fixes" the Error (E) and Warning Log (WLog) types
/// for `CdlEffect`, implementing the `HKT` and `HKT3` traits.
///
/// This creates a type-level function: `Type<T> = CdlEffect<T>` (where E and WLog are fixed).
///
/// It uses `PhantomData` to carry the fixed types without inhibiting standard traits
/// and a phantom generic placeholder.
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

// The Builder struct connecting the Effect system to the Witness
pub struct CdlBuilder;

// Implement Effect3: Fixing the Error and Warning types for the system.
impl Effect3 for CdlBuilder {
    type Fixed1 = CdlError;
    type Fixed2 = CdlWarningLog;
    type HktWitness = CdlEffectWitness<Self::Fixed1, Self::Fixed2>;
}

// --- Monad Implementation ---

// 1. Functor: fmap
impl Functor<CdlEffectWitness<CdlError, CdlWarningLog>>
    for CdlEffectWitness<CdlError, CdlWarningLog>
{
    fn fmap<A, B, Func>(m_a: CdlEffect<A>, f: Func) -> CdlEffect<B>
    where
        Func: FnMut(A) -> B,
    {
        // fmap expects FnMut
        let f = f;
        CdlEffect {
            inner: m_a.inner.map(f),
            warnings: m_a.warnings, // Warnings are preserved
        }
    }
}

// 2a. Pure Implementation
impl Pure<CdlEffectWitness<CdlError, CdlWarningLog>> for CdlEffectWitness<CdlError, CdlWarningLog> {
    fn pure<T>(value: T) -> CdlEffect<T>
    where
        T: Satisfies<NoConstraint>,
    {
        CdlEffect {
            inner: Ok(value),
            warnings: CdlWarningLog::default(),
        }
    }
}

// 2. Applicative: apply
impl Applicative<CdlEffectWitness<CdlError, CdlWarningLog>>
    for CdlEffectWitness<CdlError, CdlWarningLog>
{
    fn apply<A, B, Func>(
        f_ab: CdlEffect<Func>, // The container holding the function
        mut m_a: CdlEffect<A>, // The container holding the value
    ) -> CdlEffect<B>
    where
        Func: FnMut(A) -> B,
        A: Clone,
        B: Satisfies<NoConstraint>,
        Func: Satisfies<NoConstraint>,
    {
        let mut combined_warnings = f_ab.warnings;
        // Append warnings from m_a
        combined_warnings.append(&mut m_a.warnings);

        let new_inner = match (f_ab.inner, m_a.inner) {
            (Ok(mut func), Ok(val)) => Ok(func(val)),
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e),
        };

        CdlEffect {
            inner: new_inner,
            warnings: combined_warnings,
        }
    }
}

// 3. Monad: bind
impl Monad<CdlEffectWitness<CdlError, CdlWarningLog>>
    for CdlEffectWitness<CdlError, CdlWarningLog>
{
    fn bind<A, B, Func>(m_a: CdlEffect<A>, f: Func) -> CdlEffect<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> CdlEffect<B>,
    {
        let mut f = f;
        match m_a.inner {
            Err(e) => CdlEffect {
                inner: Err(e),
                warnings: m_a.warnings,
            },
            Ok(val) => {
                let mut m_b = f(val);
                let mut combined_warnings = m_a.warnings;
                // Append warnings from the result of the bound function
                combined_warnings.append(&mut m_b.warnings);

                CdlEffect {
                    inner: m_b.inner,
                    warnings: combined_warnings,
                }
            }
        }
    }
}

impl CdlBuilder {
    /// Helper to lift a value into the CdlEffect context (Pure)
    pub fn pure<T>(value: T) -> CdlEffect<T> {
        CdlEffectWitness::<CdlError, CdlWarningLog>::pure(value)
    }

    pub fn build() -> CdlEffect<CDL<NoData>> {
        Self::pure(CDL {
            state: NoData,
            config: CdlConfig::default(),
        })
    }
}
