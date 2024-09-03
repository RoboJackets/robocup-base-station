use rtic_nrf24l01::config::power_amplifier::PowerAmplifier;

pub mod timeout_checker;

pub mod xbox_node;

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
pub const RADIO_IRQ: u8 = 25;

/// The base amplification level of the signals to send to the robots
pub const BASE_AMPLIFICATION_LEVEL: PowerAmplifier = PowerAmplifier::PALow;
/// The Radio channel to use (f = 2400 + CHANNEL (MHz))
pub const CHANNEL: u8 = 106;
