// Render the theme for a cut at the cut's exact length: SuperCollider renders `music/theme.scd` in
// non-realtime mode, with the strings entering where the closing scene starts, and ffmpeg normalizes
// the result to -16 LUFS integrated, -1.5 dBTP true peak, in two linear passes.
//
//   tsx <shared>/scripts/music.ts <output.wav>
//
// Run from a project's directory: it reads that project's `src/timeline.ts`. `SCLANG` overrides the
// path to SuperCollider's interpreter.
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { render } from '../src/tokens';

const target = process.argv[2];
if (!target) throw new Error('usage: music.ts <output.wav>');
const { sceneTimings, mainFrames } = await import(pathToFileURL(resolve('src/timeline.ts')).href);

const seconds = mainFrames(render.fps) / render.fps;
const scenes = sceneTimings(render.fps) as { id: string; from: number }[];
const liftAt = scenes[scenes.length - 1].from / render.fps;

const theme = resolve(dirname(fileURLToPath(import.meta.url)), '../../music/theme.scd');
const sclang = process.env.SCLANG ?? '/Applications/SuperCollider.app/Contents/MacOS/sclang';
const work = mkdtempSync(join(tmpdir(), 'cfd-theme-'));
const raw = join(work, 'theme.wav');

try {
  execFileSync(sclang, [theme, raw, seconds.toFixed(6), liftAt.toFixed(6)], { stdio: ['ignore', 'ignore', 'inherit'], timeout: 600_000 });

  // Pass 1 measures the loudness; pass 2 applies one gain, so the dynamics stay as rendered.
  const loudnorm = 'loudnorm=I=-16:TP=-1.5:LRA=11';
  const probe = spawnSync('ffmpeg', ['-hide_banner', '-i', raw, '-af', `${loudnorm}:print_format=json`, '-f', 'null', '-'], { encoding: 'utf8' });
  const json = probe.stderr.slice(probe.stderr.lastIndexOf('{'));
  const m = JSON.parse(json.slice(0, json.indexOf('}') + 1));
  const measured = `measured_I=${m.input_i}:measured_TP=${m.input_tp}:measured_LRA=${m.input_lra}:measured_thresh=${m.input_thresh}:offset=${m.target_offset}:linear=true`;

  const out = resolve(target);
  mkdirSync(dirname(out), { recursive: true });
  execFileSync('ffmpeg', ['-hide_banner', '-loglevel', 'error', '-y', '-i', raw, '-af', `${loudnorm}:${measured},aresample=48000,atrim=end_sample=${Math.round(seconds * 48000)},asetpts=N/SR/TB`, '-c:a', 'pcm_s24le', out]);
  console.log(`wrote ${seconds.toFixed(2)} s of music to ${out} (measured ${m.input_i} LUFS before normalizing)`);
} finally {
  rmSync(work, { recursive: true, force: true });
}
