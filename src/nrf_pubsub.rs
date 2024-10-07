//!
//! nRF24L01+ Radio Publisher and Receiver
//! 

use std::convert::Infallible;
use std::marker::{Send, PhantomData};

use ncomm::prelude::*;
use ncomm::utils::packing::Packable;

use rtic_nrf24l01::Radio;

use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::blocking::delay::{DelayMs, DelayUs};

use robojackets_robocup_rtp::{ControlMessage, CONTROL_MESSAGE_SIZE};
use robojackets_robocup_rtp::{RobotStatusMessage, ROBOT_STATUS_SIZE};
use robojackets_robocup_rtp::ROBOT_RADIO_ADDRESSES;

pub struct NrfPublisherSubscriber<
    SPI: Transfer<u8, Error=SPIE> + Write<u8, Error=SPIE>,
    CSN: OutputPin<Error=GPIOE>,
    CE: OutputPin<Error=GPIOE>,
    DELAY: DelayMs<u32> + DelayUs<u32>,
    SPIE,
    GPIOE,
> {
    radio: Radio<CE, CSN, SPI, DELAY, GPIOE, SPIE>,
    spi: SPI,
    delay: DELAY,
    pub send_status: bool,
    pub data: Vec<RobotStatusMessage>,
    phantom: PhantomData<ControlMessage>,
}

impl<SPI, CSN, CE, DELAY, SPIE, GPIOE> NrfPublisherSubscriber<SPI, CSN, CE, DELAY, SPIE, GPIOE> where
    SPI: Transfer<u8, Error=SPIE> + Write<u8, Error=SPIE>,
    CSN: OutputPin<Error=GPIOE>,
    CE: OutputPin<Error=GPIOE>,
    DELAY: DelayMs<u32> + DelayUs<u32>,
{
    pub fn new(radio: Radio<CE, CSN, SPI, DELAY, GPIOE, SPIE>, spi: SPI, delay: DELAY) -> Self {
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

impl<SPI, CSN, CE, DELAY, SPIE, GPIOE> Publisher for NrfPublisherSubscriber<SPI, CSN, CE, DELAY, SPIE, GPIOE> where
    SPI: Transfer<u8, Error=SPIE> + Write<u8, Error=SPIE>,
    CSN: OutputPin<Error=GPIOE>,
    CE: OutputPin<Error=GPIOE>,
    DELAY: DelayMs<u32> + DelayUs<u32>,
{
    type Data = ControlMessage;
    type Error = Infallible;

    fn publish(&mut self, data: Self::Data) -> Result<(), Self::Error> {
        let target_robot = data.robot_id;

        let mut buffer = vec![0u8; ControlMessage::len()];
        data.pack(&mut buffer).unwrap();

        // Configure Radio
        self.radio.stop_listening(&mut self.spi, &mut self.delay);
        self.radio.set_payload_size(CONTROL_MESSAGE_SIZE as u8, &mut self.spi, &mut self.delay);
        self.radio.open_writing_pipe(ROBOT_RADIO_ADDRESSES[target_robot as usize], &mut self.spi, &mut self.delay);

        // Send Data
        self.send_status = self.radio.write(&buffer, &mut self.spi, &mut self.delay);

        // Get Ready For Listening
        self.radio.start_listening(&mut self.spi, &mut self.delay);
        self.radio.set_payload_size(ROBOT_STATUS_SIZE as u8, &mut self.spi, &mut self.delay);

        Ok(())
    }
}

impl<SPI, CSN, CE, DELAY, SPIE, GPIOE> Subscriber for NrfPublisherSubscriber<SPI, CSN, CE, DELAY, SPIE, GPIOE> where
    SPI: Transfer<u8, Error=SPIE> + Write<u8, Error=SPIE>,
    CSN: OutputPin<Error=GPIOE>,
    CE: OutputPin<Error=GPIOE>,
    DELAY: DelayMs<u32> + DelayUs<u32>
{
    type Target = Vec<RobotStatusMessage>;

    fn get(&mut self) -> &Self::Target {
        while self.radio.available(&mut self.spi, &mut self.delay) {
            let mut buffer = [0u8; ROBOT_STATUS_SIZE];
            self.radio.read(&mut buffer, &mut self.spi, &mut self.delay);
            let data = RobotStatusMessage::unpack(&buffer).unwrap();
            self.data.push(data);
        }
        &self.data
    }
}

unsafe impl<SPI, CSN, CE, DELAY, SPIE, GPIOE> Send for NrfPublisherSubscriber<SPI, CSN, CE, DELAY, SPIE, GPIOE> where
    SPI: Transfer<u8, Error=SPIE> + Write<u8, Error=SPIE>,
    CSN: OutputPin<Error=GPIOE>,
    CE: OutputPin<Error=GPIOE>,
    DELAY: DelayMs<u32> + DelayUs<u32>
{}