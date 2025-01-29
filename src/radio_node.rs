//!
//! The Radio Node is a single-radio solution to the sending and receiving of 
//! communication for the robots.
//! 

use super::NodeIdentifier;

use std::convert::Infallible;

use ncomm::prelude::*;
use ncomm::pubsubs::local::LocalBufferedSubscriber;
use ncomm::pubsubs::{local::{LocalPublisher, LocalSubscriber, LocalMappedSubscriber}, udp::{UdpMappedSubscriber, UdpPublisher}};

use robojackets_robocup_rtp::{
    ControlMessage, ControlMessageBuilder, RobotStatusMessage, Team
};

use rppal::gpio::Gpio;
use rppal::{
    spi::{self, SimpleHalSpiDevice, Spi},
    gpio::OutputPin,
};

use quanta::Clock;

use crate::nrf_pubsub::{IncomingMessage, NrfPublisherSubscriber, Packet};
use crate::visor_node::{RadioDiagnostic, RadioUpdateDiagnostic};
use crate::{RADIO_ONE_CE, RADIO_ONE_CSN, RADIO_TWO_CE, RADIO_TWO_CSN};

pub fn id_from_message(message: &ControlMessage) -> u8 {
    message.robot_id
}

pub fn subscriber_map(data: &Option<u8>) -> u8 {
    data.unwrap_or_default()
}

pub struct RadioNode {
    team: Team,
    num_robots: u8,
    control_message_subscriber: UdpMappedSubscriber<ControlMessage, u8, fn(&ControlMessage) -> u8>,
    radio_publisher_subscriber: NrfPublisherSubscriber<SimpleHalSpiDevice, OutputPin, OutputPin, Infallible, spi::Error>,
    robot_status_publisher: UdpPublisher<RobotStatusMessage>,
    receive_message_publisher: LocalPublisher<u8>,
    alive_robots_intra_subscriber: Option<LocalSubscriber<u16>>,

    // True if diagnostics are enabled
    diagnostics_enabled: bool,
    // Optional Diagnostics Publisher
    radio_diagnostics_publisher: Option<LocalPublisher<RadioDiagnostic>>,
    // Optional Radio Update Diagnostics Publisher
    radio_update_publisher: Option<LocalPublisher<RadioUpdateDiagnostic>>,
    // Optional diagnostics clock
    clock: Option<Clock>,
}

impl RadioNode {
    pub fn new(
        team: Team,
        radio_number: u8,
        num_robots: u8,
        control_message_bind_address: String,
        robot_status_bind_address: String,
        robot_status_send_address: String,
        gpio: &mut Gpio,
    ) -> Self {
        let radio_publisher_subscriber = match radio_number {
            0 => NrfPublisherSubscriber::new(
                team,
                SimpleHalSpiDevice::new(Spi::new(spi::Bus::Spi0, spi::SlaveSelect::Ss0, 1_000_000, spi::Mode::Mode0).unwrap()),
                gpio.get(RADIO_ONE_CSN).unwrap().into_output(),
                gpio.get(RADIO_ONE_CE).unwrap().into_output(),
            ).unwrap(),
            _ => NrfPublisherSubscriber::new(
                team,
                SimpleHalSpiDevice::new(Spi::new(spi::Bus::Spi1, spi::SlaveSelect::Ss0, 1_000_000, spi::Mode::Mode0).unwrap()),
                gpio.get(RADIO_TWO_CSN).unwrap().into_output(),
                gpio.get(RADIO_TWO_CE).unwrap().into_output(),
            ).unwrap(),
        };

        let control_message_subscriber = UdpMappedSubscriber::new(
            control_message_bind_address.parse().unwrap(),
            id_from_message as fn(&ControlMessage) -> u8,
        ).unwrap();
        let robot_status_publisher = UdpPublisher::new(
            robot_status_bind_address.parse().unwrap(),
            vec![robot_status_send_address.parse().unwrap()],
        ).unwrap();
        let receive_message_publisher = LocalPublisher::new();

        Self {
            team,
            num_robots,
            control_message_subscriber,
            radio_publisher_subscriber,
            robot_status_publisher,
            receive_message_publisher,
            alive_robots_intra_subscriber: None,
            diagnostics_enabled: false,
            radio_diagnostics_publisher: None,
            radio_update_publisher: None,
            clock: None,
        }
    }

    pub fn create_subscriber(&mut self) -> LocalMappedSubscriber<u8, u8, fn(&Option<u8>) -> u8> {
        self.receive_message_publisher.subscribe_mapped(subscriber_map as fn(&Option<u8>) -> u8)
    }

    pub fn add_alive_robots_intra_publisher(&mut self, publisher: LocalSubscriber<u16>) {
        self.alive_robots_intra_subscriber = Some(publisher);
    }

    /// Create diagnostics publishers for the radio node
    pub fn get_diagnostics(&mut self) -> (LocalBufferedSubscriber<RadioDiagnostic>, LocalBufferedSubscriber<RadioUpdateDiagnostic>) {
        self.radio_diagnostics_publisher = Some(LocalPublisher::new());
        self.radio_update_publisher = Some(LocalPublisher::new());
        self.clock = Some(Clock::new());
        self.diagnostics_enabled = true;
        (
            self.radio_diagnostics_publisher.as_mut().unwrap().subscribe_buffered(),
            self.radio_update_publisher.as_mut().unwrap().subscribe_buffered()
        )
    }

    fn send_and_await_response(&mut self, control_message: ControlMessage, robot_id: u8) {
        if self.diagnostics_enabled {
            let start_time = self.clock.as_mut().unwrap().now();
            let _ = self.radio_publisher_subscriber.publish(Packet { robot_id, data: control_message });
            let send_time_us = (self.clock.as_mut().unwrap().now() - start_time).as_micros();

            let start_time = self.clock.as_mut().unwrap().now();
            let mut responded = false;
            if let Ok(data) = self.radio_publisher_subscriber.get() {
                if let IncomingMessage::RobotStatus(status) = data {
                    self.robot_status_publisher.publish(*status).unwrap();
                    self.receive_message_publisher.publish(robot_id).unwrap();
                } else {
                    println!("Received Test Message:\n{:?}", data);
                }
                responded = true;
            }
            let wait_time_us = (self.clock.as_mut().unwrap().now() - start_time).as_micros();
            let _ = self.radio_diagnostics_publisher.as_mut().unwrap().publish(RadioDiagnostic {
                robot_id,
                send_time_us,
                wait_time_us,
                responded,
            });
        } else {
            let _ = self.radio_publisher_subscriber.publish(Packet { robot_id, data: control_message });
            if let Ok(data) = self.radio_publisher_subscriber.get() {
                if let IncomingMessage::RobotStatus(status) = data {
                    self.robot_status_publisher.publish(*status).unwrap();
                    self.receive_message_publisher.publish(robot_id).unwrap();
                } else {
                    println!("Received Test Message:\n{:?}", data);
                }
            }
        }
    }
}

impl Node<NodeIdentifier> for RadioNode {
    fn get_id(&self) -> NodeIdentifier {
        NodeIdentifier::Radio1
    }

    fn get_update_delay_us(&self) -> u128 {
        100
    }

    fn start(&mut self) { }

    fn update(&mut self) {
        // For each robot, send them a control message and wait for a response
        let start_time = if self.diagnostics_enabled {
            Some(self.clock.as_mut().unwrap().now())
        } else {
            None
        };

        for robot_id in 0..self.num_robots {
            let control_message = match self.control_message_subscriber.get().get(&robot_id) {
                Some(control_message) => *control_message,
                None => ControlMessageBuilder::new().team(self.team).build(),
            };
            self.send_and_await_response(control_message, robot_id);
        }

        if let Some(start_time) = start_time {
            let elapsed = self.clock.as_mut().unwrap().now() - start_time;
            self.radio_update_publisher.as_mut().unwrap().publish(RadioUpdateDiagnostic {
                total_elapsed_time_us: elapsed.as_micros()
            }).unwrap();
        }
    }

    // TODO: Possibly Reset Radio Status on Shutdown (?)
    fn shutdown(&mut self) {}
}
