/**
 * The end card the cuts share: the closing line; the DeepCausality logo with the tutorial address
 * beside the Center for Dynamic Causality's logo and address; and under both, the command that
 * reproduces the run and the run's wall-clock time. `pnpm sync` copies the two logos from the
 * repository's `img/` into the project's `public/logos/`.
 */
import { Img, staticFile } from 'remotion';
import { color, font } from '../tokens';

const address: React.CSSProperties = { marginTop: 30, fontFamily: font.mono, fontSize: 26, color: color.accent };

export const EndCard: React.FC<{ line: string; url: string; command: string; note: string; opacity: number }> = ({ line, url, command, note, opacity }) => (
  <div style={{ position: 'absolute', inset: 0, opacity }}>
    <div style={{ position: 'absolute', left: 160, right: 160, top: 120, textAlign: 'center', fontFamily: font.sans, fontSize: 58, fontWeight: 500, letterSpacing: '-0.01em', color: color.fg0 }}>
      {line}
    </div>
    <div style={{ position: 'absolute', left: 0, right: 0, top: 250, display: 'flex', justifyContent: 'center', gap: 140 }}>
      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
        <Img src={staticFile('logos/logo_background.jpg')} style={{ width: 640, height: 336, borderRadius: 10, border: `1px solid ${color.line1}` }} />
        <div style={address}>{url}</div>
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
        <Img src={staticFile('logos/causal_center_logo_dark.svg')} style={{ width: 336, height: 336 }} />
        <div style={address}>causalcenter.com</div>
      </div>
    </div>
    <div style={{ position: 'absolute', left: 0, right: 0, top: 712, display: 'flex', justifyContent: 'center' }}>
      <div style={{ padding: '10px 18px', borderRadius: 6, background: color.bg1, border: `1px solid ${color.line1}`, fontFamily: font.mono, fontSize: 20, color: color.fg1 }}>{command}</div>
    </div>
    <div style={{ position: 'absolute', left: 0, right: 0, top: 790, textAlign: 'center', fontFamily: font.mono, fontSize: 19, color: color.fg2 }}>{note}</div>
  </div>
);
