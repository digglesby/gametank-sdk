//! Audio demo module - example chord progressions and sequencing
//!
//! This module works with both 7ch-linear (0-16 volume) and 8ch (0-63 volume) firmwares.

use gametank::audio::{voices, MidiNote, WAVETABLE, VOICE_COUNT};

// Detect which firmware we're using based on available features
#[cfg(feature = "audio-wavetable-7ch-linear")]
const MAX_VOLUME: u8 = 16;
#[cfg(feature = "audio-wavetable-8ch")]
const MAX_VOLUME: u8 = 63;

// Both firmwares now have full-amplitude sine at WAVETABLE[0]
const SINE_WAVETABLE: u16 = WAVETABLE[0];

/// Sequencer state for the demo
pub struct DemoSequencer {
    /// Frame counter (resets every 60 frames = 1 second at 60fps)
    frame: u16,
    /// Current step in the sequence
    step: u8,
    /// Background chord volume level
    bg_level: u8,
    /// Melody voice volume level
    melody_level: u8,
    /// Counter for background fade timing
    bg_fade_counter: u8,
    /// Counter for melody fade timing
    melody_fade_counter: u8,
}

impl DemoSequencer {
    pub const fn new() -> Self {
        Self {
            frame: 0,
            step: 0,
            bg_level: MAX_VOLUME,
            melody_level: MAX_VOLUME,
            bg_fade_counter: 0,
            melody_fade_counter: 0,
        }
    }

    /// Call once per frame (60fps). Advances the sequence.
    pub fn tick(&mut self) {
        let v = voices();
        // Use the last voice for melody (works with both 7ch and 8ch)
        let melody_voice = 5;

        // Process current step BEFORE incrementing (matches original timing)
        match self.step {
            // Build up Cmaj7 chord, one note per second
            1 => {
                if self.frame == 0 {
                    v[0].set_note(MidiNote::C4);
                    v[0].set_volume(self.bg_level);
                }
            }
            2 => {
                if self.frame == 0 {
                    v[1].set_note(MidiNote::E4);
                    v[1].set_volume(self.bg_level);
                }
            }
            3 => {
                if self.frame == 0 {
                    v[2].set_note(MidiNote::G4);
                    v[2].set_volume(self.bg_level);
                }
            }
            4 => {
                if self.frame == 0 {
                    v[3].set_note(MidiNote::B4);
                    v[3].set_volume(self.bg_level);
                }
            }
            // Step 5: Add D5
            5 => {
                if self.frame == 0 {
                    v[4].set_note(MidiNote::D5);
                    v[4].set_volume(self.bg_level);
                }
            }

            // Steps 6-9: Arpeggio melody on last voice, fade background
            6..=9 => {
                // Start melody voice at step 6
                if self.step == 6 && self.frame == 0 {
                    v[melody_voice].set_volume(self.melody_level);
                }

                // Play arpeggio pattern during step 8
                if self.step == 8 {
                    match self.frame {
                        0 => v[melody_voice].set_note(MidiNote::E5),
                        20 => v[melody_voice].set_note(MidiNote::B4),
                        40 => v[melody_voice].set_note(MidiNote::G4),
                        _ => {}
                    }
                }

                // Fade out background chord using counter instead of modulo
                // For 8ch (0-63): fade every 3 frames (240/3=80 updates, covers 63->0)
                // For 7ch (0-16): fade every 14 frames (240/14=17 updates, covers 16->0)
                const BG_FADE_INTERVAL: u8 = if MAX_VOLUME > 32 { 3 } else { 14 };
                self.bg_fade_counter += 1;
                if self.bg_fade_counter >= BG_FADE_INTERVAL {
                    self.bg_fade_counter = 0;
                    if self.bg_level > 0 {
                        self.bg_level -= 1;
                        v[0].set_volume(self.bg_level);
                        v[1].set_volume(self.bg_level);
                        v[2].set_volume(self.bg_level);
                        v[3].set_volume(self.bg_level);
                        v[4].set_volume(self.bg_level);
                    }
                }
            }

            // Fade out melody
            10..=26 => {
                // Scale fade rate: 8ch needs faster fade (more levels to cover)
                const MELODY_FADE_INTERVAL: u8 = if MAX_VOLUME > 32 { 4 } else { 15 };
                self.melody_fade_counter += 1;
                if self.melody_fade_counter >= MELODY_FADE_INTERVAL {
                    self.melody_fade_counter = 0;
                    if self.melody_level > 0 {
                        self.melody_level -= 1;
                        v[melody_voice].set_volume(self.melody_level);
                    }
                }
            }

            // Sequence complete
            _ => {}
        }

        // Increment counters AFTER processing (matches original)
        self.frame += 1;
        if self.frame >= 60 {
            self.frame = 0;
            self.step += 1;
        }
    }
}

/// Initialize voices for the demo (set wavetables, mute all)
pub fn init_demo() -> DemoSequencer {
    let v = voices();

    // Set all voices to use the full-amplitude sine wavetable and mute
    for voice in v.iter_mut() {
        voice.set_wavetable(SINE_WAVETABLE);
        voice.set_volume(0);
    }

    DemoSequencer::new()
}
