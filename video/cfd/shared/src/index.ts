/**
 * What the plasma-blackout cuts share: the design tokens, the stage and its 2D views, the capsule the
 * examples fly and the 3D shot around it. The timing machinery is the separate entry
 * `@cfd-video/shared/timing`, free of React and three.js.
 */
export * from './tokens';
export * from './views/Stage';
export * from './views/Backdrop';
export * from './views/Text';
export * from './views/Overlays';
export * from './views/Question';
export * from './views/Timeline';
export * from './three/Capsule';
export * from './three/Flight';
export type { Phrase, PhraseTiming, SceneTiming, Segment } from './timing';
