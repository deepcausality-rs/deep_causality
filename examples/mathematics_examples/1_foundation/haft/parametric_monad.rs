/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::marker::PhantomData;

// ============================================================================
// Parametric Monad: Type-Safe State Machine
// ============================================================================

// ENGINEERING VALUE:
// Standard Monads (State<S, A>) assume the state type 'S' never changes.
// But in many systems (e.g., Protocols, Builders), the state TYPE changes.
// Uninitialized -> Authenticated -> Connected.
//
// Parametric Monad (Indexed Monad) allows the state type to evolve:
// bind: M<S1, S2, A> -> (A -> M<S2, S3, B>) -> M<S1, S3, B>
//
// This enforces the correct sequence of operations at COMPILE TIME.
// You cannot "Send Data" before "Connect".
fn main() {
    print_header();

    // The chain runs Disconnected -> Connected -> Authenticated -> Authenticated, and the
    // types line up at every step: S1=Disc/S2=Conn, then S2=Conn/S3=Auth, then S3=Auth/S4=Auth.
    let t1 = connect();
    let t2 = ibind(t1, authenticate);
    let t3 = ibind(t2, send_data);

    print_result(t3.val);

    // COMPILE-TIME SAFETY:
    // Calling `send_data` straight after `connect` does not compile: `connect` returns
    // Post=Connected and `send_data` expects Pre=Authenticated.
}

/// Step 1: Disconnected -> Connected.
fn connect() -> Transition<Disconnected, Connected, String> {
    print_action_connect();
    Transition::new("Connection_ID_123".to_string())
}

/// Step 2: Connected -> Authenticated.
fn authenticate(conn_id: String) -> Transition<Connected, Authenticated, String> {
    print_action_authenticate(&conn_id);
    Transition::new("User_Session_99".to_string())
}

/// Step 3: Authenticated -> Authenticated, yielding the bytes sent.
fn send_data(session: String) -> Transition<Authenticated, Authenticated, usize> {
    print_action_send(&session);
    Transition::new(1024)
}

// States
#[derive(Debug)]
struct Disconnected;
#[derive(Debug)]
struct Connected;
#[derive(Debug)]
struct Authenticated;

// The Parametric Monad: Transition<Pre, Post, Val>
// Represents a computation starting in 'Pre' state, ending in 'Post' state, yielding 'Val'.
struct Transition<Pre, Post, Val> {
    val: Val,
    _phantom: PhantomData<(Pre, Post)>,
}

impl<Pre, Post, Val> Transition<Pre, Post, Val> {
    fn new(val: Val) -> Self {
        Self {
            val,
            _phantom: PhantomData,
        }
    }
}

// Helper to simulate Parametric Bind
// (Real trait would be ParametricMonad::ibind)
fn ibind<S1, S2, S3, A, B, F>(m: Transition<S1, S2, A>, f: F) -> Transition<S1, S3, B>
where
    F: Fn(A) -> Transition<S2, S3, B>,
{
    let next = f(m.val);
    Transition::new(next.val)
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== DeepCausality HKT: Parametric Monad Pattern ===\n");
    println!("--- Protocol State Machine ---");
}

fn print_action_connect() {
    println!("Action: Connecting...");
}

fn print_action_authenticate(conn_id: &str) {
    println!("Action: Authenticating {conn_id}...");
}

fn print_action_send(session: &str) {
    println!("Action: Sending data via {session}...");
}

fn print_result(bytes: usize) {
    println!("Final Result: {bytes} bytes sent");
}
