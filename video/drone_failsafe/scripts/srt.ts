// Write each cut's captions as an SRT, out/<cut>.srt: every line of narration text, from the frame it
// starts to appear until the next line appears or its scene ends. A line shown for less than
// MIN_FRAMES joins the next line's caption, so a heading stays readable beside its first line. The
// timings are the scenes' own exported constants, so the captions follow any change to a scene.
//
//   tsx scripts/srt.ts
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { CUTS, SCENES, type SceneName } from "../src/Explainer";
import { facts, flights } from "../src/data";
import script from "../src/script.json";
import { CARD_HEADING_AT, CARD_LINE_AT } from "../src/scenes/Chapter";
import { CENTER_URL, END_LINE_AT, END_LOGOS_AT, RUN_COMMAND, TUTORIAL_URL } from "../src/scenes/End";
import { CONFIGS, screenSecondAt } from "../src/scenes/Flight";
import { FAULTS_LINE, SETTING_LINE_AT } from "../src/scenes/Intro";
import { ISSUE_HEADING_AT, ISSUE_LINE_AT } from "../src/scenes/StaticRule";
import { SUMMARY_HEADING_AT, SUMMARY_STEP_AT } from "../src/scenes/Summary";
import { SIDES, THOUSAND_REVEAL_AT } from "../src/scenes/ThousandNights";
import { SUBTITLE_AT, TITLE_AT } from "../src/scenes/Title";

const FPS = 30;
const MIN_FRAMES = 45;
const root = dirname(dirname(fileURLToPath(import.meta.url)));

/** A line of text and the frame, within its scene, at which it starts to appear. */
type Cue = { at: number; text: string };

const flight = (which: "textbook" | "ethos"): Cue[] => {
  const cfg = CONFIGS[which];
  const landed = screenSecondAt(which, flights[cfg.part].touchdown.t);
  return [
    ...cfg.log.map((l) => ({ at: Math.round(screenSecondAt(which, l.t) * FPS), text: `${l.t} s · ${l.text}${l.rule ? `\n${l.rule}` : ""}` })),
    // The outcome, from touchdown, when it has fully appeared under the log.
    { at: Math.round(landed * FPS), text: cfg.outcome },
  ];
};

const card = (heading: string, lines: string[]): Cue[] => [
  { at: CARD_HEADING_AT, text: heading },
  ...lines.map((l, k) => ({ at: CARD_LINE_AT(k), text: l })),
];

const CUES: Record<SceneName, () => Cue[]> = {
  title: () => [
    { at: TITLE_AT, text: script.title.title },
    { at: SUBTITLE_AT, text: script.title.subtitle },
  ],
  setting: () => [
    ...script.setting.lines.map((l, k) => ({ at: SETTING_LINE_AT(k), text: l })),
    { at: SETTING_LINE_AT(script.setting.lines.length), text: FAULTS_LINE },
  ],
  part1: () => card(`${script.part1Card.label}: ${script.part1Card.heading}`, [script.part1Card.line]),
  textbook: () => flight("textbook"),
  issue: () => [
    { at: ISSUE_HEADING_AT, text: script.issue.heading },
    ...script.issue.lines.map((l, k) => ({ at: ISSUE_LINE_AT(k), text: l })),
  ],
  part2: () => card(`${script.part2Card.label}: ${script.part2Card.heading}`, script.part2Card.lines),
  ethos: () => flight("ethos"),
  simulations: () => [
    { at: 0, text: `${script.simulations.heading}\n${script.simulations.description}` },
    {
      at: THOUSAND_REVEAL_AT,
      text: SIDES.map((side) => {
        const n = facts.campaign.nearPerson[side.column];
        const label = n === 1 ? script.simulations.countLabelOne : script.simulations.countLabelMany;
        const who = side.column === 0 ? script.simulations.textbookLabel : script.simulations.ethosLabel;
        return `${who}: ${n} ${label} (${side.rate(facts.campaign.nearPersonPct)})`;
      }).join("\n"),
    },
  ],
  summary: () => [
    { at: SUMMARY_HEADING_AT, text: script.summary.heading },
    ...script.summary.steps.map((s, k) => ({ at: SUMMARY_STEP_AT(k), text: `${s.name}: ${s.text}` })),
  ],
  end: () => [
    { at: END_LINE_AT, text: script.end.line },
    { at: END_LOGOS_AT, text: `${TUTORIAL_URL}\n${RUN_COMMAND}\n${CENTER_URL}` },
  ],
};

const stamp = (frame: number) => {
  const ms = Math.round((frame / FPS) * 1000);
  const pad = (n: number, w = 2) => String(n).padStart(w, "0");
  return `${pad(Math.floor(ms / 3_600_000))}:${pad(Math.floor(ms / 60_000) % 60)}:${pad(Math.floor(ms / 1000) % 60)},${pad(ms % 1000, 3)}`;
};

mkdirSync(join(root, "out"), { recursive: true });
for (const cut of CUTS) {
  const lines: { from: number; to: number; text: string }[] = [];
  let offset = 0;
  for (const name of cut.scenes) {
    const frames = SCENES[name].frames;
    // Lines that start on the same frame share one caption.
    const merged = new Map<number, string[]>();
    for (const c of CUES[name]()) merged.set(c.at, [...(merged.get(c.at) ?? []), c.text]);
    const cues = [...merged.entries()].sort((a, b) => a[0] - b[0]).map(([at, texts]) => ({ at, text: texts.join("\n") }));
    for (let k = 0; k < cues.length; k++) {
      const next = k + 1 < cues.length ? cues[k + 1].at : frames;
      if (next - cues[k].at < MIN_FRAMES && k + 1 < cues.length) {
        cues[k + 1] = { at: cues[k].at, text: `${cues[k].text}\n${cues[k + 1].text}` };
        continue;
      }
      lines.push({ from: offset + cues[k].at, to: offset + next, text: cues[k].text });
    }
    offset += frames;
  }
  const srt = lines.map((l, i) => `${i + 1}\n${stamp(l.from)} --> ${stamp(l.to)}\n${l.text}\n`).join("\n");
  const out = join(root, "out", `${cut.id}.srt`);
  writeFileSync(out, srt);
  console.log(`wrote ${lines.length} cues to ${out}`);
}
