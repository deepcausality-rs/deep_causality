/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{
    Effect3Unbound, Effect4Unbound, Effect5Unbound, HKT4Unbound, HKT5Unbound, HKT6Unbound,
    MonadEffect3Unbound, MonadEffect4Unbound, MonadEffect5Unbound,
};

// Mock Unbound Effect 3 (Error, S_in, S_out, A)
struct MockEffect3<E, S_in, S_out, A> {
    pub run: Box<dyn Fn(S_in) -> Result<(A, S_out), E>>,
}

struct MockEffect3Witness;
impl HKT4Unbound for MockEffect3Witness {
    type Type<E, S_in, S_out, A> = MockEffect3<E, S_in, S_out, A>;
}
impl Effect3Unbound for MockEffect3Witness {
    type Fixed1 = String; // Error type
    type HktWitness = MockEffect3Witness;
}
impl MonadEffect3Unbound<MockEffect3Witness> for MockEffect3Witness {
    fn pure<S, T>(value: T) -> MockEffect3<String, S, S, T>
    where T: 'static + Clone, S: 'static
    {
        MockEffect3 {
            run: Box::new(move |s| Ok((value.clone(), s))),
        }
    }

    fn ibind<S1, S2, S3, T, U, Func>(
        effect: MockEffect3<String, S1, S2, T>,
        f: Func,
    ) -> MockEffect3<String, S1, S3, U>
    where
        Func: FnMut(T) -> MockEffect3<String, S2, S3, U> + 'static,
        T: 'static, S1: 'static, S2: 'static, S3: 'static
    {
        use std::cell::RefCell;
        use std::rc::Rc;
        let f = Rc::new(RefCell::new(f));
        MockEffect3 {
            run: Box::new(move |s1| {
                match (effect.run)(s1) {
                    Ok((val, s2)) => {
                        let next = (f.borrow_mut())(val);
                        (next.run)(s2)
                    }
                    Err(e) => Err(e),
                }
            }),
        }
    }
}

#[test]
fn test_unbound_effect3() {
    // Step 1: i32 -> String
    let step1: MockEffect3<String, i32, String, i32> = MockEffect3 {
        run: Box::new(|s| Ok((s, s.to_string()))),
    };

    // Step 2: String -> bool
    let step2 = |val: i32| -> MockEffect3<String, String, bool, bool> {
        MockEffect3 {
            run: Box::new(move |s| Ok((val > 0, s.len() > 0))),
        }
    };

    let pipeline = MockEffect3Witness::ibind(step1, step2);
    let res = (pipeline.run)(10);
    assert_eq!(res.unwrap(), (true, true)); // 10 > 0, "10".len() > 0
}

// Mock Unbound Effect 4 (Error, Log, S_in, S_out, A)
struct MockEffect4<E, L, S_in, S_out, A> {
    pub run: Box<dyn Fn(S_in) -> Result<(A, Vec<L>, S_out), E>>,
}

struct MockEffect4Witness;
impl HKT5Unbound for MockEffect4Witness {
    type Type<E, L, S_in, S_out, A> = MockEffect4<E, L, S_in, S_out, A>;
}
impl Effect4Unbound for MockEffect4Witness {
    type Fixed1 = String; // Error
    type Fixed2 = String; // Log
    type HktWitness = MockEffect4Witness;
}
impl MonadEffect4Unbound<MockEffect4Witness> for MockEffect4Witness {
    fn pure<S, T>(value: T) -> MockEffect4<String, String, S, S, T>
    where T: 'static + Clone, S: 'static
    {
        MockEffect4 {
            run: Box::new(move |s| Ok((value.clone(), vec![], s))),
        }
    }

    fn ibind<S1, S2, S3, T, U, Func>(
        effect: MockEffect4<String, String, S1, S2, T>,
        f: Func,
    ) -> MockEffect4<String, String, S1, S3, U>
    where
        Func: FnMut(T) -> MockEffect4<String, String, S2, S3, U> + 'static,
        T: 'static, S1: 'static, S2: 'static, S3: 'static
    {
        use std::cell::RefCell;
        use std::rc::Rc;
        let f = Rc::new(RefCell::new(f));
        MockEffect4 {
            run: Box::new(move |s1| {
                match (effect.run)(s1) {
                    Ok((val, mut logs1, s2)) => {
                        let next = (f.borrow_mut())(val);
                        match (next.run)(s2) {
                            Ok((val2, logs2, s3)) => {
                                logs1.extend(logs2);
                                Ok((val2, logs1, s3))
                            }
                            Err(e) => Err(e),
                        }
                    }
                    Err(e) => Err(e),
                }
            }),
        }
    }
}

#[test]
fn test_unbound_effect4() {
    let step1: MockEffect4<String, String, i32, String, i32> = MockEffect4 {
        run: Box::new(|s| Ok((s, vec!["step1".to_string()], s.to_string()))),
    };

    let step2 = |val: i32| -> MockEffect4<String, String, String, bool, bool> {
        MockEffect4 {
            run: Box::new(move |s| Ok((val > 0, vec!["step2".to_string()], s.len() > 0))),
        }
    };

    let pipeline = MockEffect4Witness::ibind(step1, step2);
    let (res, logs, state) = (pipeline.run)(10).unwrap();
    
    assert_eq!(res, true);
    assert_eq!(state, true);
    assert_eq!(logs, vec!["step1", "step2"]);
}
