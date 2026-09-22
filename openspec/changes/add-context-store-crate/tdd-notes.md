# TDD protocol record: `deep_causality_context_store`

The record the `unified-math-tdd-protocol` asks each stage to keep. One section per task group,
one subsection per phase, with the observed output and counts.

## Task groups 3 and 4: aliases, constants, errors, records, traits, `block_on`

### Phase 1: API-only

Every source under `src/alias`, `src/constants`, `src/errors`, `src/types/records`,
`src/types/id_reserve`, `src/types/context_event`, `src/traits` and `src/utils_test/block_on.rs`
was landed with its full signature and every function body reduced to `unimplemented!()`. The
traits have no bodies to reduce. `cargo build -p deep_causality_context_store` succeeded; the only
warnings were unused parameters and imports, which is what a body that never reads its arguments
produces. 42 bodies were reduced.

### Phase 2: the suite, observed failing

Suite: 25 test files, 102 test functions, mirroring `src/` file for file, each registered up its
`mod.rs` chain and declared to Bazel by folder. Each file's module doc carries its corner-case
table (rows A to K) naming the test that covers each row or stating why the row is n/a, and the
provenance of every expected value (the literal handed to the constructor, or the constant the
constants test pins independently). Every `ProjectionErrorEnum` variant is constructed through its
constructor and asserted by variant in `tests/errors/projection_error_tests.rs`.

`cargo test -p deep_causality_context_store --test mod` against the API-only crate:

```
test result: FAILED. 40 passed; 62 failed; 0 ignored; 0 measured; 0 filtered out
```

All 62 failures are the `not implemented` panic, and every panic location is under
`deep_causality_context_store/src/`; none is a compile error, a missing import or a panic from
elsewhere. The 40 that pass exercise nothing a body could get wrong:

- 17 are the vocabulary tests moved in task group 2 (`relation_kind`, `time_scale`,
  `vertical_datum`, `substrate_ref`), whose code predates this stage.
- 2 exercise an alias and a constant, which have no body (`alias_tests`, `constants_tests`).
- 21 exercise only derived `PartialEq`, `Clone`, `Copy` and `Debug` on the record enums, or a
  test-local implementor of a trait (`Probe`, `NullError`, `EchoSubstrate`), never a function of
  the crate. They are kept because they pin the derives the specs require, and they are listed
  here so the count is honest.

The full run is kept beside this note during the session as `phase2-failing-run.txt` and
summarised above.

### Corner-case enumeration

Rows A to K, per file, in each test file's module doc. Rows that apply to a record vocabulary:
A empty (`List`, `Fields`, an empty reserve, an empty snapshot, an empty stream), B single, C
coinciding values under different variants and coinciding identifiers under different payloads, D
index boundaries (`IdReserve` past its end; an off-diagonal metric entry), E the version
threshold, F zero, G negative, H exact boundaries (`u64::MAX`, `i64::MIN`, `u16::MAX`,
`u8::MAX`), I non-finite (`NaN` breaks equality, infinity does not). J overflow and K precision
are n/a: the records hold `f64` and `u64` by design and perform no arithmetic.

### Error variants

`WrongVariant`, `WrongPayload`, `MissingField`, `Unrecordable`, `Scalar`, `Identity`, `Version`:
each constructed and matched by variant in `projection_error_tests.rs`. None is unreachable.

### Phase 3: the defect audit

Eleven defects were injected one at a time into the implementation, the suite run, the failing
tests collected, and the file restored byte for byte (checked by `diff -rq` against the snapshot
taken before the audit). Each row names the defect class, the injection and the test whose subject
it is.

| Class | Injection | Subject test | Result |
|---|---|---|---|
| guard removed | `IdReserve::next` never advances `taken` | `id_reserve_tests::test_a_reserve_yields_each_identifier_once` | rejected, 4 tests fail |
| off-by-one | `remaining` counts one short | `id_reserve_tests::test_a_single_identifier` | rejected, 3 |
| plausible neighbour | `RelationRecord::to` returns `from` | `relation_record_tests::test_new_and_getters` | rejected, 2 |
| constant changed | `ContextSnapshot::new` at `RECORD_VERSION + 1` | `context_snapshot_tests::test_new_is_at_the_current_version` | rejected, 2 |
| flipped label | `DataRecord::Count` reports `"Integer"` | `data_record_tests::test_eight_variants_with_distinct_names` | rejected, 2 |
| flipped label | `SpaceRecord::Ecef` reports `"Euclidean"` | `space_record_tests::test_four_variants_with_distinct_names` | rejected, 2 |
| dropped field | `WrongVariant` display loses the node | `projection_error_tests::test_wrong_variant` | rejected, 1 |
| plausible neighbour | `ExtraContextSnapshot::name` returns `""` | `extra_context_snapshot_tests::test_new_and_getters` | rejected, 2 |
| off-by-one | `ContextRecord::id` adds one | `context_record_tests::test_new_and_getters` | rejected, 6 |
| value replaced | `ContextoidRecord::id` returns 0 | `contextoid_record_tests::test_new_and_getters` | rejected, 6 |
| early return | `block_on` stops after two polls | `block_on_tests::test_a_pending_future_is_polled_again` | **missed**: the suite's only pending future was ready on the second poll |

The missed defect widened the suite: `test_a_future_pending_several_times_completes` drives a
future that is pending five times, and the same injection then fails that test (1 failure).
Eleven of eleven rejected after the widening. Tolerance loosening is n/a: the crate has no
tolerance. Input variety: the coinciding-value rows (C) are supplemented in every record file by a
case where the values differ, per the tables.

### Phase 4: implementation against the audited suite

The implementation is the snapshot the audit was run on. `cargo test -p
deep_causality_context_store`: 103 passed, 0 failed. `cargo clippy --all-targets -- -D warnings`
clean after two lints were fixed by rewriting: a `ContextEventItem` alias replaces the nested
return type of `ContextEvents::next`, and `IdReserve::next` is the `Iterator` implementation. `cargo
llvm-cov -p deep_causality_context_store --tests`:

```
TOTAL  regions 255/255 100%  functions 53/53 100%  lines 274/274 100%
```

`bazel test //deep_causality_context_store/...`: 26 targets pass, one per test file plus the doc
test; 25 test files, 25 suite targets. Cargo counts test functions (103) and Bazel counts files,
so the raw numbers differ by unit and only the file coverage is comparable.

### Phase 5: mutation testing

`cargo mutants -p deep_causality_context_store -j 8` over the whole crate, 6 minutes:

```
90 mutants tested: 45 caught, 0 missed, 45 unviable
```

Every viable mutant was caught. The 45 unviable mutants are `Default::default()` substitutions
that do not compile: inside a `const fn` body (the getters of the record structs, the
`ProjectionError` constructors, `ContextRecord::id`), or on a type with no `Default`
(`ProjectionErrorEnum`, `ContextRecord`, `ContextoidRecord`, `NodeRecord`,
`ExtraContextSnapshot`). No survivor, so no entry was added to `.cargo/mutants.toml`.
