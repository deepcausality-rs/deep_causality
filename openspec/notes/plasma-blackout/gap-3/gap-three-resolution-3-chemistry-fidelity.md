
---

## Second pass (2026-07-02): the measured failure and the transit-age resolution

The first uncalibrated measurement failed exactly where a measurement should: loudly, with
attribution. The network predicted `n_e = 5.46e14` against the `1e19` anchor (4.26 decades low);
the channel-by-channel printout pinned the miss on the atom pool (`x_N = 6.2e-5`), electron
impact moved nothing (0.02 percent), and a hand-check showed the rate pairs reproduce the anchor
at realistic dissociation levels. The rate data was right; the parcel-age model was wrong.

The second ARIZ pass, informed by the coupling-web research note
([`../ionization-chemistry.md`](../finite-rate-ionization-chemistry.md)), found the hidden fixed constraint:
the single-parcel age `t_res = standoff/u2` holds the post-shock velocity constant, but the
stagnation line is precisely the place where that is maximally false. Intensifying the
contradiction cracked it: age to zero gives nothing, age to infinity gives the dissociation
equilibrium, and the measured hand-check put that equilibrium *on the anchor*. The physical
contradiction (age finite and effectively infinite) separates in **space**: behind the shock the
velocity decelerates linearly to zero at the body, so the transit age is
`age(xi) = t_res * ln(1/(1-xi))`, a closed form from geometry and the Rankine-Hugoniot state
with no free parameter. The flight reflectometers measured the layer's peak, which lives at the
old-gas end of that profile.

The revision, all three parts cited and none tunable:

1. **Transit-age profile** on the stagnation line; the anchor gate reads the profile's peak.
2. **Zeldovich exchange** (`N2 + O -> NO + N`, RP-1232 Table II reaction 6, 37,500 K activation
   against direct dissociation's 113,100 K) folded into the N-pool clock,
   `tau_N = 1/(k_d[M] + k_z[O])`: the real low-temperature N-production path the coupling web
   names as the start of the electron chain.
3. **Dissociation at Park's classic `q = 0.7`** controlling temperature (`T_tr^0.7 * T_ve^0.3`),
   replacing the silent geometric mean for dissociation only. The research note identifies the
   `q` choice as the largest closure divergence among production codes; adopting the Park
   lineage's own published exponent for the Park rate set is a citation, not a fit. The
   calibrated lever-1 ionization controller keeps its geometric mean; electron channels keep
   `T_e = T_ve`.

Deliberately not adopted: a separate electron temperature (note lever 3) stays out because the
measured electron-impact contribution is 0.02 to 15 percent across both passes at RAM-C speeds,
so the avalanche loop that lever governs is dormant here (revisit for faster entries); the N+N
and O+O associative channels are secondary at these enthalpies; NO is treated as transient.

**Measured result of the revision:** channel 1 + pool `2.60e19` (+0.41 dec), full network
`2.99e19` (+0.48 dec, 3.0x) against the `1e19` anchor, inside the pinned ±0.7-decade band and
squarely in the production-code 2x-3x context, with the pool at the peak substantially
dissociated (`x_N = 0.46`, `x_O = 0.64`). From 4.26 decades low to 0.48 decades high, and every
step of the closing is a citation or a geometric identity.

**Harvest, extended:** when a residence-limited closure starves a downstream quantity, check
whether the operating zone's kinematics makes age a *field* rather than a number; the profile's
extremum is knob-free where a scalar age would be a fit. And the inverse of the first pass's
lesson also held: the lag amendment was directionally right, but auditing a sub-closure is not
enough — the *clock* it runs on (here, which production channels feed it and which controlling
temperature rates it) carries the same burden of proof as the closure itself.
