/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Compares the Meek closures against the definition of a compelled edge, on the graphs BRCD's
//! Algorithm 1 builds.
//!
//! # What is enumerated
//!
//! For each vertex count up to the bound: every labelled DAG, its CPDAG, and then — as Algorithm 1
//! does — for each candidate root-cause set `R` of size `k`, the CPDAG augmented with `F → r` for
//! every `r ∈ R`, under each orientation of the edge cut between `R` and `V \ R`. Inputs admitting
//! no consistent DAG extension are skipped, since an orientation of one means nothing.
//!
//! # The oracle
//!
//! An undirected edge `a — b` is compelled to `a → b` when every consistent DAG extension orients it
//! that way. Orienting every compelled edge gives the maximally oriented PDAG. `maximally_oriented`
//! computes it by enumerating extensions, so it uses no orientation rule.
//!
//! Each input is compared against that. An edge the definition compels and the R1–R3 closure leaves
//! undirected is a witness; the R1–R4 closure is then checked for whether it recovers the witness.
//!
//! # Checks on the result
//!
//! - `verify_cpdag_construction`: R1–R3 equals the definition on every pattern of a DAG.
//! - `positive_control_r4`: R4 fires on a configuration the definition compels and R1–R3 misses.
//! - `Rules::None`: the no-rule closure differs from the definition, so the comparison detects
//!   under-orientation where it exists.
//! - Soundness: R4 never orients an edge the definition leaves free.
//!
//! Any of the four failing exits non-zero, because the headline count is uninterpretable without
//! all of them.
//!
//! Run with:
//! ```bash
//! cargo run --release -p deep_causality_algorithms --example verification_meek_r4
//! bazel run //deep_causality_algorithms:verification_meek_r4
//! ```

use std::collections::BTreeSet;
use std::fmt::Write as _;

/// The state of the pair `(i, j)` with `i < j`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Cell {
    Absent,
    Undirected,
    /// `i → j`
    Forward,
    /// `j → i`
    Backward,
}

/// A partially directed graph on `n` labelled vertices, stored by unordered pair.
#[derive(Clone, PartialEq, Eq, Debug)]
struct Pdag {
    n: usize,
    /// Row-major upper triangle, indexed by `pair_index(i, j)` for `i < j`.
    cells: Vec<Cell>,
}

fn pair_index(n: usize, i: usize, j: usize) -> usize {
    debug_assert!(i < j && j < n);
    // Offset of row `i` in the strict upper triangle, plus the column offset within it.
    i * n - (i * (i + 1)) / 2 + (j - i - 1)
}

impl Pdag {
    fn empty(n: usize) -> Self {
        Pdag {
            n,
            cells: vec![Cell::Absent; n * (n - 1) / 2],
        }
    }

    fn cell(&self, i: usize, j: usize) -> Cell {
        // The diagonal is reachable: `parents` and `children` sweep every vertex, including the
        // one they are asked about.
        if i == j {
            return Cell::Absent;
        }
        if i < j {
            self.cells[pair_index(self.n, i, j)]
        } else {
            match self.cells[pair_index(self.n, j, i)] {
                Cell::Forward => Cell::Backward,
                Cell::Backward => Cell::Forward,
                other => other,
            }
        }
    }

    fn set_cell(&mut self, i: usize, j: usize, c: Cell) {
        if i < j {
            self.cells[pair_index(self.n, i, j)] = c;
        } else {
            let flipped = match c {
                Cell::Forward => Cell::Backward,
                Cell::Backward => Cell::Forward,
                other => other,
            };
            self.cells[pair_index(self.n, j, i)] = flipped;
        }
    }

    /// True when `a → b`.
    fn has_arc(&self, a: usize, b: usize) -> bool {
        self.cell(a, b) == Cell::Forward
    }

    fn is_undirected(&self, a: usize, b: usize) -> bool {
        self.cell(a, b) == Cell::Undirected
    }

    fn is_adjacent(&self, a: usize, b: usize) -> bool {
        a != b && self.cell(a, b) != Cell::Absent
    }

    /// Orients the undirected edge `a — b` as `a → b`.
    fn orient(&mut self, a: usize, b: usize) {
        self.set_cell(a, b, Cell::Forward);
    }

    fn parents(&self, v: usize) -> Vec<usize> {
        (0..self.n).filter(|&u| self.has_arc(u, v)).collect()
    }

    fn children(&self, v: usize) -> Vec<usize> {
        (0..self.n).filter(|&u| self.has_arc(v, u)).collect()
    }

    fn undirected_neighbors(&self, v: usize) -> Vec<usize> {
        (0..self.n).filter(|&u| self.is_undirected(v, u)).collect()
    }

    fn undirected_edges(&self) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        for i in 0..self.n {
            for j in (i + 1)..self.n {
                if self.cell(i, j) == Cell::Undirected {
                    out.push((i, j));
                }
            }
        }
        out
    }

    /// True when the directed-arc projection has a cycle.
    fn has_directed_cycle(&self) -> bool {
        let mut indeg: Vec<usize> = (0..self.n).map(|v| self.parents(v).len()).collect();
        let mut ready: Vec<usize> = (0..self.n).filter(|&v| indeg[v] == 0).collect();
        let mut seen = 0usize;
        while let Some(v) = ready.pop() {
            seen += 1;
            for c in self.children(v) {
                indeg[c] -= 1;
                if indeg[c] == 0 {
                    ready.push(c);
                }
            }
        }
        seen != self.n
    }

    /// The unshielded colliders `a → c ← b` with `a`, `b` non-adjacent, keyed by `(a, c, b)`
    /// with `a < b` so each is recorded once.
    fn colliders(&self) -> BTreeSet<(usize, usize, usize)> {
        let mut out = BTreeSet::new();
        for c in 0..self.n {
            let ps = self.parents(c);
            for x in 0..ps.len() {
                for y in (x + 1)..ps.len() {
                    let (a, b) = (ps[x], ps[y]);
                    if !self.is_adjacent(a, b) {
                        out.insert((a.min(b), c, a.max(b)));
                    }
                }
            }
        }
        out
    }

    fn render(&self) -> String {
        let mut s = String::new();
        for i in 0..self.n {
            for j in (i + 1)..self.n {
                match self.cell(i, j) {
                    Cell::Absent => {}
                    Cell::Undirected => {
                        let _ = write!(s, "{i}-{j} ");
                    }
                    Cell::Forward => {
                        let _ = write!(s, "{i}->{j} ");
                    }
                    Cell::Backward => {
                        let _ = write!(s, "{j}->{i} ");
                    }
                }
            }
        }
        if s.is_empty() {
            s.push_str("(no edges)");
        }
        s.trim_end().to_string()
    }
}

// ---------------------------------------------------------------------------------------------
// The orientation rules, transcribed from Meek (1995). Used for the closure under test, never as
// the oracle.
// ---------------------------------------------------------------------------------------------

/// R1: there is `c → a` with `c` not adjacent to `b`.
fn r1(g: &Pdag, a: usize, b: usize) -> bool {
    g.parents(a).into_iter().any(|c| !g.is_adjacent(c, b))
}

/// R2: there is a directed path `a → c → b`.
fn r2(g: &Pdag, a: usize, b: usize) -> bool {
    let parents_b: BTreeSet<usize> = g.parents(b).into_iter().collect();
    g.children(a).into_iter().any(|c| parents_b.contains(&c))
}

/// R3: there are `c → b` and `d → b` with `a — c`, `a — d` undirected and `c`, `d` non-adjacent.
fn r3(g: &Pdag, a: usize, b: usize) -> bool {
    let parents_b = g.parents(b);
    let und_a: BTreeSet<usize> = g.undirected_neighbors(a).into_iter().collect();
    for x in 0..parents_b.len() {
        for y in (x + 1)..parents_b.len() {
            let (c, d) = (parents_b[x], parents_b[y]);
            if und_a.contains(&c) && und_a.contains(&d) && !g.is_adjacent(c, d) {
                return true;
            }
        }
    }
    false
}

/// R4: there are `c`, `d` with `a — d` and `a — c` undirected, `d → c`, `c → b`, and `b` not
/// adjacent to `d`.
fn r4(g: &Pdag, a: usize, b: usize) -> bool {
    for d in g.undirected_neighbors(a) {
        for c in g.undirected_neighbors(a) {
            if c == d {
                continue;
            }
            if g.has_arc(d, c) && g.has_arc(c, b) && !g.is_adjacent(b, d) {
                return true;
            }
        }
    }
    false
}

/// Which rules a closure applies. `R1R2` exists only as a negative control: it is deliberately
/// incomplete, so a search that reports no difference under it is not searching.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Rules {
    /// No rules: the closure is the identity. Differs from the definition wherever the definition
    /// compels anything.
    None,
    R1R2,
    R1R3,
    R1R4,
}

/// Closes `g` under the supplied rules to a fixpoint.
fn close_with(g: &Pdag, rules: Rules) -> Pdag {
    let mut g = g.clone();
    loop {
        let mut changed = false;
        for (u, v) in g.undirected_edges() {
            if !g.is_undirected(u, v) {
                continue;
            }
            let forces = |x: usize, y: usize, g: &Pdag| match rules {
                Rules::None => false,
                Rules::R1R2 => r1(g, x, y) || r2(g, x, y),
                Rules::R1R3 => r1(g, x, y) || r2(g, x, y) || r3(g, x, y),
                Rules::R1R4 => r1(g, x, y) || r2(g, x, y) || r3(g, x, y) || r4(g, x, y),
            };
            if forces(u, v, &g) {
                g.orient(u, v);
                changed = true;
            } else if forces(v, u, &g) {
                g.orient(v, u);
                changed = true;
            }
        }
        if !changed {
            return g;
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The oracle: the maximally oriented PDAG, by enumerating consistent DAG extensions.
// ---------------------------------------------------------------------------------------------

/// Every consistent DAG extension of `p`: same skeleton, every arc of `p` preserved, acyclic, and
/// introducing no unshielded collider that `p` does not already have.
fn consistent_extensions(p: &Pdag) -> Vec<Pdag> {
    let undirected = p.undirected_edges();
    let base_colliders = p.colliders();
    let mut out = Vec::new();

    for mask in 0u32..(1u32 << undirected.len()) {
        let mut d = p.clone();
        for (bit, &(u, v)) in undirected.iter().enumerate() {
            if mask >> bit & 1 == 1 {
                d.orient(u, v);
            } else {
                d.orient(v, u);
            }
        }
        if d.has_directed_cycle() {
            continue;
        }
        // Arcs of `p` are preserved by construction, so every collider of `p` survives; the
        // extension is consistent exactly when it adds none.
        if !d.colliders().is_subset(&base_colliders) {
            continue;
        }
        out.push(d);
    }
    out
}

/// The maximally oriented PDAG: every undirected edge on whose direction all consistent
/// extensions agree, oriented that way. `None` when `p` admits no consistent extension.
fn maximally_oriented(p: &Pdag) -> Option<Pdag> {
    let extensions = consistent_extensions(p);
    if extensions.is_empty() {
        return None;
    }
    let mut out = p.clone();
    for (u, v) in p.undirected_edges() {
        let all_forward = extensions.iter().all(|d| d.has_arc(u, v));
        let all_backward = extensions.iter().all(|d| d.has_arc(v, u));
        if all_forward {
            out.orient(u, v);
        } else if all_backward {
            out.orient(v, u);
        }
    }
    Some(out)
}

// ---------------------------------------------------------------------------------------------
// Enumeration
// ---------------------------------------------------------------------------------------------

/// Every DAG on `n` labelled vertices, as a fully directed `Pdag`.
fn enumerate_dags(n: usize) -> Vec<Pdag> {
    let pairs: Vec<(usize, usize)> = (0..n)
        .flat_map(|i| ((i + 1)..n).map(move |j| (i, j)))
        .collect();
    let mut out = Vec::new();
    // Three states per pair: absent, i→j, j→i.
    let total = 3usize.pow(pairs.len() as u32);
    for code in 0..total {
        let mut g = Pdag::empty(n);
        let mut rest = code;
        for &(i, j) in &pairs {
            match rest % 3 {
                0 => {}
                1 => g.set_cell(i, j, Cell::Forward),
                _ => g.set_cell(i, j, Cell::Backward),
            }
            rest /= 3;
        }
        if !g.has_directed_cycle() {
            out.push(g);
        }
    }
    out
}

/// The CPDAG of a DAG: its skeleton, its v-structures, closed under the rules.
///
/// Uses the restricted closure, which is complete for a pattern; `verify_cpdag_construction`
/// checks that against the oracle.
fn cpdag_of(dag: &Pdag) -> Pdag {
    let mut p = Pdag::empty(dag.n);
    for i in 0..dag.n {
        for j in (i + 1)..dag.n {
            if dag.is_adjacent(i, j) {
                p.set_cell(i, j, Cell::Undirected);
            }
        }
    }
    for &(a, c, b) in &dag.colliders() {
        p.orient(a, c);
        p.orient(b, c);
    }
    close_with(&p, Rules::R1R3)
}

/// Adds the proxy node `F` with `F → r` for each `r ∈ R`, as Algorithm 1 line 3 does. `F` is the
/// new last vertex.
fn augment_with_f(cpdag: &Pdag, roots: &[usize]) -> Pdag {
    let n = cpdag.n + 1;
    let f = cpdag.n;
    let mut g = Pdag::empty(n);
    for i in 0..cpdag.n {
        for j in (i + 1)..cpdag.n {
            g.set_cell(i, j, cpdag.cell(i, j));
        }
    }
    for &r in roots {
        g.set_cell(f, r, Cell::Forward);
    }
    g
}

/// The undirected edges with exactly one endpoint in `roots` — the cut Algorithm 1 orients.
fn cut_edges(g: &Pdag, roots: &[usize]) -> Vec<(usize, usize)> {
    let in_r: BTreeSet<usize> = roots.iter().copied().collect();
    g.undirected_edges()
        .into_iter()
        .filter(|&(u, v)| in_r.contains(&u) != in_r.contains(&v))
        .collect()
}

fn subsets_of_size(n: usize, k: usize) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    let mut current = Vec::new();
    fn rec(start: usize, n: usize, k: usize, cur: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if cur.len() == k {
            out.push(cur.clone());
            return;
        }
        for v in start..n {
            cur.push(v);
            rec(v + 1, n, k, cur, out);
            cur.pop();
        }
    }
    rec(0, n, k, &mut current, &mut out);
    out
}

// ---------------------------------------------------------------------------------------------
// The search
// ---------------------------------------------------------------------------------------------

struct Witness {
    n: usize,
    dag: String,
    roots: Vec<usize>,
    input: String,
    r1r3: String,
    truth: String,
    missed: Vec<(usize, usize)>,
    r4_recovers: bool,
}

struct Stats {
    dags: usize,
    augmented: usize,
    extendable: usize,
    non_extendable: usize,
    witnesses: usize,
    r4_recovered_all: usize,
    /// Negative control: inputs the no-rule closure under-orients. Zero here would mean the
    /// comparison detects nothing, and the headline result would be meaningless.
    control_witnesses: usize,
    /// Inputs R1-R2 alone under-orients — that is, where R3 does real work.
    r3_needed: usize,
    /// Inputs where adding R4 changes the closure at all.
    r4_fires: usize,
    /// Inputs that still carry an undirected edge after the cut is oriented. If this is near zero
    /// the search is vacuous: there is nothing left for any rule to orient.
    with_free_edge: usize,
    /// Inputs where the definition compels something the input did not already fix. This is the
    /// population in which a rule can possibly be incomplete.
    oracle_orients_something: usize,
    /// Of those, the ones where R1-R3 also oriented something.
    r1r3_orients_something: usize,
    /// Inputs where R4 orients an edge the definition does not compel. Any is a transcription bug.
    r4_unsound: usize,
    r4_unsound_example: Option<String>,
}

/// For the pattern of a DAG, the R1–R3 closure equals the maximally oriented PDAG. A failure here
/// is a bug in this file.
fn verify_cpdag_construction(n: usize) -> Result<usize, String> {
    let mut checked = 0;
    for dag in enumerate_dags(n) {
        let cpdag = cpdag_of(&dag);
        let truth = maximally_oriented(&cpdag)
            .ok_or_else(|| format!("a CPDAG admitted no extension: {}", cpdag.render()))?;
        if truth != cpdag {
            return Err(format!(
                "R1-R3 is incomplete on the pattern of {} — closure {} vs definition {}",
                dag.render(),
                cpdag.render(),
                truth.render()
            ));
        }
        checked += 1;
    }
    Ok(checked)
}

/// Checks R4 on the configuration it exists for: `a — b`, `a — c`, `a — d` undirected, `d → c → b`,
/// `b` and `d` non-adjacent.
///
/// The definition compels `a → b` here: an extension orienting `b → a` is forced to `c → a` and
/// `d → a` to avoid a cycle, and `b → a ← d` is then a new unshielded collider. Asserts that the
/// definition compels it, that R1–R3 does not orient it, and that R1–R4 does.
fn positive_control_r4() -> Result<(), String> {
    let (a, b, c, d) = (0usize, 1usize, 2usize, 3usize);
    let mut p = Pdag::empty(4);
    p.set_cell(a, b, Cell::Undirected);
    p.set_cell(a, c, Cell::Undirected);
    p.set_cell(a, d, Cell::Undirected);
    p.set_cell(d, c, Cell::Forward);
    p.set_cell(c, b, Cell::Forward);

    let truth = maximally_oriented(&p).ok_or("the R4 configuration admits no extension")?;
    if !truth.has_arc(a, b) {
        return Err(format!(
            "the definition does not compel a->b here, so this is not an R4 configuration: {}",
            truth.render()
        ));
    }
    let restricted = close_with(&p, Rules::R1R3);
    if restricted.has_arc(a, b) {
        return Err(format!(
            "R1-R3 already orients a->b, so this configuration does not isolate R4: {}",
            restricted.render()
        ));
    }
    let full = close_with(&p, Rules::R1R4);
    if !full.has_arc(a, b) {
        return Err(format!(
            "R4 AS TRANSCRIBED DOES NOT FIRE on its own configuration — the rule is mis-stated: {}",
            full.render()
        ));
    }
    if full != truth {
        return Err(format!(
            "R4 fires but does not reach the definition: closure {} vs definition {}",
            full.render(),
            truth.render()
        ));
    }
    Ok(())
}

fn search(n: usize, k: usize, stats: &mut Stats, witnesses: &mut Vec<Witness>) {
    for dag in enumerate_dags(n) {
        stats.dags += 1;
        let cpdag = cpdag_of(&dag);

        for roots in subsets_of_size(n, k) {
            let augmented = augment_with_f(&cpdag, &roots);
            let cut = cut_edges(&augmented, &roots);

            for mask in 0u32..(1u32 << cut.len()) {
                let mut input = augmented.clone();
                for (bit, &(u, v)) in cut.iter().enumerate() {
                    if mask >> bit & 1 == 1 {
                        input.orient(u, v);
                    } else {
                        input.orient(v, u);
                    }
                }
                stats.augmented += 1;

                let Some(truth) = maximally_oriented(&input) else {
                    // Algorithm 1 can produce these; BRCD's validity pass rejects them.
                    stats.non_extendable += 1;
                    continue;
                };
                stats.extendable += 1;

                if !input.undirected_edges().is_empty() {
                    stats.with_free_edge += 1;
                }
                if truth != input {
                    stats.oracle_orients_something += 1;
                }
                if close_with(&input, Rules::R1R3) != input {
                    stats.r1r3_orients_something += 1;
                }

                // Negative control: no rules at all, through the same comparison.
                if close_with(&input, Rules::None) != truth {
                    stats.control_witnesses += 1;
                }
                // How much of the work each rule tier actually does on this family.
                if close_with(&input, Rules::R1R2) != truth {
                    stats.r3_needed += 1;
                }

                // Soundness of the R4 transcription: the closure may not orient an edge the
                // definition leaves free. An over-orientation is a wrong rule, not a finding.
                let full = close_with(&input, Rules::R1R4);
                if full != close_with(&input, Rules::R1R3) {
                    stats.r4_fires += 1;
                }
                for (u, v) in input.undirected_edges() {
                    if truth.is_undirected(u, v) && !full.is_undirected(u, v) {
                        stats.r4_unsound += 1;
                        if stats.r4_unsound_example.is_none() {
                            stats.r4_unsound_example = Some(format!(
                                "input {} — R4 oriented {}-{}, the definition leaves it free (closure {})",
                                input.render(),
                                u,
                                v,
                                full.render()
                            ));
                        }
                        break;
                    }
                }

                let restricted = close_with(&input, Rules::R1R3);
                if restricted == truth {
                    continue;
                }

                let missed: Vec<(usize, usize)> = restricted
                    .undirected_edges()
                    .into_iter()
                    .filter(|&(u, v)| !truth.is_undirected(u, v))
                    .collect();

                let r4_recovers = full == truth;

                stats.witnesses += 1;
                if r4_recovers {
                    stats.r4_recovered_all += 1;
                }
                if witnesses.len() < 3 {
                    witnesses.push(Witness {
                        n,
                        dag: dag.render(),
                        roots: roots.clone(),
                        input: input.render(),
                        r1r3: restricted.render(),
                        truth: truth.render(),
                        missed,
                        r4_recovers,
                    });
                }
            }
        }
    }
}

fn main() {
    println!("Meek closures vs the maximally oriented PDAG, on Algorithm 1's augmented graphs.\n");

    print!("R1-R3 equals the definition on every pattern ... ");
    for n in 2..=4 {
        match verify_cpdag_construction(n) {
            Ok(count) => print!("n={n}: {count} ok  "),
            Err(e) => {
                println!("\nHARNESS BUG: {e}");
                std::process::exit(1);
            }
        }
    }
    print!("R4 fires on its own configuration ... ");
    match positive_control_r4() {
        Ok(()) => println!("ok"),
        Err(e) => {
            println!("\nHARNESS BUG: {e}");
            std::process::exit(1);
        }
    }
    println!();

    let mut stats = Stats {
        dags: 0,
        augmented: 0,
        extendable: 0,
        non_extendable: 0,
        witnesses: 0,
        r4_recovered_all: 0,
        control_witnesses: 0,
        r3_needed: 0,
        r4_fires: 0,
        with_free_edge: 0,
        oracle_orients_something: 0,
        r1r3_orients_something: 0,
        r4_unsound: 0,
        r4_unsound_example: None,
    };
    let mut witnesses = Vec::new();

    // The bound. Vertex counts are exhaustive; `k` is the number of root causes Algorithm 1
    // ranges over. n=5 with k=2 is where the augmented-graph count stops being cheap.
    let bound: &[(usize, usize)] = &[(3, 1), (4, 1), (5, 1), (3, 2), (4, 2), (5, 2), (5, 3)];

    for &(n, k) in bound {
        let before = stats.witnesses;
        search(n, k, &mut stats, &mut witnesses);
        println!(
            "n={n} k={k}: {} witnesses",
            stats.witnesses.saturating_sub(before)
        );
    }

    println!("\n--- searched ---");
    println!("DAGs enumerated:            {}", stats.dags);
    println!("augmented graphs:           {}", stats.augmented);
    println!("  with a consistent extension: {}", stats.extendable);
    println!("  without one (skipped):       {}", stats.non_extendable);

    println!("\n--- is the search non-vacuous? ---");
    println!(
        "inputs with a free edge after the cut: {}",
        stats.with_free_edge
    );
    println!(
        "inputs the definition orients further: {}",
        stats.oracle_orients_something
    );
    println!(
        "inputs R1-R3 orients further:          {}",
        stats.r1r3_orients_something
    );

    println!("\n--- harness checks ---");
    println!(
        "negative control (no rules): {} under-oriented inputs",
        stats.control_witnesses
    );
    if stats.control_witnesses == 0 {
        println!("CONTROL FAILED: the comparison detects no incompleteness even where it exists.");
        println!("The headline result below means nothing. Fix the harness.");
        std::process::exit(1);
    }
    println!(
        "R4 transcription soundness: {} over-orientations",
        stats.r4_unsound
    );
    println!("\n--- which rules do work on this family ---");
    println!(
        "inputs where R3 is needed (R1-R2 falls short): {}",
        stats.r3_needed
    );
    println!(
        "inputs where R4 changes the closure at all:    {}",
        stats.r4_fires
    );
    if let Some(example) = &stats.r4_unsound_example {
        println!("R4 AS TRANSCRIBED IS UNSOUND — it compels an edge the definition does not:");
        println!("  {example}");
        std::process::exit(1);
    }

    println!("\n--- result ---");
    if stats.witnesses == 0 {
        println!("R1-R3 reached the maximally oriented PDAG on every extendable input.");
        println!("R4 changed no orientation up to this bound.");
    } else {
        println!(
            "{} of {} extendable inputs are under-oriented by R1-R3.",
            stats.witnesses, stats.extendable
        );
        println!(
            "R4 recovers the full orientation on {} of them.",
            stats.r4_recovered_all
        );
        if stats.r4_recovered_all < stats.witnesses {
            println!(
                "On {} it does not — so either the R4 transcription in this file is wrong, or",
                stats.witnesses - stats.r4_recovered_all
            );
            println!("more than R4 is missing. Resolve before implementing the rule.");
        }
        for (idx, w) in witnesses.iter().enumerate() {
            println!("\nwitness {} (n={}, roots={:?})", idx + 1, w.n, w.roots);
            println!("  source DAG:      {}", w.dag);
            println!("  augmented input: {}", w.input);
            println!("  R1-R3 closure:   {}", w.r1r3);
            println!("  definition:      {}", w.truth);
            println!("  left undirected: {:?}", w.missed);
            println!("  R4 recovers it:  {}", w.r4_recovers);
        }
    }
}
