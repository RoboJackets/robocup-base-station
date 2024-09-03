use ncomm::publisher_subscriber::{Receive, Publish};
use ncomm::publisher_subscriber::udp::UdpPublisher;
use ncomm::publisher_subscriber::packed_udp::{MappedPackedUdpSubscriber, PackedUdpPublisher};

use robojackets_robocup_rtp::control_message::ControlMessage;
use robojackets_robocup_rtp::robot_status_message::{RobotStatusMessage, RobotStatusMessageBuilder};

use std::thread;
use std::time::Duration;
use std::sync::Arc;

fn main() {
    let mut control_message_subscriber: MappedPackedUdpSubscriber<ControlMessage, u8, 10> = MappedPackedUdpSubscriber::new(
        "0.0.0.0:8000",
        None,
        Arc::new(|message: &ControlMessage| { *message.robot_id })
    );
    let mut robot_status_publisher: PackedUdpPublisher<'_, RobotStatusMessage> = PackedUdpPublisher::new("0.0.0.0:8001", vec!["10.42.0.1:8001"]);
    let mut alive_robots_publisher: UdpPublisher<'_, u16, 2> = UdpPublisher::new(
        "0.0.0.0:8002",
        vec!["10.42.0.1:8002"],
    );

    loop {
        control_message_subscriber.update_data();

        for robot_id in 0..6 {
            if let Some(control_message) = control_message_subscriber.data.get(&robot_id) {
                println!("Received: {:?}", control_message);
                let robot_status_response = RobotStatusMessageBuilder::new()
                    .robot_id(robot_id)
                    .build();

                robot_status_publisher.send(robot_status_response);
                alive_robots_publisher.send((1 << robot_id) as u16);
            }
        }

        thread::sleep(Duration::from_millis(250));
    }
}