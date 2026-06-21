/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `O(du)` greedy MAP-configuration finder (BRCD "MAP-config pruning", C1).
//!
//! BRCD scores a root-cause candidate by integrating over every valid orientation
//! of the undirected edges **incident on the candidate set** — a space of size
//! `2^{du}` (before validity filtering), where `du` is the number of incident
//! undirected edges. [`get_configurations_multi`](crate::brcd::brcd_augment::get_configurations_multi)
//! enumerates them all; this module finds, in `O(du)` configuration evaluations,
//! the **dominant** configuration (the MAP cut) plus the small frontier of valid
//! configurations a greedy coordinate walk visits.
//!
//! The finder is a faithful port of the `valid_start` / `greedy` / `evaluate`
//! prototype in `verification/brcd/brcd_heuristic_mapconfig.rs`, but it reuses the
//! **production** config-weight function (a config's representative-DAG data
//! log-likelihood plus `ln(mec_size)`) passed in by the driver, so the finder's
//! ranking is consistent with the scoring in BRCD Phase 2/3.
//!
//! A configuration's weight is
//!
//! ```text
//! w(config) = Σ_node logL(node | parents in the representative DAG over the data) + ln(mec_size(config))
//! ```
//!
//! identical to the per-candidate term the full path assembles. The finder picks
//! the argmax over an `O(du)` walk instead of an exhaustive `2^{du}` scan.
//!
//! For `du == 0` (a fully-directed candidate) the finder returns exactly the one
//! valid configuration, so [`ConfigStrategy::MapPrune`](crate::brcd::ConfigStrategy::MapPrune)
//! is identical to [`ConfigStrategy::Full`](crate::brcd::ConfigStrategy::Full) on
//! directed CPDAGs.

use crate::brcd::brcd_augment::incident_undirected_edges;
use crate::brcd::brcd_error::{BrcdError, BrcdErrorEnum};
use crate::brcd::brcd_validity::{baseline_parents, is_valid_configuration};
use deep_causality_num::RealField;
use deep_causality_topology::MixedGraph;
use std::collections::{BTreeMap, BTreeSet};

/// Maximum incident-undirected-edge count the MAP finder accepts.
///
/// Unlike the full path's
/// [`MAX_CONFIG_EDGES`](crate::brcd::brcd_augment::MAX_CONFIG_EDGES) (which bounds
/// an exponential `2^{du}` enumeration), the finder is `O(du)`, so this is **not**
/// a tractability limit: it is only the width of the `usize` orientation label
/// `bits` (`1usize << du` must stay well-defined), far above any realistic
/// undirected degree.
///
/// Derived from the target's pointer width so the `1usize << du` shifts stay
/// sound on every target (e.g. `62` on 64-bit, `30` on 32-bit). The `- 2` margin
/// keeps `1usize << du` and `(1usize << du) - 1` well below the overflow edge.
const MAX_MAPPRUNE_EDGES: usize = (usize::BITS - 2) as usize;

/// The pruned set of cut configurations found by the greedy MAP finder.
///
/// `configs` are Meek-completed clones of the input CPDAG (one orientation of the
/// incident undirected edges each), ranked best-first by their weight. `evals` is
/// the number of configuration weight evaluations the walk performed (the `O(du)`
/// budget), exposed so callers/tests can assert the near-linear cost.
pub struct PrunedConfigs<N> {
    /// Valid configurations the finder visited, best (highest weight) first.
    pub configs: Vec<MixedGraph<N>>,
    /// Number of configuration weight evaluations performed (`O(du)` budget).
    pub evals: usize,
}

/// Immutable inputs shared by every orientation evaluation during the walk.
struct Finder<'a, N, F> {
    cpdag: &'a MixedGraph<N>,
    targets: &'a [usize],
    /// Incident undirected edges, canonical and ascending; bit `i` is edge `i`.
    incident: &'a [(usize, usize)],
    /// Baseline parents at each target, captured before any orientation.
    baseline: &'a BTreeMap<usize, BTreeSet<usize>>,
    /// Production config-weight function (higher is better).
    weight: F,
}

impl<T, N, F> Finder<'_, N, F>
where
    T: RealField,
    N: Clone,
    F: Fn(&MixedGraph<N>) -> Result<T, BrcdError>,
{
    /// Orients the incident undirected edges of a fresh clone per `bits` (bit `i`
    /// clear ⇒ `a → b`, set ⇒ `b → a`), matching the full path's `combo` bit
    /// convention so both strategies enumerate the same configurations.
    fn orient(&self, bits: usize) -> MixedGraph<N> {
        let mut g = self.cpdag.clone();
        for (i, &(a, b)) in self.incident.iter().enumerate() {
            if (bits >> i) & 1 == 0 {
                g.orient(a, b)
                    .expect("incident edge is undirected in the clone");
            } else {
                g.orient(b, a)
                    .expect("incident edge is undirected in the clone");
            }
        }
        g
    }

    /// Evaluates orientation `bits`: orient, Meek-complete, validity-check, then
    /// score. Returns `Some(w)` if valid, `None` if invalid. **Both** outcomes are
    /// memoized in `visited` (valid as `Some(w)`, invalid as `None`), so a
    /// re-visited orientation returns the cached result without a new evaluation or
    /// Meek completion — the hill-climb revisits the same invalid neighbours across
    /// passes, and caching them keeps `evals` at unique-orientation work.
    fn eval(
        &self,
        bits: usize,
        visited: &mut BTreeMap<usize, Option<T>>,
        evals: &mut usize,
    ) -> Result<Option<T>, BrcdError> {
        if let Some(&cached) = visited.get(&bits) {
            return Ok(cached);
        }
        *evals += 1;
        let mut g = self.orient(bits);
        if !is_valid_configuration(&mut g, self.targets, self.baseline) {
            visited.insert(bits, None);
            return Ok(None);
        }
        let w = (self.weight)(&g)?;
        visited.insert(bits, Some(w));
        Ok(Some(w))
    }

    /// A valid starting orientation: all-out (bits 0), all-in (all set), then the
    /// first valid single-bit flip from all-out. `None` if none is valid.
    fn valid_start(
        &self,
        du: usize,
        visited: &mut BTreeMap<usize, Option<T>>,
        evals: &mut usize,
    ) -> Result<Option<(usize, T)>, BrcdError> {
        let all_in = (1usize << du) - 1;
        for bits in [0usize, all_in] {
            if let Some(w) = self.eval(bits, visited, evals)? {
                return Ok(Some((bits, w)));
            }
        }
        for j in 0..du {
            let bits = 1usize << j;
            if let Some(w) = self.eval(bits, visited, evals)? {
                return Ok(Some((bits, w)));
            }
        }
        Ok(None)
    }
}

/// Finds the MAP cut configuration of `targets` against `cpdag` in `O(du)`
/// configuration evaluations, returning the dominant configuration plus the
/// frontier of valid configurations visited by the greedy coordinate walk.
///
/// `weight` scores one Meek-completed configuration to its real-valued weight —
/// the driver passes a closure that reuses production family scoring so the
/// finder's ranking matches BRCD Phase 2/3. Higher is better.
///
/// The returned configurations are ranked best-first; `configs[0]` is the MAP cut.
/// The list is deduplicated by orientation (a config re-visited across the walk is
/// scored once). For `du == 0` exactly one configuration is returned (matching
/// [`get_configurations_multi`](crate::brcd::brcd_augment::get_configurations_multi)
/// on directed candidates).
///
/// # Errors
/// * [`BrcdErrorEnum::NodeOutOfBounds`] if a target is not a vertex of `cpdag`.
/// * [`BrcdErrorEnum::ConfigSpaceTooLarge`] if more than [`MAX_MAPPRUNE_EDGES`]
///   undirected edges are incident on the candidate set — the `usize`
///   orientation-label width, not a tractability bound (the finder is `O(du)`),
///   so far above the full path's `2^{du}` cap.
/// * any error surfaced by `weight`.
pub fn find_map_configs<T, N, F>(
    cpdag: &MixedGraph<N>,
    targets: &[usize],
    weight: F,
) -> Result<PrunedConfigs<N>, BrcdError>
where
    T: RealField,
    N: Clone,
    F: Fn(&MixedGraph<N>) -> Result<T, BrcdError>,
{
    let n = cpdag.num_vertices();
    if targets.iter().any(|&t| t >= n) {
        return Err(BrcdError(BrcdErrorEnum::NodeOutOfBounds));
    }

    let incident = incident_undirected_edges(cpdag, targets);
    let du = incident.len();
    if du > MAX_MAPPRUNE_EDGES {
        return Err(BrcdError(BrcdErrorEnum::ConfigSpaceTooLarge { edges: du }));
    }

    let baseline = baseline_parents(cpdag, targets);
    let finder = Finder {
        cpdag,
        targets,
        incident: &incident,
        baseline: &baseline,
        weight,
    };

    let mut visited: BTreeMap<usize, Option<T>> = BTreeMap::new();
    let mut evals = 0usize;

    // `du == 0`: a single fully-directed candidate. Meek-complete + validate the
    // lone configuration, exactly as `get_configurations_multi` does for the empty
    // orientation space. This makes MapPrune ≡ Full on directed CPDAGs.
    if du == 0 {
        return match finder.eval(0, &mut visited, &mut evals)? {
            Some(_) => Ok(PrunedConfigs {
                configs: vec![finder.completed(0)],
                evals,
            }),
            None => Ok(PrunedConfigs {
                configs: Vec::new(),
                evals,
            }),
        };
    }

    let Some((mut bits, mut cur)) = finder.valid_start(du, &mut visited, &mut evals)? else {
        // No valid configuration at all (candidate scored as −∞ by the driver).
        return Ok(PrunedConfigs {
            configs: Vec::new(),
            evals,
        });
    };

    // Hill-climb: repeatedly take the best improving single-bit flip until none
    // improves. Each pass is `O(du)` evaluations; the weight is bounded above by
    // the (finite) MAP weight and strictly increases each accepted step, so the
    // walk converges in `O(du)` passes — `O(du²)` evaluations worst case, near
    // `O(du)` in practice.
    loop {
        let mut best: Option<(usize, T)> = None;
        for j in 0..du {
            let cand = bits ^ (1usize << j);
            if let Some(w) = finder.eval(cand, &mut visited, &mut evals)? {
                let improves = w > cur;
                let beats_best = best.as_ref().is_none_or(|(_, bw)| w > *bw);
                if improves && beats_best {
                    best = Some((cand, w));
                }
            }
        }
        match best {
            Some((cand, w)) => {
                bits = cand;
                cur = w;
            }
            None => break,
        }
    }

    // Rank the visited valid configs best-first; the MAP cut is first. The cache
    // also holds the memoized invalid orientations (`None`), which are dropped here.
    let mut ranked: Vec<(usize, T)> = visited
        .into_iter()
        .filter_map(|(bits, w)| w.map(|w| (bits, w)))
        .collect();
    ranked.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            // Tie-break on bits for a deterministic order.
            .then_with(|| a.0.cmp(&b.0))
    });

    let configs = ranked
        .iter()
        .map(|(cfg_bits, _)| finder.completed(*cfg_bits))
        .collect();

    Ok(PrunedConfigs { configs, evals })
}

impl<T, N, F> Finder<'_, N, F>
where
    T: RealField,
    N: Clone,
    F: Fn(&MixedGraph<N>) -> Result<T, BrcdError>,
{
    /// The Meek-completed clone for orientation `bits`. Meek completion is
    /// deterministic, so this reproduces exactly the graph that was scored during
    /// the walk (the validity result is discarded; callers only reach this for
    /// orientations already known valid).
    fn completed(&self, bits: usize) -> MixedGraph<N> {
        let mut g = self.orient(bits);
        let _ = is_valid_configuration(&mut g, self.targets, self.baseline);
        g
    }
}
