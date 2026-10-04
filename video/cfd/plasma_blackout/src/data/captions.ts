/**
 * The numbers the captions show, rebuilt from the run and looked up in the caption text. A caption
 * that disagrees with the run throws, which fails `calculateMetadata` and so the render, as the
 * on-screen data checks do.
 */
import type { SegmentId } from '../script';
import { segment } from '../timeline';
import type { Corridor } from './corridor';

/** Scientific notation in caption markup: 3.3 × 10^16. */
const sci = (v: number) => {
  const e = Math.floor(Math.log10(v));
  return `${(v / 10 ** e).toFixed(1)} × 10^${e}`;
};

export function checkCaptions(c: Corridor): void {
  const coarse = c.branches.filter((b) => b.round === 1);
  const fine = c.branches.filter((b) => b.round === 2).sort((a, b) => a.bank - b.bank);
  if (fine.length < 2) throw new Error(`captions: the fine round needs two branches to state their spacing; the run has ${fine.length}`);
  const best = coarse.reduce((a, b) => (b.miss < a.miss ? b : a));
  const clamped = coarse.find((b) => b.flown < b.bank - 0.5);
  if (!clamped) throw new Error('captions: no clamped branch in the coarse round');
  const banks = coarse.map((b) => b.bank);

  const expected: [SegmentId, string][] = [
    ['problem', `Mach ${c.entry.mach.toFixed(0)}`],
    ['problem', `${Math.round(c.pause.t)} seconds later, at ${Math.round(c.pause.altitudeKm)} km`],
    ['question', `${c.aimOffsetM} m off its path`],
    ['pause', `at ${c.pause.t.toFixed(1)} s and ${c.pause.altitudeKm.toFixed(1)} km`],
    ['pause', `${sci(c.pause.electronDensity)} electrons`],
    ['pause', `${c.pause.navErr.toFixed(2)} m off`],
    ['pause', `all ${c.branches.length} alternatives`],
    ['fork', `${c.branchSeconds.toFixed(0)} s ahead`],
    ['fork', `${coarse.length} angles in parallel, from ${Math.min(...banks)}° to ${Math.max(...banks)}°`],
    ['fork', `${clamped.bank}° exceeds the safety envelope, so the gate clamps it to ${clamped.flown.toFixed(1)}°`],
    ['fork', `${best.bank}° lands ${best.miss.toFixed(1)} m from the aim`],
    ['fork', `${fine.length} angles around ${best.bank}°, ${(fine[1].bank - fine[0].bank).toFixed(1)}° apart`],
    ['fork', `${c.committed.bank}° comes closest: ${c.committed.miss.toFixed(2)} m from the aim, against ${c.noBankMiss.toFixed(0)} m`],
    ['fly-through', `commits ${c.committed.bank}°`],
    ['fly-through', `at ${Math.round(c.peakPassage.altitudeKm)} km the sheath holds ${sci(c.peakPassage.electronDensity)} electrons`],
    ['fly-through', `against 10^${Math.round(Math.log10(c.ramcReference))} measured`],
    ['fly-through', `Over ${Math.round(c.exit.t - c.pause.t)} s of blackout`],
    ['fly-through', `from ${c.pause.navErr.toFixed(2)} m to ${c.darkNavPeak.navErr.toFixed(1)} m`],
    ['fly-through', `At ${c.exit.altitudeKm.toFixed(1)} km`],
    ['fly-through', `to ${c.exit.navErr.toFixed(2)} m, and ${Math.round(c.endT - c.exit.t)} s later to ${c.endNavErr.toFixed(2)} m`],
    ['close', `miss the aim by ${c.noBankMiss.toFixed(0)} m`],
    ['close', `cut that to ${c.committed.miss.toFixed(2)} m, inside the ${c.scoring.navSigma.toFixed(1)} m the navigation filter`],
    ['close', `takes ${Math.round(c.runSeconds)} s`],
  ];
  for (const [id, text] of expected) {
    if (!segment(id).phrases.some((p) => p.text.includes(text))) throw new Error(`captions: scene '${id}' does not say "${text}"`);
  }
}
