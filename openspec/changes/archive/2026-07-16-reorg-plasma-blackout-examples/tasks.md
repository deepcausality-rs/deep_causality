## 1. Move the folders (history-preserving)

- [x] 1.1 Create the parent folder `examples/avionics_examples/cfd/plasma_blackout/` and
      `git mv examples/avionics_examples/cfd/plasma_blackout_corridor examples/avionics_examples/cfd/plasma_blackout/corridor`
- [x] 1.2 `git mv examples/avionics_examples/cfd/plasma_blackout_weather examples/avionics_examples/cfd/plasma_blackout/weather`
- [x] 1.3 Confirm `git status` shows renames (R) for every moved file and the old flat folders no longer exist

## 2. Functional path fixups

- [x] 2.1 `examples/avionics_examples/Cargo.toml`: update the two `[[example]] path` entries to
      `cfd/plasma_blackout/corridor/main.rs` and `cfd/plasma_blackout/weather/main.rs`; binary
      names stay `plasma_blackout_corridor` / `plasma_blackout_weather`
- [x] 2.2 Corridor `main.rs` (`table_path` closure, was line 65): record path becomes
      `cfd/plasma_blackout/corridor/corridor_branches.csv`
- [x] 2.3 Weather `model.rs` (`get_audit_dir` / `get_table_path`, were lines 401/406): paths become
      `cfd/plasma_blackout/weather/audit` and `cfd/plasma_blackout/weather/weather_table.csv`

## 3. Docs and live-note sweep

- [x] 3.1 `examples/avionics_examples/README.md` (lines 24-25): repoint both example links to
      `cfd/plasma_blackout/corridor/README.md` and `cfd/plasma_blackout/weather/README.md`
- [x] 3.2 Corridor `README.md`: cross-link becomes `../weather/README.md`; correct the stale
      module name `avionics_examples::blackout` to `avionics_examples::shared` (same paragraph,
      "Where Things Live")
- [x] 3.3 Weather `README.md` (line 12): cross-link becomes `../corridor/README.md`
- [x] 3.4 Live openspec notes: repoint the old paths in
      `openspec/notes/cfd-plasma-blackout/finite-rate-cfd-chemistry.md` (line 182),
      `openspec/notes/cfd-plasma-blackout/gap-analysis.md` (line 295), and
      `openspec/notes/cfd-plasma-retropulsion/plasma-retropulsion-descent.md` §12 (update the
      two example paths and drop the "paths as of this note; §8 moves them" parenthetical).
      Do not touch `openspec/changes/archive/` or `openspec/notes/archive/`

## 4. Verification (the spec's scenarios)

- [x] 4.1 `cargo build -p avionics_examples` compiles clean
- [x] 4.2 `cargo run --release -p avionics_examples --example plasma_blackout_corridor` exits 0
      with all gates passing, and `corridor_branches.csv` is rewritten inside
      `cfd/plasma_blackout/corridor/`
- [x] 4.3 `cargo run --release -p avionics_examples --example plasma_blackout_weather` exits 0
      with all gates passing (~4 min, 48 descents; run it, do not skip), and
      `weather_table.csv` plus `audit/` land inside `cfd/plasma_blackout/weather/`
- [x] 4.4 Grep gate: searching the repo for `cfd/plasma_blackout_corridor` or
      `cfd/plasma_blackout_weather` (excluding `openspec/changes/` — change artifacts describe
      the move itself — plus `openspec/notes/archive/`, `target/`, `.git/`) returns zero matches
- [x] 4.5 History check: `git status` shows every moved file as a rename (R) while staged;
      after the commit lands, `git log --follow --oneline -- examples/avionics_examples/cfd/plasma_blackout/corridor/main.rs`
      lists the pre-move history
- [x] 4.6 Prepare the commit message and hand it to the user (never commit; golden rule)

## 5. Review follow-ups (spec review, 5 confirmed findings + link audit)

- [x] 5.1 Stale module name `avionics_examples::blackout` corrected to
      `avionics_examples::shared` everywhere it survived 3.2: the weather `README.md` and both
      examples' `constants.rs` doc headers (repo grep for the stale name now returns zero)
- [x] 5.2 Corridor `README.md`: the companion-note link into `openspec/` gains one `../` level
      (the folder is one level deeper after the move); full relative-link audit of the four
      affected READMEs — 26 links checked, this was the only break
- [x] 5.3 Retropulsion note: §1 output.txt shorthand path updated; §3.3 cites
      `DescentSchedule::sample` by symbol; §4 gains the table-loader contract (rows arrive in
      run order — sort by `d_temp`, bracket by value, clamp out-of-range with a provenance
      stamp, reject duplicates); §7 kernel path gains `src/`; §12 weather README as a full
      repo path
- [x] 5.4 Artifacts corrected to match discovered reality: design.md inventory + Risks
      (the output.txt-link and depth-relative-link classes), proposal.md What Changes / Impact,
      spec scenario broadened to all relative README links
- [x] 5.5 Gates re-run after the fixes: link-resolver over the four READMEs (zero broken),
      grep gate (zero stale live references); no code touched, so the example runs from 4.2/4.3
      stand
