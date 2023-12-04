//!
//! The Timeout Checker checks the last received timestamp for data from the robots
//! and decides when a robot could be considered dead.  When it is dead, the TimeoutChecker
//! will continually send it a request to wake up.
//! 

use std::time::{SystemTime, UNIX_EPOCH};

use ncomm::{publisher_subscriber::{local::LocalSubscriber, Receive}, node::Node};

/// The Timeout Checker will receive the timestamps for last send from the RobotRelayNode
/// and compute (every 100 ms) whether or not a robot should be considered dead.  Then, if
/// a robot is considered dead the TimeoutCheckerNode will periodically send wake up commands
/// until the robot wakes up.
pub struct TimeoutCheckerNode {
    last_send_subscribers: Vec<LocalSubscriber<u128>>,
    alive_robots: Vec<bool>,
    num_robots: u8,
    timeout_duration: u128,
}

impl TimeoutCheckerNode {
    pub fn new(
        last_send_subscribers: Vec<LocalSubscriber<u128>>,
        num_robots: u8,
        timeout_duration: u128,
    ) -> Self {
        let mut alive_robots = Vec::with_capacity(num_robots as usize);
        for _ in 0..num_robots {
            alive_robots.push(false);
        }

        Self {
            last_send_subscribers,
            alive_robots,
            num_robots,
            timeout_duration,
        }
    }
}

impl Node for TimeoutCheckerNode {
    fn name(&self) -> String { String::from("Timeout Checker") }

    fn get_update_delay(&self) -> u128 { self.timeout_duration }

    // Wake Up The Robots
    fn start(&mut self) {}

    fn update(&mut self) {
        for robot_id in 0..self.num_robots {
            self.last_send_subscribers[robot_id as usize].update_data();
            let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

            match self.last_send_subscribers[robot_id as usize].data {
                Some(timestamp) => {
                    let elapsed = current_time - timestamp;
                    self.alive_robots[robot_id as usize] = elapsed <= self.timeout_duration;
                },
                None => self.alive_robots[robot_id as usize] = false,
            }
        }

        // TODO (Nathaniel Wert): Send alive robots to base computer
    }

    // Shutdown the robots
    fn shutdown(&mut self) {}

    fn debug(&self) -> String {
        format!(
            "{}:{:?}",
            self.name(),
            self.alive_robots,
        )
    }
}
