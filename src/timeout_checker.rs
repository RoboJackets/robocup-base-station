//!
//! The Timeout Checker checks the last received timestamp for data from the robots
//! and decides when a robot could be considered dead.  When it is dead, the TimeoutChecker
//! will continually send it a request to wake up.
//! 

use super::NodeIdentifier;

use ncomm::prelude::*;
use ncomm::pubsubs::{local::{LocalMappedSubscriber, LocalPublisher}, udp::UdpPublisher};

use ncomm::utils::packing::Packable;

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
pub struct TimeoutCheckerNode<F: Fn(&Option<u8>) -> u8 + Send> {
    num_robots: u8,
    timeout_duration: u128,
    alive_robots: Vec<bool>,
    receive_message_subscriber: LocalMappedSubscriber<u8, u8, F>,
    alive_robots_publisher: UdpPublisher<AliveRobotsMessage>,
    alive_robots_intra_publisher: LocalPublisher<u16>,
}

impl<F: Fn(&Option<u8>) -> u8 + Send> TimeoutCheckerNode<F> {
    pub fn new(
        num_robots: u8,
        timeout: u128,
        alive_robots_bind_address: String,
        alive_robots_send_address: String,
        receive_message_subscriber: LocalMappedSubscriber<u8, u8, F>
    ) -> Self {
        let alive_robots_publisher = UdpPublisher::new(
            alive_robots_bind_address.parse().unwrap(),
            vec![alive_robots_send_address.parse().unwrap()]).unwrap();
        let alive_robots = Vec::with_capacity(num_robots as usize);
        let alive_robots_intra_publisher = LocalPublisher::new();

        Self {
            num_robots,
            timeout_duration: timeout,
            alive_robots,
            receive_message_subscriber,
            alive_robots_publisher,
            alive_robots_intra_publisher,
        }
    }
}
impl<F: Fn(&Option<u8>) -> u8 + Send> Node<NodeIdentifier> for TimeoutCheckerNode<F> {
    fn get_id(&self) -> NodeIdentifier {
        NodeIdentifier::Timeout
    }

    fn get_update_delay_us(&self) -> u128 {
        self.timeout_duration * 1000
    }

    fn start(&mut self) {
        for _ in 0..self.num_robots {
            self.alive_robots.push(true);
        }
    }

    fn update(&mut self) {
        // Update Alive Robots
        let mut alive_robots = 0u16;
        for i in 0..self.num_robots {
            self.alive_robots[i as usize] = false;
            if let Some(_) = self.receive_message_subscriber.get().get(&i) {
                self.alive_robots[i as usize] = true;
                alive_robots |= 1 << i;
            }
        }

        println!("Alive Robots: {:#018b}", alive_robots);

        // Send Updated Alive Robots List
        self.alive_robots_publisher.publish(alive_robots.into()).unwrap();
        self.alive_robots_intra_publisher.publish(alive_robots.into()).unwrap();
    }

    fn shutdown(&mut self) {
        self.alive_robots_publisher.publish(0.into()).unwrap();
    }
}
