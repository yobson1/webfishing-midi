mod webfishing_player;
use core::str;
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect, Input, MultiSelect};
use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;
use log::{debug, error, info};
use midly::{MetaMessage, MidiMessage, TrackEventKind};
use simple_logger::SimpleLogger;
use std::{fs, io::stdin, path::Path, path::PathBuf, process::exit};
use tabled::{builder::Builder, settings::Style};
use webfishing_player::{PlayerSettings, WebfishingPlayer};
use xcap::Window;

const MIDI_DIR: &str = "./midi";
const WINDOW_NAMES: [&str; 3] = ["steam_app_3146520", "Fish! (On the WEB!)", "Godot_Engine"];

// https://en.wikipedia.org/wiki/General_MIDI#Program_change_events
// https://github.com/ryohey/signal/blob/main/app/src/components/TrackList/InstrumentName.tsx
const INSTRUMENTS: [&str; 128] = [
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logger = SimpleLogger::new()
        .with_level(if cfg!(debug_assertions) {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .without_timestamps();
    let multi = MultiProgress::new();
    LogWrapper::new(multi.clone(), logger).try_init()?;
    let theme = ColorfulTheme::default();

    let window = WINDOW_NAMES
        .iter()
        .find_map(|name| get_window(name))
        .ok_or_else(|| {
            error!("Could not find game window");
            pause_and_exit(-1);
        })
        .unwrap();

    info!(
        "Found window: {} {},{} {}x{}",
        window.title(),
        window.x(),
        window.y(),
        window.width(),
        window.height()
    );

    let min_framerate: u64 = Input::with_theme(&theme)
        .with_prompt("\nEnter your minimum FPS.\nHigher is better, but may skip notes. Default:")
        .default(40)
        .interact_text()?;

    // Calculate the ideal delay in milliseconds
    let input_sleep_duration: u64 = (1000 / min_framerate) as u64;

    loop {
        let mut song_queue: Vec<PlayerSettings> = Vec::new();
        let mut default_selection = 0;

        // Selection loop for adding songs to the queue
        loop {
            let (midi_file_path, selection) = get_midi_selection(&theme, default_selection);
            default_selection = selection;

            let midi_data = match fs::read(&midi_file_path) {
                Ok(data) => data,
                Err(e) => {
                    error!("Failed to read MIDI file: {}", e);
                    continue;
                }
            };

            // Ask if the user wants to loop the song
            let loop_midi = Confirm::with_theme(&theme)
                .with_prompt("Loop? (Hold ESC to stop)")
                .default(false)
                .interact()?;

            // Add the selected song to the queue
            let mut settings = match PlayerSettings::new(midi_data, loop_midi) {
                Ok(settings) => settings,
                Err(e) => {
                    error!("Failed to parse MIDI data: {}", e);
                    continue;
                }
            };

            // progams[track] = instrument
            let mut programs = vec![-1; settings.smf.tracks.len()];
            // Get the "program"/instrument of each channel
            for (i, track) in settings.smf.tracks.iter().enumerate() {
                for event in track {
                    match event.kind {
                        TrackEventKind::Midi { channel, message } => match message {
                            MidiMessage::ProgramChange { program } => {
                                debug!(
                                    "Program change: {} - {} channel {} track {}",
                                    program,
                                    INSTRUMENTS[program.as_int() as usize],
                                    channel,
                                    i
                                );
                                if channel == 9 {
                                    programs[i] = -128;
                                } else {
                                    programs[i] = program.as_int() as i8;
                                }
                                break;
                            }
                            _ => {}
                        },
                        _ => continue,
                    }
                }
            }

            // Ask the user which tracks to play
            let mut builder = Builder::new();
            builder.push_record(["Track #", "Track Name", "Program", "Instrument"]);
            for (i, track) in settings.smf.tracks.iter().enumerate() {
                let mut track_name = None;
                let mut instrument_name = None;
                let program_number = programs[i];
                let program_name = if program_number == -128 {
                    // Special case from rhythm channel
                    "Standard Drum Kit"
                } else {
                    INSTRUMENTS
                        .get(program_number as usize)
                        .unwrap_or(&"Unknown")
                };

                for event in track {
                    match event.kind {
                        TrackEventKind::Meta(MetaMessage::TrackName(name)) => {
                            track_name =
                                Some(str::from_utf8(name).unwrap_or("Failed to decode track name"));
                        }
                        TrackEventKind::Meta(MetaMessage::InstrumentName(name)) => {
                            instrument_name = Some(
                                str::from_utf8(name).unwrap_or("Failed to decode instrument name"),
                            );
                        }
                        _ => continue,
                    }
                    if track_name.is_some() && instrument_name.is_some() {
                        break;
                    }
                }

                builder.push_record([
                    i.to_string().as_str(),
                    track_name.unwrap_or("Unknown"),
                    program_name,
                    instrument_name.unwrap_or("Unknown"),
                ]);
            }
            let table = builder.build().with(Style::psql()).to_string();
            let tracks_tbl = table.split("\n").collect::<Vec<_>>();
            let tracks = &tracks_tbl[2..];
            let chosen_tracks = MultiSelect::with_theme(&theme)
                .with_prompt(
                    format!("Which tracks to play? (use arrow keys and space to select, enter to confirm)\n  {}\n  {}", tracks_tbl[0], tracks_tbl[1]),
                )
                .items(&tracks)
                .defaults(&vec![true; tracks.len()])
                .interact()?;

            settings.tracks = Some(chosen_tracks);

            song_queue.push(settings);

            if loop_midi {
                break; // Exit the selection loop
            }

            // Ask if the user wants to add another song
            let add_another_song = Confirm::with_theme(&theme)
                .with_prompt("Would you like to add another song to the queue?")
                .default(false)
                .interact()?;

            if !add_another_song {
                break; // Exit the selection loop
            }
        }

        // Play all songs in the queue
        for (index, settings) in song_queue.into_iter().enumerate() {
            let is_first_song = index == 0;

            let mut player = match WebfishingPlayer::new(
                settings,
                is_first_song,
                input_sleep_duration,
                &window,
                &multi,
            ) {
                Ok(player) => player,
                Err(e) => {
                    error!("Error creating player: {}", e);
                    continue;
                }
            };

            player.play();
        }

        // Ask if the user wants to play another song
        let confirmation = Confirm::with_theme(&theme)
            .with_prompt("Do you want to play another song?")
            .default(true)
            .interact()?;
        if !confirmation {
            break;
        }
    }

    Ok(())
}

fn get_window(name: &str) -> Option<Window> {
    let windows = Window::all().unwrap();
    windows.into_iter().find(|w| w.app_name() == name)
}

fn get_midi_selection(theme: &ColorfulTheme, default_selection: usize) -> (PathBuf, usize) {
    let mut current_dir = PathBuf::from(MIDI_DIR);

    loop {
        let (midi_files, folder_names) = collect_midi_files(&current_dir);

        let mut items: Vec<String> = Vec::new();

        // Add an option to go to the parent directory
        if current_dir != PathBuf::from(MIDI_DIR) {
            items.push("..".to_string());
        } else {
            // Replace parent option with refresh in ./midi
            items.push("[Refresh]".to_string());
        }

        // Add folder names
        items.extend(folder_names.iter().map(|name| format!("[Folder] {}", name)));

        // Add MIDI file names
        items.extend(
            midi_files
                .iter()
                .map(|path| path.file_name().unwrap().to_str().unwrap().to_string()),
        );

        let selection = FuzzySelect::with_theme(theme)
            .with_prompt("Select a midi file or folder to navigate")
            .items(&items)
            .default(default_selection)
            .interact()
            .unwrap();

        if selection == 0 && current_dir == PathBuf::from(MIDI_DIR) {
            // Refresh list
            current_dir = current_dir;
        } else if selection == 0 && current_dir.parent().is_some() {
            // Navigate to the parent folder
            current_dir = current_dir.parent().unwrap().to_path_buf();
        } else if selection < folder_names.len() + 1 {
            // Navigate into the selected folder
            let selected_folder = &folder_names[selection - 1]; // Adjust index for folder selection
            current_dir = current_dir.join(selected_folder); // Update current_dir to the selected folder
        } else {
            // Select a MIDI file
            let midi_file_index = selection - folder_names.len() - 1; // Adjust index for MIDI file selection
            return (midi_files[midi_file_index].clone(), midi_file_index);
        }
    }
}

fn collect_midi_files(dir: &Path) -> (Vec<PathBuf>, Vec<String>) {
    let mut midi_files = Vec::new();
    let mut folder_names = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                // Collect folder names
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    folder_names.push(name.to_string());
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("mid") {
                // Collect MIDI files
                midi_files.push(path);
            }
        }
    } else {
        error!("You need to place MIDI files in {}.", dir.display());
    }

    (midi_files, folder_names)
}

fn pause_and_exit(code: i32) -> ! {
    println!("Press Enter to exit...");
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    exit(code);
}
