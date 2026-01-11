Based on Section 4 of the provided lecture notes by David Tong, here is the mathematical formulation of Lattice Gauge Theory, covering the discretization of scalar fields, gauge fields (the Wilson action), the measure, and fermions.

### 1. Discretization of Spacetime
Lattice gauge theory discretizes Euclidean spacetime into a cubic, four-dimensional lattice $\Gamma$ with lattice spacing $a$.
$$
\Gamma = \left\{ x : x = \sum_{\mu=1}^4 a n_\mu \hat{\mu} \, , \,\, n_\mu \in \mathbf{Z} \right\}
$$
The lattice spacing acts as an ultra-violet cut-off: $\Lambda_{UV} = 1/a$.

### 2. Scalar Fields on the Lattice
A continuum real scalar field action:
$$
S = \int d^4x \, \frac{1}{2} (\partial_\mu \phi)^2 + V(\phi)
$$
Is discretized by replacing the integral with a sum and derivatives with finite differences:
$$
\int d^4x \rightarrow a^4 \sum_{x \in \Gamma} \quad , \quad \partial_\mu \phi(x) \rightarrow \frac{\phi(x + a\hat{\mu}) - \phi(x)}{a}
$$
The resulting lattice action is:
$$
S = a^4 \sum_{x \in \Gamma} \left[ \frac{1}{2} \sum_{\mu} \left( \frac{\phi(x + a\hat{\mu}) - \phi(x)}{a} \right)^2 + V(\phi(x)) \right]
$$
The partition function becomes a product of ordinary integrals:
$$
Z = \int \prod_{x \in \Gamma} d\phi(x) \, e^{-S}
$$

### 3. Gauge Fields (The Wilson Action)
Instead of discretizing the algebra-valued field $A_\mu$, lattice gauge theory uses group-valued variables $U_\mu(x)$ living on the **links** between sites.
$$
\text{link } x \rightarrow x + \hat{\mu}: \quad U_\mu(x) \in G \quad (\text{e.g., } SU(N))
$$
Relation to the continuum gauge field:
$$
U_\mu(x) = e^{ia A_\mu(x)}
$$
**Gauge Transformation:**
$$
U_\mu(x) \rightarrow \Omega(x) U_\mu(x) \Omega^\dagger(x + \hat{\mu})
$$
**The Wilson Loop (Plaquette):**
The simplest gauge invariant object is the trace of the product of link variables around a single square (plaquette) in the $\mu-\nu$ plane:
$$
W_{\square} = \text{tr } U_\mu(x) U_\nu(x + \hat{\mu}) U_\mu^\dagger(x + \hat{\nu}) U_\nu^\dagger(x)
$$
**The Wilson Action:**
$$
S_{\text{Wilson}} = -\frac{\beta}{2N} \sum_{\square} \left( W_{\square} + W_{\square}^\dagger \right)
$$
where $\beta$ is related to the continuum coupling $g$ by:
$$
\frac{\beta}{2N} = \frac{1}{g^2}
$$
In the continuum limit ($a \to 0$), this reproduces the Yang-Mills action:
$$
S_{\text{Wilson}} = \frac{1}{2g^2} \int d^4x \, \text{tr } F_{\mu\nu} F^{\mu\nu} + O(a^2)
$$

### 4. The Haar Measure
To preserve gauge invariance in the path integral, one must use the Haar measure for integration over the group elements on the links.
$$
Z = \int \prod_{(x, \mu)} dU_\mu(x) \, e^{-S_{\text{Wilson}}}
$$
Properties of the Haar measure:
*   **Invariance:** $\int dU \, f(U) = \int dU \, f(\Omega U) = \int dU \, f(U \Omega)$
*   **Normalization:** $\int dU \, 1 = 1$
*   ** orthogonality:** $\int dU \, U_{ij} U_{kl}^\dagger = \frac{1}{N} \delta_{il} \delta_{jk}$

**Elitzur’s Theorem:**
For any non-gauge-invariant operator $\mathcal{O}$:
$$
\langle \mathcal{O} \rangle = 0
$$

### 5. Strong Coupling Expansion
In the limit $\beta \ll 1$ (strong coupling), the partition function can be expanded in powers of $\beta$. The expectation value of a large rectangular Wilson loop $W[C]$ with area $A$ exhibits an area law (confinement):
$$
\langle W[C] \rangle \sim \left( \frac{\beta}{2N^2} \right)^{A/a^2} = e^{-\sigma A}
$$
where the string tension $\sigma$ is:
$$
\sigma = -\frac{1}{a^2} \log \left( \frac{\beta}{2N^2} \right)
$$

### 6. Fermions on the Lattice
**Naive Discretization:**
$$
S = a^4 \sum_{x \in \Gamma} \bar{\psi}(x) \sum_\mu \gamma^\mu \frac{\psi(x + a\hat{\mu}) - \psi(x - a\hat{\mu})}{2a}
$$
This leads to **fermion doubling**. The inverse propagator in momentum space is:
$$
D(k) = \frac{1}{a} \sum_\mu i \gamma^\mu \sin(k_\mu a)
$$
This has poles not just at $k_\mu = 0$, but at all corners of the Brillouin zone $k_\mu = \pi/a$ (16 species in 4D).

**Nielsen-Ninomiya Theorem:**
It is impossible to define a lattice Dirac operator $D(k)$ that simultaneously satisfies:
1.  Locality (continuity in $k$).
2.  Correct continuum limit ($D(k) \approx i\gamma^\mu k_\mu$).
3.  No doublers.
4.  Chiral symmetry ($\{D, \gamma^5\} = 0$).

**Wilson Fermions:**
Add a term to kill doublers (breaking chiral symmetry):
$$
S = a^4 \sum_{x} \bar{\psi}(x) \left[ \sum_\mu \gamma^\mu \frac{\psi(x+a\hat{\mu}) - \psi(x-a\hat{\mu})}{2a} - \frac{r}{2a} \sum_\mu (\psi(x+a\hat{\mu}) - 2\psi(x) + \psi(x-a\hat{\mu})) \right]
$$
Inverse propagator:
$$
D(k) = \frac{i}{a} \sum_\mu \gamma^\mu \sin(k_\mu a) + \frac{r}{a} \sum_\mu (1 - \cos(k_\mu a))
$$

**Ginsparg-Wilson Relation (Chiral Fermions):**
A lattice operator $D$ preserves a modified chiral symmetry if it satisfies:
$$
\{ \gamma^5, D \} = a D \gamma^5 D
$$
A solution to this is the **Overlap Operator**:
$$
D = \frac{1}{a} \left( 1 - \frac{1 - a D_0}{\sqrt{(1 - a D_0)^\dagger (1 - a D_0)}} \right)
$$
where $D_0$ is the Wilson-Dirac operator (typically with a large negative mass).