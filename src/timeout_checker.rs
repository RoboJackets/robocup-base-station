//!
//! The Timeout Checker checks the last received timestamp for data from the robots
//! and decides when a robot could be considered dead.  When it is dead, the TimeoutChecker
//! will continually send it a request to wake up.
//! 

use std::{sync::{Arc, Mutex}, time::{SystemTime, UNIX_EPOCH}};

use embedded_hal::blocking::{spi::{Transfer, Write}, delay::{DelayMs, DelayUs}};
use embedded_hal::digital::v2::OutputPin;

use ncomm::{publisher_subscriber::{local::LocalSubscriber, Publish, Receive}, node::Node};

use robojackets_robocup_rtp::control_command::ControlCommand;
use robojackets_robocup_rtp::Team;

use sx127::LoRa;

use crate::cpu_relay_node::radio_publisher::RadioPublisher;

/// The Timeout Checker will receive the timestamps for last send from the RobotRelayNode
/// and compute (every 100 ms) whether or not a robot should be considered dead.  Then, if
/// a robot is considered dead the TimeoutCheckerNode will periodically send wake up commands
/// until the robot wakes up.
pub struct TimeoutCheckerNode<
    SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>,
    CS: OutputPin,
    RESET: OutputPin,
    DELAY: DelayMs<u8> + DelayUs<u8>,
    ERR,
> {
    radio_publisher: RadioPublisher<SPI, CS, RESET, DELAY, ERR, ControlCommand>,
    last_send_subscribers: Vec<LocalSubscriber<u128>>,
    alive_robots: Vec<bool>,
    team: Team,
    num_robots: u8,
    timeout_duration: u128,
}

impl<SPI, CS, RESET, DELAY, ERR> TimeoutCheckerNode<SPI, CS, RESET, DELAY, ERR>
    where SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>, CS: OutputPin,
    RESET: OutputPin, DELAY: DelayMs<u8> + DelayUs<u8> {
    pub fn new(
        radio_peripherals: Arc<Mutex<LoRa<SPI, CS, RESET, DELAY>>>,
        last_send_subscribers: Vec<LocalSubscriber<u128>>,
        team: Team,
        num_robots: u8,
        timeout_duration: u128,
    ) -> Self {
        let radio_publisher = RadioPublisher::new(radio_peripherals);

        Self {
            radio_publisher,
            last_send_subscribers,
            alive_robots: Vec::with_capacity(num_robots as usize),
            team,
            num_robots,
            timeout_duration,
        }
    }
}

impl<SPI, CS, RESET, DELAY, ERR> Node for TimeoutCheckerNode<SPI, CS, RESET, DELAY, ERR>
    where SPI: Transfer<u8, Error = ERR> + Write<u8, Error = ERR>, CS: OutputPin,
    RESET: OutputPin, DELAY: DelayMs<u8> + DelayUs<u8> {
    fn name(&self) -> String { String::from("Timeout Checker") }

    fn get_update_delay(&self) -> u128 { 100 }

    // Wake Up The Robots
    fn start(&mut self) {
        for robot_id in 0..self.num_robots {
            self.radio_publisher.send(ControlCommand::wake_up(self.team, robot_id));
        }
    }

    fn update(&mut self) {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

        for robot_id in 0..self.num_robots {
            self.last_send_subscribers[robot_id as usize].update_data();

            match self.last_send_subscribers[robot_id as usize].data {
                Some(timestamp) => {
                    let elapsed = current_time - timestamp;
                    self.alive_robots[robot_id as usize] = elapsed <= self.timeout_duration;
                },
                None => self.alive_robots[robot_id as usize] = false,
            }
        }

        for robot_id in 0..self.num_robots {
            if !self.alive_robots[robot_id as usize] {
                self.radio_publisher.send(ControlCommand::wake_up(self.team, robot_id));
            }
        }

        // TODO (Nathaniel Wert): Send alive robots to base computer
    }

    // Shutdown the robots
    fn shutdown(&mut self) {
        for robot_id in 0..self.num_robots {
            self.radio_publisher.send(ControlCommand::shut_down(self.team, robot_id));
        }
    }

    fn debug(&self) -> String {
        format!(
            "{}:{:?}",
            self.name(),
            self.alive_robots,
        )
    }
}