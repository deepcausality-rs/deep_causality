// Lays out the main cut and the clips from the narration's durations: each scene opens with a lead,
// places its sentences one after another, and holds at its end. Writes src/data/timeline.json, which
// the compositions read, and out/main.srt, the main cut's captions.
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const voice = JSON.parse(readFileSync(join(root, "src", "data", "voice.json"), "utf8"));
const FPS = 30;
const f = (s) => Math.round(s * FPS);

// A scene: the narration segment it speaks, the lead before its first sentence, the pause between
// sentences, longer pauses before given sentences, and the hold after its last.
const SCENES = {
  hook: { segment: "hook", lead: 0.6, gap: 0.45, breaks: { 3: 1.0, 5: 1.0 }, tail: 1.2 },
  world: { segment: "world", lead: 1.0, gap: 0.4, breaks: { 3: 0.6, 6: 0.4 }, tail: 1.0 },
  run1: { segment: "run1", lead: 1.2, gap: 0.6, tail: 3.4 },
  run2: { segment: "run2", lead: 1.2, gap: 0.5, breaks: { 2: 0.8 }, tail: 3.4 },
  run3: { segment: "run3", lead: 1.2, gap: 0.6, tail: 3.6 },
  twist: { segment: "twist", lead: 0.8, gap: 0.6, tail: 2.0 },
  run4: { segment: "run4", lead: 0.8, gap: 0.45, breaks: { 1: 0.6, 9: 1.2 }, tail: 2.2 },
  campaign: { segment: "campaign", lead: 2.2, gap: 0.5, breaks: { 5: 0.8 }, tail: 2.0 },
  close: { segment: "close", lead: 0.8, gap: 0.6, tail: 4.0 },
  teaser: { segment: null, seconds: 17 },
};

function scene(name, from, segmentOverride) {
  const spec = SCENES[name];
  const segment = segmentOverride ?? spec.segment;
  if (!segment) return { name, from, durationInFrames: f(spec.seconds), cues: [] };
  let t = spec.lead;
  const cues = voice[segment].map((v, k) => {
    if (k > 0) t += spec.gap + (spec.breaks?.[k] ?? 0);
    const cue = { file: v.file, text: v.text, from: f(t), durationInFrames: f(v.seconds) };
    t += v.seconds;
    return cue;
  });
  return { name, from, durationInFrames: f(t + spec.tail), cues };
}

// A card that speaks one line: a clip's hook or its closing line.
function card(name, segment, from, hold) {
  const [v] = voice[segment];
  const lead = 0.3;
  return {
    name,
    from,
    durationInFrames: f(lead + v.seconds + hold),
    cues: [{ file: v.file, text: v.text, from: f(lead), durationInFrames: f(v.seconds) }],
  };
}

function sequence(parts) {
  let from = 0;
  const scenes = parts.map((make) => {
    const s = make(from);
    from += s.durationInFrames;
    return s;
  });
  return { durationInFrames: from, scenes };
}

const main = sequence(
  ["hook", "world", "run1", "run2", "run3", "twist", "run4", "campaign", "close"].map((n) => (from) => scene(n, from)),
);
const CLIPS = [
  ["teaser"],
  ["run1"],
  ["run2"],
  ["run3"],
  ["twist"],
  ["run4"],
  ["campaign"],
];
const clips = CLIPS.map((names, k) =>
  sequence([
    (from) => card("hookCard", `clip${k}_hook`, from, 0.7),
    ...names.map((n) => (from) => scene(n, from)),
    (from) => card("endCard", `clip${k}_close`, from, 2.4),
  ]),
);

writeFileSync(join(root, "src", "data", "timeline.json"), JSON.stringify({ fps: FPS, main, clips }, null, 1));

// The main cut's captions, one cue per sentence.
const stamp = (frame) => {
  const ms = Math.round((frame / FPS) * 1000);
  const p = (n, w = 2) => String(n).padStart(w, "0");
  return `${p(Math.floor(ms / 3600000))}:${p(Math.floor(ms / 60000) % 60)}:${p(Math.floor(ms / 1000) % 60)},${p(ms % 1000, 3)}`;
};
const srt = main.scenes
  .flatMap((s) => s.cues.map((c) => [s.from + c.from, s.from + c.from + c.durationInFrames, c.text]))
  .map(([a, b, text], k) => `${k + 1}\n${stamp(a)} --> ${stamp(b)}\n${text}\n`)
  .join("\n");
mkdirSync(join(root, "out"), { recursive: true });
writeFileSync(join(root, "out", "main.srt"), srt);
const len = (frames) => `${Math.floor(frames / FPS / 60)}:${String(Math.round(frames / FPS) % 60).padStart(2, "0")}`;
console.log(`timeline: main ${len(main.durationInFrames)}; clips ${clips.map((c) => len(c.durationInFrames)).join(", ")}`);
