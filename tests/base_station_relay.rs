use std::thread;
use std::time::Duration;

use ncomm::publisher_subscriber::packed_udp::{BufferedPackedUdpSubscriber, PackedUdpPublisher};
use ncomm::publisher_subscriber::{Receive, Publish};

use robojackets_robocup_rtp::Team;
use robojackets_robocup_rtp::control_message::ControlMessage;
use robojackets_robocup_rtp::robot_status_message::RobotStatusMessage;

#[test]
fn test_receive_software_commands() {
    let mut base_computer_subscriber: BufferedPackedUdpSubscriber<ControlMessage, 10> = BufferedPackedUdpSubscriber::new("10.42.0.252:8000", None);

    loop {
        base_computer_subscriber.update_data();

        for data in base_computer_subscriber.data.drain(..) {
            if *data.robot_id == 1 {
                println!("{:?}", data);
            }
        }

        thread::sleep(Duration::from_millis(500));
    }
}

#[test]
fn test_send_software_commands() {
    let mut base_computer_publisher: PackedUdpPublisher<'_, RobotStatusMessage> = PackedUdpPublisher::new("10.42.0.252:8001", vec!["10.42.0.1:8001"]);

    loop {
        let status = RobotStatusMessage::new(Team::Blue, 1u8, true, false, true, 10u8, 0u8, true, [0u16; 18]);
        base_computer_publisher.send(status);

        thread::sleep(Duration::from_millis(500));
    }
}