

# Specfile: `deep_causality_topology`

**Version:** 0.1.0
**Type:** Internal Core Crate
**Architecture:** `no_std` compatible (with `alloc`).

## 1. Dependencies (Internal)

```toml
[dependencies]
deep_causality_haft = { path = "../haft" }        # Traits: BoundedComonad, Adjunction
deep_causality_tensor = { path = "../tensor" }    # Data Storage
deep_causality_multivector = { path = "../multivector" } # Metric/Algebra
deep_causality_sparse = { path = "../deep_causality_sparse" }    # CSR Matrices
```

---

## 2. Core Data Structures

### 2.1. The Simplex & Skeleton
*Implementation Note: To maintain zero dependencies (dropping `indexmap`), we use a sorted `Vec` with binary search for index lookups. This is $O(\log N)$ for setup and $O(1)$ for access once indices are baked into the Sparse Matrix.*

```rust
/// A combinatorial simplex defined by sorted vertex indices.
/// Order is strictly increasing to ensure canonical representation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Simplex {
    pub vertices: Vec<usize>, 
}

/// A collection of all simplices of dimension K.
pub struct Skeleton {
    pub dim: usize,
    /// Canonical list of simplices. The index in this vector is the "Global ID".
    pub simplices: Vec<Simplex>,
}

impl Skeleton {
    /// Find the global index of a simplex via binary search.
    pub fn get_index(&self, simplex: &Simplex) -> Option<usize> {
        self.simplices.binary_search(simplex).ok()
    }
}
```

### 2.2. The Causal Complex (The Container)
This struct holds the "Static Topology." It uses `CausalSparse` from your new crate to store relationships.

```rust
pub struct CausalComplex {
    /// Storage of geometric entities (Points, Lines, Triangles...)
    pub skeletons: Vec<Skeleton>,
    
    /// The Boundary Operators (∂). 
    /// boundary[k] is a matrix of size (N_{k-1} x N_k).
    /// Maps a k-chain to a (k-1)-chain.
    /// 
    /// Implementation: deep_causality_sparse::CausalSparse<i8>
    /// Values are {-1, 0, 1} representing orientation.
    pub boundary_operators: Vec<CausalSparse<i8>>,

    /// The Coboundary / Adjacency Cache (Optional but recommended for Comonad speed).
    /// Transpose of boundary operators.
    /// coboundary[k] is a matrix of size (N_{k+1} x N_k).
    /// Used to find "Who contains me?" efficiently.
    pub coboundary_operators: Vec<CausalSparse<i8>>,
}
```

---

## 3. The "Field" Types (Data on Topology)

### 3.1. `CausalTopology<T>` (The Cochain)
This is the **Comonad Witness**. It represents a Differential Form (Field) defined on the mesh.

```rust
/// Represents a discrete field defined on the k-skeleton.
/// (e.g., Temperature on Vertices, Magnetic Flux on Faces).
pub struct CausalTopology<T> {
    /// Shared reference to the underlying mesh
    pub complex: Arc<CausalComplex>,
    
    /// The dimension of the simplices this data lives on
    pub grade: usize,
    
    /// The values (CausalTensor is essentially a dense vector here)
    pub data: CausalTensor<T>,
    
    /// The Focus (Cursor) for Comonadic extraction
    pub cursor: usize,
}
```

### 3.2. `Chain<T>` (The Geometric Selection)
This represents a geometric object (like a path or a surface) used for integration.

```rust
/// Represents a weighted collection of simplices.
/// (e.g., A path is a Chain<f64> on the 1-skeleton where weights are 1.0).
pub struct Chain<T> {
    pub complex: Arc<CausalComplex>,
    pub grade: usize,
    
    /// Sparse vector of active simplices. 
    /// Reuses CausalSparse logic (1 row, N cols) for efficient sparse operations.
    pub weights: CausalSparse<T>, 
}
```

---

## 4. Trait Implementations

### 4.1. `BoundedComonad` (The Stencil Engine)
This enables local physics (e.g., Laplacian) to be defined as a closure and applied globally.

```rust
impl BoundedComonad<CausalTopologyWitness> for CausalTopologyWitness {
    fn extract<A>(fa: &CausalTopology<A>) -> A 
    where A: Clone {
        fa.data.get_flat(fa.cursor).cloned().expect("Cursor OOB")
    }

    fn extend<A, B, Func>(fa: &CausalTopology<A>, mut f: Func) -> CausalTopology<B>
    where
        Func: FnMut(&CausalTopology<A>) -> B,
        A: Zero + Copy + Clone,
        B: Zero + Copy + Clone
    {
        let size = fa.data.size();
        let mut result_vec = Vec::with_capacity(size);

        // OPTIMIZATION: 
        // Instead of allocating a new View struct every iteration,
        // we keep the topology constant and only move the cursor integer.
        // The closure `f` receives a lightweight view.
        
        for i in 0..size {
            // 1. Create View centered at i
            // We can clone 'fa' cheaply because 'complex' is Arc 
            // and 'data' is ref-counted or cloned (depending on tensor impl).
            // For max speed, we'd introduce a View struct, but cloning struct wrapper is fine.
            let mut view = fa.clone_shallow(); 
            view.cursor = i;

            // 2. Apply Physics
            // The user's function 'f' will likely call view.laplacian() or view.neighbors()
            let val = f(&view);
            result_vec.push(val);
        }

        CausalTopology {
            complex: fa.complex.clone(),
            grade: fa.grade,
            data: CausalTensor::new(result_vec, vec![size]).unwrap(),
            cursor: 0,
        }
    }
}
```

### 4.2. `Adjunction` (Exact Conservation)
This enforces $\langle d\omega, c \rangle = \langle \omega, \partial c \rangle$. It uses the **Sparse Matrix Transpose** from `deep_causality_sparse`.

```rust
impl Adjunction<ChainWitness, CausalTopologyWitness> for CausalComplex {
    
    /// Left Adjunct: The Boundary Operator (∂)
    /// Maps Geometry -> Geometry (Chain_k -> Chain_{k-1})
    fn left_adjunct<A, B, F>(chain: Chain<A>, f: F) -> CausalTopology<B>
    where F: Fn(Chain<A>) -> B {
        // 1. Get the Boundary Operator (Sparse Matrix) for this grade
        let boundary_op = &chain.complex.boundary_operators[chain.grade];
        
        // 2. Apply Matrix: New Chain = Matrix * Old Chain
        // This calculates the geometric boundary exactly.
        // (Requires CausalSparse::apply to work on sparse vectors)
        let boundary_weights = boundary_op.apply(&chain.weights);
        
        // ... wrap and return
    }
    
    // Right Adjunct logic (Exterior Derivative) maps Cochain -> Cochain
    // using boundary_op.transpose()
}
```

---

## 5. The Regge Geometry (Metric Generation)

This bridges the Thesis (Edge Lengths) with your Algebra (`MultiVector`).

```rust
pub struct ReggeGeometry {
    // Lengths of the 1-simplices (Edges)
    pub edge_lengths: CausalTensor<f64>, 
}

impl ReggeGeometry {
    /// Computes the Riemannian Metric for a specific simplex.
    /// Used to initialize CausalMultiVector with correct signature.
    pub fn metric_at(&self, complex: &CausalComplex, grade: usize, index: usize) -> Metric {
        // 1. Retrieve indices of all edges in this simplex
        // (Using complex.skeletons[grade].simplices[index])
        let simplex = &complex.skeletons[grade].simplices[index];
        
        // 2. Retrieve lengths from edge_lengths tensor
        // 3. Construct Cayley-Menger Gram Matrix (from Thesis Eq 29)
        // 4. Check signature (Eigenvalues)
        
        // 5. Return Metric
        // If eigenvalues are all positive -> Metric::Euclidean(k)
        // If mixed -> Metric::Minkowski(k) or Metric::Custom
        // If zero eigenvalue -> Metric::PGA (Degenerate)
    }
}
```

---

## 6. The Haruna Formalism (Quantum Gates)

This logic belongs in `CausalTopology`. It uses the **Cup Product** to interact fields.

```rust
impl<T> CausalTopology<T> where T: MultiVector {
    /// The Cup Product: (k-form) U (l-form) -> (k+l)-form
    /// Used for Generative Quantum Gates: S = exp(a U a)
    pub fn cup_product(&self, other: &CausalTopology<T>) -> CausalTopology<T> {
        let k = self.grade;
        let l = other.grade;
        let dim = k + l;
        let target_skeleton = &self.complex.skeletons[dim];
        
        let mut result_data = Vec::with_capacity(target_skeleton.simplices.len());

        // Iterate over all (k+l)-simplices
        for simplex in &target_skeleton.simplices {
            // 1. Alexander-Whitney Approximation
            // Split vertices [0...n] into Front [0...k] and Back [k...n]
            let front_face = simplex.subsimplex(0..=k);
            let back_face = simplex.subsimplex(k..=dim);
            
            // 2. Look up values
            let front_idx = self.complex.skeletons[k].get_index(&front_face).unwrap();
            let back_idx = self.complex.skeletons[l].get_index(&back_face).unwrap();
            
            let v1 = self.data.get_flat(front_idx).unwrap();
            let v2 = other.data.get_flat(back_idx).unwrap();
            
            // 3. Geometric Product
            result_data.push(v1.geometric_product(v2));
        }
        
        // ... return new topology
    }
}
```

---

## 7. Summary of Operations

| Operation | Physics Meaning | Implementation |
| :--- | :--- | :--- |
| **`boundary_op.apply(chain)`** | Boundary ($\partial$) | `CausalSparse` Matrix-Vector Mul |
| **`boundary_op.transpose().apply(cochain)`** | Ext. Derivative ($d$) | `CausalSparse` Transpose Mul |
| **`cup_product(a, b)`** | Interaction / Logic Gate | Alexander-Whitney Combinatorics |
| **`extend(f)`** | Stencil / Local Law | `BoundedComonad` Iterator |
| **`metric_at(i)`** | Intrinsic Geometry | Regge Calculus $\to$ `Metric` Enum |

This spec provides the complete blueprint to implement the **Topological Layer** with zero external dependencies, reusing your optimized Sparse Matrix and MultiVector engines.