/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Unique paths for scratch entries under the OS temp directory.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{env, path, process};

static COUNTER: AtomicU64 = AtomicU64::new(0);
static KEYS: OnceLock<RandomState> = OnceLock::new();

/// Returns [`temp_path_in`] of `std::env::temp_dir()`.
pub(crate) fn temp_path(suffix: &str) -> io::Result<PathBuf> {
    temp_path_in(&env::temp_dir(), suffix)
}

/// Returns `base`, made absolute, joined with `.dct-{pid}-{random}-{counter}{suffix}`.
///
/// `pid` is the process id and `counter` a process-wide count incremented on every call; together
/// they keep names distinct within a process and across live processes. `random` is the counter
/// hashed with SipHash under one `RandomState` per process, printed as 16 hex digits. std draws
/// that state's 128-bit key from the OS random source, so another process cannot predict a name
/// and create it first. A relative `base`
/// is resolved against the current directory. The caller validates `suffix`.
///
/// # Errors
///
/// Returns the error of `std::path::absolute`: an empty `base`, or an unreadable current
/// directory when `base` is relative.
pub(crate) fn temp_path_in(base: &Path, suffix: &str) -> io::Result<PathBuf> {
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut hasher = KEYS.get_or_init(RandomState::new).build_hasher();
    hasher.write_u64(counter);
    let random = hasher.finish();
    let name = format!(".dct-{}-{random:016x}-{counter}{suffix}", process::id());
    Ok(path::absolute(base)?.join(name))
}

#[cfg(test)]
mod tests {
    use super::{temp_path, temp_path_in};
    use std::collections::HashSet;
    use std::io::ErrorKind;
    use std::path::Path;
    use std::{env, process};

    // Splits `.dct-{pid}-{random}-{counter}` (no suffix) into its three numbers.
    fn fields(name: &str) -> (u32, u64, u64) {
        let rest = name.strip_prefix(".dct-").expect("prefix");
        let parts: Vec<&str> = rest.split('-').collect();
        assert_eq!(parts.len(), 3, "{name}");
        assert_eq!(parts[1].len(), 16, "{name}");
        (
            parts[0].parse().unwrap(),
            u64::from_str_radix(parts[1], 16).unwrap(),
            parts[2].parse().unwrap(),
        )
    }

    fn name_of(suffix: &str) -> String {
        let p = temp_path(suffix).unwrap();
        assert_eq!(p.parent(), Some(env::temp_dir().as_path()));
        p.file_name().unwrap().to_str().unwrap().to_string()
    }

    #[test]
    fn name_carries_the_process_id() {
        let (pid, _, _) = fields(&name_of(""));
        assert_eq!(pid, process::id());
    }

    #[test]
    fn counter_strictly_increases_within_a_thread() {
        let (_, _, first) = fields(&name_of(""));
        let (_, _, second) = fields(&name_of(""));
        assert!(second > first, "{first} then {second}");
    }

    #[test]
    fn random_field_differs_across_thousand_names() {
        // Oracle: 1000 draws of 64 uniform bits collide with probability below 1e-13.
        let randoms: HashSet<u64> = (0..1000).map(|_| fields(&name_of("")).1).collect();
        assert_eq!(randoms.len(), 1000);
    }

    #[test]
    fn suffix_is_appended_after_the_counter() {
        let name = name_of(".sp3");
        let stem = name.strip_suffix(".sp3").expect("suffix at the end");
        fields(stem);
    }

    #[test]
    fn thousand_back_to_back_paths_are_distinct() {
        let paths: HashSet<_> = (0..1000).map(|_| temp_path(".csv").unwrap()).collect();
        assert_eq!(paths.len(), 1000);
    }

    #[test]
    fn relative_base_is_resolved_against_the_current_directory() {
        let p = temp_path_in(Path::new("rel_base"), "").unwrap();
        assert!(p.is_absolute(), "{}", p.display());
        let expected = env::current_dir().unwrap().join("rel_base");
        assert_eq!(p.parent(), Some(expected.as_path()));
    }

    #[test]
    fn absolute_base_is_kept() {
        let base = env::temp_dir();
        let p = temp_path_in(&base, "").unwrap();
        assert_eq!(p.parent(), Some(base.as_path()));
    }

    #[test]
    fn empty_base_is_an_error() {
        let err = temp_path_in(Path::new(""), "").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
    }
}
