use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use midly::{Smf, TrackEvent, TrackEventKind};
use std::{fs, path::PathBuf};

const MIDI_DIR: &str = "./midi";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let theme = ColorfulTheme::default();

    let midi_file_path = get_midi_selection(&theme);
    let midi_data = fs::read(midi_file_path)?;
    let smf = Smf::parse(&midi_data)?;

    let optimal_shift = get_optimal_shift(&smf);

    println!("Optimal shift: {}", optimal_shift);

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

fn get_notes(smf: &Smf) -> Vec<u8> {
    smf.tracks
        .iter()
        .flat_map(|track| track)
        .filter_map(|event| match event.kind {
            TrackEventKind::Midi { ref message, .. } => Some(message),
            _ => None,
        })
        .filter_map(|message| match message {
            midly::MidiMessage::NoteOn { key, .. } => Some(key.as_int()),
            _ => None,
        })
        .collect()
}

fn get_optimal_shift(smf: &Smf) -> i8 {
    let notes = get_notes(smf);
    calculate_optimal_shift(&notes)
}

fn calculate_optimal_shift(notes: &[u8]) -> i8 {
    let mut best_shift: i16 = 0;
    let mut max_playable_notes = 0;

    for shift in -127..=127i16 {
        let playable_notes = notes
            .iter()
            .filter(|&&n| (n as i16 + shift) >= 40 && (n as i16 + shift) <= 79)
            .count();

        // The best shift is the one with the most playable notes that is closest to 0
        if playable_notes > max_playable_notes
            || (playable_notes == max_playable_notes && shift.abs() < best_shift.abs())
        {
            max_playable_notes = playable_notes;
            best_shift = shift;
        }
    }

    best_shift as i8
}
