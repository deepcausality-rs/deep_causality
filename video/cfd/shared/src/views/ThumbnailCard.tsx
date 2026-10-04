/**
 * The YouTube thumbnail of a cut, laid out as the drone fail-safe video's title screen: the
 * DeepCausality logo, an eyebrow naming the series and part, the cut's title and its subtitle,
 * centered on the stage. `pnpm sync` copies the logo into the project's `public/logos/`.
 */
import { AbsoluteFill, Img, staticFile } from 'remotion';
import { color, font } from '../tokens';
import { Stage } from './Stage';

export const ThumbnailCard: React.FC<{ eyebrow: string; title: string; subtitle: string }> = ({ eyebrow, title, subtitle }) => (
  <AbsoluteFill style={{ background: color.bg0 }}>
    <Stage>
      <div style={{ position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', textAlign: 'center' }}>
        <Img src={staticFile('logos/logo_background.jpg')} style={{ width: 560, borderRadius: 10, border: `1px solid ${color.line1}` }} />
        <div style={{ marginTop: 50, fontFamily: font.mono, fontSize: 20, letterSpacing: 2, textTransform: 'uppercase', color: color.fg2 }}>{eyebrow}</div>
        <div style={{ marginTop: 18, fontFamily: font.sans, fontSize: 84, fontWeight: 500, lineHeight: 1.22, color: color.fg0 }}>{title}</div>
        <div style={{ marginTop: 18, fontFamily: font.sans, fontSize: 34, lineHeight: 1.22, color: color.fg1 }}>{subtitle}</div>
      </div>
    </Stage>
  </AbsoluteFill>
);
