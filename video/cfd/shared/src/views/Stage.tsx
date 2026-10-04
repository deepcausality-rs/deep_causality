/**
 * The 2D stage: children are laid out in 1920 × 1080 stage pixels and scaled to the render size,
 * so vector strokes and text stay sharp at 4K. The 3D canvas renders outside the stage at the full
 * render resolution.
 */
import { AbsoluteFill, useVideoConfig } from 'remotion';
import { stage } from '../tokens';

export const Stage: React.FC<{ children: React.ReactNode; style?: React.CSSProperties }> = ({ children, style }) => {
  const { width } = useVideoConfig();
  return (
    <AbsoluteFill style={{ overflow: 'hidden', ...style }}>
      <div
        style={{
          position: 'absolute',
          left: 0,
          top: 0,
          width: stage.width,
          height: stage.height,
          transform: `scale(${width / stage.width})`,
          transformOrigin: '0 0',
        }}
      >
        {children}
      </div>
    </AbsoluteFill>
  );
};
