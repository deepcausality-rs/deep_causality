use crate::{CsmGraph, GraphAlgorithms, GraphError, GraphView};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::VecDeque;
use std::slice;

/// Private enum representing the state of a node during a DFS traversal.
/// It is not part of the public API.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NodeState {
    /// The node has not yet been visited.
    Unvisited,
    /// The node is currently in the recursion stack (being visited).
    VisitingInProgress,
    /// The node and all its descendants have been fully visited and are known to be cycle-free.
    Visited,
}

impl<N, W> GraphAlgorithms<N, W> for CsmGraph<N, W>
where
    N: Clone,
    W: Clone + Default,
{
    /// Finds a cycle in the graph using an iterative Depth-First Search (DFS).
    ///
    /// This method traverses the graph to detect a back edge, which indicates a cycle.
    /// The iterative approach using an explicit stack makes it robust against stack
    /// overflows, even for very deep or large graphs. The search is comprehensive and
    /// will find a cycle in any of the graph's disconnected components if one exists.
    ///
    /// # Returns
    ///
    /// - `Some(Vec<usize>)`: If a cycle is found, this returns a vector of node indices
    ///   that form the cycle. The path explicitly starts and ends with the same node
    ///   to represent the closed loop (e.g., `[1, 2, 0, 1]`). For a self-loop on
    ///   node `n`, it returns `vec![n, n]`.
    /// - `None`: If the graph is a Directed Acyclic Graph (DAG) and contains no cycles.
    ///
    /// # Complexity
    ///
    /// - **Time Complexity:** O(V + E), where V is the number of nodes and E is the
    ///   number of edges, as each node and edge is visited exactly once.
    /// - **Space Complexity:** O(V) for storing node states, predecessors, and the DFS stack.
    ///
    fn find_cycle(&self) -> Result<Option<Vec<usize>>, GraphError> {
        let num_nodes = self.number_nodes();
        if num_nodes == 0 {
            return Ok(None);
        }

        let mut states = vec![NodeState::Unvisited; num_nodes];
        let mut predecessors = vec![None; num_nodes];

        for i in 0..num_nodes {
            if states[i] == NodeState::Unvisited {
                // We use the concrete `slice::Iter` type to avoid Box and dynamic dispatch.
                let mut stack: Vec<(usize, slice::Iter<'_, usize>)> = Vec::new();

                // By accessing the internal fields directly, we bypass the opaque `impl Trait`
                // returned by the `outbound_edges` trait method.
                let start = self.forward_edges.offsets[i];
                let end = self.forward_edges.offsets[i + 1];
                let neighbors = self.forward_edges.targets[start..end].iter();

                states[i] = NodeState::VisitingInProgress;
                stack.push((i, neighbors));

                while let Some((u_ref, neighbors)) = stack.last_mut() {
                    let u = *u_ref;

                    if let Some(&v) = neighbors.next() {
                        if states[v] == NodeState::VisitingInProgress {
                            // --- Cycle Found: Reconstruct the Path ---
                            let mut path = vec![u];
                            let mut current = u;

                            while let Some(predecessor) = predecessors[current] {
                                path.push(predecessor);
                                if predecessor == v {
                                    break;
                                }
                                current = predecessor;
                            }
                            path.reverse();
                            path.push(v); // Make the cycle explicit: [v, ..., u, v]
                            return Ok(Some(path));
                        }

                        if states[v] == NodeState::Unvisited {
                            predecessors[v] = Some(u);
                            states[v] = NodeState::VisitingInProgress;

                            let v_start = self.forward_edges.offsets[v];
                            let v_end = self.forward_edges.offsets[v + 1];
                            let v_neighbors = self.forward_edges.targets[v_start..v_end].iter();
                            stack.push((v, v_neighbors));
                        }
                    } else {
                        // If the neighbor iterator is exhausted, we are done with this node.
                        states[u] = NodeState::Visited;
                        stack.pop();
                    }
                }
            }
        }

        Ok(None) // No cycles found after checking all nodes.
    }

    /// Checks if the graph contains any directed cycles.
    ///
    /// This implementation leverages the iterative `topological_sort` method,
    /// making it a highly robust way to detect cycles in graphs of any size
    /// without risking a stack overflow.
    fn has_cycle(&self) -> Result<bool, GraphError> {
        match self.topological_sort() {
            Ok(Some(_)) => Ok(false),
            Ok(None) => Ok(true),
            Err(e) => Err(e),
        }
    }

    /// Computes a topological sort of the graph using the iterative Kahn's algorithm.
    ///
    /// This method is robust against stack overflows, making it suitable for graphs
    /// of any size. It works by repeatedly finding nodes with no incoming edges.
    ///
    /// # Returns
    /// - `Some(Vec<usize>)` if the graph is a Directed Acyclic Graph (DAG). The
    ///   vector contains the nodes in a valid topological order.
    /// - `None` if the graph contains a cycle, as a topological sort is not possible.
    fn topological_sort(&self) -> Result<Option<Vec<usize>>, GraphError> {
        let num_nodes = self.number_nodes();
        if num_nodes == 0 {
            return Ok(Some(Vec::new()));
        }

        // 1. Compute in-degrees for all nodes.
        let mut in_degrees = vec![0; num_nodes];
        for i in 0..num_nodes {
            //  Access CSR arrays directly for max performance.
            let start = self.forward_edges.offsets[i];
            let end = self.forward_edges.offsets[i + 1];
            for &neighbor in &self.forward_edges.targets[start..end] {
                in_degrees[neighbor] += 1;
            }
        }

        // 2. Initialize a queue with all nodes that have an in-degree of 0.
        let mut queue: VecDeque<usize> = (0..num_nodes).filter(|&i| in_degrees[i] == 0).collect();

        // 3. Process the queue.
        let mut sorted_list = Vec::with_capacity(num_nodes);
        while let Some(u) = queue.pop_front() {
            sorted_list.push(u);

            // For each neighbor of the dequeued node, decrement its in-degree.
            //  Access CSR arrays directly.
            let start = self.forward_edges.offsets[u];
            let end = self.forward_edges.offsets[u + 1];
            for &v in &self.forward_edges.targets[start..end] {
                in_degrees[v] -= 1;
                if in_degrees[v] == 0 {
                    queue.push_back(v);
                }
            }
        }

        // 4. Validate the result.
        if sorted_list.len() == num_nodes {
            Ok(Some(sorted_list))
        } else {
            Ok(None) // Cycle detected
        }
    }

    /// Checks if a path exists from a start to a stop index.
    fn is_reachable(&self, start_index: usize, stop_index: usize) -> Result<bool, GraphError> {
        match self.shortest_path_len(start_index, stop_index) {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Returns the length of the shortest path (in number of nodes) from a start to a stop index.
    fn shortest_path_len(
        &self,
        start_index: usize,
        stop_index: usize,
    ) -> Result<Option<usize>, GraphError> {
        if !self.contains_node(start_index) || !self.contains_node(stop_index) {
            return Ok(None);
        }
        if start_index == stop_index {
            return Ok(Some(1));
        }

        let mut queue = VecDeque::new();
        let mut visited = vec![false; self.number_nodes()];

        queue.push_back((start_index, 1)); // (node, path_length)
        visited[start_index] = true;

        while let Some((current_node, current_len)) = queue.pop_front() {
            //  Access CSR arrays directly.
            let start = self.forward_edges.offsets[current_node];
            let end = self.forward_edges.offsets[current_node + 1];
            for &neighbor in &self.forward_edges.targets[start..end] {
                if neighbor == stop_index {
                    return Ok(Some(current_len + 1));
                }
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    queue.push_back((neighbor, current_len + 1));
                }
            }
        }
        Ok(None)
    }

    /// Finds the complete shortest path from a start to a stop index.
    fn shortest_path(
        &self,
        start_index: usize,
        stop_index: usize,
    ) -> Result<Option<Vec<usize>>, GraphError> {
        if !self.contains_node(start_index) || !self.contains_node(stop_index) {
            return Ok(None);
        }
        if start_index == stop_index {
            return Ok(Some(vec![start_index]));
        }

        let mut queue = VecDeque::new();
        let mut predecessors = vec![None; self.number_nodes()];
        let mut visited = vec![false; self.number_nodes()];

        queue.push_back(start_index);
        visited[start_index] = true;

        let mut found = false;
        'bfs_loop: while let Some(current_node) = queue.pop_front() {
            //  Access CSR arrays directly.
            let start = self.forward_edges.offsets[current_node];
            let end = self.forward_edges.offsets[current_node + 1];
            for &neighbor in &self.forward_edges.targets[start..end] {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    predecessors[neighbor] = Some(current_node);
                    queue.push_back(neighbor);

                    if neighbor == stop_index {
                        found = true;
                        break 'bfs_loop;
                    }
                }
            }
        }

        if !found {
            return Ok(None);
        }

        // Reconstruct path by walking backwards from the stop index.
        let mut path = Vec::new();
        let mut current = Some(stop_index);
        while let Some(curr_index) = current {
            path.push(curr_index);
            current = predecessors[curr_index];
        }
        path.reverse();
        Ok(Some(path))
    }

    fn shortest_weighted_path(
        &self,
        start_index: usize,
        stop_index: usize,
    ) -> Result<Option<(Vec<usize>, W)>, GraphError>
    where
        W: Copy + Ord + Default + std::ops::Add<Output = W>,
    {
        if !self.contains_node(start_index) || !self.contains_node(stop_index) {
            return Ok(None);
        }

        if start_index == stop_index {
            return Ok(Some((vec![start_index], W::default())));
        }

        let num_nodes = self.number_nodes();
        let mut distances: Vec<Option<W>> = vec![None; num_nodes];
        let mut predecessors: Vec<Option<usize>> = vec![None; num_nodes];
        let mut pq: BinaryHeap<(Reverse<W>, usize)> = BinaryHeap::new();

        distances[start_index] = Some(W::default());
        pq.push((Reverse(W::default()), start_index));

        while let Some((Reverse(dist), u)) = pq.pop() {
            if u == stop_index {
                let mut path = Vec::new();
                let mut current = Some(stop_index);
                while let Some(node) = current {
                    path.push(node);
                    if node == start_index {
                        break;
                    }
                    current = predecessors[node];
                }
                path.reverse();

                return Ok(Some((path, dist)));
            }

            if let Some(known_dist) = distances[u] {
                if dist > known_dist {
                    continue;
                }
            }

            let start_offset = self.forward_edges.offsets[u];
            let end_offset = self.forward_edges.offsets[u + 1];

            for i in start_offset..end_offset {
                let v = self.forward_edges.targets[i];
                let weight = self.forward_edges.weights[i];
                let new_dist = dist + weight;

                if distances[v].is_none() || new_dist < distances[v].unwrap() {
                    distances[v] = Some(new_dist);
                    predecessors[v] = Some(u);
                    pq.push((Reverse(new_dist), v));
                }
            }
        }

        Ok(None)
    }

    /// Finds all Strongly Connected Components in the graph using Tarjan's algorithm.
    ///
    /// # Returns
    /// A vector of vectors, where each inner vector is a list of node indices
    /// belonging to a single SCC.
    fn strongly_connected_components(&self) -> Result<Vec<Vec<usize>>, GraphError> {
        let num_nodes = self.number_nodes();
        if num_nodes == 0 {
            return Ok(Vec::new());
        }

        let mut dfs_num: Vec<Option<usize>> = vec![None; num_nodes];
        let mut low_link: Vec<Option<usize>> = vec![None; num_nodes];
        let mut on_stack: Vec<bool> = vec![false; num_nodes];
        let mut tarjan_stack: Vec<usize> = Vec::new();
        let mut time: usize = 0;
        let mut sccs: Vec<Vec<usize>> = Vec::new();

        // Stack for iterative DFS. Stores (node_index, iterator_over_neighbors)
        // The iterator is used to keep track of which neighbor to visit next.
        let mut dfs_stack: Vec<(usize, slice::Iter<'_, usize>)> = Vec::new();

        for i in 0..num_nodes {
            if dfs_num[i].is_none() {
                // Start DFS from node i
                let start_offset = self.forward_edges.offsets[i];
                let end_offset = self.forward_edges.offsets[i + 1];
                let neighbors_iter = self.forward_edges.targets[start_offset..end_offset].iter();
                dfs_stack.push((i, neighbors_iter));

                // Simulate recursion
                while let Some((u, neighbors)) = dfs_stack.last_mut() {
                    // On first visit to u (pre-order traversal)
                    if dfs_num[*u].is_none() {
                        dfs_num[*u] = Some(time);
                        low_link[*u] = Some(time);
                        time += 1;
                        tarjan_stack.push(*u);
                        on_stack[*u] = true;
                    }

                    // Process neighbors
                    if let Some(&v) = neighbors.next() {
                        if dfs_num[v].is_none() {
                            // Neighbor v not visited, "recurse"
                            let v_start_offset = self.forward_edges.offsets[v];
                            let v_end_offset = self.forward_edges.offsets[v + 1];
                            let v_neighbors_iter =
                                self.forward_edges.targets[v_start_offset..v_end_offset].iter();
                            dfs_stack.push((v, v_neighbors_iter));
                        } else if on_stack[v] {
                            // Neighbor v is on stack, back-edge
                            low_link[*u] = Some(
                                low_link[*u]
                                    .ok_or(GraphError::AlgorithmError("low_link for u not set"))?
                                    .min(dfs_num[v].ok_or(GraphError::AlgorithmError(
                                        "dfs_num for v not set",
                                    ))?),
                            );
                        }
                    } else {
                        // All neighbors processed, "return" from u (post-order traversal)

                        // If u is the root of an SCC
                        if dfs_num[*u] == low_link[*u] {
                            let mut current_scc = Vec::new();
                            loop {
                                let node = tarjan_stack
                                    .pop()
                                    .ok_or(GraphError::AlgorithmError("tarjan_stack is empty"))?;
                                on_stack[node] = false;
                                current_scc.push(node);
                                if node == *u {
                                    break;
                                }
                            }
                            current_scc.reverse(); // SCC nodes are popped in reverse order
                            sccs.push(current_scc);
                        }

                        // Update parent's low_link if u is not the root of an SCC
                        // This must happen AFTER processing the current node's SCC, but BEFORE popping u from dfs_stack
                        let popped_u = dfs_stack
                            .pop()
                            .ok_or(GraphError::AlgorithmError("DFS stack was empty in a post-order step, which should be impossible"))?
                            .0;

                        if let Some((parent_node, _)) = dfs_stack.last() {
                            low_link[*parent_node] = Some(
                                low_link[*parent_node]
                                    .ok_or(GraphError::AlgorithmError(
                                        "low_link for parent_node not set",
                                    ))?
                                    .min(low_link[popped_u].ok_or(GraphError::AlgorithmError(
                                        "low_link for popped_u not set",
                                    ))?),
                            );
                        }
                    }
                }
            }
        }

        Ok(sccs)
    }
}
