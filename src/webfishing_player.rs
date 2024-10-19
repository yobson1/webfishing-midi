use log::info;
use midly::{Format, Smf, TrackEvent, TrackEventKind};
use std::{cmp::Ordering, collections::BinaryHeap, thread::sleep, time::Duration};

const MIN_NOTE: u8 = 40;
const MAX_NOTE: u8 = 79;

#[derive(Debug, Eq, PartialEq)]
struct TimedEvent<'a> {
    absolute_time: u64,
    event: TrackEvent<'a>,
}

impl<'a> Ord for TimedEvent<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.absolute_time.cmp(&self.absolute_time)
    }
}

impl<'a> PartialOrd for TimedEvent<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct WebfishingPlayer<'a> {
    smf: Smf<'a>,
    shift: i8,
    micros_per_tick: u64,
    events: BinaryHeap<TimedEvent<'a>>,
}

impl<'a> WebfishingPlayer<'a> {
    pub fn new(smf: Smf<'a>) -> WebfishingPlayer<'a> {
        if smf.header.format != Format::Parallel {
            unimplemented!("Only parallel format is supported");
        }

        let notes = WebfishingPlayer::get_notes(&smf);
        let shift = WebfishingPlayer::calculate_optimal_shift(&notes);
        let mut player = WebfishingPlayer {
            smf,
            shift,
            micros_per_tick: 0,
            events: BinaryHeap::new(),
        };
        player.prepare_events();
        player
    }

    fn prepare_events(&mut self) {
        for track in self.smf.tracks.clone() {
            let mut absolute_time = 0;
            for event in track {
                absolute_time += event.delta.as_int() as u64;
                self.events.push(TimedEvent {
                    absolute_time,
                    event,
                });
            }
        }
    }

    pub fn play(&mut self) {
        let timing = self.smf.header.timing;
        let ticks_per_beat = match timing {
            midly::Timing::Metrical(ppq) => ppq.as_int() as u64,
            _ => panic!("Timecode timing not supported"),
        };

        let mut last_time = 0;

        while let Some(timed_event) = self.events.pop() {
            let wait_time = timed_event.absolute_time - last_time;
            sleep(Duration::from_micros(wait_time * self.micros_per_tick));
            last_time = timed_event.absolute_time;

            match timed_event.event.kind {
                TrackEventKind::Meta(midly::MetaMessage::Tempo(tempo)) => {
                    self.micros_per_tick = tempo.as_int() as u64 / ticks_per_beat;
                    info!("Tempo change: {}µs per tick", self.micros_per_tick);
                }
                TrackEventKind::Midi {
                    channel: _,
                    message,
                } => match message {
                    midly::MidiMessage::NoteOn { key, vel } => {
                        if vel.as_int() > 0 {
                            let note = (key.as_int() as i8 + self.shift) as u8;
                            self.play_note(note);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn play_note(&self, note: u8) {
        let note = note.clamp(MIN_NOTE, MAX_NOTE);
        // todo!();
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
