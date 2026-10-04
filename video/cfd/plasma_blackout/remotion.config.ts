import { Config } from '@remotion/cli/config';

// ANGLE gives the headless browser a GPU-backed WebGL context, which the 3D register needs.
Config.setChromiumOpenGlRenderer('angle');
Config.setVideoImageFormat('jpeg');
Config.setJpegQuality(95);
Config.setOverwriteOutput(true);
