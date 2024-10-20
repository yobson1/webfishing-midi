mod webfishing_player;
use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use log::{error, info};
use midly::Smf;
use simple_logger::SimpleLogger;
use std::{fs, io::stdin, path::PathBuf, process::exit};
use webfishing_player::WebfishingPlayer;
use xcap::Window;

const MIDI_DIR: &str = "./midi";
const WINDOW_NAMES: [&str; 2] = ["steam_app_3146520", "Fish! (On the WEB!)"];

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

    let mut midi_data;
    let mut default_selection = 0;
    let mut player = loop {
        let (midi_file_path, selection) = get_midi_selection(&theme, default_selection);
        default_selection = selection;
        midi_data = match fs::read(&midi_file_path) {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to read MIDI file: {}", e);
                continue;
            }
        };

        let smf = match Smf::parse(&midi_data) {
            Ok(smf) => smf,
            Err(e) => {
                error!("Failed to parse MIDI file: {}", e);
                continue;
            }
        };

        match WebfishingPlayer::new(smf, &window) {
            Ok(player) => break player,
            Err(e) => {
                error!("Error creating player: {}", e);
                continue;
            }
        }
    };

    player.play();

    Ok(())
}

fn get_window(name: &str) -> Option<Window> {
    let windows = Window::all().unwrap();
    windows.into_iter().find(|w| w.app_name() == name)
}

fn get_midi_selection(theme: &ColorfulTheme, default_selection: usize) -> (PathBuf, usize) {
    // Get a list of the .mid files from ./midi
    let midi_files: Vec<_> = fs::read_dir(MIDI_DIR)
        .unwrap_or_else(|e| {
            error!("You need to place MIDI files in {} - {}", MIDI_DIR, e);
            pause_and_exit(e.raw_os_error().unwrap_or(-1) as i32)
        })
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.extension().and_then(|s| s.to_str()) == Some("mid") {
                    Some(path)
                } else {
                    None
                }
            })
        })
        .collect();

    let midi_file_names = midi_files
        .iter()
        .map(|path| path.file_name().unwrap().to_str().unwrap())
        .collect::<Vec<_>>();

    let selection = FuzzySelect::with_theme(theme)
        .with_prompt("Select a midi file to play")
        .items(&midi_file_names)
        .default(default_selection)
        .interact()
        .unwrap();

    (midi_files[selection].clone(), selection)
}

fn pause_and_exit(code: i32) -> ! {
    println!("Press Enter to exit...");
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    exit(code);
}
