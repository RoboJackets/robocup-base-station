//!
//! The Timeout Checker checks the last received timestamp for data from the robots
//! and decides when a robot could be considered dead.  When it is dead, the TimeoutChecker
//! will continually send it a request to wake up.
//! 

use std::time::Duration;

use super::NodeIdentifier;

use ncomm::prelude::*;
use ncomm::pubsubs::local::LocalSubscriber;
use ncomm::pubsubs::{local::{LocalMappedSubscriber, LocalPublisher}, udp::UdpPublisher};

use ncomm::utils::packing::Packable;
use quanta::{Clock, Instant};

/// A message where alive_robots[robot_id] tells whether a robot is alive (true) or unresponsive (false)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AliveRobotsMessage {
    alive_robots: Vec<bool>,
}

impl From<u16> for AliveRobotsMessage {
    fn from(value: u16) -> Self {
        Self::unpack(&value.to_le_bytes()).unwrap()
    }
}

impl Packable for AliveRobotsMessage {
    fn len() -> usize {
        2
    }

    fn pack(self, buffer: &mut [u8]) -> Result<(), ncomm::utils::packing::PackingError> {
        for (i, robot) in self.alive_robots.iter().enumerate() {
            if (0..=7).contains(&i) {
                buffer[0] |= (*robot as u8) << i;
            } else if (8..=15).contains(&i) {
                buffer[1] |= (*robot as u8) << i - 8;
            }
        }
        Ok(())
    }

    fn unpack(data: &[u8]) -> Result<Self, ncomm::utils::packing::PackingError> {
        let mut alive_robots = Vec::with_capacity(16);
        for data in data {
            for robot in 0..8 {
                alive_robots.push(data | (1 << robot) != 0);
            }
        }

        Ok(Self {
            alive_robots,
        })
    }
}

/// The Timeout Checker will receive the timestamps for last send from the RobotRelayNode
/// and compute (every 100 ms) whether or not a robot should be considered dead.  Then, if
/// a robot is considered dead the TimeoutCheckerNode will periodically send wake up commands
/// until the robot wakes up.
pub struct TimeoutCheckerNode {
    /// The number of robots in the system
    num_robots: u8,
    /// The amount of time (in microseconds) before we consider a robot unresponsive
    timeout_duration_us: u64,
    /// The timestamp each robot last responded
    receive_timestamp_subscriber: LocalMappedSubscriber<(u8, Instant), u8, fn(&Option<(u8, Instant)>) -> u8>,
    /// The network publisher for publishing alive robots
    alive_robots_publisher: UdpPublisher<AliveRobotsMessage>,
    /// The local publisher publishing what robots are alive
    local_alive_robots_publisher: LocalPublisher<AliveRobotsMessage>,
    /// The quanta reference clock
    clock: Clock,
}

impl TimeoutCheckerNode {
    /// Initialize a new timeout checker node that will publish a list of alive robots to
    /// `alive_robots_send_address`, while bound to the `alive_robots_bind_address`
    pub fn new(
        num_robots: u8,
        timeout_duration_us: u64,
        alive_robots_bind_address: String,
        alive_robots_send_address: String,
        receive_timestamp_subscriber: LocalMappedSubscriber<(u8, Instant), u8, fn(&Option<(u8, Instant)>) -> u8>
    ) -> Self {
        Self {
            num_robots,
            timeout_duration_us,
            receive_timestamp_subscriber,
            alive_robots_publisher: UdpPublisher::new(
                alive_robots_bind_address.parse().unwrap(),
                vec![alive_robots_send_address.parse().unwrap()]
            ).expect("Unable to create the alive robots publisher"),
            local_alive_robots_publisher: LocalPublisher::new(),
            clock: Clock::new(),
        }
    }

    // Subscribe to the alive robots publisher
    pub fn subscribe_to_alive_robots(&mut self) -> LocalSubscriber<AliveRobotsMessage> {
        self.local_alive_robots_publisher.subscribe()
    }
}
impl Node<NodeIdentifier> for TimeoutCheckerNode {
    fn get_id(&self) -> NodeIdentifier {
        NodeIdentifier::Timeout
    }

    fn get_update_delay_us(&self) -> u128 {
        self.timeout_duration_us as u128
    }

    fn start(&mut self) {
        let alive_robots = AliveRobotsMessage {
            alive_robots: vec![false; self.num_robots as usize],
        };
        self.alive_robots_publisher.publish(alive_robots.clone()).unwrap();
        self.local_alive_robots_publisher.publish(alive_robots).unwrap();
    }

    fn update(&mut self) {
        let mut alive_robots = AliveRobotsMessage {
            alive_robots: vec![false; self.num_robots as usize],
        };

        for robot_id in 0..self.num_robots {
            if let Some(timestamp) = self.receive_timestamp_subscriber.get().get(&robot_id) {
                if let Some(timestamp) = *(timestamp.as_ref()) {
                    if timestamp.1 + Duration::from_micros(self.timeout_duration_us) < self.clock.now() {
                        alive_robots.alive_robots[robot_id as usize] = true;
                    }
                }
            }
        }

        self.alive_robots_publisher.publish(alive_robots.clone()).unwrap();
        self.local_alive_robots_publisher.publish(alive_robots).unwrap();
    }

    fn shutdown(&mut self) {
        let alive_robots = AliveRobotsMessage {
            alive_robots: vec![false; self.num_robots as usize],
        };
        self.alive_robots_publisher.publish(alive_robots.clone()).unwrap();
        self.local_alive_robots_publisher.publish(alive_robots).unwrap();
    }
}
