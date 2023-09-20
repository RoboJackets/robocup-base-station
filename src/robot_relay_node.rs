//!
//! The Robot Relay Node takes statuses from the robots, conglomerates them
//! and sends them back to the base computer every x milliseconds.
//! 
//! The goal of this is to handle the determining of robots being alive or dead
//! on the base station end because rust is much easier to modify than our massive
//! C++ codebase.
//! 

use std::sync::{Arc, Mutex};

use ncomm::publisher_subscriber::{Publish, udp::UdpPublisher};
use ncomm::node::Node;

use rtp::robot_status_message::RobotStatusMessage;

pub mod radio_subscriber;
use radio_subscriber::RadioSubscriber;

use embedded_hal::blocking::{spi::{Transfer, Write}, delay::{DelayMs, DelayUs}};
use embedded_hal::digital::v2::OutputPin;
use sx127::LoRa;

pub struct RobotRelayNode<
    'a,
    SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>,
    CS: OutputPin,
    RESET: OutputPin,
    DELAY: DelayMs<u8> + DelayUs<u8>,
    ERR
> {
    base_computer_publisher: UdpPublisher<'a, [RobotStatusMessage; 6], 126>,
    radio_subscriber: RadioSubscriber<SPI, CS, RESET, DELAY, ERR, RobotStatusMessage>,
    robot_statuses: [RobotStatusMessage; 6],
}

impl<'a, SPI, CS, RESET, DELAY, ERR> RobotRelayNode<'a, SPI, CS, RESET, DELAY, ERR>
    where SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>, CS: OutputPin,
    RESET: OutputPin, DELAY: DelayMs<u8> + DelayUs<u8> {
    pub fn new(bind_address: &'a str, publish_addresses: Vec<&'a str>, radio_peripherals: Arc<Mutex<LoRa<SPI, CS, RESET, DELAY>>>) -> Self {
        let base_computer_publisher = UdpPublisher::new(bind_address, publish_addresses);
        let radio_subscriber = RadioSubscriber::new(radio_peripherals);

        Self {
            base_computer_publisher,
            radio_subscriber,
            robot_statuses: [RobotStatusMessage::default(); 6],
        }
    }
}

impl<'a, SPI, CS, RESET, DELAY, ERR> Node for RobotRelayNode<'a, SPI, CS, RESET, DELAY, ERR>
    where SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>, CS: OutputPin,
    RESET: OutputPin, DELAY: DelayMs<u8> + DelayUs<u8> {
    fn name(&self) -> String { String::from("Robot --> Cpu Node") }

    fn get_update_delay(&self) -> u128 { 10 }

    fn start(&mut self) {
        self.base_computer_publisher.send(self.robot_statuses);
    }

    fn update(&mut self) {
        // TODO: Retrieve any incoming radio packets

        self.base_computer_publisher.send(self.robot_statuses);
    }

    fn shutdown(&mut self) {
        // TODO: Send Robots Dead
        self.base_computer_publisher.send([RobotStatusMessage::default(); 6]);
    }

    fn debug(&self) -> String {
        format!(
            "{}\n{:?}",
            self.name(),
            self.robot_statuses,
        )
    }
}