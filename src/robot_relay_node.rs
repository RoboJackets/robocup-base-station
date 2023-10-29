//!
//! The Robot Relay Node takes statuses from the robots, conglomerates them
//! and sends them back to the base computer every x milliseconds.
//! 
//! The goal of this is to handle the determining of robots being alive or dead
//! on the base station end because rust is much easier to modify than our massive
//! C++ codebase.
//! 

use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use ncomm::publisher_subscriber::{Receive, Subscribe};
use ncomm::publisher_subscriber::local::{LocalPublisher, LocalSubscriber};
use ncomm::publisher_subscriber::{Publish, udp::UdpPublisher};
use ncomm::node::Node;

use robojackets_robocup_rtp::Team;
use robojackets_robocup_rtp::robot_status_message::RobotStatusMessage;

pub mod radio_subscriber;
use radio_subscriber::RadioSubscriber;

use embedded_hal::blocking::{spi::{Transfer, Write}, delay::{DelayMs, DelayUs}};
use embedded_hal::digital::v2::OutputPin;
use sx127::LoRa;

/// The Robot Relay Node Subscribes to the Messages coming from the robots via radio and sends this
/// information to the base computer.  It also notes when the last time it received data from the
/// robots and forwards this data to the timeout checker so that it can deal with determining which
/// robots are alive.
pub struct RobotRelayNode<
    'a,
    SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>,
    CS: OutputPin,
    RESET: OutputPin,
    DELAY: DelayMs<u8> + DelayUs<u8>,
    ERR
> {
    robot_state_publisher: UdpPublisher<'a, RobotStatusMessage, 126>,
    robot_status_subscriber: RadioSubscriber<SPI, CS, RESET, DELAY, ERR, RobotStatusMessage>,
    last_send_publishers: Vec<LocalPublisher<u128>>,
    num_robots: u8,
    _team: Team,
}

impl<'a, SPI, CS, RESET, DELAY, ERR> RobotRelayNode<'a, SPI, CS, RESET, DELAY, ERR>
    where SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>, CS: OutputPin,
    RESET: OutputPin, DELAY: DelayMs<u8> + DelayUs<u8> {
    pub fn new(
        bind_address: &'a str,
        publish_addresses: Vec<&'a str>,
        radio_peripherals: Arc<Mutex<LoRa<SPI, CS, RESET, DELAY>>>,
        team: Team,
        num_robots: u8,
    ) -> Self {
        let robot_state_publisher = UdpPublisher::new(bind_address, publish_addresses);
        let robot_status_subscriber = RadioSubscriber::new(radio_peripherals);
        let mut last_send_publishers = Vec::with_capacity(num_robots as usize);
        for _ in 0..num_robots {
            last_send_publishers.push(LocalPublisher::new());
        }

        Self {
            robot_state_publisher,
            robot_status_subscriber,
            last_send_publishers,
            num_robots,
            _team: team,
        }
    }

    pub fn create_subscriber(&mut self) -> Vec<LocalSubscriber<u128>> {
        let mut subscribers = Vec::with_capacity(self.num_robots as usize);
        for publisher in self.last_send_publishers.iter_mut() {
            subscribers.push(publisher.create_subscriber());
        }
        subscribers
    }
}

impl<'a, SPI, CS, RESET, DELAY, ERR> Node for RobotRelayNode<'a, SPI, CS, RESET, DELAY, ERR>
    where SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>, CS: OutputPin,
    RESET: OutputPin, DELAY: DelayMs<u8> + DelayUs<u8> {
    fn name(&self) -> String { String::from("Robot --> Cpu Node") }

    fn get_update_delay(&self) -> u128 { 0 }

    // Nothing to do on start
    fn start(&mut self) { }

    fn update(&mut self) {
        self.robot_status_subscriber.update_data();

        if !self.robot_status_subscriber.data.is_empty() {
            for data in self.robot_status_subscriber.data.drain(0..) {
                println!("Received Data From Robots:\n{:?}", data);
                // Tell Timeout Checker We Have Received Data from the Robot
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
                self.last_send_publishers.get_mut(*data.robot_id as usize).unwrap().send(now);
            
                // Forward the Data Along
                self.robot_state_publisher.send(data);
            }
        }
    }

    // Nothing to do on shutdown
    fn shutdown(&mut self) { }

    fn debug(&self) -> String {
        self.name()
    }
}
