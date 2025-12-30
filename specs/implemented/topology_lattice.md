# Lattice Topology Extension Specification

## 1. Overview

This specification defines extensions to `deep_causality_topology` for **lattice structures** and related
discrete topological objects. These are foundational mathematical abstractions used across:

- **Lattice gauge theory and field theory**: Wilson loops, plaquettes, gauge transformations
- **Discrete differential geometry**: Discrete exterior calculus, Hodge decomposition
- **Crystallography and condensed matter physics**: Band structure, defect topology, Brillouin zones
- **Graph-based algorithms on regular grids**: Image processing, finite difference methods

**Scope:** Pure abstract mathematics. No application-specific (QEC, physics simulation) content.

---

## 2. Mathematical Background

### 2.1 Lattice vs Simplicial Complex

| Structure | Cells | Regularity | Dual |
|-----------|-------|------------|------|
| Simplicial Complex | Simplices (triangles, tetrahedra) | Irregular | Not canonical |
| **CW Complex** | Arbitrary cells | Irregular | Poincaré dual |
| **Lattice** | Hypercubes | **Regular** | **Natural dual** |

A **lattice** is a regular CW complex where all k-cells are k-dimensional hypercubes arranged in a grid.

### 2.2 Key Concepts

- **Primal lattice**: The original grid (vertices, edges, faces, cubes, ...)
- **Dual lattice**: Each k-cell maps to a (D-k)-cell (vertices ↔ cubes, edges ↔ faces)
- **Boundary operator**: ∂: C_k → C_{k-1} (k-chains to (k-1)-chains)
- **Coboundary operator**: δ: C_k → C_{k+1} (adjoint of ∂)
- **Homology**: H_k = ker(∂_k) / im(∂_{k+1})

### 2.3 Applications in Condensed Matter Physics

| Concept | Lattice Representation | Physical Meaning |
|---------|------------------------|------------------|
| **Crystal structure** | Lattice<3> vertices | Atomic positions |
| **Bonds** | 1-cells (edges) | Nearest-neighbor hopping |
| **Plaquettes** | 2-cells (faces) | Magnetic flux |
| **Brillouin zone** | Dual lattice | Momentum space |
| **Defects** | Non-trivial homology | Dislocations, vortices |
| **Berry phase** | Holonomy around cycles | Topological invariants |

The homology computation H_k directly computes:
- **H_0**: Connected components (domains)
- **H_1**: Non-contractible loops (vortex lines, dislocation lines)
- **H_2**: Enclosed voids (bubble defects)

---

## 3. New Types

### 3.1 `Lattice<const D: usize>`

A D-dimensional regular lattice with optional periodic boundaries.

```rust
/// A D-dimensional regular lattice.
pub struct Lattice<const D: usize> {
    /// Dimensions of the lattice [L₀, L₁, ..., L_{D-1}]
    shape: [usize; D],
    /// Periodic boundary conditions per dimension
    periodic: [bool; D],
}

impl<const D: usize> Lattice<D> {
    // --- Constructors ---
    
    /// Create a new lattice with given shape and boundary conditions.
    pub fn new(shape: [usize; D], periodic: [bool; D]) -> Self;
    
    /// Create a fully periodic (toroidal) lattice.
    pub fn torus(shape: [usize; D]) -> Self;
    
    /// Create an open-boundary lattice.
    pub fn open(shape: [usize; D]) -> Self;
    
    // --- Getters ---
    
    /// Shape of the lattice [L₀, L₁, ..., L_{D-1}].
    pub fn shape(&self) -> &[usize; D];
    
    /// Periodic boundary conditions per dimension.
    pub fn periodic(&self) -> &[bool; D];
    
    /// Dimension of the lattice (always D).
    pub const fn dim(&self) -> usize { D }
    
    // --- Cell Access ---
    
    /// Total number of k-cells in the lattice.
    pub fn num_cells(&self, k: usize) -> usize;
    
    /// Iterator over all k-cells (as multi-indices + orientation).
    pub fn cells(&self, k: usize) -> impl Iterator<Item = LatticeCell<D>>;
    
    /// Get the boundary of a k-cell as a chain of (k-1)-cells.
    pub fn boundary(&self, cell: &LatticeCell<D>) -> Chain<LatticeCell<D>>;
    
    /// Get the coboundary of a k-cell as a chain of (k+1)-cells.
    pub fn coboundary(&self, cell: &LatticeCell<D>) -> Chain<LatticeCell<D>>;
}
```

### 3.2 `LatticeCell<const D: usize>`

A single k-cell in a D-dimensional lattice.

```rust
/// A k-cell in a D-dimensional lattice.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LatticeCell<const D: usize> {
    /// Base vertex position [x₀, x₁, ..., x_{D-1}]
    position: [usize; D],
    /// Orientation mask: bit i set means cell extends in dimension i
    /// k = popcount(orientation) is the cell dimension
    orientation: u32,
}

impl<const D: usize> LatticeCell<D> {
    // --- Constructors ---
    
    /// Create a new cell at the given position with orientation mask.
    pub fn new(position: [usize; D], orientation: u32) -> Self;
    
    /// Create a vertex (0-cell) at the given position.
    pub fn vertex(position: [usize; D]) -> Self;
    
    /// Create an edge (1-cell) at position extending in dimension `dir`.
    pub fn edge(position: [usize; D], dir: usize) -> Self;
    
    // --- Getters ---
    
    /// Base vertex position.
    pub fn position(&self) -> &[usize; D];
    
    /// Orientation bitmask.
    pub fn orientation(&self) -> u32;
    
    /// The dimension k of this cell.
    pub fn cell_dim(&self) -> usize;
    
    // --- Predicates ---
    
    /// Is this a vertex (0-cell)?
    pub fn is_vertex(&self) -> bool;
    
    /// Is this an edge (1-cell)?
    pub fn is_edge(&self) -> bool;
    
    /// Is this a face (2-cell)?
    pub fn is_face(&self) -> bool;
    
    // --- Operations ---
    
    /// The dual cell in the dual lattice.
    pub fn dual(&self) -> LatticeCell<D>;
    
    /// Vertices of this cell.
    pub fn vertices(&self) -> Vec<[usize; D]>;
}
```

### 3.3 `DualLattice<const D: usize>`

The Poincaré dual of a lattice.

```rust
/// The dual of a lattice, where k-cells become (D-k)-cells.
pub struct DualLattice<const D: usize> {
    primal: Lattice<D>,
}

impl<const D: usize> DualLattice<D> {
    /// Create the dual of a primal lattice.
    pub fn new(primal: Lattice<D>) -> Self;
    
    /// Access the primal lattice.
    pub fn primal(&self) -> &Lattice<D>;
    
    /// Map a primal k-cell to its dual (D-k)-cell.
    pub fn dual_cell(&self, cell: &LatticeCell<D>) -> LatticeCell<D>;
    
    /// The primal boundary operator becomes the dual coboundary.
    pub fn coboundary(&self, cell: &LatticeCell<D>) -> Chain<LatticeCell<D>>;
}
```

### 3.4 `CellComplex<C>`

Generalization of simplicial complexes to arbitrary cell shapes.

```rust
/// A CW complex with arbitrary cell types.
pub struct CellComplex<C: Cell> {
    /// cells[k] = all k-cells (private, access via methods)
    cells: Vec<Vec<C>>,
    /// incidence[k] = ∂_k matrix (private, computed lazily)
    incidence: Vec<SparseMatrix<i8>>,
}

pub trait Cell: Clone + Eq + Hash {
    /// Dimension of this cell.
    fn dim(&self) -> usize;
    
    /// Boundary as signed sum of lower-dimensional cells.
    fn boundary(&self) -> Chain<Self>;
}

impl<C: Cell> CellComplex<C> {
    // --- Constructors ---
    
    /// Build from a collection of cells with incidence relations.
    pub fn from_cells(cells: Vec<C>) -> Self;
    
    // --- Getters ---
    
    /// All k-cells in the complex.
    pub fn cells(&self, k: usize) -> &[C];
    
    /// Number of k-cells.
    pub fn num_cells(&self, k: usize) -> usize;
    
    /// Maximum dimension of cells in the complex.
    pub fn dimension(&self) -> usize;
    
    // --- Operators ---
    
    /// The boundary operator ∂_k as a sparse matrix.
    pub fn boundary_matrix(&self, k: usize) -> &SparseMatrix<i8>;
    
    // --- Homology ---
    
    /// Compute the k-th Betti number β_k = dim(H_k).
    pub fn betti_number(&self, k: usize) -> usize;
    
    /// A basis for the k-th homology group H_k.
    pub fn homology_basis(&self, k: usize) -> Vec<Chain<C>>;
}
```

### 3.5 `BoundaryOperator<C>`

The discrete boundary operator ∂.

```rust
/// The boundary operator ∂: C_k → C_{k-1} for a cell complex.
pub struct BoundaryOperator<C: Cell> {
    /// Sparse matrix representation (rows = (k-1)-cells, cols = k-cells)
    matrix: SparseMatrix<i8>,
    /// Reference to the complex
    complex: Arc<CellComplex<C>>,
    /// Dimension k of the source chains
    k: usize,
}

impl<C: Cell> BoundaryOperator<C> {
    // --- Getters ---
    
    /// The sparse matrix representation.
    pub fn matrix(&self) -> &SparseMatrix<i8>;
    
    /// Source dimension k.
    pub fn source_dim(&self) -> usize;
    
    /// Target dimension k-1.
    pub fn target_dim(&self) -> usize;
    
    // --- Operations ---
    
    /// Apply ∂ to a k-chain.
    pub fn apply(&self, chain: &Chain<C>) -> Chain<C>;
    
    /// The adjoint coboundary operator δ = ∂*.
    pub fn adjoint(&self) -> CoboundaryOperator<C>;
    
    /// Kernel of ∂ (cycles).
    pub fn kernel(&self) -> Vec<Chain<C>>;
    
    /// Image of ∂ (boundaries).
    pub fn image(&self) -> Vec<Chain<C>>;
}
```

---

## 4. Higher-Kinded Type Extensions

Following the pattern established in `deep_causality_topology/src/extensions/`, we define HKT witnesses
for the new types to enable generic functional programming patterns.

### 4.1 `LatticeWitness`

```rust
// deep_causality_topology/src/extensions/hkt_lattice.rs

use deep_causality_haft::{HKT, BoundedAdjunction};

/// HKT witness for Lattice<D> as a functor over field values.
pub struct LatticeWitness<const D: usize>;

impl<const D: usize> HKT for LatticeWitness<D> {
    /// Lattice with field values of type T at each k-cell.
    type Type<T> = LatticeField<D, T>;
}

/// A field (assignment of values) over lattice cells.
pub struct LatticeField<const D: usize, T> {
    lattice: Arc<Lattice<D>>,
    grade: usize,  // Which k-cells hold values
    values: CausalTensor<T>,
}

impl<const D: usize> BoundedAdjunction<LatticeWitness<D>, TopologyWitness, Arc<Lattice<D>>>
    for Lattice<D>
{
    /// Left Adjunct: Lift a function on cell values to a topology.
    fn left_adjunct<A, B, F>(ctx: &Arc<Lattice<D>>, a: A, f: F) -> Topology<B>
    where
        F: Fn(LatticeField<D, A>) -> B,
        A: Clone,
        B: Clone,
    {
        // Implementation: Create basis fields, apply f, collect results
        todo!()
    }

    /// Right Adjunct: Integrate a field-valued function over a chain.
    fn right_adjunct<A, B, F>(ctx: &Arc<Lattice<D>>, field: LatticeField<D, A>, f: F) -> B
    where
        F: FnMut(A) -> Topology<B>,
        A: Clone,
        B: Clone + Zero + Add<Output = B>,
    {
        // Implementation: Sum f(value) over all cells weighted by chain
        todo!()
    }

    /// Unit: Embed a scalar into a constant field.
    fn unit<A>(ctx: &Arc<Lattice<D>>, a: A) -> Topology<LatticeField<D, A>>
    where
        A: Clone,
    {
        // Implementation: Constant field with value a everywhere
        todo!()
    }

    /// Counit: Integrate a field of topologies.
    fn counit<B>(ctx: &Arc<Lattice<D>>, field: LatticeField<D, Topology<B>>) -> B
    where
        B: Clone + Zero + Add<Output = B>,
    {
        // Implementation: Sum all values
        todo!()
    }
}
```

### 4.2 `CellComplexWitness`

```rust
// deep_causality_topology/src/extensions/hkt_cell_complex.rs

/// HKT witness for CellComplex<C> as a functor over field values.
pub struct CellComplexWitness<C: Cell>;

impl<C: Cell> HKT for CellComplexWitness<C> {
    /// CellComplex with field values at each cell.
    type Type<T> = CellField<C, T>;
}

/// A field over an arbitrary cell complex.
pub struct CellField<C: Cell, T> {
    complex: Arc<CellComplex<C>>,
    grade: usize,
    values: CausalTensor<T>,
}
```

### 4.3 Extension Pattern Summary

| Type | Witness | HKT Functor |
|------|---------|-------------|
| `SimplicialComplex` | `ChainWitness` | `Chain<T>` |
| `Graph` | `GraphWitness` | `Graph<N, E>` |
| `Lattice<D>` | `LatticeWitness<D>` | `LatticeField<D, T>` |
| `CellComplex<C>` | `CellComplexWitness<C>` | `CellField<C, T>` |

---

## 5. Integration with Existing Types

### 5.1 Chain Compatibility

The existing `Chain<T>` type works directly with lattice cells:

```rust
// Usage with lattice cells
let edge: LatticeCell<3> = lattice.cells(1).next().unwrap();
let boundary: Chain<LatticeCell<3>> = lattice.boundary(&edge);
```

### 5.2 SimplicialComplex Embedding

A simplicial complex can be embedded in a lattice for barycentric subdivision:

```rust
impl<const D: usize> Lattice<D> {
    /// Triangulate the lattice into a simplicial complex.
    /// Each D-cube becomes D! simplices via barycentric subdivision.
    pub fn triangulate(&self) -> SimplicialComplex;
}
```

### 5.3 Graph Extraction

Extract the 1-skeleton as a Graph:

```rust
impl<const D: usize> Lattice<D> {
    /// The 1-skeleton (vertices + edges) as a Graph.
    pub fn skeleton_graph(&self) -> Graph<[usize; D], ()>;
}
```

---

## 6. Common Lattice Types

### 6.1 Factory Functions

```rust
/// Standard lattice constructors.
impl Lattice<2> {
    /// L×L square lattice with periodic boundaries (torus).
    pub fn square_torus(l: usize) -> Self;
    
    /// L×L square lattice with open boundaries.
    pub fn square_open(l: usize) -> Self;
}

impl Lattice<3> {
    /// L×L×L cubic lattice with periodic boundaries (3-torus).
    pub fn cubic_torus(l: usize) -> Self;
    
    /// L×L×L cubic lattice with open boundaries.
    pub fn cubic_open(l: usize) -> Self;
}

impl Lattice<4> {
    /// L^4 hypercubic lattice (for lattice gauge theory).
    pub fn hypercubic_torus(l: usize) -> Self;
}
```

### 6.2 Specialized Lattices

```rust
/// Honeycomb (hexagonal) lattice in 2D.
/// Used in graphene, Haldane model, Kitaev model.
pub struct HoneycombLattice {
    size: [usize; 2],
    periodic: [bool; 2],
}

impl HoneycombLattice {
    pub fn new(size: [usize; 2], periodic: [bool; 2]) -> Self;
    pub fn size(&self) -> &[usize; 2];
    pub fn periodic(&self) -> &[bool; 2];
    
    /// Convert to CellComplex for homology computations.
    pub fn as_cell_complex(&self) -> CellComplex<HoneycombCell>;
}

/// Triangular lattice in 2D.
/// Used in antiferromagnetic systems, spin liquids.
pub struct TriangularLattice {
    size: [usize; 2],
    periodic: [bool; 2],
}

/// Kagome lattice in 2D.
/// Used in frustrated magnets, spin liquids.
pub struct KagomeLattice {
    size: [usize; 2],
    periodic: [bool; 2],
}

/// Heavy-Hex lattice (IBM quantum hardware topology).
pub struct HeavyHexLattice {
    rows: usize,
    cols: usize,
}

impl HeavyHexLattice {
    pub fn new(rows: usize, cols: usize) -> Self;
    pub fn rows(&self) -> usize;
    pub fn cols(&self) -> usize;
}
```

---

## 7. File Structure

```
deep_causality_topology/src/
├── types/
│   ├── lattice/                        # NEW
│   │   ├── mod.rs
│   │   ├── lattice.rs                  # Lattice<D> struct
│   │   ├── lattice_cell.rs             # LatticeCell<D> struct
│   │   ├── lattice_field.rs            # LatticeField<D, T> struct
│   │   ├── dual_lattice.rs             # DualLattice<D> struct
│   │   ├── constructors.rs             # Factory functions
│   │   └── specialized/
│   │       ├── mod.rs
│   │       ├── honeycomb.rs            # HoneycombLattice
│   │       ├── triangular.rs           # TriangularLattice
│   │       ├── kagome.rs               # KagomeLattice
│   │       └── heavy_hex.rs            # HeavyHexLattice
│   ├── cell_complex/                   # NEW
│   │   ├── mod.rs
│   │   ├── cell_complex.rs             # CellComplex<C>
│   │   ├── cell_field.rs               # CellField<C, T>
│   │   ├── cell_trait.rs               # Cell trait
│   │   ├── boundary_operator.rs        # BoundaryOperator<C>
│   │   └── homology.rs                 # Betti numbers, homology basis
│   └── [existing modules...]
├── extensions/
│   ├── hkt_lattice.rs                  # NEW: LatticeWitness<D>
│   ├── hkt_cell_complex.rs             # NEW: CellComplexWitness<C>
│   └── [existing HKT modules...]
└── traits/
    ├── cw_complex.rs                   # NEW: CWComplex trait
    └── [existing traits...]
```

---

## 8. Trait Hierarchy

```rust
/// Marker trait for cell types.
pub trait Cell: Clone + Eq + Hash {
    fn dim(&self) -> usize;
    fn boundary(&self) -> Chain<Self>;
}

/// Types that form a CW complex.
pub trait CWComplex {
    type CellType: Cell;
    
    fn cells(&self, k: usize) -> impl Iterator<Item = Self::CellType>;
    fn boundary_matrix(&self, k: usize) -> SparseMatrix<i8>;
    fn betti_number(&self, k: usize) -> usize;
}

// Implementations
impl<const D: usize> CWComplex for Lattice<D> { 
    type CellType = LatticeCell<D>;
    // ...
}

impl<C: Cell> CWComplex for CellComplex<C> { 
    type CellType = C;
    // ...
}

impl CWComplex for SimplicialComplex {
    type CellType = Simplex;
    // ... (implement for existing type)
}
```

---

## 9. GPU Acceleration via CausalTensor

For large lattices, boundary/coboundary operations can be accelerated:

```rust
impl<const D: usize> Lattice<D> {
    /// Boundary operator as a CausalTensor sparse matrix for GPU acceleration.
    pub fn boundary_tensor<B: TensorBackend>(&self, k: usize) -> BackendTensor<f32, B>;
    
    /// Apply boundary operator to a chain vector on GPU.
    pub fn apply_boundary_gpu<B: LinearAlgebraBackend>(
        &self,
        chain_coeffs: &BackendTensor<f32, B>,
        k: usize
    ) -> BackendTensor<f32, B> {
        B::matmul(&self.boundary_tensor::<B>(k), chain_coeffs)
    }
}
```

---

## 10. Testing Strategy

| Category | Tests |
|----------|-------|
| **Cell counting** | num_cells matches expected formula for L^D lattice |
| **Boundary² = 0** | ∂∂ = 0 for all k |
| **Homology** | H_k of D-torus = (D choose k) |
| **Duality** | dual(dual(cell)) = cell |
| **Triangulation** | triangulate() produces valid simplicial complex |
| **HKT laws** | unit/counit satisfy adjunction equations |
| **Encapsulation** | All struct fields are private |

---

## 11. Example Usage

```rust
use deep_causality_topology::{Lattice, Chain, CWComplex};

// Create a 10×10 toric lattice
let lattice = Lattice::square_torus(10);

// Count cells
assert_eq!(lattice.num_cells(0), 100);  // 100 vertices
assert_eq!(lattice.num_cells(1), 200);  // 200 edges (2 per vertex)
assert_eq!(lattice.num_cells(2), 100);  // 100 faces

// Homology of torus: β₀=1, β₁=2, β₂=1
assert_eq!(lattice.betti_number(0), 1);
assert_eq!(lattice.betti_number(1), 2);
assert_eq!(lattice.betti_number(2), 1);

// Get a non-trivial 1-cycle (homology generator)
let cycles = lattice.homology_basis(1);
assert_eq!(cycles.len(), 2);  // Two independent cycles on torus

// Condensed matter: Detect vortex cores
let vortex_lines = lattice.homology_basis(1);  // H_1 = vortex lines
```

---

## 12. Condensed Matter Physics Applications

### 12.1 Band Structure

```rust
// Brillouin zone is the dual lattice
let real_space = Lattice::cubic_torus(32);
let momentum_space = DualLattice::new(real_space);

// k-points are vertices of the dual lattice
for k_point in momentum_space.primal().cells(0) {
    // Compute Hamiltonian eigenvalues at k
}
```

### 12.2 Topological Defects

```rust
// Vortex lines in a superfluid/superconductor
let lattice = Lattice::cubic_torus(64);
let vortex_lines = lattice.homology_basis(1);  // H_1 generators

// Monopoles (if any)
let monopoles = lattice.homology_basis(0);  // H_0 - 1 = number of cuts
```

### 12.3 Tight-Binding Models

```rust
// Graphene on honeycomb lattice
let graphene = HoneycombLattice::new([128, 128], [true, true]);
let graph = graphene.skeleton_graph();

// Each vertex has 3 neighbors
for vertex in graph.vertices() {
    assert_eq!(graph.degree(vertex), 3);
}
```
