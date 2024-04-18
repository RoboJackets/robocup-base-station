pub mod timeout_checker;

// All Functionality Involving 1 Radio Communication
pub mod one_radio;

// Radio Publishers
pub mod publishers;

// PIN DEFINITIONS
/// Chip Select (CSN) for the Radio
pub const RADIO_CSN: u8 = 8;
/// Chip Enable (CE) for the Radio
pub const RADIO_CE: u8 = 22;
/// Radio Interrupt (IRQ) Pin
pub const RADIO_IRQ: u8 = 10;