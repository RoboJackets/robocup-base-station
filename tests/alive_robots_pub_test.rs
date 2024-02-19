use ncomm::publisher_subscriber::udp::UdpPublisher;
use ncomm::publisher_subscriber::Publish;

use std::thread;
use std::time::Duration;

#[test]
fn test_send_alive_robots() {
    let mut alive_robots_publisher: UdpPublisher<'_, u16, 2> = UdpPublisher::new("0.0.0.0:8002", vec!["10.42.0.1:8001"]);

    let mut counter = 0;
    loop {
        let alive_robots = 1 << counter;
        counter = (counter + 1) % 16;

        println!("Sending Alive Robots: {:016b}", alive_robots);

        alive_robots_publisher.send(alive_robots);

        thread::sleep(Duration::from_millis(500));
    }
}