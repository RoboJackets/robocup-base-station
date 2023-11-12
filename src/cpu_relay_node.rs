//!
//! The CPU Relay Node takes commands from the Base Computer and Relays them to the robot.
//! 

use std::sync::{Arc, Mutex};

use ncomm::publisher_subscriber::Publish;
use ncomm::publisher_subscriber::{Receive, packed_udp::BufferedPackedUdpSubscriber};
use ncomm::node::Node;

use robojackets_robocup_rtp::Team;
use robojackets_robocup_rtp::control_message::ControlMessage;

use sx127::LoRa;

pub mod radio_publisher;
use radio_publisher::RadioPublisher;

use embedded_hal::blocking::{spi::{Transfer, Write}, delay::{DelayMs, DelayUs}};
use embedded_hal::digital::v2::OutputPin;

/// The Cpu Relay Node has a udp subscriber that subscribes to the base computer.  With this subscriber, this
/// node relays information coming from the base computer directly to the robots.  It has an incredibly low update
/// time (1 ms) to minimize the delay between receiving information from the base computer and sending that data.
pub struct CpuRelayNode<
    SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>,
    CS: OutputPin,
    RESET: OutputPin,
    DELAY: DelayMs<u8> + DelayUs<u8>,
    ERR
> {
    base_computer_subscriber: BufferedPackedUdpSubscriber<ControlMessage, 10>,
    control_message_publisher: RadioPublisher<SPI, CS, RESET, DELAY, ERR, ControlMessage>,
    _team: Team,
    robots: u8,
}

impl<SPI, CS, RESET, DELAY, ERR> CpuRelayNode<SPI, CS, RESET, DELAY, ERR> where 
    SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>, CS: OutputPin,
    RESET: OutputPin, DELAY: DelayMs<u8> + DelayUs<u8>{
    pub fn new(
        bind_address: &str,
        radio_peripherals: Arc<Mutex<LoRa<SPI, CS, RESET, DELAY>>>,
        team: Team,
        robots: u8,
    ) -> Self {
        let base_computer_subscriber = BufferedPackedUdpSubscriber::new(bind_address, None);
        let control_message_publisher = RadioPublisher::new(radio_peripherals);

        Self {
            base_computer_subscriber,
            control_message_publisher,
            _team: team,
            robots,
        }
    }
}

impl<SPI, CS, RESET, DELAY, ERR> Node for CpuRelayNode<SPI, CS, RESET, DELAY, ERR> where
    SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>, CS: OutputPin,
    RESET: OutputPin, DELAY: DelayMs<u8> + DelayUs<u8> {
    fn name(&self) -> String { String::from("Cpu --> Robot Node") }

    // Basically, this node should always be running
    fn get_update_delay(&self) -> u128 { 0 }

    // The Timeout Checker tells the robots to wake up so this process doesn't have to do anything
    fn start(&mut self) { }

    fn update(&mut self) {
        // Update Data from Base Computer
        self.base_computer_subscriber.update_data();

        // If Data from Base Computer, Publish it to the Robots
        for data in self.base_computer_subscriber.data.drain(..) {
            if *data.robot_id < self.robots {
                println!("Received Data from CPU:\n{:?}", data);
                self.control_message_publisher.send(data);
            }
        }
    }

    // The Timeout Checker also tells the robots to sleep so this process doesn't have to do anything
    fn shutdown(&mut self) { }

    fn debug(&self) -> String { 
        self.name()
    }
}
