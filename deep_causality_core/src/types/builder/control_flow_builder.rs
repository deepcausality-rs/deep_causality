/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::builder::executable_node_type::NodeType;
use crate::{CausalProtocol, FromProtocol, ToProtocol};
use crate::{ExecutableEdge, ExecutableGraph, ExecutableNode};

#[cfg(all(feature = "alloc", not(feature = "strict-zst")))]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

pub struct ControlFlowBuilder<P> {
    nodes: Vec<ExecutableNode<P>>,
    edges: Vec<ExecutableEdge>,
}

impl<P: CausalProtocol> ControlFlowBuilder<P> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Adds a causaloid (function) to the graph.
    ///
    /// # Arguments
    /// * `logic`: A strictly typed function `Fn(I) -> O`.
    ///
    /// If `strict-zst` feature is enabled:
    /// `logic` MUST be a Zero-Sized Type (ZST), i.e., a function item, not a closure or pointer.
    ///
    /// # Returns
    /// A `NodeType<I, O>` that preserves the type information for linking.
    pub fn add_node<I, O, F>(&mut self, logic: F) -> NodeType<I, O>
    where
        F: Fn(I) -> O + Copy + Send + Sync + 'static,
        I: FromProtocol<P> + 'static,
        O: ToProtocol<P> + 'static,
    {
        let id = self.nodes.len();

        #[cfg(feature = "strict-zst")]
        {
            // Suppress unused variable warning as we use F type, not the value
            let _ = logic;

            // Enforce ZST requirement for strict safety
            if core::mem::size_of::<F>() != 0 {
                panic!(
                    "ControlFlowBuilder only supports ZST function items (static functions) for safety reasons."
                );
            }

            // THE ADAPTER:
            // A static function wrapper that constructs the ZST function F on the fly.
            // This avoids Box<dyn Fn> and closures, using only a plain function pointer.
            fn adapter_wrapper<P, I, O, F>(input_enum: P) -> P
            where
                P: CausalProtocol,
                I: FromProtocol<P>,
                O: ToProtocol<P>,
                F: Fn(I) -> O + Copy + Send + Sync + 'static,
            {
                // Safety: We checked that F is ZST.
                // Constructing a ZST from zeroed memory is safe for function items.
                let f: F = unsafe { core::mem::MaybeUninit::zeroed().assume_init() };

                match I::from_protocol(input_enum) {
                    Ok(typed_input) => {
                        let typed_output = f(typed_input);
                        typed_output.to_protocol()
                    }
                    Err(e) => {
                        // Note: We cannot capture 'id' in a plain function pointer.
                        P::error(&e)
                    }
                }
            }

            self.nodes.push(ExecutableNode {
                id,
                func: adapter_wrapper::<P, I, O, F>,
            });
        }

        #[cfg(not(feature = "strict-zst"))]
        {
            // THE ADAPTER:
            // This closure acts as the firewall between the untyped runtime
            // and the strictly typed user logic.
            let adapter = move |input_enum: P| -> P {
                // 1. Try to extract the strictly typed input (I) from the Enum (P).
                // Note: In a correctly wired graph, this 'match' will always succeed.
                // The error branch is unreachable code via topological guarantee,
                // but required for Rust safety.
                match I::from_protocol(input_enum) {
                    Ok(typed_input) => {
                        // 2. Execute the user's logic
                        let typed_output = logic(typed_input);
                        // 3. Wrap the result back into the Enum (P)
                        typed_output.to_protocol()
                    }
                    Err(e) => {
                        // Runtime safety net
                        // We can capture ID here, but to keep it zero-alloc we pass the error directly.
                        // If we want to include ID, we'd need a stack-based formatter or similar,
                        // but for now we prioritize zero-alloc.
                        P::error(&e)
                    }
                }
            };

            self.nodes.push(ExecutableNode {
                id,
                func: Box::new(adapter),
            });
        }

        NodeType::new(id)
    }

    /// Creates a directed connection between two nodes.
    ///
    /// # Compiler Enforcement
    /// This function signature enforces that the Output type of the Source
    /// matches the Input type of the Target.
    ///
    /// If `T` does not match, the code will fail to compile with E0308.
    pub fn connect<S, T, U>(&mut self, source: NodeType<S, T>, target: NodeType<T, U>) {
        self.edges.push(ExecutableEdge {
            from: source.id,
            to: target.id,
        });
    }

    /// Finalizes the builder and returns the executable graph.
    /// This consumes the builder (Type State Pattern).
    pub fn build(self) -> ExecutableGraph<P> {
        // Optimization: Convert edge list to adjacency matrix if needed.
        let mut adjacency = vec![Vec::new(); self.nodes.len()];
        for edge in self.edges {
            adjacency[edge.from].push(edge.to);
        }

        ExecutableGraph {
            nodes: self.nodes,
            adjacency,
        }
    }
}

impl<P: CausalProtocol> Default for ControlFlowBuilder<P> {
    fn default() -> Self {
        Self::new()
    }
}
