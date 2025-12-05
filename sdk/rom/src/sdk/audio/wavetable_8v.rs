//! wavetable-8v audio firmware interface
//!
//! This module provides a Rust interface to the 8-voice wavetable synthesizer
//! running on the GameTank's Audio Coprocessor (ACP).
//!
//! # Memory Layout (ACP side)
//! 
//! Each voice occupies 7 bytes starting at `VOICE_BASE = 0x0041`:
//! 
//! | Offset | Name     | Description                          |
//! |--------|----------|--------------------------------------|
//! | +0     | PHASE_L  | Phase accumulator low byte           |
//! | +1     | PHASE_H  | Phase accumulator high byte          |
//! | +2     | FREQ_L   | Frequency increment low byte         |
//! | +3     | FREQ_H   | Frequency increment high byte        |
//! | +4     | WAVEPTR_L| Wavetable pointer low byte           |
//! | +5     | WAVEPTR_H| Wavetable pointer high byte          |
//! | +6     | VOLUME   | Volume level (0-63)                  |
//!
//! From the main CPU, the ACP's 4KB RAM is mapped at `0x3000`, so voices
//! are accessed at `0x3041`.

use crate::sdk::audio::pitch_table::{midi_inc, MidiNote};

/// Base address for voice registers (CPU-side address, ACP RAM at 0x3000)
pub const VOICE_BASE: usize = 0x3041;
/// Number of bytes per voice
pub const VOICE_SIZE: usize = 7;
/// Number of voices
pub const VOICE_COUNT: usize = 8;

/// Base address for wavetables in ACP RAM (CPU-side)
pub const WAVETABLE_BASE: usize = 0x3400;
/// Size of each wavetable in bytes
pub const WAVETABLE_SIZE: usize = 256;
/// Number of wavetables available
pub const WAVETABLE_COUNT: usize = 8;

/// Wavetable slot addresses (CPU-side)
pub const WAVETABLE: [u16; WAVETABLE_COUNT] = [
    0x0400, 0x0500, 0x0600, 0x0700,
    0x0800, 0x0900, 0x0A00, 0x0B00,
];

/// A single synthesizer voice.
///
/// This struct is laid out to match the ACP firmware's memory layout exactly.
/// All fields are little-endian as expected by the 6502.
#[repr(C, packed)]
pub struct Voice {
    /// Phase accumulator (16.8 fixed point, high byte indexes wavetable)
    phase: u16,
    /// Frequency increment added to phase each sample
    frequency: u16,
    /// Pointer to 256-byte wavetable in ACP RAM
    wavetable: u16,
    /// Volume level (0 = silence, 63 = max)
    volume: u8,
}

impl Voice {
    /// Set the voice frequency from a MIDI note number.
    #[inline]
    pub fn set_note(&mut self, note: MidiNote) {
        self.frequency = midi_inc(note);
    }

    /// Set the voice frequency directly as a 16-bit increment value.
    /// 
    /// Use `pitch_table::midi_inc()` to convert from MIDI notes,
    /// or calculate directly: `inc = (freq_hz * 65536) / SAMPLE_RATE`
    #[inline]
    pub fn set_frequency(&mut self, freq_inc: u16) {
        self.frequency = freq_inc;
    }

    /// Set the volume level (0 = silence, 63 = maximum).
    /// 
    /// Values above 63 may cause clipping/distortion.
    #[inline]
    pub fn set_volume(&mut self, volume: u8) {
        self.volume = volume;
    }

    /// Set which wavetable this voice uses.
    /// 
    /// Pass the ACP-side address (e.g., `WAVETABLE[0]` = 0x0400).
    #[inline]
    pub fn set_wavetable(&mut self, wavetable_addr: u16) {
        self.wavetable = wavetable_addr;
    }

    /// Silence this voice immediately.
    #[inline]
    pub fn mute(&mut self) {
        self.volume = 0;
    }

    /// Reset the phase accumulator to zero (useful for hard sync effects).
    #[inline]
    pub fn reset_phase(&mut self) {
        self.phase = 0;
    }

    /// Get the current volume level.
    #[inline]
    pub fn get_volume(&self) -> u8 {
        self.volume
    }
}

/// Get a mutable reference to all 8 voices.
///
/// # Safety
/// This function creates a mutable reference to memory-mapped hardware.
/// The caller must ensure exclusive access to the voice registers.
#[inline]
pub fn voices() -> &'static mut [Voice; VOICE_COUNT] {
    unsafe { &mut *(VOICE_BASE as *mut [Voice; VOICE_COUNT]) }
}

/// Get a mutable reference to a single voice by index (0-7).
///
/// # Panics
/// Panics if `index >= 8`.
#[inline]
pub fn voice(index: usize) -> &'static mut Voice {
    assert!(index < VOICE_COUNT, "voice index out of range");
    unsafe { &mut *((VOICE_BASE + index * VOICE_SIZE) as *mut Voice) }
}

/// Silence all voices.
#[inline]
pub fn mute_all() {
    let v = voices();
    for voice in v.iter_mut() {
        voice.mute();
    }
}
