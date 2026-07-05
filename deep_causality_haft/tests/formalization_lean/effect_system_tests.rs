/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/EffectSystem.lean` (Moggi 1991; Wadler 1995
//! §2.7 for the Writer monoid).

use deep_causality_haft::{Effect3, Functor, HKT, HKT3, MonadEffect3, NoConstraint, Satisfies};

// Sum-encoded carrier: `Result` holds value XOR error (the W-invariant by construction),
// so bind's error branch needs neither `U::default()` nor a spurious run of `f` —
// demonstrating deviations D6/D7 against the product-encoded reference carrier.
#[derive(Debug, PartialEq, Clone)]
struct LawfulEff<T> {
    result: Result<T, String>,
    logs: Vec<String>,
}

struct LawfulEffWitness;
impl HKT3<String, Vec<String>> for LawfulEffWitness {
    type Type<T> = LawfulEff<T>;
}
impl HKT for LawfulEffWitness {
    type Constraint = NoConstraint;
    type Type<T> = LawfulEff<T>;
}
impl Functor<LawfulEffWitness> for LawfulEffWitness {
    fn fmap<A, B, Func>(m_a: LawfulEff<A>, mut f: Func) -> LawfulEff<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> B,
    {
        LawfulEff {
            result: m_a.result.map(&mut f),
            logs: m_a.logs,
        }
    }
}

struct LawfulEffect;
impl Effect3 for LawfulEffect {
    type Fixed1 = String;
    type Fixed2 = Vec<String>;
    type HktWitness = LawfulEffWitness;
}

struct LawfulMonadEffect3;
impl MonadEffect3<LawfulEffect> for LawfulMonadEffect3 {
    fn pure<T>(value: T) -> LawfulEff<T> {
        LawfulEff {
            result: Ok(value),
            logs: vec![],
        }
    }

    fn bind<T, U, Func>(effect: LawfulEff<T>, mut f: Func) -> LawfulEff<U>
    where
        Func: FnMut(T) -> LawfulEff<U>,
    {
        match effect.result {
            Err(e) => LawfulEff {
                result: Err(e),
                logs: effect.logs, // error short-circuits; f is NOT run
            },
            Ok(t) => {
                let next = f(t);
                let mut logs = effect.logs;
                logs.extend(next.logs);
                LawfulEff {
                    result: next.result,
                    logs,
                }
            }
        }
    }
}

/// THEOREM_MAP: haft.effect3.monad_laws
#[test]
fn test_effect3_monad_laws() {
    let f = |t: i32| LawfulEff {
        result: Ok(t + 1),
        logs: vec!["f".to_string()],
    };
    let g = |u: i32| LawfulEff {
        result: if u % 2 == 0 {
            Ok(u * 10)
        } else {
            Err("odd".to_string())
        },
        logs: vec!["g".to_string()],
    };

    // Left identity: bind (pure t) f = f t (pure's log is empty)
    assert_eq!(
        LawfulMonadEffect3::bind(LawfulMonadEffect3::pure(5), f),
        f(5)
    );

    // Right identity: bind m pure = m — including the errored carrier (sum encoding)
    let cases = [
        LawfulEff {
            result: Ok(5),
            logs: vec!["a".to_string()],
        },
        LawfulEff {
            result: Err("boom".to_string()),
            logs: vec!["a".to_string()],
        },
    ];
    for m in cases {
        assert_eq!(
            LawfulMonadEffect3::bind(m.clone(), LawfulMonadEffect3::pure),
            m
        );
    }

    // Associativity, with log-append associativity doing the Writer-monoid work
    let m = LawfulEff {
        result: Ok(3),
        logs: vec!["m".to_string()],
    };
    assert_eq!(
        LawfulMonadEffect3::bind(LawfulMonadEffect3::bind(m.clone(), f), g),
        LawfulMonadEffect3::bind(m, |t| LawfulMonadEffect3::bind(f(t), g))
    );

    // Raise is a left zero: f must NOT run on an errored carrier
    // (deviation D7, now also implemented by the reference carriers in utils_tests.rs)
    let mut ran = false;
    let errored = LawfulEff::<i32> {
        result: Err("e".to_string()),
        logs: vec![],
    };
    let out: LawfulEff<i32> = LawfulMonadEffect3::bind(errored.clone(), |t| {
        ran = true;
        LawfulEff {
            result: Ok(t),
            logs: vec![],
        }
    });
    assert!(
        !ran,
        "continuation must not run when the carrier holds an error"
    );
    assert_eq!(out, errored);
}
