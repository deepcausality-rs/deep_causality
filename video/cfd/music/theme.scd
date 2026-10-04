// The plasma-blackout theme: one piece of music for the three CFD cuts, rendered by SuperCollider in
// non-realtime mode to a cut's exact length.
//
//   sclang theme.scd <output.wav> <seconds> <lift seconds>
//
// D minor at 88 BPM, two bars per chord over Dm9, Bbmaj7, Fmaj7 and Cadd9. An intro of pads and a
// felt-piano motif; a middle that adds the sub bass, an arpeggio through a dotted-eighth delay, and
// soft percussion; strings from the lift (the start of the cut's closing scene); and an ending that
// resolves to D major and decays to silence on the last frame. Every sound is synthesized here, and
// noise sources take fixed seeds, so the same arguments render the same file.
(
var argv = thisProcess.argv;
var outPath = argv[0];
var total = argv[1].asFloat;
var liftAt = argv[2].asFloat;

var beat = 60 / 88;
var bar = 4 * beat;
var introBars = 6;
// The final chord starts on the last bar line that leaves it at least 13 s to ring out.
var finalBar = ((total - 13) / bar).floor.asInteger;
var liftBar = (liftAt / bar).round.asInteger.clip(introBars + 4, finalBar - 4);

// MIDI notes. The voicings keep common tones between neighbours.
var chords = [
	[50, 53, 57, 60, 64], // Dm9
	[46, 50, 53, 57, 62], // Bbmaj7
	[48, 53, 57, 60, 64], // Fmaj7
	[48, 55, 62, 64, 67]  // Cadd9
];
var roots = [38, 34, 41, 36];
var finalChord = [50, 54, 57, 62, 66, 69]; // D major
var motifs = [[69, 65, 64, 62], [72, 69, 65, 64]];
var motifBeats = [0, 1.5, 2, 3];
var arpOrder = [0, 2, 4, 3, 1, 3, 4, 2];

// Buses: reverb send, delay send.
var verbBus = 16, delayBus = 18;
var defs, events = List.new, fx, score;
var note = { |t, name, args| events.add([t, ['/s_new', name, -1, 0, 0] ++ args]) };
// Intensity of the arrangement at a bar: quiet intro, a slow rise through the middle, full at the lift.
var level = { |b|
	case
	{ b < introBars } { 0.5 }
	{ b < liftBar } { 0.7 + (0.15 * (b - introBars) / (liftBar - introBars)) }
	{ 1.0 }
};
// The arpeggio and percussion thin out over the four bars before the final chord.
var thin = { |b| (finalBar - 2 - b).linlin(0, 2, 0, 1) };

defs = [
	SynthDef(\pad, { |freq = 220, amp = 0.05, dur = 4, atk = 1.5, rel = 2.5, cut = 1200, pan = 0, send = 0.5|
		var env = EnvGen.kr(Env.linen(atk, dur, rel, 1, \sine), doneAction: 2);
		var sig = Mix(Saw.ar(freq * [0.995, 1.0, 1.006], 0.33));
		sig = RLPF.ar(sig, cut, 0.5);
		sig = Pan2.ar(sig, pan) * env * amp;
		Out.ar(0, sig * (1 - send));
		Out.ar(verbBus, sig * send);
	}),
	SynthDef(\bass, { |freq = 55, amp = 0.03, dur = 4, atk = 0.3, rel = 1.5|
		var env = EnvGen.kr(Env.linen(atk, dur, rel, 1, \sine), doneAction: 2);
		var sig = SinOsc.ar(freq) + SinOsc.ar(freq * 2, 0, 0.15);
		sig = sig * env * amp;
		Out.ar(0, sig ! 2);
	}),
	SynthDef(\pluck, { |freq = 440, amp = 0.06, cut = 1800, pan = 0, dec = 0.45, send = 0.35|
		var env = EnvGen.kr(Env.perc(0.004, dec, 1, -4), doneAction: 2);
		var sig = Pulse.ar(freq, 0.35, 0.6) + SinOsc.ar(freq * 2, 0, 0.2);
		sig = RLPF.ar(sig, cut * (1 + env), 0.6);
		sig = Pan2.ar(sig, pan) * env * amp;
		Out.ar(0, sig);
		Out.ar(delayBus, sig * send);
	}),
	SynthDef(\felt, { |freq = 440, amp = 0.12, pan = 0, send = 0.55|
		var partials = [1, 2.003, 3.01, 4.02];
		var weights = [1, 0.35, 0.12, 0.05];
		var decays = [1, 0.6, 0.35, 0.2];
		var sig = Mix.fill(4, { |i|
			SinOsc.ar(freq * partials[i]) * weights[i] * EnvGen.kr(Env.perc(0.008, 5 * decays[i], 1, -5))
		});
		var env = EnvGen.kr(Env.perc(0.008, 5.5, 1, -5), doneAction: 2);
		sig = Pan2.ar(LPF.ar(sig, 3000), pan) * env * amp;
		Out.ar(0, sig * (1 - send));
		Out.ar(verbBus, sig * send);
	}),
	SynthDef(\strings, { |freq = 440, amp = 0.03, dur = 5, atk = 3, rel = 4, pan = 0, send = 0.6|
		var env = EnvGen.kr(Env.linen(atk, dur, rel, 1, \sine), doneAction: 2);
		var vib = SinOsc.kr(5.2, 0, freq * 0.004);
		var sig = Mix(Saw.ar((freq + vib) * [0.997, 1.0, 1.003], 0.33));
		sig = HPF.ar(LPF.ar(sig, 3500), 180);
		sig = Pan2.ar(sig, pan) * env * amp;
		Out.ar(0, sig * (1 - send));
		Out.ar(verbBus, sig * send);
	}),
	SynthDef(\hat, { |amp = 0.02, pan = 0, seed = 0|
		var env, sig;
		RandSeed.ir(1, seed);
		env = EnvGen.kr(Env.perc(0.002, 0.06), doneAction: 2);
		sig = BPF.ar(WhiteNoise.ar, 9000, 0.6);
		sig = Pan2.ar(sig, pan) * env * amp;
		Out.ar(0, sig);
		Out.ar(verbBus, sig * 0.3);
	}),
	SynthDef(\kick, { |amp = 0.05|
		var env = EnvGen.kr(Env.perc(0.005, 0.5, 1, -6), doneAction: 2);
		var pitch = EnvGen.kr(Env([110, 48, 42], [0.04, 0.3], \exp));
		Out.ar(0, (SinOsc.ar(pitch) * env * amp) ! 2);
	}),
	// Radio static under the whole piece: band-passed noise and sparse crackle.
	SynthDef(\static, { |amp = 0.02, dur = 180, seed = 7|
		var env, sig;
		RandSeed.ir(1, seed);
		env = EnvGen.kr(Env.linen(4, dur - 8, 4), doneAction: 2);
		sig = BPF.ar(PinkNoise.ar([1, 1]), LFNoise1.kr(0.05).exprange(800, 3000), 0.3) + (Dust.ar([3, 2.5]) * 0.3);
		sig = sig * env * amp;
		Out.ar(0, sig * 0.5);
		Out.ar(verbBus, sig * 0.5);
	}),
	SynthDef(\delay, { |time = 0.5|
		var sig = CombC.ar(In.ar(delayBus, 2), 1, time, 3);
		sig = LPF.ar(sig, 3500);
		Out.ar(0, sig * 0.3);
		Out.ar(verbBus, sig * 0.2);
	}),
	SynthDef(\verb, {
		var sig = HPF.ar(In.ar(verbBus, 2), 150);
		Out.ar(0, FreeVerb2.ar(sig[0], sig[1], 1, 0.92, 0.45));
	}),
	// The master: a short fade at either end, then a limiter.
	SynthDef(\master, { |dur = 180, fadeIn = 2, fadeOut = 1.5|
		var env = EnvGen.kr(Env([0, 1, 1, 0], [fadeIn, dur - fadeIn - fadeOut, fadeOut], \sine));
		var sig = LeakDC.ar(In.ar(0, 2)) * env;
		ReplaceOut.ar(0, Limiter.ar(sig, 0.89, 0.01));
	})
];

// The effects run at the tail of the root group, in order, after every note.
fx = [
	[0.0, ['/s_new', \delay, 1000, 1, 0, \time, 0.75 * beat]],
	[0.0, ['/s_new', \verb, 1001, 1, 0]],
	[0.0, ['/s_new', \master, 1002, 1, 0, \dur, total]]
];
note.(0.0, \static, [\dur, total]);

(0..finalBar - 1).do { |b|
	var t = b * bar;
	var c = (b div: 2) % 4;
	var lvl = level.(b);

	// Pads and bass change chord every two bars.
	if(b.even) {
		chords[c].do { |m, i|
			note.(t, \pad, [\freq, m.midicps, \amp, 0.03 + (0.025 * lvl), \dur, 2 * bar - 0.5, \atk, if(b < introBars, 3, 1.2),
				\rel, 2.5, \cut, 900 + (2100 * lvl), \pan, i.linlin(0, 4, -0.5, 0.5)]);
		};
		if(b >= 4) { note.(t, \bass, [\freq, roots[c].midicps, \amp, 0.03, \dur, 2 * bar - 1.2]) };
		if(b >= liftBar) {
			chords[c].keep(-3).do { |m, i|
				note.(t, \strings, [\freq, (m + 12).midicps, \amp, 0.06, \dur, 2 * bar - 1.5,
					\atk, if(b < (liftBar + 2), 3, 1.5), \pan, i.linlin(0, 2, -0.4, 0.4)]);
			};
		};
	};

	// The felt-piano motif opens every eight bars, on the Dm9, alternating its two forms.
	if((b % 8 == 1) and: { b < (finalBar - 4) }) {
		var motif = motifs[(b div: 8) % 2];
		motif.do { |m, i| note.(t + (motifBeats[i] * beat), \felt, [\freq, m.midicps, \amp, 0.1, \pan, 0.15]) };
	};

	// The arpeggio, in eighths, through the middle.
	if((b >= introBars) and: { b < (finalBar - 2) }) {
		var tones = chords[c] + 12;
		8.do { |k|
			note.(t + (k * beat / 2), \pluck, [\freq, tones[arpOrder[k]].midicps,
				\amp, 0.055 * lvl * thin.(b) * if(k.even, 1, 0.75), \cut, 2000 + (3000 * lvl),
				\pan, if(k.even, -0.3, 0.3)]);
		};
	};

	// Soft percussion: off-beat hats from four bars into the middle, a kick from eight.
	if((b >= (introBars + 4)) and: { b < (finalBar - 2) }) {
		[1, 3, 5, 7].do { |k|
			note.(t + (k * beat / 2), \hat, [\amp, 0.06 * lvl * thin.(b), \pan, 0.2, \seed, (b * 8) + k]);
		};
	};
	if((b >= (introBars + 8)) and: { b < (finalBar - 2) }) {
		[0, 2].do { |k| note.(t + (k * beat), \kick, [\amp, 0.05 * lvl * thin.(b)]) };
	};
};

// The ending: D major on the final bar line, ringing until the last frame.
{
	var t = finalBar * bar;
	var hold = total - t;
	finalChord.do { |m, i|
		note.(t, \pad, [\freq, m.midicps, \amp, 0.045, \dur, hold - 6.5, \atk, 1.5, \rel, 5, \cut, 1600,
			\pan, i.linlin(0, 5, -0.5, 0.5)]);
	};
	note.(t, \bass, [\freq, 38.midicps, \amp, 0.03, \dur, hold - 6, \atk, 0.5, \rel, 5]);
	finalChord.keep(-3).do { |m, i|
		note.(t, \strings, [\freq, (m + 12).midicps, \amp, 0.06, \dur, hold - 7, \atk, 2, \rel, 5,
			\pan, i.linlin(0, 2, -0.4, 0.4)]);
	};
	[69, 66, 64, 62].do { |m, i| note.(t + (motifBeats[i] * beat), \felt, [\freq, m.midicps, \amp, 0.12, \pan, 0.15]) };
	note.(t + (6 * beat), \felt, [\freq, 74.midicps, \amp, 0.09, \pan, -0.1]);
}.value;

score = Score(
	defs.collect { |d| [0.0, ['/d_recv', d.asBytes]] }
	++ fx
	++ events.asArray.sort { |a, b| a[0] <= b[0] }
	++ [[total, ['/c_set', 0, 0]]]
);
score.recordNRT(
	outputFilePath: outPath,
	sampleRate: 48000,
	headerFormat: "WAV",
	sampleFormat: "int24",
	options: ServerOptions.new.numOutputBusChannels_(2).numInputBusChannels_(0).memSize_(65536),
	duration: total,
	action: { ("theme: wrote " ++ outPath).postln; 0.exit }
);
)
