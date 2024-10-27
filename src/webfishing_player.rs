use device_query::{DeviceQuery, DeviceState, Keycode};
use enigo::{
    Button, Coordinate,
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Mouse, Settings,
};
use log::{info, warn};
use midly::{Format, Smf, TrackEvent, TrackEventKind};
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
    io::Error,
    thread::sleep,
    time::{Duration, Instant},
};
use xcap::Window;

const MIN_NOTE: u8 = 40;
const MAX_NOTE: u8 = 79;

#[derive(Debug, Eq, PartialEq)]
struct TimedEvent<'a> {
    absolute_time: u64,
    event: TrackEvent<'a>,
    track: u32,
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

pub struct PlayerSettings<'a> {
    _data: Vec<u8>,
    pub smf: Smf<'a>,
    pub loop_midi: bool,
    pub tracks: Option<Vec<usize>>,
}

impl<'a> PlayerSettings<'a> {
    pub fn new(midi_data: Vec<u8>, loop_midi: bool) -> Result<Self, midly::Error> {
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

pub struct WebfishingPlayer<'a> {
    smf: &'a Smf<'a>,
    shift: i8,
    micros_per_tick: u64,
    events: BinaryHeap<TimedEvent<'a>>,
    enigo: Enigo,
    window: &'a Window,
    cur_string_positions: HashMap<i32, i32>,
    strings_played: [bool; 6],
    last_string_usage_time: [Instant; 6],
    input_sleep_duration: u64,
    loop_midi: bool,
    wait_for_user: bool,
    tracks: &'a Vec<usize>,
}

struct GuitarPosition {
    string: i32, // 0-5, where 0 is the lowest E string
    fret: i32,   // 0 means open string, 1-15 for frets
}

impl<'a> WebfishingPlayer<'a> {
    pub fn new(
        smf: &'a Smf<'a>,
        loop_midi: bool,
        wait_for_user: bool,
        input_sleep_duration: u64,
        window: &'a Window,
        tracks: &'a Vec<usize>,
    ) -> Result<Self, Error> {
        if smf.header.format != Format::Parallel {
            warn!("Format not parallel");
        }

        let notes = WebfishingPlayer::get_notes(smf);
        let shift = WebfishingPlayer::calculate_optimal_shift(&notes);
        let mut player = WebfishingPlayer {
            smf,
            shift,
            micros_per_tick: 0,
            events: BinaryHeap::new(),
            enigo: Enigo::new(&Settings::default()).unwrap(),
            window,
            cur_string_positions: HashMap::new(),
            strings_played: [false; 6],
            last_string_usage_time: [Instant::now(); 6],
            input_sleep_duration,
            loop_midi,
            wait_for_user,
            tracks,
        };

        // For each 6 strings initialize the cur pos as 0
        for i in 0..6 {
            player.cur_string_positions.insert(i, 0);
        }

        player.prepare_events();
        Ok(player)
    }

    fn prepare_events(&mut self) {
        for (track_num, track) in self.smf.tracks.clone().iter().enumerate() {
            let should_play = self.tracks.contains(&track_num);

            let mut absolute_time = 0;
            for event in track {
                absolute_time += event.delta.as_int() as u64;
                // Skip non-meta events
                if !should_play && !matches!(event.kind, TrackEventKind::Meta(_)) {
                    continue;
                }
                self.events.push(TimedEvent {
                    absolute_time,
                    event: *event,
                    track: track_num as u32,
                });
            }
        }
    }

    fn find_best_string(&mut self, note: u8) -> Option<GuitarPosition> {
        let string_notes = [
            [
                40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55,
            ], // low E
            [
                45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60,
            ], // A
            [
                50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65,
            ], // D
            [
                55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70,
            ], // G
            [
                59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74,
            ], // B
            [
                64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79,
            ], // high E
        ];

        let int_note = note as i32;
        let current_time = Instant::now();

        // Create a vector to hold candidates based on last usage time
        let mut candidates: Vec<(i32, i32)> = Vec::new();

        for (string_index, notes) in string_notes.iter().enumerate() {
            if self.strings_played[string_index] {
                continue; // Skip if this string has already been played
            }

            if let Some(fret) = notes.iter().position(|&n| n == int_note) {
                // Found a match, add to candidates
                candidates.push((string_index as i32, fret.try_into().unwrap()));
            }
        }

        // Sort candidates by last usage time (ascending order)
        candidates.sort_by_key(|&index| {
            let string_index = index.0 as usize;
            self.last_string_usage_time[string_index]
        });

        // Select the best candidate (the one with the least last usage time)
        if let Some(&(string_index, fret)) = candidates.first() {
            // Update last usage time for the selected string
            self.last_string_usage_time[string_index as usize] = current_time;

            return Some(GuitarPosition {
                string: string_index,
                fret,
            });
        }

        None // No suitable string found
    }

    pub fn play(&mut self) {
        let timing = self.smf.header.timing;
        let ticks_per_beat = match timing {
            midly::Timing::Metrical(ppq) => ppq.as_int() as u64,
            _ => unimplemented!("Timecode timing not supported"),
        };

        let device_state = DeviceState::new();

        // Attempt to press space in-case the user's OS requires a permission pop-up for input
        if self.wait_for_user {
            self.enigo.key(Key::Space, Click).unwrap();
            println!("Tab over to the game and press backspace to start playing");
            loop {
                if device_state.get_keys().contains(&Keycode::Backspace) {
                    break;
                }
            }
        }

        // Reset the guitar to all open string
        self.set_fret(6, 0);

        loop {
            // Start a new loop for playback
            let mut last_time = 0; // Reset last_time for each loop iteration

            while let Some(timed_event) = self.events.pop() {
                let keys = device_state.get_keys();
                if keys.contains(&Keycode::Escape) {
                    info!("Song interrupted");
                    return; // Exit the play method
                }

                let wait_time = timed_event.absolute_time - last_time;
                if wait_time > 0 {
                    self.strings_played = [false; 6];
                    sleep(Duration::from_micros(wait_time * self.micros_per_tick));
                }
                last_time = timed_event.absolute_time;

                match timed_event.event.kind {
                    TrackEventKind::Meta(midly::MetaMessage::Tempo(tempo)) => {
                        self.micros_per_tick = tempo.as_int() as u64 / ticks_per_beat;
                        info!(
                            "Tempo change: {}µs per tick - track {}",
                            self.micros_per_tick, timed_event.track
                        );
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

            if self.loop_midi {
                info!("Looping the MIDI playback (Hold ESC to stop)");
                self.prepare_events();
            } else {
                break;
            }
        }
    }

    fn play_note(&mut self, note: u8) {
        let note = note.clamp(MIN_NOTE, MAX_NOTE);

        // Use the find_best_string function to get the guitar position
        if let Some(position) = self.find_best_string(note) {
            info!(
                "Playing note {} on string {} fret {}",
                note,
                position.string + 1,
                position.fret
            );

            // Set fret position
            self.set_fret(position.string, position.fret);

            // Strum the string
            self.strum_string(position.string);

            self.strings_played[position.string as usize] = true;
        } else {
            warn!("No suitable string found for note {}", note);
        }
    }

    fn set_fret(&mut self, string: i32, fret: i32) {
        // Don't attempt to change to this position if it's already set
        // It will just unset it
        if self.cur_string_positions.get(&string).unwrap_or(&-1) == &fret {
            return;
        }

        let cur_string_pos = self.cur_string_positions.entry(string).or_default();
        *cur_string_pos = fret;

        // These values need to be adjusted based on your screen resolution and game window position
        let scale_x = self.window.width() as f32 / 2560.0;
        let scale_y = self.window.height() as f32 / 1440.0;

        // Offset from the left where the strings start
        let scaled_left = (460.0 * scale_x) as i32;
        // Offset from the top where the frets start
        let scaled_top = (130.0 * scale_y) as i32;
        // Distance centre to centre of the strings
        let scaled_string = (44.0 * scale_x) as i32;
        // Distance centre to centre of the frets
        let scaled_fret = (82.0 * scale_y) as i32;

        let fret_x = self.window.x() + (scaled_left + (string * scaled_string));
        let fret_y = self.window.y() + (scaled_top + (fret * scaled_fret));

        info!(
            "x: {} y: {} | scale_x {:.3} scale_y {:.3}",
            fret_x, fret_y, scale_x, scale_y
        );

        self.enigo
            .move_mouse(fret_x, fret_y, Coordinate::Abs)
            .unwrap();
        self.enigo.button(Button::Left, Click).unwrap();
    }

    fn strum_string(&mut self, string: i32) {
        let key = match string {
            0 => Key::Unicode('q'),
            1 => Key::Unicode('w'),
            2 => Key::Unicode('e'),
            3 => Key::Unicode('r'),
            4 => Key::Unicode('t'),
            5 => Key::Unicode('y'),
            _ => return,
        };

        self.enigo.key(key, Press).unwrap();
        // NOTE: This sleep is needed for the game to read the input
        // espesially when it is low FPS since it checks input
        // once per frame
        sleep(Duration::from_millis(self.input_sleep_duration));
        self.enigo.key(key, Release).unwrap();
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
            "Total notes: {} | Playable notes: {} | Clamped notes {} : {}% playable",
            total_notes,
            max_playable_notes,
            total_notes - max_playable_notes,
            max_playable_notes as f32 / total_notes as f32 * 100.0
        );

        best_shift as i8
    }
}
