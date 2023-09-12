//!
//! The Robot Relay Node takes statuses from the robots, conglomerates them
//! and sends them back to the base computer every x milliseconds.
//! 
//! The goal of this is to handle the determining of robots being alive or dead
//! on the base station end because rust is much easier to modify than our massive
//! C++ codebase.
//! 

use ncomm::publisher_subscriber::{Publish, udp::UdpPublisher};
use ncomm::node::Node;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Data {

}

pub struct RobotRelayNode<'a> {
    base_computer_publisher: UdpPublisher<'a, [Data; 6], 24>,
    robot_statuses: [Data; 6],
}

impl<'a> RobotRelayNode<'a> {
    pub fn new(bind_address: &'a str, publish_addresses: Vec<&'a str>) -> Self {
        let base_computer_publisher = UdpPublisher::new(bind_address, publish_addresses);

        Self {
            base_computer_publisher,
            robot_statuses: [Data {}; 6],
        }
    }
}

impl<'a> Node for RobotRelayNode<'a> {
    fn name(&self) -> String { String::from("Robot --> Cpu Node") }

    fn get_update_rate(&self) -> u128 { 10 }

    fn start(&mut self) {
        self.base_computer_publisher.send(self.robot_statuses);
    }

    fn update(&mut self) {
        // TODO: Retrieve any incoming radio packets

        self.base_computer_publisher.send(self.robot_statuses);
    }

    fn shutdown(&mut self) {
        // TODO: Send Robots Dead
        self.base_computer_publisher.send([Data {}; 6]);
    }

    fn debug(&self) -> String {
        format!(
            "{}\n{:?}",
            self.name(),
            self.robot_statuses
        )
    }
}