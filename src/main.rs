mod webfishing_player;
use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use midly::Smf;
use simple_logger::SimpleLogger;
use std::{fs, path::PathBuf};
use webfishing_player::WebfishingPlayer;

const MIDI_DIR: &str = "./midi";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .without_timestamps()
        .init()
        .unwrap();
    let theme = ColorfulTheme::default();

    let midi_file_path = get_midi_selection(&theme);
    let midi_data = fs::read(midi_file_path)?;
    let smf = Smf::parse(&midi_data)?;

    let mut player = WebfishingPlayer::new(smf);
    player.play();

    Ok(())
}

fn get_midi_selection(theme: &ColorfulTheme) -> PathBuf {
    // Get a list of the .mid files from ./midi
    let midi_files: Vec<_> = fs::read_dir(MIDI_DIR)
        .expect(&format!("You need to place MIDI files in {}", MIDI_DIR))
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
        .default(0)
        .interact()
        .unwrap();

    midi_files[selection].clone()
}
