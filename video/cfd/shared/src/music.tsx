/**
 * The theme under a cut. `pnpm music` renders `music/theme.scd` into the project's `public/music/`
 * at the cut's exact length, so the track plays once from the first frame and its ending lands on
 * the last.
 */
import { Html5Audio, staticFile } from 'remotion';

export const MUSIC_FILE = 'music/theme.wav';

export const Music: React.FC = () => <Html5Audio src={staticFile(MUSIC_FILE)} />;

const fail = (msg: string): never => {
  throw new Error(`music: ${msg}`);
};

/**
 * Check that the theme exists and lasts `seconds`, to within half a frame at `fps`, by reading the
 * WAV header: the `fmt ` chunk's byte rate and the `data` chunk's size. A missing or stale file throws,
 * which fails `calculateMetadata` and so the render.
 */
export async function checkMusic(seconds: number, fps: number): Promise<void> {
  const res = await fetch(staticFile(MUSIC_FILE));
  if (!res.ok || !res.body) return fail(`no public/${MUSIC_FILE}; run pnpm music`);
  const reader = res.body.getReader();
  const { value } = await reader.read();
  await reader.cancel();
  if (!value) return fail(`public/${MUSIC_FILE} is empty; run pnpm music`);
  const view = new DataView(value.buffer, value.byteOffset, value.byteLength);
  const tag = (at: number) => String.fromCharCode(...value.subarray(at, at + 4));
  if (tag(0) !== 'RIFF' || tag(8) !== 'WAVE') return fail(`public/${MUSIC_FILE} is not a WAV file`);
  let byteRate = 0;
  for (let at = 12; at + 8 <= view.byteLength; ) {
    const size = view.getUint32(at + 4, true);
    if (tag(at) === 'fmt ') byteRate = view.getUint32(at + 16, true);
    if (tag(at) === 'data') {
      if (byteRate === 0) break;
      const length = size / byteRate;
      if (Math.abs(length - seconds) > 0.5 / fps) {
        fail(`public/${MUSIC_FILE} lasts ${length.toFixed(3)} s; the cut lasts ${seconds.toFixed(3)} s. Run pnpm music`);
      }
      return;
    }
    at += 8 + size + (size % 2);
  }
  fail(`public/${MUSIC_FILE} has no readable fmt and data chunks`);
}
