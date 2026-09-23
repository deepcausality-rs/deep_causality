/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Unique paths for scratch entries under the OS temp directory.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, process};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Returns `std::env::temp_dir()` joined with `.dct-{pid}-{nanos}-{counter}{suffix}`.
///
/// `pid` is the process id, `nanos` the wall clock in nanoseconds since the Unix epoch, and
/// `counter` a process-wide count incremented on every call. The pid and counter keep names
/// distinct across live processes and within one process; the clock separates a process from an
/// earlier one that had the same pid. A clock set before the epoch reads as 0. The caller
/// validates `suffix`.
pub(crate) fn temp_path(suffix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    env::temp_dir().join(format!(".dct-{}-{nanos}-{counter}{suffix}", process::id()))
}

#[cfg(test)]
mod tests {
    use super::temp_path;
    use std::collections::HashSet;
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::{env, process};

    fn now_nanos() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }

    // Splits `.dct-{pid}-{nanos}-{counter}` (no suffix) into its three numbers.
    fn fields(name: &str) -> (u32, u128, u64) {
        let rest = name.strip_prefix(".dct-").expect("prefix");
        let parts: Vec<&str> = rest.split('-').collect();
        assert_eq!(parts.len(), 3, "{name}");
        (
            parts[0].parse().unwrap(),
            parts[1].parse().unwrap(),
            parts[2].parse().unwrap(),
        )
    }

    fn name_of(suffix: &str) -> String {
        let p = temp_path(suffix);
        assert_eq!(p.parent(), Some(env::temp_dir().as_path()));
        p.file_name().unwrap().to_str().unwrap().to_string()
    }

    #[test]
    fn name_carries_pid_clock_and_counter() {
        let before = now_nanos();
        let (pid, nanos, _) = fields(&name_of(""));
        let after = now_nanos();
        // Oracle: the process id from std, and the clock read by the test around the call.
        assert_eq!(pid, process::id());
        assert!(
            before <= nanos && nanos <= after,
            "{before} <= {nanos} <= {after}"
        );
    }

    #[test]
    fn counter_strictly_increases_within_a_thread() {
        let (_, _, first) = fields(&name_of(""));
        let (_, _, second) = fields(&name_of(""));
        assert!(second > first, "{first} then {second}");
    }

    #[test]
    fn suffix_is_appended_after_the_counter() {
        let name = name_of(".sp3");
        let stem = name.strip_suffix(".sp3").expect("suffix at the end");
        fields(stem);
    }

    #[test]
    fn thousand_back_to_back_paths_are_distinct() {
        let paths: HashSet<_> = (0..1000).map(|_| temp_path(".csv")).collect();
        assert_eq!(paths.len(), 1000);
    }
}
