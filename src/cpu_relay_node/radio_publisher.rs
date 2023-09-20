//!
//! The Radio Publisher Utilizes the LoRa Radio to publish Data to the robots.
//! 

use std::sync::{Arc, Mutex};
use std::marker::{Send, PhantomData};

use ncomm::publisher_subscriber::Publish;

use packed_struct::PackedStruct;

use packed_struct::types::bits::ByteArray;
use sx127::LoRa;

use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::blocking::delay::{DelayMs, DelayUs};

/// The Radio Publisher wraps the SX127 Library with the ncomm Publish trait to
/// make the codebase more uniform.
pub struct RadioPublisher<
    SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>,
    CS: OutputPin,
    RESET: OutputPin,
    DELAY: DelayMs<u8> + DelayUs<u8>,
    ERR,
    Data: PackedStruct + Clone + Send
> {
    radio: Arc<Mutex<LoRa<SPI, CS, RESET, DELAY>>>,
    phantom: PhantomData<Data>,
}

impl<SPI, CS, RESET, DELAY, ERR, Data: PackedStruct + Clone + Send> RadioPublisher<SPI, CS, RESET, DELAY, ERR, Data>
    where SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>, CS: OutputPin, 
    RESET: OutputPin, DELAY: DelayMs<u8> + DelayUs<u8> {
    pub fn new(radio: Arc<Mutex<LoRa<SPI, CS, RESET, DELAY>>>) -> Self {
        Self {
            radio,
            phantom: PhantomData,
        }
    }
}

impl<SPI, CS, RESET, DELAY, ERR, Data: PackedStruct + Clone + Send> Publish<Data> for RadioPublisher<SPI, CS, RESET, DELAY, ERR, Data>
    where SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>, CS: OutputPin, RESET: OutputPin,
    DELAY: DelayMs<u8> + DelayUs<u8> {
    fn send(&mut self, data: Data) {
        let packed_data = match data.pack() {
            Ok(bytes) => bytes,
            Err(err) => {
                println!("Unable to Send: {:?}", err);
                return;
            },
        };

        let packed_data = packed_data.as_bytes_slice();

        let mut packet = [0u8; 255];
        packet[0..packed_data.len()].copy_from_slice(packed_data);

        let mut radio = self.radio.lock().unwrap();
        match radio.transmit_payload_busy(packet, packed_data.len()) {
            Ok(_) => return,
            Err(_) => {
                println!("Unable to Send");
                return;
            }
        }
    }
}

// I wrapped the radio peripheral with an arc mutex so this node is thread safe.
unsafe impl<SPI, CS, RESET, DELAY, ERR, Data> Send for RadioPublisher<SPI, CS, RESET, DELAY, ERR, Data> where
    SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>, CS: OutputPin,
    RESET: OutputPin, DELAY: DelayMs<u8> + DelayUs<u8>, Data: PackedStruct + Clone + Send {}