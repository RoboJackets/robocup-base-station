use ncomm::prelude::*;
use ncomm::pubsubs::udp::{UdpPublisher, UdpMappedSubscriber};

use robojackets_robocup_rtp::control_message::ControlMessage;
use robojackets_robocup_rtp::robot_status_message::RobotStatusMessageBuilder;

use std::thread;
use std::time::Duration;

fn main() {
    let mut control_message_subscriber = UdpMappedSubscriber::new(
        "0.0.0.0:8000".parse().unwrap(),
        |message: &ControlMessage| { message.robot_id }
    ).unwrap();
    let mut robot_status_publisher = UdpPublisher::new(
        "0.0.0.0:8001".parse().unwrap(),
        vec!["10.42.0.1:8001".parse().unwrap()]
    ).unwrap();
    let mut alive_robots_publisher = UdpPublisher::new(
        "0.0.0.0:8002".parse().unwrap(),
        vec!["10.42.0.1:8002".parse().unwrap()],
    ).unwrap();

    loop {
        for robot_id in 0..6 {
            if let Some(control_message) = control_message_subscriber.get().get(&robot_id) {
                println!("Received: {:?}", control_message);
                let robot_status_response = RobotStatusMessageBuilder::new()
                    .robot_id(robot_id)
                    .build();

                robot_status_publisher.publish(robot_status_response).unwrap();
                alive_robots_publisher.publish((1 << robot_id) as u16).unwrap();
            }
        }

        thread::sleep(Duration::from_millis(250));
    }
}