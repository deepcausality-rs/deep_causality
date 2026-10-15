<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Group 1: ultragraph_traits and the EdgeKey bound

Change `ultragraph-stable-ids-keyed-edges`, design D8. The five phases of
`openspec/specs/unified-math-tdd-protocol/spec.md`, recorded as they ran on 2026-10-15.

## Corner cases A–K

One test named per row, or `n/a` with the reason.

| # | Class | Instance in this group | Test |
|---|---|---|---|
| A | Empty | n/a: the group adds a trait and its impls, no collection | — |
| B | Single element | One impl per primitive type, each called on its own | `test_<type>_has_the_unit_key`, sixteen tests in `ultragraph_traits/tests/extensions/edge_key_primitives_tests.rs` |
| C | Coinciding quantities | `()`, where the key equals the payload; also a payload whose key differs from the payload (a kind plus a weight) | `test_unit_has_the_unit_key`; `test_edge_key_ignores_the_data` in `tests/traits/edge_key_tests.rs` |
| D | Degenerate index | n/a: no index | — |
| E | Thresholds, both sides | n/a: no threshold | — |
| F | Zero | `0` of every numeric type, `0.0` and `-0.0` of both floats, `false` | the sixteen primitive tests |
| G | Negative | `MIN` of every signed type, `-1.5` of both floats | the signed and float primitive tests |
| H | Exact boundary | `MAX` of every integer type, `MIN` of the signed ones | the integer primitive tests |
| I | Non-finite | `NAN`, `INFINITY`, `NEG_INFINITY` of both floats | `test_f32_has_the_unit_key`, `test_f64_has_the_unit_key` |
| J | Overflow reach | n/a: no arithmetic | — |
| K | Every payload kind | `()`; a numeric weight; a kind key (`RelationKind`, key = itself); an enum with the unit key (`TeloidRelation`); a struct keyed by one field (`Edge` in the trait tests) | the primitive tests; `edge_key_tests.rs` in the store, ethos and `ultragraph_traits` suites |

## Phase 1 — the API, every new body `unimplemented!()`

New crate `ultragraph_traits` at the repository root: `EdgeKey { type Key: Copy + Ord; fn edge_key(&self) -> Self::Key; }`
in `src/traits/edge_key.rs`; sixteen impls (`()`, `bool`, the twelve integer types, `f32`, `f64`)
in `src/extensions/edge_key_primitives.rs`, each body `unimplemented!()`. Workspace member;
`[workspace.dependencies]` entry at `0.1`, `default-features = false`; `ultragraph` depends on it
and re-exports `EdgeKey` at its root.

In the calling crates, the stubs the protocol asks for: `impl EdgeKey for RelationKind` in
`deep_causality_context_store/src/types/relation_kind/edge_key.rs` and `impl EdgeKey for
TeloidRelation` in `deep_causality_ethos/src/types/teloid_relation/edge_key.rs`, both bodies
`unimplemented!()`. `RelationKind`'s order cannot be stubbed as a derive, so phase 1 landed a manual
`Ord` with an `unimplemented!()` body in `relation_kind/ord.rs`; phase 4 fills it.

Exit: `cargo build -p ultragraph_traits -p ultragraph` and `bazel build //ultragraph_traits
//ultragraph //deep_causality_context_store //deep_causality_ethos` succeed. One ordering fact:
Bazel refuses an empty `glob` in a `rust_test_suite`, so the test files of phase 2 had to exist
before the Bazel build of phase 1 could pass.

## Phase 2 — the suite, written first, observed failing

| Scenario | Test |
|---|---|
| The manifest declares nothing | no test can read a manifest without scraping it; recorded instead: `cargo tree -p ultragraph_traits -e normal` prints the crate alone, and `[lints] workspace = true` is in its `Cargo.toml` |
| One trait under two paths | `ultragraph/tests/traits/edge_key_tests.rs::test_one_trait_under_two_paths`: an impl of `ultragraph_traits::EdgeKey` satisfies a bound on `ultragraph::EdgeKey` |
| The crate builds without std | `make check_no_std`: "All 27 no-std crates build for thumbv7em-none-eabihf" (26 before this group) |
| The unit and primitive numeric types take an empty key | sixteen tests, each with the compile-time pin `unit_key::<T: EdgeKey<Key = ()>>` and a `let () =` binding per value |
| The key is the relation | `deep_causality_context_store/tests/types/relation_kind/edge_key_tests.rs`, two tests |
| The order is declaration order | `.../relation_kind/ord_tests.rs`, three tests; the expected order is written out as a constant, not derived from the enum |
| The tier block records the position | a documentation change; `AGENTS.md` diff |
| `TeloidRelation` has the unit key | `deep_causality_ethos/tests/types/teloid_relation/edge_key_tests.rs` |

Exit run against the stubs: **22 failed, 7 passed**. Every failure is the `not implemented` panic:
16 in `ultragraph_traits`, 5 in the store (`edge_key_tests`, `ord_tests`), 1 in ethos. Four of the passes
are the three trait tests, whose impl is a local type written inside the test, and the two-paths test
in `ultragraph`, for the same reason; they pin the trait's declared shape, which phase 1 provides, and
cannot fail on a body. The other three were not itemised.

## Phase 3 — the defect audit

The throwaway was the candidate implementation itself; each defect was injected, the targeted suite
run, and the file restored before the next (`audit_g1.py`, scratch).

| Defect | Class | Caught by |
|---|---|---|
| D1 `impl EdgeKey for u16` removed | guard removed (a missing impl) | compile error `u16: EdgeKey` not satisfied at `test_u16_has_the_unit_key` |
| D2 `u64` keyed by its own value (`Key = u64`) | plausible neighbour | compile error `<u64 as EdgeKey>::Key == ()` at `test_u64_has_the_unit_key` |
| D3 `RelationKind::edge_key` returns `Datial` for every variant | plausible neighbour | `test_the_key_is_the_relation`, `test_distinct_relations_have_distinct_keys` |
| D4 `Spatial` keyed as `Datial`, the rest as themselves | early return skipping a case | the same two tests |
| D5 the order reversed | flipped comparison | the three `ord_tests` |
| D6 the order rotated by one | off-by-one | the three `ord_tests` |

Not admitted by this group's code: a constant factor, a tolerance (no arithmetic, no tolerance).
`TeloidRelation::edge_key` returns `()` and admits no defect other than a panic. Input variety:
row C has both the coinciding case (`()`) and the differing one (a kind plus a weight).

## Phase 4 — implementation against the audited suite

Bodies: the primitives return `()`; `RelationKind::edge_key` returns `*self`; `Ord` compares the
`#[repr(u8)]` discriminants, which is declaration order; `TeloidRelation::edge_key` returns `()`.
The `W: Clone + Default + EdgeKey` bound is on `UltraGraphContainer`, `GraphState`, `CsmGraph`,
`CsrAdjacency`, `DynamicGraph` (struct and every impl; the struct carried no bound before) and the
`Freezable` and `Unfreezable` traits. Clippy's `unit_cmp` refused `assert_eq!` on unit values, so the
value pins are `let () = x.edge_key()` bindings.

Every added body is called by a named test: the sixteen primitive impls, the four `RelationKind`
variants through `edge_key` and every ordered pair through `cmp`, both `TeloidRelation` variants.
No coverage tool was run.

Executed counts, Cargo against Bazel (`tests(//<crate>/...)` targets of the same run):

| Crate | Cargo | Bazel |
|---|---|---|
| `ultragraph_traits` | 19 | 19 |
| `ultragraph` | 132 | 132 |
| `deep_causality_context_store` | 171 | 171 |
| `deep_causality_context` | 798 | 798 |
| `deep_causality` | 704 | 702 |
| `deep_causality_ethos` | 129 | 129 |
| `deep_causality_quantum` | 871 | 932 |

The two mismatches are in crates whose tests this group did not touch, and the Cargo figure sums
the doc-test line as well; they are not this group's doing and were not investigated here.

## Phase 5 — mutation testing

`scripts/mutants.sh ultragraph_traits`: 0 mutants (bodies returning `()` offer nothing to mutate).
`deep_causality_context_store src/types/relation_kind/edge_key.rs`: 1 mutant, caught.
`.../relation_kind/ord.rs`: 3 mutants, 1 caught, 2 unviable. `deep_causality_ethos
src/types/teloid_relation/edge_key.rs`: 0 mutants. No survivor; nothing added to
`.cargo/mutants.toml`.

## Close-out checks

`make format && make fix`: clean. `make check_no_std`: 27 crates build. `cargo publish -p
ultragraph_traits --dry-run --allow-dirty`: passes. `make check_publishable` stops at the
uncommitted `Cargo.toml`, as it does for any change before its commit.
