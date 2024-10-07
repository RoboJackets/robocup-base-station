use ncomm::pubsubs::udp::UdpPublisher;
use ncomm::prelude::*;

use std::thread;
use std::time::Duration;

#[test]
fn test_send_alive_robots() {
    let mut alive_robots_publisher = UdpPublisher::new(
        "0.0.0.0:8200".parse().unwrap(),
        vec!["10.42.0.1:8001".parse().unwrap()],
    ).unwrap();

    let mut counter = 0;
    loop {
        let alive_robots = 1 << counter;
        counter = (counter + 1) % 16;

        println!("Sending Alive Robots: {:016b}", alive_robots);

        alive_robots_publisher.publish(alive_robots).unwrap();

        thread::sleep(Duration::from_millis(500));
    }
}