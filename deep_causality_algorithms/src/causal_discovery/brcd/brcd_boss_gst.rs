/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Grow-shrink tree (GST) — the per-variable parent-set cache BOSS searches over.
//!
//! A faithful port of causal-learn's `gst.GST` (Andrews et al., NeurIPS 2023).
//! For a fixed target `vertex`, [`Gst::trace`] returns that vertex's
//! best-scoring parent set and score given the variables that **precede** it in
//! the current order (the `prefix`). The tree is grown **lazily** and cached, so
//! the order search — which traces the same vertex against many prefixes —
//! reuses partial results instead of re-scoring from scratch.
//!
//! Scoring is the higher-is-better BIC ([`crate::brcd::brcd_boss_score`]):
//! *grow* keeps parents that strictly **increase** the score; *shrink* removes
//! parents that strictly **increase** the score. With the correct sign that is
//! exactly "add parents that improve the fit beyond their penalty", so the tree
//! recovers real parent sets.

use crate::brcd::BrcdError;
use crate::brcd::brcd_boss_score::FamilyScorer;
use deep_causality_num::RealField;
use std::cmp::Ordering;

/// A node in the grow-shrink tree.
///
/// `add` is the parent this node introduces (`None` at the root). `branches`
/// (the grow step) and `remove` (the shrink step) are filled lazily on first
/// visit and cached thereafter.
struct GstNode<T> {
    add: Option<usize>,
    grow_score: T,
    shrink_score: T,
    branches: Option<Vec<GstNode<T>>>,
    remove: Option<Vec<usize>>,
}

impl<T: RealField> GstNode<T> {
    /// A leaf for parent `add` with its single-parent extension score.
    fn leaf(add: usize, score: T) -> Self {
        Self {
            add: Some(add),
            grow_score: score,
            shrink_score: score,
            branches: None,
            remove: None,
        }
    }

    /// Grows this node: scores every still-available parent as a one-step
    /// extension of `parents`, keeps the strictly-improving ones, and sorts them
    /// ascending by score (matching `GSTNode.__lt__`).
    fn grow<S: FamilyScorer<T>>(
        &mut self,
        vertex: usize,
        available: &[usize],
        parents: &mut Vec<usize>,
        scorer: &S,
    ) -> Result<(), BrcdError> {
        let mut branches = Vec::new();
        for &add in available {
            parents.push(add);
            let score = scorer.score(vertex, parents)?;
            parents.pop();
            if score > self.grow_score {
                branches.push(GstNode::leaf(add, score));
            }
        }
        branches.sort_by(|a, b| {
            a.grow_score
                .partial_cmp(&b.grow_score)
                .unwrap_or(Ordering::Equal)
        });
        self.branches = Some(branches);
        Ok(())
    }

    /// Shrinks `parents`: repeatedly removes the parent whose removal most
    /// improves the score, to a fixpoint. Mutates `parents` in place (dropping
    /// the removed parents) and records them in `self.remove`.
    fn shrink<S: FamilyScorer<T>>(
        &mut self,
        vertex: usize,
        parents: &mut Vec<usize>,
        scorer: &S,
    ) -> Result<(), BrcdError> {
        let mut removed = Vec::new();
        loop {
            let mut best: Option<usize> = None;
            // The score depends only on the parent *set*, so testing each
            // candidate by removing and restoring it (order may shuffle) is safe.
            let candidates = parents.clone();
            for r in candidates {
                let pos = parents.iter().position(|&x| x == r).expect("present");
                parents.remove(pos);
                let score = scorer.score(vertex, parents)?;
                parents.insert(pos, r);
                if score > self.shrink_score {
                    self.shrink_score = score;
                    best = Some(r);
                }
            }
            match best {
                None => break,
                Some(b) => {
                    removed.push(b);
                    parents.retain(|&x| x != b);
                }
            }
        }
        self.remove = Some(removed);
        Ok(())
    }

    /// Traces `prefix` from this node, accumulating the chosen parents into
    /// `parents` and returning the node's shrink score. Lazily grows/shrinks on
    /// first visit; reuses the cached tree afterwards.
    fn trace<S: FamilyScorer<T>>(
        &mut self,
        vertex: usize,
        prefix: &[usize],
        available: &mut Vec<usize>,
        parents: &mut Vec<usize>,
        scorer: &S,
    ) -> Result<T, BrcdError> {
        if self.branches.is_none() {
            self.grow(vertex, available, parents, scorer)?;
        }

        // Follow the first branch whose added parent is in the prefix, removing
        // each visited branch's parent from `available` as we descend (so a
        // child never re-considers an already-used parent).
        let branches = self.branches.as_mut().expect("grown above");
        for branch in branches.iter_mut() {
            let add = branch.add.expect("non-root branch");
            available.retain(|&x| x != add);
            if prefix.contains(&add) {
                parents.push(add);
                return branch.trace(vertex, prefix, available, parents, scorer);
            }
        }

        // No prefix parent extends this node: shrink to the local optimum.
        // First visit: `shrink` mutates `parents` in place (and caches `remove`).
        if self.remove.is_none() {
            self.shrink(vertex, parents, scorer)?;
            return Ok(self.shrink_score);
        }
        // Cached visit: replay the recorded removals onto `parents`.
        if let Some(removed) = &self.remove {
            for &r in removed {
                parents.retain(|&x| x != r);
            }
        }
        Ok(self.shrink_score)
    }
}

/// A grow-shrink tree for one target variable.
pub struct Gst<T> {
    vertex: usize,
    num_vars: usize,
    forbidden: Vec<usize>,
    root: GstNode<T>,
}

impl<T: RealField> Gst<T> {
    /// Builds the tree for `vertex`, seeding the root with the no-parent score.
    ///
    /// # Errors
    /// Propagates any scorer error (e.g. [`crate::brcd::BrcdErrorEnum::NodeOutOfBounds`]
    /// if `vertex` is out of range).
    pub fn new<S: FamilyScorer<T>>(vertex: usize, scorer: &S) -> Result<Self, BrcdError> {
        let empty = scorer.score(vertex, &[])?;
        Ok(Self {
            vertex,
            num_vars: scorer.num_vars(),
            forbidden: vec![vertex],
            root: GstNode {
                add: None,
                grow_score: empty,
                shrink_score: empty,
                branches: None,
                remove: None,
            },
        })
    }

    /// The target variable this tree scores.
    pub fn vertex(&self) -> usize {
        self.vertex
    }

    /// Returns `vertex`'s best parent set (a subset of `prefix`) and its score,
    /// given that the variables in `prefix` precede `vertex` in the order.
    ///
    /// Only variables that are both in `prefix` and not forbidden (i.e. not the
    /// vertex itself) can become parents.
    ///
    /// # Errors
    /// Propagates any scorer error encountered while growing/shrinking.
    pub fn trace<S: FamilyScorer<T>>(
        &mut self,
        prefix: &[usize],
        scorer: &S,
    ) -> Result<(Vec<usize>, T), BrcdError> {
        let mut available: Vec<usize> = (0..self.num_vars)
            .filter(|i| !self.forbidden.contains(i))
            .collect();
        let mut parents = Vec::new();
        let score = self
            .root
            .trace(self.vertex, prefix, &mut available, &mut parents, scorer)?;
        parents.sort_unstable();
        Ok((parents, score))
    }
}
