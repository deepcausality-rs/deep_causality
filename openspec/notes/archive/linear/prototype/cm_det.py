import math, itertools, copy

def laplace(m):
    n=len(m)
    if n==1: return m[0][0]
    if n==2: return m[0][0]*m[1][1]-m[0][1]*m[1][0]
    d=0.0
    for c in range(n):
        sub=[[m[r][cc] for cc in range(n) if cc!=c] for r in range(1,n)]
        d += (1 if c%2==0 else -1)*m[0][c]*laplace(sub)
    return d

def gaussian_as_written(m, eps=2.22e-16*100):
    """Exactly lazy_hodge_star.rs:97 - no row pivoting, bails on a small pivot."""
    m=[row[:] for row in m]; n=len(m); det=1.0
    for i in range(n):
        p=m[i][i]
        if abs(p) < eps: return 0.0          # <-- the bail
        det *= p
        for j in range(i+1,n):
            f=m[j][i]/p
            for k in range(i,n): m[j][k]-=f*m[i][k]
    return det

def gaussian_partial_pivot(m):
    m=[row[:] for row in m]; n=len(m); det=1.0
    for i in range(n):
        piv=max(range(i,n), key=lambda r: abs(m[r][i]))
        if abs(m[piv][i])==0.0: return 0.0
        if piv!=i: m[i],m[piv]=m[piv],m[i]; det=-det
        det*=m[i][i]
        for j in range(i+1,n):
            f=m[j][i]/m[i][i]
            for k in range(i,n): m[j][k]-=f*m[i][k]
    return det

def cayley_menger(pts):
    n=len(pts); d=n+1
    m=[[0.0]*d for _ in range(d)]
    for i in range(1,d): m[0][i]=1.0; m[i][0]=1.0
    for i in range(n):
        for j in range(n):
            m[i+1][j+1]=sum((a-b)**2 for a,b in zip(pts[i],pts[j]))
    return m

# regular unit tetrahedron -> k=3 -> 5x5 CM, the exact shape at curvature.rs:254
tet=[(0,0,0),(1,0,0),(0.5,math.sqrt(3)/2,0),(0.5,math.sqrt(3)/6,math.sqrt(2/3))]
cm=cayley_menger(tet)
print("Cayley-Menger 5x5, regular unit tetrahedron")
print("  m[0][0] =", cm[0][0])
L=laplace(cm); G=gaussian_as_written(cm); P=gaussian_partial_pivot(cm)
print(f"  Laplace (det_recursive / determinant_impl) : {L:.12f}")
print(f"  gaussian_determinant AS WRITTEN            : {G:.12f}")
print(f"  elimination WITH partial pivoting          : {P:.12f}")
# vol^2 = (-1)^(k+1)/(2^k (k!)^2) * det, k=3
k=3
for name,d in (("Laplace",L),("as-written",G),("pivoted",P)):
    v2=((-1)**(k+1))/(2**k*math.factorial(k)**2)*d
    print(f"    vol^2 via {name:11s} = {v2: .12f}   vol = {math.sqrt(v2) if v2>0 else float('nan'):.12f}")
print(f"    exact regular-tet volume            =  {math.sqrt(2)/12:.12f}")

# triangle: k=2 -> 4x4
tri=[(0,0),(1,0),(0,1)]
cm3=cayley_menger(tri)
print("\nCayley-Menger 4x4, right triangle (area 0.5)")
print(f"  Laplace                        : {laplace(cm3):.12f}")
print(f"  gaussian_determinant AS WRITTEN: {gaussian_as_written(cm3):.12f}")
print(f"  partial pivoting               : {gaussian_partial_pivot(cm3):.12f}")

# Gram matrix (what lazy_hodge_star actually feeds it) - SPD, positive diagonal
gram=[[3.0,1.0,0.5],[1.0,2.0,0.25],[0.5,0.25,1.5]]
print("\nGram matrix 3x3 (what lazy_hodge_star feeds gaussian_determinant)")
print(f"  Laplace                        : {laplace(gram):.12f}")
print(f"  gaussian_determinant AS WRITTEN: {gaussian_as_written(gram):.12f}")
print(f"  partial pivoting               : {gaussian_partial_pivot(gram):.12f}")
