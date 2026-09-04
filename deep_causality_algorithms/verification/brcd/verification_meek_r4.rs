/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Compares BRCD's **production** Meek closures against the definition of a compelled edge, on the
//! graphs BRCD's Algorithm 1 builds.
//!
//! # What is under test
//!
//! The shipped closures, not a transcription of them:
//! [`MixedGraph::meek_complete_r1_r3`](deep_causality_topology::MixedGraph::meek_complete_r1_r3) and
//! [`MixedGraph::meek_complete`](deep_causality_topology::MixedGraph::meek_complete). This file
//! contains no copy of the four rules, so an omission in the shipped R4 cannot pass here.
//!
//! # What is enumerated
//!
//! For each vertex count up to the bound: every labelled DAG and its CPDAG. Then, for each
//! candidate root-cause set `R` of size `k`, exactly what
//! [`get_configurations_multi`] enumerates — every orientation of **every undirected edge incident
//! on any target**, including the edges internal to `R`, kept only when production's validity pass
//! accepts it (Meek-complete, acyclic, no new unshielded collider at a target).
//!
//! The enumeration is checked against `get_configurations_multi` itself on every `(CPDAG, R)`: the
//! accepted set must match production's, edge for edge. A mismatch is a harness bug and exits
//! non-zero, so the population cannot silently drift from Algorithm 1's.
//!
//! The graph production actually Meek-closes is the CPDAG with those edges oriented — the F-node is
//! added afterwards by [`augmented_graph`] and the closure is never re-run. That is checked too: the
//! F-augmented graph must be a fixpoint of the production closure.
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
//! - `verify_cpdag_construction`: production R1–R3 equals the definition on every pattern of a DAG.
//! - `positive_control_r4`: production R4 fires on a configuration the definition compels and
//!   production R1–R3 misses.
//! - Negative control: the no-rule closure (the identity) differs from the definition on a large
//!   population, so the comparison detects under-orientation where it exists.
//! - Soundness: production R1–R4 never orients an edge the definition leaves free.
//! - Every witness R1–R3 leaves must be recovered by production R1–R4.
//!
//! Any of these failing exits non-zero, because the headline count is uninterpretable without all of
//! them.
//!
//! Run with:
//! ```bash
//! cargo run --release -p deep_causality_algorithms --example verification_meek_r4
//! bazel run //deep_causality_algorithms:verification_meek_r4
//! ```

use std::collections::BTreeSet;
use std::fmt::Write as _;

use deep_causality_algorithms::brcd::brcd_augment::{augmented_graph, get_configurations_multi};
use deep_causality_algorithms::brcd::brcd_validity::{
    baseline_parents, has_new_unshielded_collider_any,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;

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
///
/// This is the oracle's representation only: it carries no orientation rule. Every closure goes
/// through the production `MixedGraph` (see `meek_r1_r3` / `meek_r1_r4`).
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
// The bridge to production. Every orientation rule this file exercises lives behind these two
// functions; nothing here re-states a rule.
// ---------------------------------------------------------------------------------------------

/// The same graph as a `MixedGraph`, the type the production closure operates on.
fn to_mixed(p: &Pdag) -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); p.n], vec![p.n])
        .expect("a unit payload of length n is a valid 1-D tensor");
    let mut g = MixedGraph::<()>::new(p.n, data, 0).expect("n >= 1 with a matching payload");
    for i in 0..p.n {
        for j in (i + 1)..p.n {
            let added = match p.cell(i, j) {
                Cell::Absent => Ok(()),
                Cell::Undirected => g.add_undirected(i, j),
                Cell::Forward => g.add_arc(i, j),
                Cell::Backward => g.add_arc(j, i),
            };
            added.expect("each unordered pair is written exactly once");
        }
    }
    g
}

/// The inverse of `to_mixed`, for the oracle's comparisons. Only directed and undirected edges
/// occur here: BRCD builds no bidirected or circle endpoints.
fn from_mixed(g: &MixedGraph<()>) -> Pdag {
    let mut p = Pdag::empty(g.num_vertices());
    for (u, v) in g.undirected_edges() {
        p.set_cell(u, v, Cell::Undirected);
    }
    for (u, v) in g.arcs() {
        p.set_cell(u, v, Cell::Forward);
    }
    p
}

/// The production R1–R3 closure.
fn meek_r1_r3(p: &Pdag) -> Pdag {
    let mut g = to_mixed(p);
    g.meek_complete_r1_r3();
    from_mixed(&g)
}

/// The production R1–R4 closure.
fn meek_r1_r4(p: &Pdag) -> Pdag {
    let mut g = to_mixed(p);
    g.meek_complete();
    from_mixed(&g)
}

/// The edge content of a graph: its arcs and its undirected edges, both in ascending canonical
/// order. Two `MixedGraph`s with the same signature are the same partially directed graph.
type Signature = (Vec<(usize, usize)>, Vec<(usize, usize)>);

/// The edge content of a graph, for comparing two `MixedGraph`s by structure alone.
fn signature(g: &MixedGraph<()>) -> Signature {
    (g.arcs(), g.undirected_edges())
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

/// The CPDAG of a DAG: its skeleton, its v-structures, closed under the production R1–R3 rules —
/// the same closure `dag_to_cpdag` uses.
///
/// `verify_cpdag_construction` checks the result against the oracle.
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
    meek_r1_r3(&p)
}

/// Every undirected edge with **at least one** endpoint in `targets` — the set
/// `brcd_augment::incident_undirected_edges` collects, in the same ascending canonical order, so
/// bit `i` of a configuration selects the same edge here as it does in production.
///
/// Note the `||`: an edge with *both* endpoints in `targets` is enumerated too. Restricting this to
/// the cut (exactly one endpoint in `targets`) would make the search miss part of Algorithm 1's
/// configuration space.
fn incident_undirected_edges(p: &Pdag, targets: &[usize]) -> Vec<(usize, usize)> {
    let target_set: BTreeSet<usize> = targets.iter().copied().collect();
    p.undirected_edges()
        .into_iter()
        .filter(|&(a, b)| target_set.contains(&a) || target_set.contains(&b))
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
    /// Raw orientations of the incident undirected edges, before any filter.
    configurations: usize,
    /// Of those, the ones production's validity pass rejects (cyclic after Meek completion, or a
    /// new unshielded collider at a target). These never reach Algorithm 1's downstream.
    rejected_invalid: usize,
    /// Production-valid configurations that still admit no consistent DAG extension, so an
    /// orientation of them means nothing. Reported, not searched.
    non_extendable: usize,
    /// The searched population: production-valid and extendable.
    extendable: usize,
    witnesses: usize,
    r4_recovered_all: usize,
    /// Inputs that still carry an undirected edge after the incident edges are oriented. If this is
    /// near zero the search is vacuous: there is nothing left for any rule to orient.
    with_free_edge: usize,
    /// Inputs where the definition compels something the input did not already fix. This is both
    /// the population in which a rule can possibly be incomplete and the negative control: the
    /// no-rule closure is the identity, so this is exactly the count of inputs it under-orients.
    oracle_orients_something: usize,
    /// Of those, the ones where production R1-R3 also oriented something.
    r1r3_orients_something: usize,
    /// Inputs where adding R4 changes the production closure at all.
    r4_fires: usize,
    /// Inputs where production R1-R4 orients an edge the definition does not compel. Any is a bug
    /// in the shipped rules.
    r4_unsound: usize,
    r4_unsound_example: Option<String>,
    /// Configurations whose F-node-augmented graph is not a fixpoint of the production closure.
    /// Algorithm 1 never re-closes after augmenting, so any of these would be an under-oriented
    /// graph handed to `mec_size`.
    f_augment_compels: usize,
    f_augment_example: Option<String>,
}

/// For the pattern of a DAG, the production R1–R3 closure equals the maximally oriented PDAG. A
/// failure here is a bug in the shipped closure or in this file.
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

/// Checks the production R4 on the configuration it exists for: `a — b`, `a — c`, `a — d`
/// undirected, `d → c → b`, `b` and `d` non-adjacent.
///
/// The definition compels `a → b` here: an extension orienting `b → a` is forced to `c → a` and
/// `d → a` to avoid a cycle, and `b → a ← d` is then a new unshielded collider. Asserts that the
/// definition compels it, that production R1–R3 does not orient it, and that production R1–R4 does.
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
    let restricted = meek_r1_r3(&p);
    if restricted.has_arc(a, b) {
        return Err(format!(
            "R1-R3 already orients a->b, so this configuration does not isolate R4: {}",
            restricted.render()
        ));
    }
    let full = meek_r1_r4(&p);
    if !full.has_arc(a, b) {
        return Err(format!(
            "THE PRODUCTION R4 DOES NOT FIRE on its own configuration — the shipped rule is wrong: {}",
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

/// The population mismatch a `(CPDAG, R)` can report: the harness's accepted configurations must be
/// exactly `get_configurations_multi`'s.
fn cross_check_population(
    cpdag: &MixedGraph<()>,
    roots: &[usize],
    accepted: &mut Vec<Signature>,
) -> Result<(), String> {
    let production = get_configurations_multi(cpdag, roots)
        .map_err(|e| format!("get_configurations_multi failed: {e:?}"))?;
    let mut produced: Vec<_> = production.iter().map(signature).collect();
    produced.sort();
    accepted.sort();
    if &produced != accepted {
        return Err(format!(
            "the harness enumerated {} configurations, get_configurations_multi {} — \
             the searched population is not Algorithm 1's",
            accepted.len(),
            produced.len()
        ));
    }
    Ok(())
}

fn search(
    n: usize,
    k: usize,
    stats: &mut Stats,
    witnesses: &mut Vec<Witness>,
) -> Result<(), String> {
    for dag in enumerate_dags(n) {
        stats.dags += 1;
        let cpdag = cpdag_of(&dag);
        let cpdag_graph = to_mixed(&cpdag);

        for roots in subsets_of_size(n, k) {
            let incident = incident_undirected_edges(&cpdag, &roots);
            let baseline = baseline_parents(&cpdag_graph, &roots);
            let mut accepted = Vec::new();

            for combo in 0u32..(1u32 << incident.len()) {
                stats.configurations += 1;

                // Production's bit convention: bit i clear orients (a, b), set orients (b, a).
                let mut input = cpdag.clone();
                for (bit, &(a, b)) in incident.iter().enumerate() {
                    if combo >> bit & 1 == 0 {
                        input.orient(a, b);
                    } else {
                        input.orient(b, a);
                    }
                }

                // Production's closure, then production's validity pass. A rejected configuration
                // never reaches Algorithm 1's downstream, so it is not part of the population.
                let mut completed = to_mixed(&input);
                completed.meek_complete();
                if completed.has_cycle()
                    || has_new_unshielded_collider_any(&completed, &roots, &baseline)
                {
                    stats.rejected_invalid += 1;
                    continue;
                }
                accepted.push(signature(&completed));

                // Algorithm 1 adds the F-node after the closure and never re-closes. Check that
                // costs nothing: the augmented graph must already be a fixpoint.
                let augmented = augmented_graph(&completed, &roots)
                    .map_err(|e| format!("augmented_graph failed: {e:?}"))?;
                let mut augmented_closed = augmented.clone();
                augmented_closed.meek_complete();
                if signature(&augmented_closed) != signature(&augmented) {
                    stats.f_augment_compels += 1;
                    if stats.f_augment_example.is_none() {
                        stats.f_augment_example = Some(format!(
                            "config {} with roots {:?} — F-augmented graph is not closed: {} becomes {}",
                            input.render(),
                            roots,
                            from_mixed(&augmented).render(),
                            from_mixed(&augmented_closed).render()
                        ));
                    }
                }

                let Some(truth) = maximally_oriented(&input) else {
                    // Production's validity pass accepts these; they still pin no orientation.
                    stats.non_extendable += 1;
                    continue;
                };
                stats.extendable += 1;

                if !input.undirected_edges().is_empty() {
                    stats.with_free_edge += 1;
                }
                // Negative control and incompleteness population in one: the no-rule closure is the
                // identity, so `truth != input` is exactly "the no-rule closure under-orients".
                if truth != input {
                    stats.oracle_orients_something += 1;
                }

                let restricted = meek_r1_r3(&input);
                let full = from_mixed(&completed);

                if restricted != input {
                    stats.r1r3_orients_something += 1;
                }
                if full != restricted {
                    stats.r4_fires += 1;
                }

                // Soundness: the closure may not orient an edge the definition leaves free. An
                // over-orientation is a wrong rule, not a finding about completeness.
                for (u, v) in input.undirected_edges() {
                    if truth.is_undirected(u, v) && !full.is_undirected(u, v) {
                        stats.r4_unsound += 1;
                        if stats.r4_unsound_example.is_none() {
                            stats.r4_unsound_example = Some(format!(
                                "input {} — the closure oriented {}-{}, the definition leaves it free (closure {})",
                                input.render(),
                                u,
                                v,
                                full.render()
                            ));
                        }
                        break;
                    }
                }

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

            cross_check_population(&cpdag_graph, &roots, &mut accepted)
                .map_err(|e| format!("n={n} k={k} dag {} roots {roots:?}: {e}", dag.render()))?;
        }
    }
    Ok(())
}

fn main() {
    println!(
        "Production Meek closures vs the maximally oriented PDAG, on Algorithm 1's configurations.\n"
    );

    print!("R1-R3 equals the definition on every pattern ... ");
    for n in 2..=4 {
        match verify_cpdag_construction(n) {
            Ok(count) => print!("n={n}: {count} ok  "),
            Err(e) => {
                println!("\nFAILED: {e}");
                std::process::exit(1);
            }
        }
    }
    println!();
    print!("R4 fires on its own configuration ... ");
    match positive_control_r4() {
        Ok(()) => println!("ok"),
        Err(e) => {
            println!("\nFAILED: {e}");
            std::process::exit(1);
        }
    }
    println!();

    let mut stats = Stats {
        dags: 0,
        configurations: 0,
        rejected_invalid: 0,
        non_extendable: 0,
        extendable: 0,
        witnesses: 0,
        r4_recovered_all: 0,
        with_free_edge: 0,
        oracle_orients_something: 0,
        r1r3_orients_something: 0,
        r4_fires: 0,
        r4_unsound: 0,
        r4_unsound_example: None,
        f_augment_compels: 0,
        f_augment_example: None,
    };
    let mut witnesses = Vec::new();

    // The bound. Vertex counts are exhaustive; `k` is the number of root causes Algorithm 1
    // ranges over.
    let bound: &[(usize, usize)] = &[(3, 1), (4, 1), (5, 1), (3, 2), (4, 2), (5, 2), (5, 3)];

    for &(n, k) in bound {
        let before = stats.witnesses;
        if let Err(e) = search(n, k, &mut stats, &mut witnesses) {
            println!("HARNESS BUG: {e}");
            std::process::exit(1);
        }
        println!(
            "n={n} k={k}: {} witnesses",
            stats.witnesses.saturating_sub(before)
        );
    }

    println!("\n--- searched ---");
    println!("DAGs enumerated:                   {}", stats.dags);
    println!(
        "cut configurations enumerated:     {}",
        stats.configurations
    );
    println!(
        "  rejected by BRCD's validity pass:   {}",
        stats.rejected_invalid
    );
    println!(
        "  valid but not extendable (skipped): {}",
        stats.non_extendable
    );
    println!(
        "  valid and extendable (searched):     {}",
        stats.extendable
    );

    println!("\n--- is the search non-vacuous? ---");
    println!(
        "inputs with a free edge after orienting the incident set: {}",
        stats.with_free_edge
    );
    println!(
        "inputs the definition orients further:                    {}",
        stats.oracle_orients_something
    );
    println!(
        "inputs production R1-R3 orients further:                  {}",
        stats.r1r3_orients_something
    );

    println!("\n--- harness checks ---");
    println!(
        "negative control (no rules): {} under-oriented inputs",
        stats.oracle_orients_something
    );
    if stats.oracle_orients_something == 0 {
        println!("CONTROL FAILED: the comparison detects no incompleteness even where it exists.");
        println!("The headline result below means nothing. Fix the harness.");
        std::process::exit(1);
    }
    println!(
        "production R1-R4 soundness:  {} over-orientations",
        stats.r4_unsound
    );
    println!(
        "F-augmented graph is closed: {} counterexamples",
        stats.f_augment_compels
    );
    println!(
        "inputs where R4 changes the production closure: {}",
        stats.r4_fires
    );

    let mut failed = false;
    if let Some(example) = &stats.r4_unsound_example {
        println!(
            "\nTHE PRODUCTION CLOSURE IS UNSOUND — it compels an edge the definition does not:"
        );
        println!("  {example}");
        failed = true;
    }
    if let Some(example) = &stats.f_augment_example {
        println!("\nTHE F-AUGMENTED GRAPH IS NOT CLOSED — Algorithm 1 hands `mec_size` an");
        println!("under-oriented PDAG, so the equivalence-class size is over-counted:");
        println!("  {example}");
        failed = true;
    }

    println!("\n--- result ---");
    if stats.witnesses == 0 {
        println!("Production R1-R3 reached the maximally oriented PDAG on every searched input.");
        println!("R4 changed no orientation up to this bound.");
    } else {
        println!(
            "{} of {} searched inputs are under-oriented by production R1-R3.",
            stats.witnesses, stats.extendable
        );
        println!(
            "Production R1-R4 recovers the full orientation on {} of them.",
            stats.r4_recovered_all
        );
        if stats.r4_recovered_all < stats.witnesses {
            println!(
                "On {} it does not — the shipped closure does not reach the maximally oriented",
                stats.witnesses - stats.r4_recovered_all
            );
            println!("PDAG on inputs Algorithm 1 constructs. This is a production defect.");
            failed = true;
        }
        for (idx, w) in witnesses.iter().enumerate() {
            println!("\nwitness {} (n={}, roots={:?})", idx + 1, w.n, w.roots);
            println!("  source DAG:      {}", w.dag);
            println!("  configuration:   {}", w.input);
            println!("  R1-R3 closure:   {}", w.r1r3);
            println!("  definition:      {}", w.truth);
            println!("  left undirected: {:?}", w.missed);
            println!("  R4 recovers it:  {}", w.r4_recovers);
        }
    }

    if failed {
        std::process::exit(1);
    }
}
