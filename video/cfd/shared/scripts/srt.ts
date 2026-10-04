// Write a cut's narration captions as an SRT, from the same timing table the main composition
// renders with, so every cue starts and ends on the frames where the burned-in caption does.
//
//   tsx <shared>/scripts/srt.ts <output.srt>
//
// Run from a project's directory: it reads that project's `src/timeline.ts`.
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { render } from '../src/tokens';

const target = process.argv[2];
if (!target) throw new Error('usage: srt.ts <output.srt>');
const { sceneTimings } = await import(pathToFileURL(resolve('src/timeline.ts')).href);

const stamp = (frame: number) => {
  const ms = Math.round((frame / render.fps) * 1000);
  const pad = (n: number, w = 2) => String(n).padStart(w, '0');
  return `${pad(Math.floor(ms / 3_600_000))}:${pad(Math.floor(ms / 60_000) % 60)}:${pad(Math.floor(ms / 1000) % 60)},${pad(ms % 1000, 3)}`;
};

// Caption exponents, `10^19`, as Unicode superscripts: 10¹⁹.
const SUPERSCRIPT: Record<string, string> = { '-': '⁻', 0: '⁰', 1: '¹', 2: '²', 3: '³', 4: '⁴', 5: '⁵', 6: '⁶', 7: '⁷', 8: '⁸', 9: '⁹' };
const plain = (text: string) => text.replace(/\^(-?\d+)/g, (_, e: string) => [...e].map((ch) => SUPERSCRIPT[ch]).join(''));

type Scene = { from: number; phrases: { start: number; length: number; text: string }[] };
const cues = (sceneTimings(render.fps) as Scene[]).flatMap((scene) =>
  scene.phrases.map((p) => ({ from: scene.from + p.start, to: scene.from + p.start + p.length, text: plain(p.text) }))
);
const srt = cues.map((c, i) => `${i + 1}\n${stamp(c.from)} --> ${stamp(c.to)}\n${c.text}\n`).join('\n');

const out = resolve(target);
mkdirSync(dirname(out), { recursive: true });
writeFileSync(out, srt);
console.log(`wrote ${cues.length} cues to ${out}`);
