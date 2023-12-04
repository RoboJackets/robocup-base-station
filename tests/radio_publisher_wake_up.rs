use std::{thread::sleep, time::Duration};

use ncomm::publisher_subscriber::Publish;
use robocup_base_station::cpu_relay_node::radio_publisher::RadioPublisher;

use rppal::{spi::{Spi, Bus, SlaveSelect, Mode}, gpio::Gpio, hal::Delay};
use sx127::LoRa;

use robojackets_robocup_rtp::control_command::ControlCommand;
use robojackets_robocup_rtp::Team;

#[test]
fn test_publish_wake_up() {
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0).unwrap();
    let gpio = Gpio::new().unwrap();
    let cs = gpio.get(8u8).unwrap().into_output();
    let reset = gpio.get(21u8).unwrap().into_output();
    let delay = Delay::new();

    // Create Radio
    let radio = LoRa::new(spi, cs, reset, 915, delay).unwrap();

    let mut radio_publisher = RadioPublisher::new(radio);

    for _ in 0..10 {
        println!("Sending Wake Up");
        radio_publisher.send(
            ControlCommand::wake_up(Team::Blue, 0u8)
        );
        sleep(Duration::from_millis(1000));
    }
}