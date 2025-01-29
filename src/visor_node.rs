//!
//! The visor (or supervisor) node supervises each of the other nodes to obtain stats on the performance
//! of nodes.
//! 

use super::NodeIdentifier;

use ncomm::prelude::*;
use ncomm::pubsubs::local::LocalBufferedSubscriber;

/// Diagnostic sent by the radio node for each transmitted piece of data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RadioDiagnostic {
    /// The id of the robot in question
    pub robot_id: u8,
    /// The amount of time the radio took to send data
    pub send_time_us: u128,
    /// The amount of time the radio waited for incoming data
    pub wait_time_us: u128,
    /// Whether the base station received a response from the robot
    pub responded: bool,
}

/// Diagnostic sent by the radio node each update
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RadioUpdateDiagnostic {
    /// The amount of time the radio spent relaying messages
    pub total_elapsed_time_us: u128,
}

/// The visor node is supposed to supervise the other nodes.  This involves listening to 
/// messages from other nodes (specifically the radio node) and displaying / logging
/// important information
pub struct VisorNode {
    /// The average send time for each robot
    average_send_times: Vec<u128>,
    /// The average wait time for each robot
    average_wait_times: Vec<u128>,
    /// The number of times each robot has responded to radio transmissions
    total_responses: Vec<u128>,
    /// The number of radio diagnostics received per robot
    radio_diagnostics_received: Vec<u128>,

    /// The average elapsed time the radio spent relaying messages
    average_elapsed_time: u128,
    /// The number of radio update diagnostics received
    update_diagnostics_received: u128,

    /// Subscriber to radio diagnostics information
    radio_diagnostics_subscriber: LocalBufferedSubscriber<RadioDiagnostic>,
    /// Subscriber to radio update loop diagnostics information
    radio_update_subscriber: LocalBufferedSubscriber<RadioUpdateDiagnostic>,
}

impl VisorNode {
    /// Create a new visor node
    pub fn new(
        radio_diagnostics_subscriber: LocalBufferedSubscriber<RadioDiagnostic>,
        radio_update_subscriber: LocalBufferedSubscriber<RadioUpdateDiagnostic>,
        num_robots: usize,
    ) -> Self {
        Self {
            average_send_times: vec![0; num_robots],
            average_wait_times: vec![0; num_robots],
            total_responses: vec![0; num_robots],
            radio_diagnostics_received: vec![0; num_robots],
            average_elapsed_time: 0,
            update_diagnostics_received: 0,
            radio_diagnostics_subscriber,
            radio_update_subscriber
        }
    }
}

impl Node<NodeIdentifier> for VisorNode {
    fn get_id(&self) -> NodeIdentifier {
        NodeIdentifier::Visor
    }

    fn get_update_delay_us(&self) -> u128 {
        1_000_000
    }

    fn update(&mut self) {
        for radio_diagnostic in self.radio_diagnostics_subscriber.get() {
            if let Some(radio_diagnostic) = radio_diagnostic.as_ref() {
                let robot_id = radio_diagnostic.robot_id as usize;
                self.average_send_times[robot_id] = (self.average_send_times[robot_id] * self.radio_diagnostics_received[robot_id] + radio_diagnostic.send_time_us) / (self.radio_diagnostics_received[robot_id] + 1);
                self.average_wait_times[robot_id] = (self.average_wait_times[robot_id] * self.radio_diagnostics_received[robot_id] + radio_diagnostic.wait_time_us) / (self.radio_diagnostics_received[robot_id] + 1);
                self.radio_diagnostics_received[robot_id] += 1;
                if radio_diagnostic.responded {
                    self.total_responses[robot_id] += 1;
                }
            }
        }
        self.radio_diagnostics_subscriber.clear();

        for update_diagnostic in self.radio_update_subscriber.get() {
            if let Some(update_diagnostic) = update_diagnostic.as_ref() {
                self.average_elapsed_time = (self.average_elapsed_time * self.update_diagnostics_received + update_diagnostic.total_elapsed_time_us) / (self.update_diagnostics_received + 1);
                self.update_diagnostics_received += 1;
            }
        }
        self.radio_update_subscriber.clear();

        println!(
            "Radio Diagnostics:\nRobot Send Times: {:?}\nRobot Wait Times: {:?}Response Rates: {:?}\nAverage Elapsed Time: {}\n",
            self.average_send_times,
            self.average_wait_times,
            self.total_responses.iter().enumerate().map(|(i, v)| *v as f32 / self.radio_diagnostics_received[i] as f32).collect::<Vec<f32>>(),
            self.average_elapsed_time,
        );
    }

    fn shutdown(&mut self) {
        
    }
}
