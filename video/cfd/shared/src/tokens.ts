/**
 * The design system's tokens (`website/web_design/01-foundations.md`), plus the video's stage and
 * timing. The stage is authored at 1920 × 1080 and scaled to the render size, so every 2D length
 * below is in stage pixels.
 */
export const color = {
  bg0: '#070b10',
  bg1: '#0b1118',
  bg2: '#121a23',
  line1: '#1f2a36',
  line2: '#2a3a4a',
  fg0: '#e6edf3',
  fg1: '#aab3bd',
  fg2: '#6b7682',
  accent: '#5cd4e1',
  /** The GPS L1 band and the blackout window, nowhere else. */
  warn: '#e3b341',
  ok: '#8ed4a8',
} as const;

export const font = {
  sans: 'Geist, system-ui, sans-serif',
  mono: '"JetBrains Mono", ui-monospace, monospace',
} as const;

export const stage = { width: 1920, height: 1080 } as const;
export const render = { width: 3840, height: 2160, fps: 30 } as const;

/** Narration pace used to time a scene until its recording exists. */
export const WORDS_PER_MINUTE = 150;
