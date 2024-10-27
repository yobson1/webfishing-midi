// https://en.wikipedia.org/wiki/General_MIDI#Program_change_events
// https://github.com/ryohey/signal/blob/main/app/src/components/TrackList/InstrumentName.tsx
pub const INSTRUMENTS: [&str; 128] = [
    // Piano
    "Acoustic Grand Piano",
    "Bright Acoustic Piano",
    "Electric Grand Piano",
    "Honky-tonk Piano",
    "Electric Piano 1",
    "Electric Piano 2",
    "Harpsichord",
    "Clavinet",

    // Chromatic Percussion
    "Celesta",
    "Glockenspiel",
    "Music Box",
    "Vibraphone",
    "Marimba",
    "Xylophone",
    "Tubular Bells",
    "Dulcimer or Santoor",

    // Organ
    "Drawbar Organ",
    "Percussive Organ",
    "Rock Organ",
    "Church Organ",
    "Reed Organ",
    "Accordion",
    "Harmonica",
    "Bandoneon or Tango Accordion",

    // Guitar
    "Acoustic Guitar (nylon)",
    "Acoustic Guitar (steel)",
    "Electric Guitar (jazz)",
    "Electric Guitar (clean)",
    "Electric Guitar (muted)",
    "Electric Guitar (overdrive)",
    "Electric Guitar (distortion)",
    "Electric Guitar (harmonics)",

    // Bass
    "Acoustic Bass",
    "Electric Bass (finger)",
    "Electric Bass (picked)",
    "Electric Bass (fretless)",
    "Slap Bass 1",
    "Slap Bass 2",
    "Synth Bass 1",
    "Synth Bass 2",

    // Strings
    "Violin",
    "Viola",
    "Cello",
    "Contrabass",
    "Tremolo Strings",
    "Pizzicato Strings",
    "Orchestral Harp",
    "Timpani",

    // Ensemble
    "String Ensemble 1",
    "String Ensemble 2",
    "Synth Strings 1",
    "Synth Strings 2",
    "Choir Aahs",
    "Voice Oohs or Doos",
    "Synth Voice or Synth Choir",
    "Orchestra Hit",

    // Brass
    "Trumpet",
    "Trombone",
    "Tuba",
    "Muted Trumpet",
    "French Horn",
    "Brass Section",
    "Synth Brass 1",
    "Synth Brass 2",

    // Reed
    "Soprano Sax",
    "Alto Sax",
    "Tenor Sax",
    "Baritone Sax",
    "Oboe",
    "English Horn",
    "Bassoon",
    "Clarinet",

    // Pipe
    "Piccolo",
    "Flute",
    "Recorder",
    "Pan Flute",
    "Blown Bottle",
    "Shakuhachi",
    "Whistle",
    "Ocarina",

    // Synth Lead
    "Lead 1 (square, often chorused)",
    "Lead 2 (sawtooth or saw, often chorused)",
    "Lead 3 (calliope, usually resembling a woodwind)",
    "Lead 4 (chiff)",
    "Lead 5 (charang, a guitar-like lead)",
    "Lead 6 (voice, derived from 'synth voice' with faster attack)",
    "Lead 7 (fifths)",
    "Lead 8 (bass and lead or solo lead)",

    // Synth Pad
    "Pad 1 (new age, pad stacked with a bell, often derived from 'Fantasia' patch from Roland D-50)",
    "Pad 2 (warm, a mellower pad with slow attack)",
    "Pad 3 (polysynth or poly, a saw-like percussive pad resembling an early 1980s polyphonic synthesizer)",
    "Pad 4 (choir, identical to 'synth voice' with longer decay)",
    "Pad 5 (bowed glass or bowed, a sound resembling a glass harmonica)",
    "Pad 6 (metallic, often created from a piano or guitar sample with removed attack)",
    "Pad 7 (halo, choir-like pad, often with a filter effect)",
    "Pad 8 (sweep, pad with a pronounced 'wah' filter effect)",

    // Synth Effects
    "FX 1 (rain, a bright pluck with echoing pulses that decreases in pitch)",
    "FX 2 (soundtrack, a bright perfect fifth pad)",
    "FX 3 (crystal, a synthesized bell sound)",
    "FX 4 (atmosphere, usually a classical guitar-like sound)",
    "FX 5 (brightness, bright pad stacked with choir or bell)",
    "FX 6 (goblins, a slow-attack pad with chirping or murmuring sounds)",
    "FX 7 (echoes or echo drops, similar to 'rain')",
    "FX 8 (sci-fi or star theme, usually an electric guitar-like pad)",

    // Ethnic
    "Sitar",
    "Banjo",
    "Shamisen",
    "Koto",
    "Kalimba",
    "Bagpipe",
    "Fiddle",
    "Shanai",

    // Percussive
    "Tinkle Bell",
    "Agogô or Cowbell",
    "Steel Drums",
    "Woodblock",
    "Taiko Drum or Surdo",
    "Melodic Tom",
    "Synth Drum",
    "Reverse Cymbal",

    // Sound Effects
    "Guitar Fret Noise",
    "Breath Noise",
    "Seashore",
    "Bird Tweet",
    "Telephone Ring",
    "Helicopter",
    "Applause",
    "Gunshot",
];
