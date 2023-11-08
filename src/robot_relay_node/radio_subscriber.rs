//!
//! The Radio Subscriber Utilizes the LoRa Radio to Receive Data from the robots.
//! 

use std::sync::{Arc, Mutex};
use std::marker::{Send, PhantomData};

use ncomm::publisher_subscriber::Receive;

use packed_struct::PackedStruct;
use packed_struct::PackedStructSlice;
use packed_struct::types::bits::ByteArray;

use sx127::{LoRa, RadioMode};

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
    radio: Arc<Mutex<LoRa<SPI, CS, RESET, DELAY>>>,
    phantom: PhantomData<Data>,
    pub data: Vec<Data>,
}

impl<SPI, CS, RESET, DELAY, ERR, Data> RadioSubscriber<SPI, CS, RESET, DELAY, ERR, Data>
    where SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>, CS: OutputPin,
    RESET: OutputPin, DELAY: DelayMs<u8> + DelayUs<u8>, Data: PackedStruct + Clone + Send {

    pub fn new(radio: Arc<Mutex<LoRa<SPI, CS, RESET, DELAY>>>) -> Self {
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
        let data_size = Data::ByteArray::len();

        let mut radio = self.radio.lock().unwrap();
        if let Ok(buffer) = radio.read_packet() {
            match Data::unpack_from_slice(&buffer[0..data_size]) {
                Ok(data) => {
                    self.data.push(data);
                },
                Err(err) => println!("Unablet to decode: {:?}", err),
            }
        }
    }
}

// I wrapped the radio peripheral with an arc mutex so this node is thread safe.
unsafe impl<SPI, CS, RESET, DELAY, ERR, Data> Send for RadioSubscriber<SPI, CS, RESET, DELAY, ERR, Data>
    where SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>, CS: OutputPin,
    RESET: OutputPin, DELAY: DelayMs<u8> + DelayUs<u8>, Data: PackedStruct + Clone + Send {}