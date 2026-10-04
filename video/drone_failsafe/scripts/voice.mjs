// Synthesizes every narration sentence into its own WAV file in public/vo/ and records each file's
// duration in src/data/voice.json. A sentence whose text has not changed keeps its file.
//
// The voice comes from one function, `synthesize`. With VOICE_CMD set, it runs that command with
// the sentence on stdin and `{out}` replaced by the WAV path, so any open-source voice tool that
// writes a WAV fits, for example:
//   VOICE_CMD='piper --model en_US-lessac-medium.onnx --output_file {out}' pnpm voice
// Without VOICE_CMD it uses the macOS `say` voice as a placeholder.
import { execFileSync, execSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const narration = JSON.parse(readFileSync(join(root, "narration.json"), "utf8"));
const voDir = join(root, "public", "vo");
const outFile = join(root, "src", "data", "voice.json");
mkdirSync(voDir, { recursive: true });
const previous = existsSync(outFile) ? JSON.parse(readFileSync(outFile, "utf8")) : {};
const voiceName = process.env.VOICE_CMD ? `cmd:${process.env.VOICE_CMD}` : "say:Samantha:172";

function synthesize(text, wav) {
  if (process.env.VOICE_CMD) {
    execSync(process.env.VOICE_CMD.replace("{out}", JSON.stringify(wav)), { input: text });
  } else {
    const aiff = wav.replace(/\.wav$/, ".aiff");
    execFileSync("say", ["-v", "Samantha", "-r", "172", "-o", aiff, text]);
    execFileSync("ffmpeg", ["-y", "-loglevel", "error", "-i", aiff, "-ar", "48000", "-ac", "1", wav]);
    execFileSync("rm", [aiff]);
  }
}

const seconds = (wav) =>
  Number(execFileSync("ffprobe", ["-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", wav]).toString().trim());

const voice = {};
let made = 0;
for (const [segment, sentences] of Object.entries(narration)) {
  voice[segment] = sentences.map((text, k) => {
    const file = `${segment}_${k}.wav`;
    const wav = join(voDir, file);
    const hash = createHash("sha256").update(`${voiceName}\n${text}`).digest("hex").slice(0, 16);
    const old = previous[segment]?.[k];
    if (!(old && old.hash === hash && existsSync(wav))) {
      synthesize(text, wav);
      made += 1;
    }
    return { file: `vo/${file}`, text, seconds: seconds(wav), hash };
  });
}
writeFileSync(outFile, JSON.stringify(voice, null, 1));
const total = Object.values(voice).flat().reduce((s, v) => s + v.seconds, 0);
console.log(`voice: ${made} sentences synthesized, ${Object.values(voice).flat().length} in all, ${total.toFixed(1)} s of narration (${voiceName})`);
