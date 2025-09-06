#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MidiNote {
    CNeg1, CsNeg1, DNeg1, DsNeg1, ENeg1, FNeg1, FsNeg1, GNeg1, GsNeg1, ANeg1, AsNeg1, BNeg1, // 0..11
    C0, Cs0, D0, Ds0, E0, F0, Fs0, G0, Gs0, A0, As0, B0,                                     // 12..23
    C1, Cs1, D1, Ds1, E1, F1, Fs1, G1, Gs1, A1, As1, B1,                                     // 24..35
    C2, Cs2, D2, Ds2, E2, F2, Fs2, G2, Gs2, A2, As2, B2,                                     // 36..47
    C3, Cs3, D3, Ds3, E3, F3, Fs3, G3, Gs3, A3, As3, B3,                                     // 48..59
    C4, Cs4, D4, Ds4, E4, F4, Fs4, G4, Gs4, A4, As4, B4,                                     // 60..71
    C5, Cs5, D5, Ds5, E5, F5, Fs5, G5, Gs5, A5, As5, B5,                                     // 72..83
    C6, Cs6, D6, Ds6, E6, F6, Fs6, G6, Gs6, A6, As6, B6,                                     // 84..95
    C7, Cs7, D7, Ds7, E7, F7, Fs7, G7, Gs7, A7, As7, B7,                                     // 96..107
    C8, Cs8, D8, Ds8, E8, F8, Fs8, G8, Gs8, A8, As8, B8,                                     // 108..119
    C9, Cs9, D9, Ds9, E9, F9, Fs9, G9,                                                       // 120..127
}
