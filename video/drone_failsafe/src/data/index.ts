// Typed access to the JSON scripts/data.mjs and scripts/timeline.mjs write.
import worldJson from "./world.json";
import flightsJson from "./flights.json";
import patchesJson from "./patches.json";
import ethosJson from "./ethos.json";
import campaignJson from "./campaign.json";
import factsJson from "./facts.json";
import timelineJson from "./timeline.json";

export type Row = {
  t: number;
  x: number;
  y: number;
  agl: number;
  faults: boolean[];
  decision: string;
  below: string;
  maneuver: string;
  target: [number, number] | null;
  plan: string;
};
export type Touchdown = { t: number; x: number; y: number; outcome: string; surface: string; nearestPersonM: number };
export type Part = 1 | 2 | 3 | 4;
export type Cue = { file: string; text: string; from: number; durationInFrames: number };
export type SceneSpec = { name: string; from: number; durationInFrames: number; cues: Cue[] };
export type Cut = { durationInFrames: number; scenes: SceneSpec[] };

export const world = worldJson as {
  x0: number;
  y0: number;
  step: number;
  nx: number;
  ny: number;
  elevation: number[];
  surface: number[];
  canopy: number[];
  crew: [number, number][];
};
export const flights = flightsJson as unknown as Record<string, { rows: Row[]; touchdown: Touchdown }>;
const patches = patchesJson as unknown as Record<string, [number, number, number, number, number][]>;
export const ethos = ethosJson as {
  rulings: { t: number; urgency: string; patch: [number, number]; proposal: string; review: string; cost: number | null; norms: number[] }[];
  lastResort: { t: number; kind: string; patch: [number, number]; cost: number }[];
};
export const campaign = campaignJson as number[][];
export const facts = factsJson;
export const timeline = timelineJson as { fps: number; main: Cut; clips: Cut[] };

export const GROUNDS = ["Safe", "Steep", "Water", "Person", "Unsure", "Trees"] as const;
export const ENDINGS = ["Resumed", "Safe", "NearPerson", "Ditched", "IntoRavine", "TippedAndRolled", "HitTrees", "Fell"] as const;

/** The drone at flight time `t`, between two recorded seconds; it rests at the touchdown. */
export function droneAt(part: Part, t: number): { x: number; y: number; agl: number; row: Row; landed: boolean } {
  const { rows, touchdown } = flights[part];
  if (t >= touchdown.t) return { x: touchdown.x, y: touchdown.y, agl: 0, row: rows[rows.length - 1], landed: true };
  const k = Math.max(0, Math.min(rows.length - 1, Math.floor(t)));
  const a = rows[k];
  const b = k + 1 < rows.length ? rows[k + 1] : { ...a, x: touchdown.x, y: touchdown.y, agl: 0 };
  const s = Math.max(0, Math.min(1, t - a.t));
  return { x: a.x + (b.x - a.x) * s, y: a.y + (b.y - a.y) * s, agl: a.agl + (b.agl - a.agl) * s, row: a, landed: false };
}

/** Each patch's latest judgement and fused slope uncertainty up to flight time `t`. */
export function patchesAt(part: Part, t: number): Map<string, { i: number; j: number; g: number; sigma: number }> {
  const out = new Map<string, { i: number; j: number; g: number; sigma: number }>();
  const rows = patches[part];
  if (!rows) return out;
  for (const [pt, i, j, g, sigma] of rows) {
    if (pt > t) break;
    out.set(`${i},${j}`, { i, j, g, sigma });
  }
  return out;
}

/** The drone's ground track up to flight time `t`. */
export function trackTo(part: Part, t: number): [number, number][] {
  const { rows } = flights[part];
  const pts: [number, number][] = rows.filter((r) => r.t <= t).map((r) => [r.x, r.y]);
  const now = droneAt(part, t);
  pts.push([now.x, now.y]);
  return pts;
}

/** The latest Effect Ethos round at or before flight time `t`. */
export function roundAt(t: number) {
  const times = [...new Set(ethos.rulings.map((r) => r.t))].filter((rt) => rt <= t);
  if (times.length === 0) return null;
  const rt = Math.max(...times);
  return { t: rt, rulings: ethos.rulings.filter((r) => r.t === rt) };
}
