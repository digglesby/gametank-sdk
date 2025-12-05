//! Audio subsystem interface for the GameTank ACP (Audio Coprocessor).
//!
//! The GameTank has a dedicated 6502-based Audio Coprocessor that runs its own
//! firmware. This module provides interfaces to communicate with the ACP.
//!
//! # Audio Firmware Selection
//!
//! Enable one of the following Cargo features to select an audio firmware:
//! - `audio-wavetable-8v`: 8-voice wavetable synthesizer (~14kHz, ~660 cycles/sample)
//!
//! # Example
//!
//! ```rust,ignore
//! use gametank_sdk::audio::{voices, MidiNote, WAVETABLE};
//!
//! // Get the voice array
//! let v = voices();
//!
//! // Set up voice 0: C4, full volume, first wavetable
//! v[0].set_note(MidiNote::C4);
//! v[0].set_volume(63);
//! v[0].set_wavetable(WAVETABLE[0]);
//! ```

// Audio firmware binary - selected via Cargo.toml features
#[cfg(feature = "audio-wavetable-8v")]
pub static FIRMWARE: &[u8; 4096] = include_bytes!("../../../../audiofw/wavetable-8v.bin");

#[cfg(feature = "audio-fm-4op")]
pub static FIRMWARE: &[u8; 4096] = include_bytes!("../../../../audiofw/fm-4op.bin");

// Audio interface modules - selected via Cargo.toml features
#[cfg(feature = "audio-wavetable-8v")]
pub mod wavetable_8v;
#[cfg(feature = "audio-wavetable-8v")]
pub use wavetable_8v::*;

// Shared
pub mod pitch_table;
pub use pitch_table::MidiNote;

