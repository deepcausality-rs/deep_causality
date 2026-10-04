/**
 * The numbers the captions show, rebuilt from the run and looked up in the caption text. A caption
 * that disagrees with the run throws, which fails `calculateMetadata` and so the render, as the
 * on-screen data checks do.
 */
import type { SegmentId } from '../script';
import { segment } from '../timeline';
import type { Weather } from './weather';

const fail = (msg: string): never => {
  throw new Error(`captions: ${msg}`);
};
/** Round up to `digits` decimals: the bound a "within" claim may state. */
const ceilTo = (v: number, digits: number) => (Math.ceil(v * 10 ** digits) / 10 ** digits).toFixed(digits);

export function checkCaptions(w: Weather): void {
  const by = (name: string) => w.worlds.find((x) => x.name === name) ?? fail(`no ${name} world`);
  const polar = by('polar_winter');
  const early = w.worlds.reduce((a, x) => (x.onset < a.onset ? x : a));
  const late = w.worlds.reduce((a, x) => (x.onset > a.onset ? x : a));
  const dwells = w.worlds.map((x) => x.dwell);
  const [dMin, dMax] = [Math.min(...dwells), Math.max(...dwells)];
  const sds = w.worlds.map((x) => x.driftSd);
  const descents = w.worlds.length * w.drawsPerWorld;
  const worstTerminal = Math.max(...w.worlds.map((x) => x.terminalMax));
  const alternated = w.worlds.length - 1;

  const expected: [SegmentId, string][] = [
    ['entry', `through ${w.worlds.length} counterfactual atmospheres`],
    ['worlds', `alternates ${alternated} atmospheres`],
    ['worlds', `${w.drawsPerWorld} receiver-noise draws: ${descents} full descents`],
    ['window', `the ${early.label} loses GPS at ${early.onset.toFixed(1)} s, the ${late.label} at ${late.onset.toFixed(1)} s`],
    ['window', `lasts between ${dMin.toFixed(1)} and ${dMax.toFixed(1)} s`],
    ['window', `shifts the window by ${w.spreads.onset.toFixed(1)} s`],
    ['drift', `it reaches ${w.worlds[0].driftMean.toFixed(2)} m on average; in polar winter, ${polar.driftMean.toFixed(2)} m, ${w.coldFactor.toFixed(2)} times as far`],
    ['why', `${Math.round((dMax / dMin - 1) * 100)}% longer than the shortest, which moves the drift by about ${Math.round(((dMax / dMin) ** 2 - 1) * 100)}%`],
    ['why', `by ${Math.round(w.imuPerK * 100)}% per kelvin`],
    ['why', `Polar winter sits ${Math.abs(polar.dTemp)} K from the calibration point and flies ${polar.departure.toFixed(2)} times the bias`],
    ['why', `within ${ceilTo(w.predictionError * 100, 1)}%`],
    ['resolved', `by ${Math.min(...sds).toFixed(2)} to ${Math.max(...sds).toFixed(2)} m, one standard deviation`],
    ['resolved', `${w.resolved.separation.toFixed(2)} m apart: ${w.resolved.ratio.toFixed(1)} times their combined scatter`],
    ['resolved', `within ${ceilTo(worstTerminal, 2)} m`],
    ['close', `shifts it by ${w.spreads.onset.toFixed(1)} s`],
    ['close', `drifts ${Math.round((w.coldFactor - 1) * 100)}% further`],
    ['close', `${descents} counterfactual descents`],
    ['close', `about ${Math.round(w.runSeconds / 60)} minutes`],
  ];
  for (const [id, text] of expected) {
    if (!segment(id).phrases.some((p) => p.text.includes(text))) fail(`scene '${id}' does not say "${text}"`);
  }
}
