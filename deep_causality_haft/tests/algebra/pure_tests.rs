/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{OptionWitness, Pure, ResultWitness, VecWitness};

#[test]
fn test_pure_option() {
    let val = OptionWitness::pure(42);
    assert_eq!(val, Some(42));

    let val_str = OptionWitness::pure("hello");
    assert_eq!(val_str, Some("hello"));
}

#[test]
fn test_pure_result() {
    // ResultWitness requires type annotation for Error type usually, but Pure::pure returns F::Type<T>
    // ResultWitness is HKT with Constraint=NoConstraint?
    // Let's check ResultWitness definition. It's usually ResultWitness<E>.
    // But in haft, it might be just ResultWitness if E is fixed or handled differently.
    // Looking at other tests: `ResultWitness` is used directly.
    // `type Type<T> = Result<T, String>;` ??? Only if ResultWitness is for String error?
    // Wait, `ResultWitness` in `lib.rs`?
    // Let's check `tests/algebra/applicative_tests.rs` again.
    // `let pure_val: Result<i32, &str> = ResultWitness::pure(5);`
    // This implies `ResultWitness` might be generic or handle defaults.
    // Actually, `ResultWitness` likely implements `HKT` where `Type<T> = Result<T, E>`.
    // But `E` needs to be defined.
    // In `applicative_tests.rs`: `let pure_val: Result<i32, &str> = ResultWitness::pure(5);`
    // If `ResultWitness` implements `Pure<ResultWitness>`, then `ResultWitness` itself must determine `E`.
    // Maybe `ResultWitness` is `ResultWitness` struct?
    // Let's check `deep_causality_haft/src/lib.rs` or `mod.rs` to see `ResultWitness`.
    // Or just look at `applicative_tests.rs` again.
    // `use deep_causality_haft::{Applicative, OptionWitness, Pure, ResultWitness};`
    // It seems `ResultWitness` is a specific type.
    // If I use `ResultWitness::pure(5)`, it returns something.
    // `applicative_tests.rs` line 29: `let pure_val: Result<i32, &str> = ResultWitness::pure(5);`
    // If the return type is `Result<i32, &str>`, then `ResultWitness::Type<i32>` must be `Result<i32, &str>`.
    // This implies `ResultWitness` might have a default generic or is bound to a specific error type in the test context?
    // Or `ResultWitness` is `struct ResultWitness<E>(PhantomData<E>)`?
    // If so, `ResultWitness::pure` would need `ResultWitness` to be instantiated or type inferred?
    // `Pure` trait: `fn pure<T>(value: T) -> F::Type<T>`
    // If `F` is `ResultWitness`, then `ResultWitness` must satisfy `HKT`.
    // If `ResultWitness` is generic `ResultWitness<E>`, then we need to specify `E`.
    // But `applicative_tests.rs` imports `ResultWitness` directly.
    // Let's assume standard `ResultWitness` used in tests implies `Result<T, &str>` or `String`.
    // Wait, `applicative_tests.rs`: `let pure_val: Result<i32, &str> = ResultWitness::pure(5);`
    // This suggests `ResultWitness` might be `ResultWitness<'static str>` or similar?
    // OR `ResultWitness` is a type alias?
    // To be safe, I'll validte against what I saw in `applicative_tests.rs`.
    // It uses `ResultWitness` as if it's a concrete type or unit struct that implements `Pure`.
    // But for `Result<T, E>`, `Type<T>` depends on `E`.
    // If `ResultWitness` is used without generics in `use`, it must be a struct.
    // If it implements `HKT`, it must define `Type<T>`.
    // If `Type<T>` is `Result<T, E>`, `E` must be fixed for that witness.
    // So `ResultWitness` probably fixes `E` to something, or there's some magic.
    //
    // Let's actually Look at `deep_causality_haft/src/hkt/mod.rs` or similar if I can, but I saw `applicative_tests.rs` working.
    // I will try to match the pattern in `applicative_tests.rs`.
    // `let pure_val: Result<i32, &str> = ResultWitness::pure(5);`
    // This line compiles in `applicative_tests.rs`.
    // So I will use that.

    // Note: The compiler infers the return type.
    // If `ResultWitness::type Type<T> = Result<T, &str>`, then it works.
    // I'll assume `ResultWitness` in this context (from prelude or haft root) is likely for `Result<T, &str>` or generic and inferred?
    // The previous `applicative_tests.rs` showed `Result<i32, &str>`.
    // I'll assume that works.

    let val: Result<i32, &str> = ResultWitness::pure(42);
    assert_eq!(val, Ok(42));
}

#[test]
fn test_pure_vec() {
    let val = VecWitness::pure(10);
    assert_eq!(val, vec![10]);
}
