[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# DeepCausality Tempfile

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue

[crates-url]: https://crates.io/crates/deep_causality_tempfile

[docs-badge]: https://img.shields.io/badge/Docs.rs-Latest-blue

[docs-url]: https://docs.rs/deep_causality_tempfile/latest/deep_causality_tempfile/

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

The crate provides scratch files and directories for the DeepCausality test suites. It depends
only on `std`. It has two types:

* `TempDir` is a new, empty directory under `std::env::temp_dir()`. Dropping it removes the
  directory and everything inside it.
* `NamedTempFile` is a new, empty file under `std::env::temp_dir()`. You write to it through
  `std::io::Write`, and dropping it removes the file. `NamedTempFile::with_suffix(".csv")` ends
  the file name with an extension.

```rust
use std::io::Write;
use deep_causality_tempfile::{NamedTempFile, TempDir};

let dir = TempDir::new().unwrap();
std::fs::write(dir.path().join("table.csv"), b"a,b\n").unwrap();

let mut f = NamedTempFile::with_suffix(".csv").unwrap();
f.write_all(b"a,b\n1,2\n").unwrap();
f.flush().unwrap();
assert_eq!(std::fs::read(f.path()).unwrap(), b"a,b\n1,2\n");
```

Creation never opens a path that already exists. On Unix, files are created with mode `0o600` and
directories with mode `0o700`, before the umask applies. A suffix that contains `/` or `\` is rejected
with `ErrorKind::InvalidInput`.
