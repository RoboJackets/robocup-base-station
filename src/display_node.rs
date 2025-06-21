//!
//! The display node is responsible for conglomerating local system information and making it
//! accessible via a display.
//! 

use crate::relay_node::RadioStatus;
use crate::timeout_checker::AliveRobotsMessage;

use super::NodeIdentifier;

use ncomm::prelude::*;
use ncomm::pubsubs::local::{LocalMappedTTLSubscriber, LocalSubscriber};
use robojackets_robocup_rtp::RobotStatusMessage;

use rppal::gpio::Gpio;

/// The display node is responsible for making system information present for users of the base
/// station.  Basically, this node should make sure the team knows what information the base
/// station has.
pub struct DisplayNode {
    /// A subscription to what robots are alive
    alive_robots_subscription: LocalSubscriber<AliveRobotsMessage>,
    /// A subscription to the most recent status of each robot
    robot_status_subscription: LocalMappedTTLSubscriber<RobotStatusMessage, u8, fn(&Option<RobotStatusMessage>) -> u8>,
    /// A subscription to the radio stats
    radio_status_subscription: LocalSubscriber<RadioStatus>,
}

impl DisplayNode {
    /// Create a new DisplayNode with given subscriptions
    pub fn new(
        alive_robots_subscription: LocalSubscriber<AliveRobotsMessage>,
        robot_status_subscription: LocalMappedTTLSubscriber<RobotStatusMessage, u8, fn(&Option<RobotStatusMessage>) -> u8>,
        radio_status_subscription: LocalSubscriber<RadioStatus>,
        _gpio: &mut Gpio,
    ) -> Self {
        Self {
            alive_robots_subscription,
            robot_status_subscription,
            radio_status_subscription,
        }
    }
}

impl Node<NodeIdentifier> for DisplayNode {
    fn get_id(&self) -> NodeIdentifier {
        NodeIdentifier::Display
    }

    fn get_update_delay_us(&self) -> u128 {
        100_000
    }

    fn start(&mut self) {
        // TODO: Display some image on startup
        println!("Robocup Base Station ... Starting");
    }

    fn update(&mut self) {
        // TODO: Check the subscriptions and update the display
        if let Some(alive_robots) = self.alive_robots_subscription.get().as_ref() {
            println!("{:?}", alive_robots);
        }
    }

    fn shutdown(&mut self) {
        // TODO: Display some image on shutdown
        println!("Robocup Base Station ... Shutting Down")
    }
}
