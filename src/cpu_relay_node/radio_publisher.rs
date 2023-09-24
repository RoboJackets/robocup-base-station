//!
//! The Radio Publisher Utilizes the LoRa Radio to publish Data to the robots.
//! 

use std::sync::{Arc, Mutex};
use std::marker::{Send, PhantomData};

use ncomm::publisher_subscriber::Publish;

use packed_struct::PackedStruct;

use packed_struct::types::bits::ByteArray;
use sx127::{LoRa, RadioMode};

use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::blocking::delay::{DelayMs, DelayUs};

use robojackets_robocup_rtp::RTPHeader;

/// The Radio Publisher wraps the SX127 Library with the ncomm Publish trait to
/// make the codebase more uniform.  The data prefix will be sent before messages published.
/// It is mainly to allow the receiver to decipher the message type.
pub struct RadioPublisher<
    SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>,
    CS: OutputPin,
    RESET: OutputPin,
    DELAY: DelayMs<u8> + DelayUs<u8>,
    ERR,
    Data: PackedStruct + Clone + Send + RTPHeader,
> {
    radio: Arc<Mutex<LoRa<SPI, CS, RESET, DELAY>>>,
    phantom: PhantomData<Data>,
}

impl<SPI, CS, RESET, DELAY, ERR, Data: PackedStruct + Clone + Send + RTPHeader> RadioPublisher<SPI, CS, RESET, DELAY, ERR, Data>
    where SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>, CS: OutputPin, 
    RESET: OutputPin, DELAY: DelayMs<u8> + DelayUs<u8> {
    pub fn new(radio: Arc<Mutex<LoRa<SPI, CS, RESET, DELAY>>>) -> Self {
        Self {
            radio,
            phantom: PhantomData,
        }
    }
}

impl<SPI, CS, RESET, DELAY, ERR, Data: PackedStruct + Clone + Send + RTPHeader> Publish<Data> for RadioPublisher<SPI, CS, RESET, DELAY, ERR, Data>
    where SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>, CS: OutputPin, RESET: OutputPin,
    DELAY: DelayMs<u8> + DelayUs<u8> {
    fn send(&mut self, data: Data) {
        let packed_data = match data.pack() {
            Ok(bytes) => bytes,
            Err(err) => {
                println!("Unable to Pack Data: {:?}", err);
                return;
            },
        };

        let packed_data = packed_data.as_bytes_slice();

        let mut packet = [0u8; 255];
        packet[0] = Data::get_header() as u8;
        packet[1..1+packed_data.len()].copy_from_slice(packed_data);

        let mut radio = self.radio.lock().unwrap();

        if radio.set_tx_power(17, 1).is_err() {
            println!("Unable to set transmit power");
        }

        if let Err(err) = radio.transmit_payload_busy(packet, packed_data.len()) {
            match err {
                sx127::Error::CS(_) => println!("CS Error"),
                sx127::Error::Reset(_) => println!("RESET Error"),
                sx127::Error::SPI(_) => println!("SPI Error"),
                sx127::Error::Transmitting => println!("Transmitting Error"),
                _ => println!("Unknown Error"),
            }
        } else {
            println!("SUCCESS");
        }
    }
}

// I wrapped the radio peripheral with an arc mutex so this node is thread safe.
unsafe impl<SPI, CS, RESET, DELAY, ERR, Data> Send for RadioPublisher<SPI, CS, RESET, DELAY, ERR, Data> where
    SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>, CS: OutputPin,
    RESET: OutputPin, DELAY: DelayMs<u8> + DelayUs<u8>, Data: PackedStruct + Clone + Send + RTPHeader {}
