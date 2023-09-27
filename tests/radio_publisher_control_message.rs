use std::{sync::{Arc, Mutex}, thread::sleep, time::Duration};

use ncomm::publisher_subscriber::Publish;
use robocup_base_station::cpu_relay_node::radio_publisher::RadioPublisher;

use rppal::{spi::{Spi, Bus, SlaveSelect, Mode}, gpio::Gpio, hal::Delay};
use sx127::LoRa;

use robojackets_robocup_rtp::control_message::{ControlMessage, TriggerMode};
use robojackets_robocup_rtp::Team;

#[test]
fn test_publish_control_message() {
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0).unwrap();
    let gpio = Gpio::new().unwrap();
    let cs = gpio.get(8u8).unwrap().into_output();
    let reset = gpio.get(21u8).unwrap().into_output();
    let delay = Delay::new();

    // Create Radio
    let radio = LoRa::new(spi, cs, reset, 915, delay).unwrap();
    // Wrap Radio in Mutex
    let radio = Arc::new(Mutex::new(radio));

    let mut radio_publisher = RadioPublisher::new(radio);

    for _ in 0..10 {
        println!("Sending Wake Up");
        let message = ControlMessage::new(
            Team::Blue,
            0u8,
            true,
            TriggerMode::StandDown,
            10f32,
            21.1f32,
            1.8f32,
            0i8,
            0u8,
            0u8,
        );
        radio_publisher.send(message);
        println!("Sent Data");
        sleep(Duration::from_millis(200));
    }
}