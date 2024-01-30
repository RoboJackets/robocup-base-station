//!
//! nRF24L01+ Radio Publisher and Receiver
//! 

use std::marker::{Send, PhantomData};

use ncomm::publisher_subscriber::{Publish, Receive};

use packed_struct::types::bits::ByteArray;
use packed_struct::PackedStruct;
use packed_struct::PackedStructSlice;

use robojackets_robocup_rtp::RTPHeader;
use rtic_nrf24l01::Radio;

use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::blocking::delay::{DelayMs, DelayUs};

pub struct NrfPublisherSubscriber<
    SPI: Transfer<u8, Error=SPIE> + Write<u8, Error=SPIE>,
    CSN: OutputPin<Error=GPIOE>,
    CE: OutputPin<Error=GPIOE>,
    DELAY: DelayMs<u32> + DelayUs<u32>,
    SPIE,
    GPIOE,
    SData: PackedStruct + Clone + Send + RTPHeader,
    RData: PackedStruct + Clone + Send,
> {
    radio: Radio<CE, CSN, SPI, DELAY, GPIOE, SPIE>,
    spi: SPI,
    delay: DELAY,
    pub send_status: bool,
    pub data: Vec<RData>,
    phantom: PhantomData<SData>,
}

impl<SPI, CSN, CE, DELAY, SPIE, GPIOE, SData, RData> NrfPublisherSubscriber<SPI, CSN, CE, DELAY, SPIE, GPIOE, SData, RData> where
    SPI: Transfer<u8, Error=SPIE> + Write<u8, Error=SPIE>,
    CSN: OutputPin<Error=GPIOE>,
    CE: OutputPin<Error=GPIOE>,
    DELAY: DelayMs<u32> + DelayUs<u32>,
    SData: PackedStruct + Clone + Send + RTPHeader,
    RData: PackedStruct + Clone + Send,
{
    pub fn new(radio: Radio<CE, CSN, SPI, DELAY, GPIOE, SPIE>, spi: SPI, delay: DELAY) -> Self {
        // TODO: Initialize the Radio Correctly

        Self {
            radio,
            spi,
            delay,
            send_status: true,
            data: Vec::new(),
            phantom: PhantomData,
        }
    }
}

impl<SPI, CSN, CE, DELAY, SPIE, GPIOE, SData, RData> Publish<SData> for NrfPublisherSubscriber<SPI, CSN, CE, DELAY, SPIE, GPIOE, SData, RData> where
    SPI: Transfer<u8, Error=SPIE> + Write<u8, Error=SPIE>,
    CSN: OutputPin<Error=GPIOE>,
    CE: OutputPin<Error=GPIOE>,
    DELAY: DelayMs<u32> + DelayUs<u32>,
    SData: PackedStruct + Clone + Send + RTPHeader,
    RData: PackedStruct + Clone + Send,
{
    fn send(&mut self, data: SData) {
        let packed_data = match data.pack() {
            Ok(bytes) => bytes,
            Err(err) => panic!("Unable to Pack Data: {:?}", err),
        };

        let packed_data = packed_data.as_bytes_slice();

        self.radio.stop_listening(&mut self.spi, &mut self.delay);

        self.radio.set_payload_size(packed_data.len() as u8, &mut self.spi, &mut self.delay);
        
        // TODO: Map Header to Address

        self.send_status = self.radio.write(packed_data, &mut self.spi, &mut self.delay);

        self.radio.start_listening(&mut self.spi, &mut self.delay);

        self.radio.set_payload_size(SData::ByteArray::len() as u8, &mut self.spi, &mut self.delay);
    }
}

impl<SPI, CSN, CE, DELAY, SPIE, GPIOE, SData, RData> Receive for NrfPublisherSubscriber<SPI, CSN, CE, DELAY, SPIE, GPIOE, SData, RData> where
    SPI: Transfer<u8, Error=SPIE> + Write<u8, Error=SPIE>,
    CSN: OutputPin<Error=GPIOE>,
    CE: OutputPin<Error=GPIOE>,
    DELAY: DelayMs<u32> + DelayUs<u32>,
    SData: PackedStruct + Clone + Send + RTPHeader,
    RData: PackedStruct + Clone + Send
{
    fn update_data(&mut self) {
        let data_size = RData::ByteArray::len();

        while self.radio.available(&mut self.spi, &mut self.delay) {
            let mut buffer = [0u8; 32];
            self.radio.read(&mut buffer, &mut self.spi, &mut self.delay);
            match RData::unpack_from_slice(&buffer[..data_size]) {
                Ok(data) => self.data.push(data),
                _ => return,
            }
        }
    }
}

unsafe impl<SPI, CSN, CE, DELAY, SPIE, GPIOE, SData, RData> Send for NrfPublisherSubscriber<SPI, CSN, CE, DELAY, SPIE, GPIOE, SData, RData> where
    SPI: Transfer<u8, Error=SPIE> + Write<u8, Error=SPIE>,
    CSN: OutputPin<Error=GPIOE>,
    CE: OutputPin<Error=GPIOE>,
    DELAY: DelayMs<u32> + DelayUs<u32>,
    SData: PackedStruct + Clone + Send + RTPHeader,
    RData: PackedStruct + Clone + Send
{}