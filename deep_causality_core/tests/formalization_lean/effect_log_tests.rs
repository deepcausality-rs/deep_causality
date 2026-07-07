/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the free-monoid (Writer output) laws of `EffectLog`.
//!
//! Mirrors `lean/DeepCausalityFormal/Core/EffectLog.lean`. The Lean proof abstracts the log as
//! `List Λ` (timestamps quotiented away, exactly as `EffectLog`'s `PartialEq` does); these
//! witnesses check the same four laws on the real `EffectLog`/`LogAppend` at representative
//! inputs. One `#[test]` per THEOREM_MAP id.
//!
//! `LogAppend::append(&mut self, other)` is the monoid operation (`Vec::append` — order-preserving
//! concatenation, no dedup/cap/reorder); `EffectLog::new()` is the identity; `==` compares the
//! message sequence (`log_effect.rs` `PartialEq`).

use deep_causality_core::EffectLog;
use deep_causality_haft::{LogAddEntry, LogAppend};

/// Build a log with one entry per message, in order.
fn log(messages: &[&str]) -> EffectLog {
    let mut l = EffectLog::new();
    for m in messages {
        l.add_entry(m);
    }
    l
}

// ---- core.effect_log.left_id : append empty x = x ----------------------------------------------

/// THEOREM_MAP: core.effect_log.left_id
#[test]
fn test_effect_log_left_identity() {
    let mut lhs = EffectLog::new(); // empty
    let mut x = log(&["a", "b"]);
    lhs.append(&mut x); // empty ++ x
    assert_eq!(lhs, log(&["a", "b"]));
}

// ---- core.effect_log.right_id : append x empty = x ---------------------------------------------

/// THEOREM_MAP: core.effect_log.right_id
#[test]
fn test_effect_log_right_identity() {
    let mut lhs = log(&["a", "b"]);
    let mut empty = EffectLog::new();
    lhs.append(&mut empty); // x ++ empty
    assert_eq!(lhs, log(&["a", "b"]));
}

// ---- core.effect_log.assoc : append (append x y) z = append x (append y z) ---------------------

/// THEOREM_MAP: core.effect_log.assoc
#[test]
fn test_effect_log_associativity() {
    // (x ++ y) ++ z
    let mut left = log(&["a"]);
    let mut y1 = log(&["b"]);
    let mut z1 = log(&["c"]);
    left.append(&mut y1);
    left.append(&mut z1);

    // x ++ (y ++ z)
    let mut right = log(&["a"]);
    let mut yz = log(&["b"]);
    let mut z2 = log(&["c"]);
    yz.append(&mut z2);
    right.append(&mut yz);

    assert_eq!(left, right);
    assert_eq!(left, log(&["a", "b", "c"]));
}

// ---- core.effect_log.monotone : the incoming log is a prefix of the combined log ---------------

/// THEOREM_MAP: core.effect_log.monotone
#[test]
fn test_effect_log_monotone_prefix() {
    let mut combined = log(&["a", "b"]);
    let mut tail = log(&["c", "d"]);
    combined.append(&mut tail); // self.logs ++ next.logs — self is a prefix

    let messages: Vec<&str> = combined.messages().collect();
    // The original incoming entries survive as a prefix (no audit history lost).
    assert_eq!(&messages[..2], &["a", "b"]);
    assert_eq!(messages.len(), 4);
}
