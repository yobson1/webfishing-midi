use log::info;
use midly::{Smf, TrackEvent, TrackEventKind};

pub struct WebfishingPlayer<'a> {
    smf: Smf<'a>,
    shift: i8,
}

impl<'a> WebfishingPlayer<'a> {
    pub fn new(smf: Smf<'a>) -> WebfishingPlayer {
        let mut player = WebfishingPlayer { smf, shift: 0 };
        player.calculate_optimal_shift();
        player
    }

    pub fn play(&self) {
        for track in &self.smf.tracks {
            self.play_track(&track);
        }
    }

    fn play_track(&self, track: &Vec<TrackEvent<'_>>) {
        let timing = self.smf.header.timing;

        for event in track {}

        unimplemented!();
    }

    fn play_note(note: u8) {
        let note = note.clamp(40, 79);
        unimplemented!();
    }

    fn get_notes(&self) -> Vec<u8> {
        self.smf
            .tracks
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

    fn calculate_optimal_shift(&mut self) {
        let notes = self.get_notes();
        let mut best_shift: i16 = 0;
        let mut max_playable_notes = 0;
        let total_notes = notes.len();

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

        info!("Optimal shift: {}", best_shift);
        info!(
            "Total notes: {} | Playable notes: {} | Skipped notes {} : {}% playable",
            total_notes,
            max_playable_notes,
            total_notes - max_playable_notes,
            max_playable_notes as f32 / total_notes as f32 * 100.0
        );

        self.shift = best_shift as i8;
    }
}
