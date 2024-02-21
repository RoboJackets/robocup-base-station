use ncomm::publisher_subscriber::{Receive, Publish};
use ncomm::publisher_subscriber::udp::UdpPublisher;
use ncomm::publisher_subscriber::packed_udp::{MappedPackedUdpSubscriber, PackedUdpPublisher};

use robojackets_robocup_rtp::control_message::{ControlMessage, ControlMessageBuilder, CONTROL_MESSAGE_SIZE};
use robojackets_robocup_rtp::robot_status_message::{RobotStatusMessage, RobotStatusMessageBuilder};
use robojackets_robocup_rtp::Team;
use robojackets_robocup_rtp::{BASE_STATION_ADDRESS, ROBOT_RADIO_ADDRESSES};

#[test]
fn test_send_and_receive_to_field_computer() {
    let mut control_message_subscriber = MappedPackedUdpSubscriber::new(
        "0.0.0.0:8000",
        None,
        Arc::new(|message: &ControlMessage| { *message.robot_id })
    );
    let mut robot_status_publisher = PackedUdpPublisher::new("0.0.0.0:8001", vec!["10.42.0.1:8001"]);
    let mut alive_robots_publisher = UdpPublisher::new(
        "0.0.0.0:8002",
        vec!["10.42.0.1:8002"],
    );

    loop {
        control_message_subscriber.update_data();

        for robot_id in 0..6 {
            if let Some(control_message) = self.control_message_subscriber.data.get(&robot_id) {
                prinltn!("Received: {:?}", control_message);
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