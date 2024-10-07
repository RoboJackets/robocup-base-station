use ncomm::prelude::*;
use ncomm::pubsubs::udp::UdpPublisher;

use robojackets_robocup_rtp::{RobotStatusMessageBuilder, Team};

use std::thread;
use std::time::Duration;

#[test]
fn test_publish_robot_status() {
    let mut robot_status_publisher = UdpPublisher::new(
        "0.0.0.0:8001".parse().unwrap(),
        vec!["10.42.0.1:8000".parse().unwrap()]
    ).unwrap();

    loop {
        let robot_status_message = RobotStatusMessageBuilder::new()
            .robot_id(0)
            .ball_sense_status(true)
            .battery_voltage(10)
            .fpga_status(true)
            .kick_healthy(true)
            .kick_status(true)
            .motor_errors(0)
            .team(Team::Blue)
            .build();

        println!("Sending Robot Status Message\n{:?}", robot_status_message);
        
        robot_status_publisher.publish(robot_status_message).unwrap();

        thread::sleep(Duration::from_millis(100));
    }
}