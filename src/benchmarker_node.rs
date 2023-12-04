//!
//! The Benchmarker Node keeps track of received packet times send times
//! to diagnose how well the base computer is keeping up with the incoming packets.
//! 
//! In the future, it may be useful for the benchmarker to alert the user in the case of
//! poor performance.
//! 

use ncomm::publisher_subscriber::{Publish, Receive, local::LocalSubscriber};
use ncomm::node::Node;

pub struct Benchmarker {
    from_field_receive_timestamps: Vec<LocalSubscriber<u128>>,
    from_robot_receive_timestamps: Vec<LocalSubscriber<u128>>,
    elapsed_times: Vec<u128>,
}

impl Benchmarker {
    pub fn new(
        from_field_subscribers: Vec<LocalSubscriber<u128>>,
        from_robot_subscribers: Vec<LocalSubscriber<u128>>,
    ) -> Self {
        Self {
            from_field_receive_timestamps: from_field_subscribers,
            from_robot_receive_timestamps: from_robot_subscribers,
            elapsed_times: Vec::new(),
        }
    }
}

impl Node for Benchmarker {
    fn name(&self) -> String { String::from("Benchmarker") }

    fn get_update_delay(&self) -> u128 { 100u128 }

    fn start(&mut self) {}

    fn update(&mut self) {
        for field_subscriber in self.from_field_receive_timestamps.iter_mut() {
            field_subscriber.update_data();
        }
    }

    fn shutdown(&mut self) {
        
    }

    fn debug(&self) -> String { String::from("Benchmarker") }
}