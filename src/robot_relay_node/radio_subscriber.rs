//!
//! The Radio Subscriber Utilizes the LoRa Radio to Receive Data from the robots.
//! 

use std::marker::{Send, PhantomData};

use ncomm::publisher_subscriber::Receive;

use packed_struct::PackedStruct;
use packed_struct::PackedStructSlice;
use packed_struct::types::bits::ByteArray;

use robojackets_robocup_rtp::MessageType;
use sx127::LoRa;

use embedded_hal::blocking::{spi::{Transfer, Write}, delay::{DelayMs, DelayUs}};
use embedded_hal::digital::v2::OutputPin;


pub struct RadioSubscriber<
    SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>,
    CS: OutputPin,
    RESET: OutputPin,
    DELAY: DelayMs<u8> + DelayUs<u8>,
    ERR,
    Data: PackedStruct + Clone + Send
> {
    radio: LoRa<SPI, CS, RESET, DELAY>,
    phantom: PhantomData<Data>,
    pub data: Vec<Data>,
}

impl<SPI, CS, RESET, DELAY, ERR, Data> RadioSubscriber<SPI, CS, RESET, DELAY, ERR, Data>
    where SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>, CS: OutputPin,
    RESET: OutputPin, DELAY: DelayMs<u8> + DelayUs<u8>, Data: PackedStruct + Clone + Send {

    pub fn new(radio: LoRa<SPI, CS, RESET, DELAY>) -> Self {
        Self {
            radio,
            phantom: PhantomData,
            data: Vec::new(),
        }
    }
}

impl<SPI, CS, RESET, DELAY, ERR, Data> Receive for RadioSubscriber<SPI, CS, RESET, DELAY, ERR, Data>
    where SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>, CS: OutputPin,
    RESET: OutputPin, DELAY: DelayMs<u8> + DelayUs<u8>, Data: PackedStruct + Clone + Send {

    fn update_data(&mut self) {
        if let Ok(buffer) = self.radio.read_packet() {
            match MessageType::from(buffer[0]) {
                MessageType::RobotStatusMessage => {
                    let target_message_length = <Data as PackedStruct>::ByteArray::len();
                    match Data::unpack_from_slice(&buffer[1..(1+target_message_length)]) {
                        Ok(message) => {
                            self.data.push(message);
                            println!("Received Message");
                        },
                        Err(_) => {
                            println!("Unable to decode message");
                        }
                    }
                },
                _ => (),
            }
        }
    }
}

// I wrapped the radio peripheral with an arc mutex so this node is thread safe.
unsafe impl<SPI, CS, RESET, DELAY, ERR, Data> Send for RadioSubscriber<SPI, CS, RESET, DELAY, ERR, Data>
    where SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>, CS: OutputPin,
    RESET: OutputPin, DELAY: DelayMs<u8> + DelayUs<u8>, Data: PackedStruct + Clone + Send {}