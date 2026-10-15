[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# ultragraph_traits

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue

[crates-url]: https://crates.io/crates/ultragraph_traits

[docs-badge]: https://img.shields.io/badge/Docs.rs-Latest-blue

[docs-url]: https://docs.rs/ultragraph_traits/latest/ultragraph_traits/

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

The `EdgeKey` trait of [ultragraph](https://crates.io/crates/ultragraph), in a crate of its own
so that a crate declaring an edge payload can implement it without depending on the graph.

An `ultragraph` edge is identified by its source, its target and the key its payload returns
from `edge_key`. Two `add_edge` calls with the same key on the same pair are one edge: the second
replaces the first's payload. Payloads with different keys are distinct edges between one pair.

`()`, `bool` and the primitive numeric types return the key `()`, so a graph whose payload is a
weight or nothing holds at most one edge per ordered pair.

```rust
use ultragraph_traits::EdgeKey;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Relation { Follows, Cites }

#[derive(Clone, Default)]
struct Edge { relation: Option<Relation>, weight: u32 }

impl EdgeKey for Edge {
    type Key = Option<Relation>;
    fn edge_key(&self) -> Self::Key { self.relation }
}
```

The crate has no dependencies and builds on `core` alone.

## License

MIT. See [LICENSE](LICENSE).
