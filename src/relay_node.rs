//!
//! The Relay Node runs on a max priority real-time thread and is responsible for receiving
//! Robot commands from the field computer and relaying this message to the robots.  It
//! is also responsible for relaying the status of each robot back to the field computer.
//! 

use std::collections::HashMap;
use std::convert::Infallible;
use std::time::Duration;

use ncomm::prelude::*;
use ncomm::pubsubs::local::{LocalMappedSubscriber, LocalMappedTTLSubscriber, LocalPublisher, LocalSubscriber};
use ncomm::pubsubs::udp::{UdpMappedTTLSubscriber, UdpPublisher};
use ncomm::utils::packing::Packable;

use robojackets_robocup_rtp::control_message::Mode;
use robojackets_robocup_rtp::{ControlMessage, ControlMessageBuilder, RobotStatusMessage, Team, BASE_STATION_ADDRESSES, ROBOT_RADIO_ADDRESSES};

use quanta::{Clock, Instant};

use rppal::hal::Delay;
use rtic_nrf24l01::Radio;

use rppal::{
    spi::{self, SimpleHalSpiDevice, Spi},
    gpio::{Gpio, OutputPin},
};

use crate::{NodeIdentifier, BASE_AMPLIFICATION_LEVEL, CHANNEL, RADIO_ONE_CE, RADIO_ONE_CSN};

fn id_from_message(message: &ControlMessage) -> u8 { message.robot_id }
fn subscriber_map(data: &Option<(u8, Instant)>) -> u8 { data.as_ref().unwrap().0 }
fn status_map(data: &Option<RobotStatusMessage>) -> u8 { data.as_ref().unwrap().robot_id }

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// Information pertaining to the status of the radio (i.e. latency, ...)
pub struct RadioStatus {

}

/// The relay node is responsible for relaying commands from the base computer to the robots and
/// vice versa
pub struct RelayNode {
    /// The team the radio is sending to
    team: Team,
    /// The number of robots the radio is publishing to
    num_robots: u8,

    /// The subscription to the control messages from the base computer
    command_subscriber: UdpMappedTTLSubscriber<ControlMessage, u8, fn(&ControlMessage) -> u8>,
    /// The publisher of robot statuses to the base computer
    robot_status_publisher: UdpPublisher<RobotStatusMessage>,
    /// A local publisher publishing the timestamp of receiving data from each robot
    receive_timestamp_publisher: LocalPublisher<(u8, Instant)>,
    /// A local publisher publishing the status of each robot
    local_status_publisher: LocalPublisher<RobotStatusMessage>,
    /// A local publisher publishing information about the radio's state
    radio_status_publisher: LocalPublisher<RadioStatus>,

    /// A reference clock
    ref_clock: Clock,
    /// The nrf radio
    radio: Radio<OutputPin, OutputPin, SimpleHalSpiDevice, Infallible, spi::Error>,
    /// An embedded-hal implemented delay
    delay: Delay,
    /// The spi peripheral for the radio
    spi: SimpleHalSpiDevice,
    /// The most recent status of each robot
    robot_data: HashMap<u8, RobotStatusMessage>,
}

impl RelayNode {
    /// Initialize a new RelayNode for a given team with a set number of robots.
    /// 
    /// Additionally, bind the control message subscriber to the `control_messages_bind_address` and
    /// bind a socket to the `robot_status_bind_address` listening for packets from the 
    /// `robot_status_send_address`.
    pub fn new(
        team: Team,
        num_robots: u8,
        control_message_bind_address: String,
        robot_status_bind_address: String,
        robot_status_send_address: String,
        gpio: &mut Gpio,
    ) -> Self {
        let spi = SimpleHalSpiDevice::new(
            Spi::new(
                spi::Bus::Spi0,
                spi::SlaveSelect::Ss0,
                1_000_000,
                spi::Mode::Mode0
            ).expect("Unable to initialize the radio's spi"));
        let csn = gpio.get(RADIO_ONE_CSN)
            .expect("Unable to take radio one's csn pin")
            .into_output();
        let ce = gpio.get(RADIO_ONE_CE)
            .expect("Unabel to take radio one's ce pin")
            .into_output();
        let delay = Delay::new();
        let radio = Radio::new(ce, csn);
        
        Self {
            team,
            num_robots,
            command_subscriber: UdpMappedTTLSubscriber::new(
                control_message_bind_address.parse().unwrap(),
                Duration::from_millis(100),
                id_from_message as fn(&ControlMessage) -> u8,
            ).expect("Unable to create control message subscriber"),
            robot_status_publisher: UdpPublisher::new(
                robot_status_bind_address.parse().unwrap(),
                vec![robot_status_send_address.parse().unwrap()],
            ).expect("Unable to create robot status publisher"),
            receive_timestamp_publisher: LocalPublisher::new(),
            local_status_publisher: LocalPublisher::new(),
            radio_status_publisher: LocalPublisher::new(),
            ref_clock: Clock::new(),
            radio,
            delay,
            spi,
            robot_data: HashMap::new(),
        }
    }

    /// Set the team the radio should be sending to
    pub fn set_team(&mut self, team: Team) {
        if self.team == team {
            return;
        }

        self.team = team;
        self.set_radio_read_write_pipe(team);
    }

    /// Set the number of robots the robot should be sending to
    pub fn set_num_robots(&mut self, num_robots: u8) {
        self.num_robots = num_robots;
    }

    /// Subscribe to receive the most recent timestamp each robot has relayed their status
    pub fn subscribe_to_receive_timestamps(&mut self) -> LocalMappedSubscriber<(u8, Instant), u8, fn(&Option<(u8, Instant)>) -> u8> {
        self.receive_timestamp_publisher.subscribe_mapped(subscriber_map)
    }

    /// Subscribe to the most recent status of each robot
    pub fn subscribe_to_robot_statuses(&mut self, timeout: Duration) -> LocalMappedTTLSubscriber<RobotStatusMessage, u8, fn(&Option<RobotStatusMessage>) -> u8> {
        self.local_status_publisher.subscribe_mapped_ttl(
            status_map,
            timeout
        )
    }

    /// Subscribe the the most recent status of the radio
    pub fn subscribe_to_radio_status(&mut self) -> LocalSubscriber<RadioStatus> {
        self.radio_status_publisher.subscribe()
    }

    /// Set the read and write pipe addresses for the relay node's radio
    fn set_radio_read_write_pipe(&mut self, team: Team) {
        self.radio.stop_listening(&mut self.spi, &mut self.delay);
        self.radio.open_reading_pipe(
            1,
            ROBOT_RADIO_ADDRESSES[(team == Team::Yellow) as usize][0],
            &mut self.spi,
            &mut self.delay,
        );
        self.radio.open_writing_pipe(
            BASE_STATION_ADDRESSES[(team == Team::Yellow) as usize],
            &mut self.spi,
            &mut self.delay,
        );
    }

    /// Send a control packet to a robot
    fn send_control_message(&mut self, control_message: ControlMessage) -> Mode {
        let mut buffer = vec![0u8; ControlMessage::len()];
        control_message.pack(&mut buffer).unwrap();

        self.radio.stop_listening(&mut self.spi, &mut self.delay);
        self.radio.open_writing_pipe(
            ROBOT_RADIO_ADDRESSES[(self.team == Team::Yellow) as usize][control_message.robot_id as usize],
            &mut self.spi,
            &mut self.delay,
        );
        self.radio.set_payload_size(ControlMessage::len() as u8, &mut self.spi, &mut self.delay);
        let _ = self.radio.write(&buffer, &mut self.spi, &mut self.delay);
        control_message.mode
    }

    /// Attempt to receive the status from a robot
    fn receive_status(&mut self, robot_id: u8) -> Option<RobotStatusMessage> {
        let mut buffer = vec![0u8; RobotStatusMessage::len()];
        self.radio.set_payload_size(RobotStatusMessage::len() as u8, &mut self.spi, &mut self.delay);
        self.radio.start_listening(&mut self.spi, &mut self.delay);

        let wait_time = self.ref_clock.now() + Duration::from_micros(5_000);
        while self.ref_clock.now() < wait_time {
            if self.radio.available(&mut self.spi, &mut self.delay) {
                self.radio.read(&mut buffer, &mut self.spi, &mut self.delay);
                let message = RobotStatusMessage::unpack(&buffer).unwrap();
                self.robot_data.insert(robot_id, message);
                if message.robot_id == robot_id {
                    return Some(message);
                }
            }
        }
        None
    }
}

impl Node<NodeIdentifier> for RelayNode {
    fn get_id(&self) -> NodeIdentifier {
        NodeIdentifier::Relay
    }

    fn get_update_delay_us(&self) -> u128 {
        0
    }

    fn start(&mut self) {
        self.radio.begin(&mut self.spi, &mut self.delay)
            .expect("Unable to start the radio");
        self.radio.set_pa_level(BASE_AMPLIFICATION_LEVEL, &mut self.spi, &mut self.delay);
        self.radio.set_payload_size(RobotStatusMessage::len() as u8, &mut self.spi, &mut self.delay);
        self.radio.set_channel(CHANNEL, &mut self.spi, &mut self.delay);
        self.set_radio_read_write_pipe(self.team);
    }

    fn update(&mut self) {
        for robot_id in 0..self.num_robots {
            let control_message = match self.command_subscriber.get().get(&robot_id) {
                Some(control_message) => control_message.0,
                None => ControlMessageBuilder::new().team(self.team).build(),
            };

            self.send_control_message(control_message);
            if let Some(status) = self.receive_status(robot_id) {
                // We received a response from the robot in the alloted time
                self.receive_timestamp_publisher.publish((robot_id, self.ref_clock.now())).unwrap();
                self.local_status_publisher.publish(status.clone()).unwrap();
                self.robot_status_publisher.publish(status).unwrap()
            }
            if let Some(status) = self.robot_data.remove(&robot_id) {
                // We received a response from the robot when sending a different command
                
                // I know that we didn't technically receive this now, but we received it last iteration
                // so this is close enough
                self.receive_timestamp_publisher.publish((robot_id, self.ref_clock.now())).unwrap();
                self.local_status_publisher.publish(status.clone()).unwrap();
                self.robot_status_publisher.publish(status).unwrap();
            }
        }

        // TODO: Occasionally publish information on the radio
    }

    fn shutdown(&mut self) {
        self.radio.stop_listening(&mut self.spi, &mut self.delay);
    }
}
