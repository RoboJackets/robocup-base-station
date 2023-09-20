//!
//! The CPU Relay Node takes commands from the Base Computer and Relays them to the robot.
//! 

use std::sync::{Arc, Mutex};

use ncomm::publisher_subscriber::{Receive, udp::UdpSubscriber};
use ncomm::node::Node;

use rtp::control_message::ControlMessage;

use sx127::LoRa;

pub mod radio_publisher;
use radio_publisher::RadioPublisher;

use embedded_hal::blocking::{spi::{Transfer, Write}, delay::{DelayMs, DelayUs}};
use embedded_hal::digital::v2::OutputPin;

pub struct CpuRelayNode<
    SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>,
    CS: OutputPin,
    RESET: OutputPin,
    DELAY: DelayMs<u8> + DelayUs<u8>,
    ERR
> {
    base_computer_subscriber: UdpSubscriber<ControlMessage, 80>,
    radio_publisher: RadioPublisher<SPI, CS, RESET, DELAY, ERR, ControlMessage>,
}

impl<SPI, CS, RESET, DELAY, ERR> CpuRelayNode<SPI, CS, RESET, DELAY, ERR> where 
    SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>, CS: OutputPin,
    RESET: OutputPin, DELAY: DelayMs<u8> + DelayUs<u8>{
    pub fn new(bind_address: &str, radio_peripherals: Arc<Mutex<LoRa<SPI, CS, RESET, DELAY>>>) -> Self {
        let base_computer_subscriber = UdpSubscriber::new(bind_address, None);
        let radio_publisher = RadioPublisher::new(radio_peripherals);

        Self {
            base_computer_subscriber,
            radio_publisher,
        }
    }
}

impl<SPI, CS, RESET, DELAY, ERR> Node for CpuRelayNode<SPI, CS, RESET, DELAY, ERR> where
    SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>, CS: OutputPin,
    RESET: OutputPin, DELAY: DelayMs<u8> + DelayUs<u8> {
    fn name(&self) -> String { String::from("Cpu --> Robot Node") }

    // Basically, this node should always be running
    fn get_update_delay(&self) -> u128 { 0 }

    fn start(&mut self) {
        // TODO: Tell the robots we're starting up
        todo!()
    }

    fn update(&mut self) {
        self.base_computer_subscriber.update_data();

        // TODO: Send Most Recent Subscriber Data
        todo!()
    }

    fn shutdown(&mut self) {
        // TODO: Tell the robots we're shutting down
        todo!()
    }

    fn debug(&self) -> String { 
        self.name()
    }
}