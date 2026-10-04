/**
 * The numbers the captions show, rebuilt from the run and looked up in the caption text. A caption
 * that disagrees with the run throws, which fails `calculateMetadata` and so the render, as the
 * on-screen data checks do.
 */
import type { SegmentId } from '../script';
import { segment } from '../timeline';
import type { Retro } from './retro';

const by = (r: Retro, name: string) => r.branches.find((b) => b.name === name) ?? fail(`no ${name} branch`);
const fail = (msg: string): never => {
  throw new Error(`captions: ${msg}`);
};

export function checkCaptions(r: Retro): void {
  const { belief: b, events: e, landings: l } = r;
  const coast = by(r, 'coast');
  const burning = r.branches.filter((x) => x.flown > 0);
  const dip = burning.reduce((a, x) => (x.deceleration < a.deceleration ? x : a));
  const high = by(r, 'high');
  const hard = by(r, 'hard');
  const throttles = burning.map((x) => x.commanded.toFixed(2));
  const list = `${throttles.slice(0, -1).join(', ')} and ${throttles[throttles.length - 1]}`;
  const separation = Math.round(l.informed.printed.lightM - l.uninformed.printed.lightM);

  const expected: [SegmentId, string][] = [
    ['coast', `Mach ${e.entry.mach.toFixed(0)} on a day ${Math.abs(b.measuredDT)} K colder than standard, and loses GPS for ${Math.round(e.blackout.dwell)} s`],
    ['coast', `for another ${Math.round(e.coastS)} s`],
    ['coast', `At ${e.ignition.altitudeKm.toFixed(1)} km and Mach ${e.ignition.mach.toFixed(2)}`],
    ['pause', `forks it ${r.branches.length} ways`],
    ['pause', `throttles of ${list}`],
    ['fork', `${r.branchSeconds.toFixed(0)} s from the same state`],
    ['fork', `Coasting slows the capsule at ${coast.deceleration.toFixed(2)} m/s^2`],
    ['fork', `Burning at ${dip.flown.toFixed(2)} slows it at only ${dip.deceleration.toFixed(2)}`],
    ['fork', `${high.deceleration.toFixed(2)} at ${high.commanded.toFixed(2)}, and ${hard.deceleration.toFixed(2)} at ${hard.commanded.toFixed(2)}, which the envelope trims to ${hard.flown.toFixed(2)}`],
    ['plan', `${b.informed.mean.toFixed(2)} m of drift, plus or minus ${b.informed.sd.toFixed(2)}`],
    ['plan', `mean plus ${b.k} standard deviations: ${b.informed.margin.toFixed(2)} m`],
    ['plan', `would use ${b.uninformed.margin.toFixed(2)} m`],
    ['landing', `Mach ${e.subsonic.mach.toFixed(1)} at ${e.subsonic.altitudeKm.toFixed(1)} km`],
    ['landing', `stopping burn at ${l.informed.printed.lightM.toFixed(2)} m`],
    ['landing', `touches down at ${l.informed.contactMs.toFixed(2)} m/s, with ${r.propellantLeftKg.toFixed(1)} kg`],
    ['beliefs', `lights at ${l.informed.printed.lightM.toFixed(2)} m; sized for a standard day, at ${l.uninformed.printed.lightM.toFixed(2)} m`],
    ['beliefs', `at ${l.informed.contactMs.toFixed(2)} and ${l.uninformed.contactMs.toFixed(2)} m/s`],
    ['beliefs', `extra ${separation} m of margin costs ${l.informed.printed.reserveKg.toFixed(2)} kg`],
    ['close', `The ${r.branches.length} forked burns`],
    ['close', `costs ${l.informed.printed.reserveKg.toFixed(2)} kg for ${separation} m`],
    ['close', `${r.totalSteps} coupled steps, runs in under ${Math.ceil(r.runSeconds / 60)} minutes`],
  ];
  for (const [id, text] of expected) {
    if (!segment(id).phrases.some((p) => p.text.includes(text))) fail(`scene '${id}' does not say "${text}"`);
  }
}
