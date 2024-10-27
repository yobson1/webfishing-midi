mod webfishing_player;
use core::str;
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect, Input, MultiSelect};
use log::{error, info};
use midly::{MetaMessage, Smf, TrackEventKind};
use simple_logger::SimpleLogger;
use std::{fs, io::stdin, path::Path, path::PathBuf, process::exit};
use webfishing_player::WebfishingPlayer;
use xcap::Window;

const MIDI_DIR: &str = "./midi";
const WINDOW_NAMES: [&str; 3] = ["steam_app_3146520", "Fish! (On the WEB!)", "Godot_Engine"];

struct PlayerSettings<'a> {
    _data: Vec<u8>,
    smf: Smf<'a>,
    loop_midi: bool,
    tracks: Option<Vec<usize>>,
}

impl<'a> PlayerSettings<'a> {
    fn new(midi_data: Vec<u8>, loop_midi: bool) -> Result<Self, midly::Error> {
        let smf = Smf::parse(&midi_data)?;
        // This is safe because we keep midi_data & smf alive in the struct
        let smf = unsafe { std::mem::transmute::<Smf<'_>, Smf<'a>>(smf) };

        Ok(PlayerSettings {
            _data: midi_data,
            smf,
            loop_midi,
            tracks: None,
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .without_timestamps()
        .init()?;
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

    let mut song_queue: Vec<PlayerSettings> = Vec::new();

    let min_framerate: u64 = Input::with_theme(&theme)
        .with_prompt("\nEnter your minimum FPS.\nHigher is better, but may skip notes. Default:")
        .default(40)
        .interact_text()?;

    // Calculate the ideal delay in milliseconds
    let input_sleep_duration: u64 = (1000 / min_framerate) as u64;

    loop {
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

            // Ask the user which tracks to play
            let mut tracks = Vec::new();
            for (i, track) in settings.smf.tracks.iter().enumerate() {
                let mut track_name = "Unknown";
                for event in track {
                    match event.kind {
                        TrackEventKind::Meta(MetaMessage::TrackName(name)) => {
                            track_name =
                                str::from_utf8(name).unwrap_or("Failed to decode track name");
                            break;
                        }
                        _ => continue,
                    }
                }
                tracks.push(format!("{}: {}", i, track_name));
            }
            let chosen_tracks = MultiSelect::with_theme(&theme)
                .with_prompt("Which tracks to play?")
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
        for (index, settings) in song_queue.iter().enumerate() {
            let is_first_song = index == 0;
            let tracks = settings.tracks.clone().expect("Failed to get tracks");

            let mut player = match WebfishingPlayer::new(
                &settings.smf,
                settings.loop_midi,
                is_first_song,
                input_sleep_duration,
                &window,
                &tracks,
            ) {
                Ok(player) => player,
                Err(e) => {
                    error!("Error creating player: {}", e);
                    continue;
                }
            };

            player.play();
        }

        // Clear the queue after playing
        song_queue.clear();

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
