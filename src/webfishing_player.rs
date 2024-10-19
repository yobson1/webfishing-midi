use log::info;
use midly::{Smf, TrackEvent, TrackEventKind};

const MIN_NOTE: u8 = 40;
const MAX_NOTE: u8 = 79;

pub struct WebfishingPlayer<'a> {
    smf: Smf<'a>,
    shift: i8,
}

impl<'a> WebfishingPlayer<'a> {
    pub fn new(smf: Smf<'a>) -> WebfishingPlayer {
        let notes = WebfishingPlayer::get_notes(&smf);
        let shift = WebfishingPlayer::calculate_optimal_shift(&notes);
        WebfishingPlayer { smf, shift }
    }

    pub fn play(&self) {
        for track in &self.smf.tracks {
            self.play_track(track);
        }
    }

    fn play_track(&self, track: &[TrackEvent<'_>]) {
        let timing = self.smf.header.timing;

        for event in track {}

        unimplemented!();
    }

    fn play_note(note: u8) {
        let note = note.clamp(MIN_NOTE, MAX_NOTE);
        unimplemented!();
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

    fn calculate_optimal_shift(notes: &Vec<u8>) -> i8 {
        let mut best_shift: i16 = 0;
        let mut max_playable_notes = 0;
        let total_notes = notes.len();

        for shift in -127..=127i16 {
            let playable_notes = notes
                .iter()
                .filter(|&&n| {
                    (n as i16 + shift) >= MIN_NOTE as i16 && (n as i16 + shift) <= MAX_NOTE as i16
                })
                .count();

            // The best shift is the one with the most playable notes that is closest to 0
            if playable_notes > max_playable_notes
                || (playable_notes == max_playable_notes && shift.abs() < best_shift.abs())
            {
                max_playable_notes = playable_notes;
                best_shift = shift;
            }
        }

        info!("Optimal shift: {}", best_shift);
        info!(
            "Total notes: {} | Playable notes: {} | Skipped notes {} : {}% playable",
            total_notes,
            max_playable_notes,
            total_notes - max_playable_notes,
            max_playable_notes as f32 / total_notes as f32 * 100.0
        );

        best_shift as i8
    }
}
