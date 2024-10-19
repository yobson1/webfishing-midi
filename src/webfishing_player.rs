use log::info;
use midly::{Smf, TrackEvent, TrackEventKind};
use std::{thread::sleep, time::Duration};

const MIN_NOTE: u8 = 40;
const MAX_NOTE: u8 = 79;

pub struct WebfishingPlayer<'a> {
    smf: Smf<'a>,
    shift: i8,
    micros_per_tick: u64,
}

impl<'a> WebfishingPlayer<'a> {
    pub fn new(smf: Smf<'a>) -> WebfishingPlayer {
        let notes = WebfishingPlayer::get_notes(&smf);
        let shift = WebfishingPlayer::calculate_optimal_shift(&notes);
        WebfishingPlayer {
            smf,
            shift,
            micros_per_tick: 0,
        }
    }

    pub fn play(&mut self) {
        for track in self.smf.tracks.clone() {
            self.play_track(&track);
        }
    }

    fn play_track(&mut self, track: &[TrackEvent<'_>]) {
        let timing = self.smf.header.timing;
        let ticks_per_beat = match timing {
            midly::Timing::Metrical(ppq) => ppq,
            _ => unimplemented!("Timecode timing not supported"),
        };

        for event in track {
            // Wait the delta
            let micros_to_wait = event.delta.as_int() as u64 * self.micros_per_tick;
            info!(
                "{} | {}µs - Waiting: {}µs | {}s",
                event.delta.as_int(),
                self.micros_per_tick,
                micros_to_wait,
                micros_to_wait as f32 / 1_000_000.0
            );
            sleep(Duration::from_micros(micros_to_wait));

            match event.kind {
                TrackEventKind::Meta(midly::MetaMessage::Tempo(micros_per_beat)) => {
                    self.micros_per_tick =
                        micros_per_beat.as_int() as u64 / ticks_per_beat.as_int() as u64;
                    info!("Tick length: {}µs", self.micros_per_tick);
                }
                TrackEventKind::Midi { ref message, .. } => {
                    if let midly::MidiMessage::NoteOn { key, .. } = message {
                        self.play_note((key.as_int() as i8 + self.shift) as u8);
                    }
                }
                _ => {}
            }
        }
    }

    fn play_note(&self, note: u8) {
        info!("Note: {}", note);
        let note = note.clamp(MIN_NOTE, MAX_NOTE);
        todo!();
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
