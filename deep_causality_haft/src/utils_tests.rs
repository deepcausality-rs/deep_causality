/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Effect3, Effect4, Effect5, Functor, Monad, MonadEffect3, MonadEffect4, MonadEffect5};
use crate::{HKT, HKT3, HKT4, HKT5, Placeholder};

// --- MonadEffect3 Setup ---

#[derive(Debug, PartialEq)]
pub struct MyCustomEffectType<T, E, W> {
    pub value: T,
    pub error: Option<E>,
    pub warnings: Vec<W>,
}

pub struct MyEffectHktWitness<E, W>(Placeholder, E, W);

impl<E, W> HKT for MyEffectHktWitness<E, W> {
    type Type<T> = MyCustomEffectType<T, E, W>;
}

impl<E, W> HKT3<E, W> for MyEffectHktWitness<E, W> {
    type Type<T> = MyCustomEffectType<T, E, W>;
}

pub struct MyEffect;

impl Effect3 for MyEffect {
    type Fixed1 = String;
    type Fixed2 = String;
    type HktWitness = MyEffectHktWitness<Self::Fixed1, Self::Fixed2>;
}

impl Functor<MyEffectHktWitness<String, String>> for MyEffectHktWitness<String, String> {
    fn fmap<A, B, Func>(
        m_a: MyCustomEffectType<A, String, String>,
        f: Func,
    ) -> MyCustomEffectType<B, String, String>
    where
        Func: FnOnce(A) -> B,
    {
        MyCustomEffectType {
            value: f(m_a.value),
            error: m_a.error,
            warnings: m_a.warnings,
        }
    }
}

impl Monad<MyEffectHktWitness<String, String>> for MyEffectHktWitness<String, String> {
    fn pure<T>(value: T) -> MyCustomEffectType<T, String, String> {
        MyCustomEffectType {
            value,
            error: None,
            warnings: Vec::new(),
        }
    }

    fn bind<A, B, Func>(
        m_a: MyCustomEffectType<A, String, String>,
        f: Func,
    ) -> MyCustomEffectType<B, String, String>
    where
        Func: FnOnce(A) -> MyCustomEffectType<B, String, String>,
    {
        if m_a.error.is_some() {
            return MyCustomEffectType {
                value: f(m_a.value).value,
                error: m_a.error,
                warnings: m_a.warnings,
            };
        }
        let mut next_effect = f(m_a.value);
        let mut combined_warnings = m_a.warnings;
        combined_warnings.extend(next_effect.warnings);
        next_effect.warnings = combined_warnings;
        next_effect
    }
}

pub struct MyMonadEffect3;

impl MonadEffect3<MyEffect> for MyMonadEffect3 {
    fn pure<T>(
        value: T,
    ) -> <<MyEffect as Effect3>::HktWitness as HKT3<
        <MyEffect as Effect3>::Fixed1,
        <MyEffect as Effect3>::Fixed2,
    >>::Type<T> {
        <MyEffect as Effect3>::HktWitness::pure(value)
    }

    fn bind<T, U, Func>(
        effect: <<MyEffect as Effect3>::HktWitness as HKT3<
            <MyEffect as Effect3>::Fixed1,
            <MyEffect as Effect3>::Fixed2,
        >>::Type<T>,
        f: Func,
    ) -> <<MyEffect as Effect3>::HktWitness as HKT3<
        <MyEffect as Effect3>::Fixed1,
        <MyEffect as Effect3>::Fixed2,
    >>::Type<U>
    where
        Func: FnMut(
            T,
        ) -> <<MyEffect as Effect3>::HktWitness as HKT3<
            <MyEffect as Effect3>::Fixed1,
            <MyEffect as Effect3>::Fixed2,
        >>::Type<U>,
    {
        <MyEffect as Effect3>::HktWitness::bind(effect, f)
    }
}

pub trait LoggableEffect3<E: Effect3>: MonadEffect3<E>
where
    E::HktWitness: Functor<E::HktWitness> + Sized,
{
    fn log<T>(
        effect: <E::HktWitness as HKT3<E::Fixed1, E::Fixed2>>::Type<T>,
        log: E::Fixed2,
    ) -> <E::HktWitness as HKT3<E::Fixed1, E::Fixed2>>::Type<T>;
}

impl LoggableEffect3<MyEffect> for MyMonadEffect3 {
    fn log<T>(
        mut effect: <<MyEffect as Effect3>::HktWitness as HKT3<
            <MyEffect as Effect3>::Fixed1,
            <MyEffect as Effect3>::Fixed2,
        >>::Type<T>,
        log: <MyEffect as Effect3>::Fixed2,
    ) -> <<MyEffect as Effect3>::HktWitness as HKT3<
        <MyEffect as Effect3>::Fixed1,
        <MyEffect as Effect3>::Fixed2,
    >>::Type<T> {
        effect.warnings.push(log);
        effect
    }
}

// --- MonadEffect4 Setup ---

#[derive(Debug, PartialEq)]
pub struct MyCustomEffectType4<T, F1, F2, F3> {
    pub value: T,
    pub f1: Option<F1>,
    pub f2: Vec<F2>,
    pub f3: Vec<F3>,
}

pub struct MyEffectHktWitness4<F1, F2, F3>(Placeholder, F1, F2, F3);

impl<F1, F2, F3> HKT for MyEffectHktWitness4<F1, F2, F3> {
    type Type<T> = MyCustomEffectType4<T, F1, F2, F3>;
}

impl<F1, F2, F3> HKT4<F1, F2, F3> for MyEffectHktWitness4<F1, F2, F3> {
    type Type<T> = MyCustomEffectType4<T, F1, F2, F3>;
}

pub struct MyEffect4;

impl Effect4 for MyEffect4 {
    type Fixed1 = String; // Error
    type Fixed2 = String; // Log
    type Fixed3 = u64; // Counter
    type HktWitness = MyEffectHktWitness4<Self::Fixed1, Self::Fixed2, Self::Fixed3>;
}

impl Functor<MyEffectHktWitness4<String, String, u64>>
    for MyEffectHktWitness4<String, String, u64>
{
    fn fmap<A, B, Func>(
        m_a: MyCustomEffectType4<A, String, String, u64>,
        f: Func,
    ) -> MyCustomEffectType4<B, String, String, u64>
    where
        Func: FnOnce(A) -> B,
    {
        MyCustomEffectType4 {
            value: f(m_a.value),
            f1: m_a.f1,
            f2: m_a.f2,
            f3: m_a.f3,
        }
    }
}

impl Monad<MyEffectHktWitness4<String, String, u64>> for MyEffectHktWitness4<String, String, u64> {
    fn pure<T>(value: T) -> MyCustomEffectType4<T, String, String, u64> {
        MyCustomEffectType4 {
            value,
            f1: None,
            f2: Vec::new(),
            f3: Vec::new(),
        }
    }

    fn bind<A, B, Func>(
        m_a: MyCustomEffectType4<A, String, String, u64>,
        f: Func,
    ) -> MyCustomEffectType4<B, String, String, u64>
    where
        Func: FnOnce(A) -> MyCustomEffectType4<B, String, String, u64>,
    {
        if m_a.f1.is_some() {
            return MyCustomEffectType4 {
                value: f(m_a.value).value, // Need a value of type B
                f1: m_a.f1,
                f2: m_a.f2,
                f3: m_a.f3,
            };
        }
        let mut next_effect = f(m_a.value);
        let mut combined_f2 = m_a.f2;
        combined_f2.extend(next_effect.f2);
        next_effect.f2 = combined_f2;

        let mut combined_f3 = m_a.f3;
        combined_f3.extend(next_effect.f3);
        next_effect.f3 = combined_f3;

        next_effect
    }
}

pub struct MyMonadEffect4;

impl MonadEffect4<MyEffect4> for MyMonadEffect4 {
    fn pure<T>(
        value: T,
    ) -> <<MyEffect4 as Effect4>::HktWitness as HKT4<
        <MyEffect4 as Effect4>::Fixed1,
        <MyEffect4 as Effect4>::Fixed2,
        <MyEffect4 as Effect4>::Fixed3,
    >>::Type<T> {
        <MyEffect4 as Effect4>::HktWitness::pure(value)
    }

    fn bind<T, U, Func>(
        effect: <<MyEffect4 as Effect4>::HktWitness as HKT4<
            <MyEffect4 as Effect4>::Fixed1,
            <MyEffect4 as Effect4>::Fixed2,
            <MyEffect4 as Effect4>::Fixed3,
        >>::Type<T>,
        f: Func,
    ) -> <<MyEffect4 as Effect4>::HktWitness as HKT4<
        <MyEffect4 as Effect4>::Fixed1,
        <MyEffect4 as Effect4>::Fixed2,
        <MyEffect4 as Effect4>::Fixed3,
    >>::Type<U>
    where
        Func: FnMut(
            T,
        ) -> <<MyEffect4 as Effect4>::HktWitness as HKT4<
            <MyEffect4 as Effect4>::Fixed1,
            <MyEffect4 as Effect4>::Fixed2,
            <MyEffect4 as Effect4>::Fixed3,
        >>::Type<U>,
    {
        <MyEffect4 as Effect4>::HktWitness::bind(effect, f)
    }
}

pub trait LoggableEffect4<E: Effect4>: MonadEffect4<E>
where
    E::HktWitness: Functor<E::HktWitness> + Sized,
{
    fn log<T>(
        effect: <E::HktWitness as HKT4<E::Fixed1, E::Fixed2, E::Fixed3>>::Type<T>,
        log: E::Fixed3,
    ) -> <E::HktWitness as HKT4<E::Fixed1, E::Fixed2, E::Fixed3>>::Type<T>;
}

impl LoggableEffect4<MyEffect4> for MyMonadEffect4 {
    fn log<T>(
        mut effect: <<MyEffect4 as Effect4>::HktWitness as HKT4<
            <MyEffect4 as Effect4>::Fixed1,
            <MyEffect4 as Effect4>::Fixed2,
            <MyEffect4 as Effect4>::Fixed3,
        >>::Type<T>,
        log: <MyEffect4 as Effect4>::Fixed3,
    ) -> <<MyEffect4 as Effect4>::HktWitness as HKT4<
        <MyEffect4 as Effect4>::Fixed1,
        <MyEffect4 as Effect4>::Fixed2,
        <MyEffect4 as Effect4>::Fixed3,
    >>::Type<T> {
        effect.f3.push(log);
        effect
    }
}

// --- MonadEffect5 Setup ---

#[derive(Debug, PartialEq)]
pub struct MyCustomEffectType5<T, F1, F2, F3, F4> {
    pub value: T,
    pub f1: Option<F1>,
    pub f2: Vec<F2>,
    pub f3: Vec<F3>,
    pub f4: Vec<F4>,
}

pub struct MyEffectHktWitness5<F1, F2, F3, F4>(Placeholder, F1, F2, F3, F4);

impl<F1, F2, F3, F4> HKT for MyEffectHktWitness5<F1, F2, F3, F4> {
    type Type<T> = MyCustomEffectType5<T, F1, F2, F3, F4>;
}

impl<F1, F2, F3, F4> HKT5<F1, F2, F3, F4> for MyEffectHktWitness5<F1, F2, F3, F4> {
    type Type<T> = MyCustomEffectType5<T, F1, F2, F3, F4>;
}

pub struct MyEffect5;

impl Effect5 for MyEffect5 {
    type Fixed1 = String;
    type Fixed2 = String;
    type Fixed3 = u64;
    type Fixed4 = String;
    type HktWitness = MyEffectHktWitness5<Self::Fixed1, Self::Fixed2, Self::Fixed3, Self::Fixed4>;
}

impl Functor<MyEffectHktWitness5<String, String, u64, String>>
    for MyEffectHktWitness5<String, String, u64, String>
{
    fn fmap<A, B, Func>(
        m_a: MyCustomEffectType5<A, String, String, u64, String>,
        f: Func,
    ) -> MyCustomEffectType5<B, String, String, u64, String>
    where
        Func: FnOnce(A) -> B,
    {
        MyCustomEffectType5 {
            value: f(m_a.value),
            f1: m_a.f1,
            f2: m_a.f2,
            f3: m_a.f3,
            f4: m_a.f4,
        }
    }
}

impl Monad<MyEffectHktWitness5<String, String, u64, String>>
    for MyEffectHktWitness5<String, String, u64, String>
{
    fn pure<T>(value: T) -> MyCustomEffectType5<T, String, String, u64, String> {
        MyCustomEffectType5 {
            value,
            f1: None,
            f2: Vec::new(),
            f3: Vec::new(),
            f4: Vec::new(),
        }
    }

    fn bind<A, B, Func>(
        m_a: MyCustomEffectType5<A, String, String, u64, String>,
        f: Func,
    ) -> MyCustomEffectType5<B, String, String, u64, String>
    where
        Func: FnOnce(A) -> MyCustomEffectType5<B, String, String, u64, String>,
    {
        if m_a.f1.is_some() {
            return MyCustomEffectType5 {
                value: f(m_a.value).value, // Need a value of type B
                f1: m_a.f1,
                f2: m_a.f2,
                f3: m_a.f3,
                f4: m_a.f4,
            };
        }
        let mut next_effect = f(m_a.value);

        let mut combined_f2 = m_a.f2;
        combined_f2.extend(next_effect.f2);
        next_effect.f2 = combined_f2;

        let mut combined_f3 = m_a.f3;
        combined_f3.extend(next_effect.f3);
        next_effect.f3 = combined_f3;

        let mut combined_f4 = m_a.f4;
        combined_f4.extend(next_effect.f4);
        next_effect.f4 = combined_f4;

        next_effect
    }
}

pub struct MyMonadEffect5;

impl MonadEffect5<MyEffect5> for MyMonadEffect5 {
    fn pure<T>(
        value: T,
    ) -> <<MyEffect5 as Effect5>::HktWitness as HKT5<
        <MyEffect5 as Effect5>::Fixed1,
        <MyEffect5 as Effect5>::Fixed2,
        <MyEffect5 as Effect5>::Fixed3,
        <MyEffect5 as Effect5>::Fixed4,
    >>::Type<T> {
        <MyEffect5 as Effect5>::HktWitness::pure(value)
    }

    fn bind<T, U, Func>(
        effect: <<MyEffect5 as Effect5>::HktWitness as HKT5<
            <MyEffect5 as Effect5>::Fixed1,
            <MyEffect5 as Effect5>::Fixed2,
            <MyEffect5 as Effect5>::Fixed3,
            <MyEffect5 as Effect5>::Fixed4,
        >>::Type<T>,
        f: Func,
    ) -> <<MyEffect5 as Effect5>::HktWitness as HKT5<
        <MyEffect5 as Effect5>::Fixed1,
        <MyEffect5 as Effect5>::Fixed2,
        <MyEffect5 as Effect5>::Fixed3,
        <MyEffect5 as Effect5>::Fixed4,
    >>::Type<U>
    where
        Func: FnMut(
            T,
        ) -> <<MyEffect5 as Effect5>::HktWitness as HKT5<
            <MyEffect5 as Effect5>::Fixed1,
            <MyEffect5 as Effect5>::Fixed2,
            <MyEffect5 as Effect5>::Fixed3,
            <MyEffect5 as Effect5>::Fixed4,
        >>::Type<U>,
    {
        <MyEffect5 as Effect5>::HktWitness::bind(effect, f)
    }
}

pub trait LoggableEffect5<E: Effect5>: MonadEffect5<E>
where
    E::HktWitness: Functor<E::HktWitness> + Sized,
{
    #[allow(clippy::type_complexity)]
    fn log<T>(
        effect: <E::HktWitness as HKT5<E::Fixed1, E::Fixed2, E::Fixed3, E::Fixed4>>::Type<T>,
        log: E::Fixed4,
    ) -> <E::HktWitness as HKT5<E::Fixed1, E::Fixed2, E::Fixed3, E::Fixed4>>::Type<T>;
}

impl LoggableEffect5<MyEffect5> for MyMonadEffect5 {
    #[allow(clippy::type_complexity)]
    fn log<T>(
        mut effect: <<MyEffect5 as Effect5>::HktWitness as HKT5<
            <MyEffect5 as Effect5>::Fixed1,
            <MyEffect5 as Effect5>::Fixed2,
            <MyEffect5 as Effect5>::Fixed3,
            <MyEffect5 as Effect5>::Fixed4,
        >>::Type<T>,
        log: <MyEffect5 as Effect5>::Fixed4,
    ) -> <<MyEffect5 as Effect5>::HktWitness as HKT5<
        <MyEffect5 as Effect5>::Fixed1,
        <MyEffect5 as Effect5>::Fixed2,
        <MyEffect5 as Effect5>::Fixed3,
        <MyEffect5 as Effect5>::Fixed4,
    >>::Type<T> {
        effect.f4.push(log);
        effect
    }
}
