use rtic_nrf24l01::config::power_amplifier::PowerAmplifier;

pub mod timeout_checker;

// All Functionality Involving 1 Radio Communication
pub mod radio_node;

// Radio Publishers
pub mod nrf_pubsub;

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

/// Identifier for Nodes
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NodeIdentifier {
    /// The 1 Radio Node (or first radio node)
    Radio1,
    /// The node keeping track of timeouts
    Timeout,
}
