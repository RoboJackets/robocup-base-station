//!
//! The input node is responsible for handling user inputs via buttons and sending necessary update
//! commands to other nodes in the system
//! 

use crate::{TEAM_TOGGLE, START_STOP, INCREMENT_ROBOTS, DECREMENT_ROBOTS};

use super::NodeIdentifier;

use ncomm::{prelude::*, pubsubs::local::LocalSubscriber};
use ncomm::pubsubs::local::LocalPublisher;

use robojackets_robocup_rtp::Team;
use rppal::gpio::{Gpio, InputPin};

/// The input node is responsible for handling input presses to alter the state of execution
/// of the base station
pub struct InputNode {
    /// The team selector switch
    team_switch: InputPin,
    /// The current team
    team: Team,
    /// The publisher to publish the current team
    team_publisher: LocalPublisher<Team>,

    /// The button to start / stop the base station
    start_stop_switch: InputPin,
    /// Is the base-station running?
    running: bool,
    /// The publisher to stop the base station
    start_stop_publisher: LocalPublisher<bool>,

    /// The button to increment the number of robots
    increment_robots_button: InputPin,
    /// The last state of the increment robots button
    last_increment_button: bool,
    /// The button to decrement the number of robots
    decrement_robots_button: InputPin,
    /// The last state of the decrement robots button
    last_decrement_button: bool,
    /// The number of robots
    num_robots: u8,
    /// The publisher to publish the number of robots
    num_robots_publisher: LocalPublisher<u8>,
}

impl InputNode {
    /// Create a new input node
    pub fn new(
        team: Team,
        num_robots: u8,
        gpio: &mut Gpio
    ) -> Self {
        Self {
            team_switch: gpio.get(TEAM_TOGGLE)
                .expect("Unable to get team toggle pin")
                .into_input_pullup(),
            team,
            team_publisher: LocalPublisher::new(),
            start_stop_switch: gpio.get(START_STOP)
                .expect("Unable to get start / stop pin")
                .into_input_pullup(),
            running: false,
            start_stop_publisher: LocalPublisher::new(),
            increment_robots_button: gpio.get(INCREMENT_ROBOTS)
                .expect("Unable to get increment robots pin")
                .into_input_pullup(),
            last_increment_button: true,
            decrement_robots_button: gpio.get(DECREMENT_ROBOTS)
                .expect("Unable to get decrement robots pin")
                .into_input_pullup(),
            last_decrement_button: true,
            num_robots,
            num_robots_publisher: LocalPublisher::new(),
        }
    }

    /// Subscribe to the team publisher
    pub fn subscribe_team(&mut self) -> LocalSubscriber<Team> {
        self.team_publisher.subscribe()
    }

    /// Subscribe to the start stop publisher
    pub fn subscribe_start_stop(&mut self) -> LocalSubscriber<bool> {
        self.start_stop_publisher.subscribe()
    }

    /// Subscribe to the number of robots publisher
    pub fn subscribe_num_robots(&mut self) -> LocalSubscriber<u8> {
        self.num_robots_publisher.subscribe()
    }
}

impl Node<NodeIdentifier> for InputNode {
    fn get_id(&self) -> NodeIdentifier {
        NodeIdentifier::Input
    }

    fn get_update_delay_us(&self) -> u128 {
        10_000
    }

    fn start(&mut self) {
        self.running = true;
        self.start_stop_publisher.publish(true).unwrap();
        self.team_publisher.publish(self.team).unwrap();
        self.num_robots_publisher.publish(self.num_robots).unwrap();
    }

    fn update(&mut self) {
        // See if we need to switch teams
        let team_value = self.team_switch.is_high();
        if !team_value && self.team == Team::Yellow {
            self.team = Team::Blue;
            self.team_publisher.publish(self.team).unwrap();
        } else if team_value && self.team == Team::Blue {
            self.team = Team::Yellow;
            self.team_publisher.publish(self.team).unwrap();
        }

        // See if we need to start or stop the execution of other nodes
        let running = self.start_stop_switch.is_high();
        if running && !self.running {
            self.running = true;
            self.start_stop_publisher.publish(true).unwrap();
        } else if !running && self.running {
            self.running = false;
            self.start_stop_publisher.publish(false).unwrap();
        }

        // See if we need to increment the number of robots
        let increment_robots_value = self.increment_robots_button.is_high();
        if increment_robots_value && !self.last_increment_button {
            self.num_robots += 1;
            self.last_increment_button = true;
            self.num_robots_publisher.publish(self.num_robots).unwrap();
        }
        self.last_increment_button = increment_robots_value;

        // See if we need to decrement the number of robots
        let decrement_robots_value = self.decrement_robots_button.is_high();
        if decrement_robots_value && !self.last_decrement_button {
            self.num_robots = self.num_robots.saturating_sub(1);
            self.last_decrement_button = true;
            self.num_robots_publisher.publish(self.num_robots).unwrap();
        }
        self.last_decrement_button = decrement_robots_value;
    }

    fn shutdown(&mut self) {
        self.running = false;
        self.start_stop_publisher.publish(false).unwrap();
    }
}
