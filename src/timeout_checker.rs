//!
//! The Timeout Checker checks the last received timestamp for data from the robots
//! and decides when a robot could be considered dead.  When it is dead, the TimeoutChecker
//! will continually send it a request to wake up.
//! 

use ncomm::{publisher_subscriber::{local::{MappedLocalSubscriber, LocalPublisher}, udp::UdpPublisher, Publish, Receive}, node::Node};

/// The Timeout Checker will receive the timestamps for last send from the RobotRelayNode
/// and compute (every 100 ms) whether or not a robot should be considered dead.  Then, if
/// a robot is considered dead the TimeoutCheckerNode will periodically send wake up commands
/// until the robot wakes up.
pub struct TimeoutCheckerNode<'a> {
    num_robots: u8,
    timeout_duration: u128,
    alive_robots: Vec<bool>,
    receive_message_subscriber: MappedLocalSubscriber<u8, u8>,
    alive_robots_publisher: UdpPublisher<'a, u16, 2>,
    alive_robots_intra_publisher: LocalPublisher<u16>,
}

impl<'a> TimeoutCheckerNode<'a> {
    pub fn new(
        num_robots: u8,
        timeout: u128,
        alive_robots_bind_address: &'a str,
        alive_robots_send_address: &'a str,
        receive_message_subscriber: MappedLocalSubscriber<u8, u8>
    ) -> Self {
        let alive_robots_publisher = UdpPublisher::new(
            alive_robots_bind_address,
            vec![alive_robots_send_address]);
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
impl<'a> Node for TimeoutCheckerNode<'a> {
    fn name(&self) -> String { String::from("Timeout Checker Node") }

    fn get_update_delay(&self) -> u128 { self.timeout_duration }

    fn start(&mut self) {
        for _ in 0..self.num_robots {
            self.alive_robots.push(true);
        }
    }

    fn update(&mut self) {
        self.receive_message_subscriber.update_data();

        // Update Alive Robots
        let mut alive_robots = 0u16;
        for i in 0..self.num_robots {
            self.alive_robots[i as usize] = false;
            if let Some(_) = self.receive_message_subscriber.data.get(&i) {
                self.alive_robots[i as usize] = true;
                alive_robots |= 1 << i;
                self.receive_message_subscriber.data.remove(&i);
            }
        }

        // Send Updated Alive Robots List
        self.alive_robots_publisher.send(alive_robots);
        self.alive_robots_intra_publisher.send(alive_robots);
    }

    fn shutdown(&mut self) {
        self.alive_robots_publisher.send(0);
    }

    fn debug(&self) -> String {
        format!("{}: {:?}", self.name(), self.alive_robots)
    }
}