/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Core — Csv: the IO CSV codec round-trips under its stated precondition.

Rust source: `deep_causality_core/src/types/io/write_csv.rs` (`WriteCsv::render` — `','`-join each
line, `'\n'`-terminate every line including the last) and `read_csv.rs` (`ReadCsv::run` — split on
`'\n'` via `lines()` dropping a single trailing empty, then split each line on `','`).

Layering: the base is the IO action monad `haft.io.laws` (`Haft/Io.lean`); the codec is a pure
`render`/`parse` pair observed through `IoAction::run`. This file proves the *content* law: `parse`
is a left inverse of `render` — `parse (render header rows) = header :: rows` — under the EXPLICIT
precondition that no field contains the separator `','` or the line terminator `'\n'` (the Rust
codec applies no quoting/escaping; deviation D16 accepts this as conditional). The precondition is a
theorem **hypothesis**, not assumed away: without it the round-trip genuinely fails (a comma in a
field would split into two).

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witnesses: `deep_causality_core/tests/formalization_lean/csv_tests.rs`.
-/

namespace DeepCausalityFormal.Core.Csv

variable {sep nl : Char}

/-- A field (or line) is *free* of char `x` when `x` never occurs in it. -/
def Free (x : Char) : List Char → Prop
  | []      => True
  | c :: cs => c ≠ x ∧ Free x cs

/-- Every field of a row is free of `x`. -/
def RowFree (x : Char) : List (List Char) → Prop
  | []      => True
  | f :: fs => Free x f ∧ RowFree x fs

/-- `[f].join(sep)`: `','`-join a row's fields (no trailing separator; single field ↦ itself). -/
def joinRow (sep : Char) : List (List Char) → List Char
  | []       => []
  | [f]      => f
  | f :: fs  => f ++ sep :: joinRow sep fs

/-- `str::split(sep)`: always ≥ 1 group; a separator starts a new (possibly empty) group. -/
def splitOn (sep : Char) : List Char → List (List Char)
  | []      => [[]]
  | c :: cs =>
    if c = sep then [] :: splitOn sep cs
    else match splitOn sep cs with
      | []      => [[c]]
      | g :: gs => (c :: g) :: gs

/-- `render` writes each line `'\n'`-terminated (including the last). -/
def unlines (nl : Char) : List (List Char) → List Char
  | []      => []
  | l :: ls => l ++ nl :: unlines nl ls

/-- Drop a single trailing empty group — `str::lines()` yields no empty final row for a trailing
    `'\n'`. -/
def dropLastIfEmpty (xs : List (List Char)) : List (List Char) :=
  match xs.reverse with
  | []      => []
  | l :: rest => if l = [] then rest.reverse else xs

/-- `str::lines()`: split on `'\n'`, dropping the trailing empty. -/
def lines (nl : Char) (s : List Char) : List (List Char) :=
  dropLastIfEmpty (splitOn nl s)

/-- `render(header, rows)` — the exact bytes `WriteCsv` writes. -/
def render (sep nl : Char) (header : List (List Char)) (rows : List (List (List Char))) :
    List Char :=
  unlines nl (joinRow sep header :: rows.map (joinRow sep))

/-- `parse(contents)` — the exact structure `ReadCsv` returns. -/
def parse (sep nl : Char) (s : List Char) : List (List (List Char)) :=
  (lines nl s).map (splitOn sep)

-- ------------------------------------------------------------------
-- Inverse lemmas.
-- ------------------------------------------------------------------

/-- On a separator-free field, `split` returns the singleton `[f]`. -/
theorem splitOn_free : ∀ (f : List Char), Free sep f → splitOn sep f = [f]
  | [],      _ => rfl
  | c :: cs, h => by
    have hc : c ≠ sep := h.1
    have ih := splitOn_free cs h.2
    simp only [splitOn, if_neg hc, ih]

/-- `split` of `field ++ sep :: tail`, with `field` separator-free, peels off `field`. -/
theorem splitOn_append_sep :
    ∀ (f : List Char), Free sep f →
      ∀ (tail : List Char), splitOn sep (f ++ sep :: tail) = f :: splitOn sep tail
  | [],      _, tail => by simp [splitOn]
  | c :: cs, h, tail => by
    have hc : c ≠ sep := h.1
    have ih := splitOn_append_sep cs h.2 tail
    show splitOn sep (c :: (cs ++ sep :: tail)) = (c :: cs) :: splitOn sep tail
    simp only [splitOn, if_neg hc]
    rw [ih]

/-- `split (join r) = r` for a row whose fields are separator-free and which has at least one field
    (the empty row is not representable — `WriteCsv` always writes ≥ 1 field). -/
theorem splitOn_joinRow : ∀ (r : List (List Char)), r ≠ [] → RowFree sep r →
    splitOn sep (joinRow sep r) = r
  | [],        hne, _ => (hne rfl).elim
  | [f],       _,   h => by
    simpa [joinRow] using splitOn_free (sep := sep) f h.1
  | f :: g :: gs, _, h => by
    have hfree : Free sep f := h.1
    have hrest : RowFree sep (g :: gs) := h.2
    have ih := splitOn_joinRow (g :: gs) (by simp) hrest
    simp only [joinRow, splitOn_append_sep f hfree, ih]

/-- `split (unlines rows) = rows ++ [[]]` when every rendered line is newline-free (the trailing
    `'\n'` leaves one empty final group). -/
theorem splitOn_unlines : ∀ (rows : List (List Char)), (∀ l ∈ rows, Free nl l) →
    splitOn nl (unlines nl rows) = rows ++ [[]]
  | [],        _ => rfl
  | l :: ls, h => by
    have hl : Free nl l := h l (by simp)
    have hls : ∀ x ∈ ls, Free nl x := fun x hx => h x (by simp [hx])
    have ih := splitOn_unlines ls hls
    simp only [unlines, splitOn_append_sep l hl, ih, List.cons_append]

/-- Dropping the trailing empty recovers the original list. -/
theorem dropLastIfEmpty_append_nil (rows : List (List Char)) :
    dropLastIfEmpty (rows ++ [[]]) = rows := by
  simp [dropLastIfEmpty, List.reverse_append, List.reverse_reverse]

/-- `lines (unlines rows) = rows` when every line is newline-free. -/
theorem lines_unlines (rows : List (List Char)) (h : ∀ l ∈ rows, Free nl l) :
    lines nl (unlines nl rows) = rows := by
  simp only [lines, splitOn_unlines rows h, dropLastIfEmpty_append_nil]

/-- Field-wise inverse over every row: `split ∘ join` is the identity on each non-empty,
    separator-free row (equation-style recursion to avoid hypothesis reversion). -/
theorem map_split_join : ∀ (rows : List (List (List Char))),
    (∀ r ∈ rows, r ≠ []) → (∀ r ∈ rows, RowFree sep r) →
    (rows.map (joinRow sep)).map (splitOn sep) = rows
  | [],        _,   _    => rfl
  | r :: rs, hne, hsep => by
    have hrne : r ≠ [] := hne r (by simp)
    have hrsep : RowFree sep r := hsep r (by simp)
    have ih := map_split_join rs (fun x hx => hne x (by simp [hx])) (fun x hx => hsep x (by simp [hx]))
    simp only [List.map_cons, splitOn_joinRow r hrne hrsep, ih]

-- ------------------------------------------------------------------
-- The round-trip.
-- ------------------------------------------------------------------

/-- The CSV codec round-trips under the no-comma/no-newline precondition:
    `parse (render header rows) = header :: rows`, provided every field is separator-free and every
    rendered line is newline-free, and both the header and every row have at least one field.

    THEOREM_MAP: `core.io.csv_roundtrip` -/
theorem csv_roundtrip
    (header : List (List Char)) (rows : List (List (List Char)))
    (hHeadNe : header ≠ []) (hHeadSep : RowFree sep header)
    (hRowsNe : ∀ r ∈ rows, r ≠ []) (hRowsSep : ∀ r ∈ rows, RowFree sep r)
    (hLinesNl : ∀ l ∈ (joinRow sep header :: rows.map (joinRow sep)), Free nl l) :
    parse sep nl (render sep nl header rows) = header :: rows := by
  unfold parse render
  rw [lines_unlines _ hLinesNl, List.map_cons, splitOn_joinRow header hHeadNe hHeadSep,
    map_split_join rows hRowsNe hRowsSep]

end DeepCausalityFormal.Core.Csv
