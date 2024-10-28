mod instruments;
mod webfishing_player;
use core::str;
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect, Input, MultiSelect};
use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;
use instruments::INSTRUMENTS;
use log::{debug, error, info};
use midly::{MetaMessage, MidiMessage, Smf, TrackEventKind};
use rusqlite::{params, Connection};
use simple_logger::SimpleLogger;
use std::{fs, io::stdin, path::Path, path::PathBuf, process::exit};
use tabled::{builder::Builder, settings::Style};
use webfishing_player::{PlayerSettings, WebfishingPlayer};
use xcap::Window;

const MIDI_DIR: &str = "./midi";
const WINDOW_NAMES: [&str; 3] = ["steam_app_3146520", "Fish! (On the WEB!)", "Godot_Engine"];

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

    let conn = Connection::open("webfishing-midi.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS track_selections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            tracks TEXT);",
        (),
    )?;

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

            info!("Selected: {}", midi_file_path.display());

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

            let chosen_tracks =
                get_tracks_selection(&midi_file_path, &settings.smf, &theme, &conn)?;
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

fn get_tracks_from_db(
    midi_path: &str,
    conn: &Connection,
) -> Result<Option<Vec<usize>>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT tracks FROM track_selections WHERE path = ?1;")?;
    let result = stmt.query_row([midi_path], |row| {
        let track_ids_str: String = row.get(0)?;
        let track_ids = track_ids_str
            .split(',')
            .filter_map(|id| id.parse::<usize>().ok())
            .collect();
        Ok(track_ids)
    });

    match result {
        Ok(track_ids) => Ok(Some(track_ids)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err),
    }
}

fn insert_tracks_to_db(
    midi_path: &str,
    tracks: &[usize],
    conn: &Connection,
) -> Result<(), rusqlite::Error> {
    let track_ids_str = tracks
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>()
        .join(","); // Convert track_ids to a comma-separated string

    debug!("Inserting tracks: {}", track_ids_str);

    conn.execute(
        "INSERT INTO track_selections (path, tracks)
        VALUES (?1, ?2)
        ON CONFLICT(path) DO UPDATE SET tracks = excluded.tracks;",
        params![midi_path, track_ids_str],
    )?;

    Ok(())
}

fn get_tracks_selection(
    midi_path: &PathBuf,
    smf: &Smf,
    theme: &ColorfulTheme,
    conn: &Connection,
) -> Result<Vec<usize>, dialoguer::Error> {
    let midi_path = match midi_path.to_str() {
        Some(midi_path) => midi_path,
        None => {
            error!("Could not convert path to string");
            pause_and_exit(-1);
        }
    };

    // If the midi_path is already in the DB then get the tracks from the DB
    let saved_tracks = match get_tracks_from_db(midi_path, conn) {
        Ok(tracks) => tracks,
        Err(err) => {
            error!("Failed to get tracks from database: {}", err);
            pause_and_exit(-1);
        }
    };

    // progams[track] = instrument
    let mut programs = vec![-1; smf.tracks.len()];
    // Get the "program"/instrument of each channel
    for (i, track) in smf.tracks.iter().enumerate() {
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
    for (i, track) in smf.tracks.iter().enumerate() {
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
                    instrument_name =
                        Some(str::from_utf8(name).unwrap_or("Failed to decode instrument name"));
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

    let defaults = if let Some(saved_tracks) = saved_tracks {
        let mut defaults = vec![false; tracks.len()];
        for track_index in saved_tracks {
            if track_index < defaults.len() {
                defaults[track_index] = true;
            }
        }
        defaults
    } else {
        vec![true; tracks.len()]
    };

    let chosen_tracks = MultiSelect::with_theme(theme)
        .with_prompt(
            format!("Which tracks to play? (use arrow keys and space to select, enter to confirm)\n  {}\n  {}", tracks_tbl[0], tracks_tbl[1]),
        )
        .items(&tracks)
        .defaults(&defaults)
        .interact()?;

    match insert_tracks_to_db(midi_path, &chosen_tracks, conn) {
        Ok(_) => {}
        Err(err) => {
            error!("Failed to insert tracks to database: {}", err);
            pause_and_exit(-1);
        }
    };

    Ok(chosen_tracks)
}

fn pause_and_exit(code: i32) -> ! {
    println!("Press Enter to exit...");
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    exit(code);
}
