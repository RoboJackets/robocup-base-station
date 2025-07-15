use rtic_nrf24l01::config::power_amplifier::PowerAmplifier;

pub mod timeout_checker;
pub mod relay_node;
pub mod display_node;
pub mod input_node;

// PIN DEFINITIONS
/// Chip Select (CSN) for the first Radio
pub const RADIO_ONE_CSN: u8 = 8;
/// Chip Enable (CE) for the second Radio
pub const RADIO_ONE_CE: u8 = 22;
/// The channel radio 1 is on
pub const RADIO_ONE_CHANNEL: u8 = 106;
/// Chip Select (CSN) for the second Radio
pub const RADIO_TWO_CSN: u8 = 10;
/// Chip Enable (CE) for the second Radio
pub const RADIO_TWO_CE: u8 = 11;
/// The channel radio 2 is on
pub const RADIO_TWO_CHANNEL: u8 = 107;
/// The Display Reset
pub const DISPLAY_RESET: u8 = 12;
/// The display DC
pub const DISPLAY_DC: u8 = 13;
/// The team select toggle button
pub const TEAM_TOGGLE: u8 = 14;
/// The start / stop switch
pub const START_STOP: u8 = 15;
/// The button to increment the number of robots
pub const INCREMENT_ROBOTS: u8 = 16;
/// The button to decrement the number of robots
pub const DECREMENT_ROBOTS: u8 = 17;

/// The base amplification level of the signals to send to the robots
pub const BASE_AMPLIFICATION_LEVEL: PowerAmplifier = PowerAmplifier::PALow;
/// The Radio channel to use (f = 2400 + CHANNEL (MHz))
pub const CHANNEL: u8 = 106;

/// Identifier for Nodes
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NodeIdentifier {
    /// The node keeping track of timeouts
    Timeout,
    /// The radio relay node
    Relay,
    /// The node displaying statistics and information
    Display,
    /// The node handling user inputs (i.e. buttons)
    Input,
}
