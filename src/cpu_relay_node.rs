//!
//! The CPU Relay Node takes commands from the Base Computer and Relays them to the robot.
//! 

use ncomm::publisher_subscriber::{Receive, udp::UdpSubscriber};
use ncomm::node::Node;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Data {

}

pub struct CpuRelayNode {
    base_computer_subscriber: UdpSubscriber<Data, 24>,
}

impl CpuRelayNode {
    pub fn new(bind_address: &str) -> Self {
        let base_computer_subscriber = UdpSubscriber::new(bind_address, None);

        Self {
            base_computer_subscriber
        }
    }
}

impl Node for CpuRelayNode {
    fn name(&self) -> String { String::from("Cpu --> Robot Node") }

    // Basically, this node should always be running
    fn get_update_delay(&self) -> u128 { 0 }

    fn start(&mut self) {
        // TODO: Tell the robots we're starting up
        todo!()
    }

    fn update(&mut self) {
        self.base_computer_subscriber.update_data();

        // TODO: Send Most Recent Subscriber Data
        todo!()
    }

    fn shutdown(&mut self) {
        // TODO: Tell the robots we're shutting down
        todo!()
    }

    fn debug(&self) -> String { 
        self.name()
    }
}